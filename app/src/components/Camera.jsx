import { useNavigate } from "@solidjs/router";
import {
  createEffect,
  createResource,
  createSignal,
  onCleanup,
  Show,
  Suspense,
} from "solid-js";
//@ts-expect-error TS6192 Can not handle new JSDoc syntax (yet?)
// https://devblogs.microsoft.com/typescript/announcing-typescript-5-5/#the-jsdoc-@import-tag
/** @import { Match, Signal } from "solid-js" */
// import Worker from "../workers/saveFile.js?worker&url";

const fileWorker = new Worker(
  new URL("../workers/saveFile.js", import.meta.url),
  {
    type: "module",
  }
);

export default function Camera() {
  // 1. Ask for permission in app
  // 2. User agent asks for permission after user accepts
  // 3. If granted, remember permission and get stream
  // 5. Later when user comes back we see they granted permission and load stream directly which should not cause a
  // user agent prompt
  const [isPermissionGranted, setIsPermissionGranted] = createSignal(
    "is camera permission granted?" in localStorage
  );

  const [devices] = createResource(async () => {
    if (!isPermissionGranted()) return;
    const devices = await navigator.mediaDevices.enumerateDevices();
    // We only care about video devices
    return devices.filter((device) => device.kind === "videoinput");
  });

  // Use first device and swap through them with user interaction
  const [activeDeviceIndex, setActiveDeviceIndex] = createSignal(0);

  function swapDevice() {
    const length = devices()?.length;
    if (length === undefined || length < 2) return;

    setActiveDeviceIndex((index) => {
      // Loop through indices
      if (index === length - 1) return 0;

      return index + 1;
    });
  }

  //TODO progressively enhance to use ImageCapture API

  const activeDevice = () => {
    const videoDevices = devices();
    if (videoDevices === undefined) return;
    return videoDevices.at(activeDeviceIndex());
  };

  // devices -> activeDevice -> mediaStream
  const [mediaStream] = createResource(activeDevice, async (device) => {
    if (device === undefined) return;
    return await navigator.mediaDevices.getUserMedia({
      video: {
        deviceId: device.deviceId,
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
      async () => {
        // Restart recording when user comes back to continue showing the feed
        console.debug("Visibility change", !document.hidden);
        mediaStream()
          ?.getTracks()
          // This should disable the camera and turn off the camera light indicator
          //TODO test if bugged and light does not turn off. Use isEnabled signal then and stop/start media stream as an effect/resource
          .forEach((track) => (track.enabled = !document.hidden));
      },
      { signal: focusController.signal }
    );
  });

  // Component being removed can also be seen as losing focus
  onCleanup(async () => {
    focusController.abort();
    // Fully stop the streams
    mediaStream()
      ?.getTracks()
      .forEach((track) => track.stop());
  });

  async function handleGrantAccess() {
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

  /** @type {Signal<HTMLVideoElement | undefined>} */
  const [video, setVideo] = createSignal();

  createEffect(async () => {
    const videoElement = video();
    if (videoElement === undefined) return;
    const stream = mediaStream();
    if (stream === undefined) return;
    videoElement.srcObject = stream;
  });

  const navigate = useNavigate();
  async function takePhoto() {
    const videoElement = video();
    if (videoElement === undefined) return;
    // We take the full height and cut the sides as it is vertical conent
    // Convert to 9/16 aspect ratio based on height
    console.debug(
      "Video height",
      videoElement.videoHeight,
      "width",
      videoElement.videoWidth
    );
    const canvas = new OffscreenCanvas(
      //TODO check why this does not work but manually setting 1080 and 1920 does
      //   videoElement.videoWidth,
      //   Math.round((videoElement.videoHeight * 9) / 16)
      1080,
      1920
    );
    const context = canvas.getContext("2d");

    if (context === null) {
      console.error("Expected to get context");
      return;
    }

    context.drawImage(videoElement, 0, 0, 1080, 1920);

    /** @type {Parameters<Parameters<HTMLCanvasElement["toBlob"]>[0]>[0]} (toBlob Callback parameter) */
    //TODO set quality and image type
    const blob = await canvas.convertToBlob();
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

    //TODO Only persist image through restarts if origin private file system is supported and writable without a worker
    if ("createWritable" in fileHandle) {
      // The chromium and firefox case
      const writer = await fileHandle.createWritable({
        keepExistingData: false,
      });
      await writer.write(blob);
      // Don't forget to close the writer to persist the file
      await writer.close();
    } else {
      // The safari case
      let buffer;
      if (crossOriginIsolated) {
        // Might be faster to create an object url but that requires cleanup
        // Need to test which is faster
        buffer = new SharedArrayBuffer(blob.size);
        //TODO is this copying the buffer? Does this even have any benefit?
        const view = new Uint8Array(buffer);
        const blobBuffer = await blob.arrayBuffer();
        view.set(new Uint8Array(blobBuffer));
      } else {
        buffer = await blob.arrayBuffer();
      }

      // Create promise before posting message to worker to avoid (unlikely) race condition
      // Wait for worker to signal it is finished
      // If we don't do this the worker is disposed from navigating before it is finished
      const save = /** @type {Promise<void>} */ (
        new Promise((resolve) =>
          fileWorker.addEventListener("message", () => resolve(), {
            once: true,
          })
        )
      );

      fileWorker.postMessage(buffer);

      await save;
    }

    // Navigate to preview
    navigate("/preview");
  }

  const isSwapButtonVisible = () => {
    const count = devices()?.length;
    // Only show button if there is a camera to swap to
    return count !== undefined && count > 1;
  };

  return (
    <>
      {/* Couldn't get this to work without a container with fixed width */}
      <div class="aspect-[9/16] max-h-full min-w-80 rounded-large grid grid-cols-3 place-items-center grid-rows-[1fr_auto] overflow-hidden mx-auto">
        <video
          ref={setVideo}
          autoplay
          class="object-cover h-full w-full col-start-1 row-start-1 row-span-2 col-span-3 pointer-events-none"
        ></video>

        {/* Need to add z-index because Safari is buggy with stacking order. See this codepen for reproduction: https://codepen.io/santaclaas/pen/dyxQrKY */}
        <button
          onClick={takePhoto}
          class="bg-light-primary col-start-2 z-10 dark:bg-dark-primary block row-start-2 size-24 rounded-full self-center mb-5 ring-4 ring-light-inverse-primary dark:ring-dark-inverse-primary"
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

        <Suspense>
          <Show when={isSwapButtonVisible()}>
            {/* Need to add z-index because Safari is buggy with stacking order. See this codepen for reproduction: https://codepen.io/santaclaas/pen/dyxQrKY */}
            <button
              onClick={swapDevice}
              class="col-start-3 z-10 row-start-2 size-12 place-content-center"
            >
              <span class="sr-only">Swap camera</span>
              <svg
                xmlns="http://www.w3.org/2000/svg"
                height="24px"
                viewBox="0 -960 960 960"
                width="24px"
                class="fill-light-inverse-on-surface mx-auto"
              >
                <path d="M480-80q-143 0-253-90T88-400h82q28 106 114 173t196 67q86 0 160-42.5T756-320H640v-80h240v240h-80v-80q-57 76-141 118T480-80Zm0-280q-50 0-85-35t-35-85q0-50 35-85t85-35q50 0 85 35t35 85q0 50-35 85t-85 35ZM80-560v-240h80v80q57-76 141-118t179-42q143 0 253 90t139 230h-82q-28-106-114-173t-196-67q-86 0-160 42.5T204-640h116v80H80Z" />
              </svg>
            </button>
          </Show>
        </Suspense>
      </div>
    </>
  );
}
