import { createResource } from "solid-js";

export default function Debug() {
  const [devices] = createResource(async () => {
    // if (!isPermissionGranted()) return;
    const devices = await navigator.mediaDevices.enumerateDevices();
    // We only care about video devices
    return devices.filter((device) => device.kind === "videoinput");
  });

  const json = () => {
    return JSON.stringify(devices(), null, 2);
  };
  return <pre>{json()}</pre>;
}
