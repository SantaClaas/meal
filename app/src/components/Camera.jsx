import { useNavigate } from "@solidjs/router";
import {
  createEffect,
  createResource,
  createSignal,
  onCleanup,
  onMount,
  Show,
  Switch,
} from "solid-js";

/** @import { Match, Signal } from "solid-js" */

export default function Camera() {
  // 1. Ask for permission in app
  // 2. User agent asks for permission after user accepts
  // 3. If granted, remember permission and get stream
  // 5. Later when user comes back we see they granted permission and load stream directly which should not cause a
  // user agent prompt
  const [isPermissionGranted, setIsPermissionGranted] = createSignal(
    "is camera permission granted?" in localStorage
  );

  //   /** @type {Signal<MediaStream | undefined>} */
  //   const [mediaStream, setMediaStream] = createSignal();
  //TODO progressively enhance to use ImageCapture API
  const [mediaStream, { mutate, refetch }] = createResource(async () => {
    // Wait for permission
    if (!isPermissionGranted()) return;
    return await navigator.mediaDevices.getUserMedia({
      video: {
        aspectRatio: { ideal: 9 / 16 },
      },
      audio: false,
    });
  });

  // Stop preview and activated camera when user is not in app
  const focusController = new AbortController();
  //   document.addEventListener("focusout", () => console.debug("Lost focus"));
  // If we encounter issues with this then read this: https://stackoverflow.com/questions/28993157/visibilitychange-event-is-not-triggered-when-switching-program-window-with-altt
  createEffect(() => {
    if (!isPermissionGranted()) return;
    window.addEventListener(
      "visibilitychange",
      () => {
        if (document.hidden) {
          mediaStream()
            ?.getTracks()
            .forEach((track) => track.stop());
          return;
        }

        // Restart recording when user comes back to continue showing the feed
        refetch();
      },
      { signal: focusController.signal }
    );
  });

  // Component being removed can also be seen as losing focus
  onCleanup(() => {
    focusController.abort();
  });

  //   createEffect((interval) => {
  //     const stream = mediaStream();
  //     if (stream === undefined) return interval;
  //     if (interval !== undefined) clearInterval(interval);

  //     const newInterval = setInterval(
  //       () => console.debug("Has fucus", document.hasFocus()),
  //       1_000
  //     );
  //     return newInterval;
  //   });

  async function handleGrantAccess() {
    const mediaStream = await navigator.mediaDevices.getUserMedia({
      video: true,
      audio: false,
    });

    mutate(mediaStream);

    localStorage.setItem("is camera permission granted?", "");
    setIsPermissionGranted(true);
  }

  if (!isPermissionGranted()) {
    return (
      <>
        <h2>Camera Permission</h2>
        <p>
          Hey there, we need your permission to use your camera to take photos.
        </p>
        <button onClick={handleGrantAccess}>Grant access</button>
      </>
    );
  }

  /** @type {HTMLVideoElement | undefined} */
  let video;

  createEffect(() => {
    if (video === undefined) return;
    const stream = mediaStream();
    if (stream === undefined) return;
    video.srcObject = stream;
  });

  /** @type {HTMLCanvasElement | undefined} */
  let canvas;

  /** @type {Signal<Blob | undefined>} */
  const [photo, setPhoto] = createSignal();
  const navigate = useNavigate();
  async function takePhoto() {
    if (canvas === undefined || video === undefined) return;

    const context = canvas.getContext("2d");
    // We take the full height and cut the sides as it is vertical conent
    canvas.height = video.videoHeight;
    // Convert to 9/16 aspect ratio based on height
    canvas.width = Math.round((video.videoHeight * 9) / 16);

    if (context === null) {
      console.error("Expected to get context");
      return;
    }
    context.drawImage(video, 0, 0);

    /** @type {Parameters<Parameters<HTMLCanvasElement["toBlob"]>[0]>[0]} (toBlob Callback parameter) */
    //TODO set quality and image type
    const blob = await new Promise((resolve) => canvas.toBlob(resolve));

    if (blob === null) {
      console.error("Could not create photo blob");
      return;
    }

    // Stop recording
    mediaStream()
      ?.getTracks()
      .forEach((track) => track.stop());

    // Store photo to origin private file system to persist it in case of unexpected app closing
    // Fall back to only keeping it in memory when we can't persist it
    //TODO checkout persist
    // navigator.storage.persist()
    const directory = await navigator.storage.getDirectory();

    const fileHandle = await directory.getFileHandle("preview", {
      create: true,
    });
    const writer = await fileHandle.createWritable({ keepExistingData: false });
    await writer.write(blob);
    // Don't forget to close the writer to persist the file
    await writer.close();

    console.debug("Wrote preview to file");

    // Navigate to preview
    navigate("/preview");
  }

  return (
    <>
      {/* Couldn't get this to work without a container with fixed width */}
      <div class="aspect-[9/16] max-w-2xl max-h-full min-w-80 rounded-large grid grid-cols-1 place-items-center grid-rows-[1fr_auto] overflow-hidden mx-auto">
        <Show when={photo() === undefined}>
          <video
            ref={video}
            autoplay
            class="object-cover h-full w-full col-start-1 row-start-1 row-span-2 pointer-events-none"
          ></video>

          <button
            onClick={takePhoto}
            class="bg-light-primary dark:bg-dark-primary block col-start-1 row-start-2 size-24 rounded-full self-center mb-5 ring-4 ring-light-inverse-primary dark:ring-dark-inverse-primary"
          >
            <span class="sr-only">Take Photo</span>
            <svg
              xmlns="http://www.w3.org/2000/svg"
              height="48px"
              viewBox="0 -960 960 960"
              width="48px"
              class="mx-auto fill-dark-on-primary dark:fill-light-on-primary "
            >
              <path d="M480-260q75 0 127.5-52.5T660-440q0-75-52.5-127.5T480-620q-75 0-127.5 52.5T300-440q0 75 52.5 127.5T480-260Zm0-80q-42 0-71-29t-29-71q0-42 29-71t71-29q42 0 71 29t29 71q0 42-29 71t-71 29ZM160-120q-33 0-56.5-23.5T80-200v-480q0-33 23.5-56.5T160-760h126l74-80h240l74 80h126q33 0 56.5 23.5T880-680v480q0 33-23.5 56.5T800-120H160Zm0-80h640v-480H638l-73-80H395l-73 80H160v480Zm320-240Z" />
            </svg>
          </button>
        </Show>

        <canvas
          ref={canvas}
          width={1080}
          height={1920}
          classList={{ hidden: photo() === undefined }}
          class="h-full"
        ></canvas>
      </div>
    </>
  );
}
