import { createEffect, createResource, createSignal, onMount } from "solid-js";

/** @import { Signal } from "solid-js" */

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

  const [mediaStream, { mutate }] = createResource(async () => {
    // Wait for permission
    if (!isPermissionGranted()) return;
    return await navigator.mediaDevices.getUserMedia({
      video: {
        aspectRatio: { ideal: 9 / 16 },
      },
      audio: false,
    });
  });

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
    console.debug("Video", video, mediaStream());
    if (video === undefined) return;
    const stream = mediaStream();
    if (stream === undefined) return;

    console.debug("Setting stream");
    video.srcObject = stream;
  });

  return (
    <>
      {/* Couldn't get this to work without a container with fixed width */}
      <div class="aspect-[9/16] w-3/4 rounded-large grid grid-cols-1 place-items-center grid-rows-[1fr_auto] overflow-hidden mx-auto">
        <video
          ref={video}
          autoplay
          class="object-cover h-full w-full col-start-1 row-start-1 row-span-2 pointer-events-none"
        ></video>
        <button class="bg-light-primary dark:bg-dark-primary block col-start-1 row-start-2 size-24 rounded-full self-center mb-5 ring-4 ring-light-inverse-primary dark:ring-dark-inverse-primary">
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
      </div>
    </>
  );
}
