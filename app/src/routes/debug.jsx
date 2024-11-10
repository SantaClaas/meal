import { createResource, createSignal } from "solid-js";

export default function Debug() {
  const [devices] = createResource(async () => {
    // if (!isPermissionGranted()) return;
    const devices = await navigator.mediaDevices.enumerateDevices();
    // We only care about video devices
    return devices.filter((device) => device.kind === "videoinput");
  });

  const devicesJson = () => {
    return JSON.stringify(devices(), null, 2);
  };

  // /**
  //  * @type {Signal<ConstrainDOMString>}
  //  */
  const [facingMode, setFacingMode] = createSignal("environment");
  const [mediaStream] = createResource(facingMode, async (facingMode) => {
    if (facingMode === undefined) return;
    return await navigator.mediaDevices.getUserMedia({
      video: {
        aspectRatio: { ideal: 9 / 16 },
        facingMode,
      },
      audio: false,
    });
  });

  const constraints = () => {
    return mediaStream()
      ?.getTracks()
      .map((track) => track.getCapabilities());
  };

  const constraintsJson = () => {
    return JSON.stringify(constraints(), null, 2);
  };

  function swapCamera() {
    setFacingMode((facingMode) => {
      if (facingMode === "environment") {
        return "user";
      }
      return "environment";
    });
  }

  return (
    <>
      <article>
        <pre>{devicesJson()}</pre>
        <pre>{constraintsJson()}</pre>

        <button
          onClick={swapCamera}
          class="p-4 bg-dark-primary text-dark-on-primary rounded-full"
        >
          Switch
        </button>
      </article>
    </>
  );
}
