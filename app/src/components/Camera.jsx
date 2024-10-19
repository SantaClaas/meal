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
        // aspectRatio:
        // { ideal: 9 / 16 }
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

  /** @type {HTMLDivElement | undefined} */
  let div;

  onMount(() => {
    if (video === undefined || div === undefined) return;
  });

  return (
    <>
      {/* Couldn't get this to work without a container with fixed width */}
      <div
        ref={div}
        class="aspect-[9/16] w-1/2 rounded-large relative overflow-hidden"
      >
        <video
          ref={video}
          autoplay
          class="object-cover h-full absolute inset-0 w-full"
        ></video>
        {/* <button onPointerDown={handleTakePhoto}>Take Photo</button> */}
      </div>
    </>
  );
}
