/**
 *
 * @param {ServiceWorkerRequest} message
 */
export async function sendMessage(message) {
  if (!("serviceWorker" in navigator))
    throw new Error("Service worker not supported");

  const worker = await navigator.serviceWorker.ready;
  // This should not happen as we awaited ready
  if (worker.active === null) throw new Error("No active service worker");
  worker.active.postMessage(message);
}

/**
 *
 * @param {ServiceWorkerRegistration} message
 */
export async function sendRequest(message) {
  if (!("serviceWorker" in navigator))
    throw new Error("Service worker not supported");

  const worker = await navigator.serviceWorker.ready;
  // This should not happen as we awaited ready
  if (worker.active === null) throw new Error("No active service worker");
  worker.active.postMessage(message);
}
