export function register() {
  if (!("serviceWorker" in navigator)) return;

  // Be carefult changing these paths
  const swPath = import.meta.env.DEV
    ? "/dev-sw.js?dev-sw"
    : "/serviceWorker.js";

  console.debug("Registering service worker:", swPath);
  navigator.serviceWorker.register(swPath, { type: "module", scope: "/" });
}
