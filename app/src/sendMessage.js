/**
 *
 * @param {ServiceWorkerRequest} message
 */
export async function postMessage(message) {
  if (!("serviceWorker" in navigator))
    throw new Error("Service worker not supported");

  console.debug("Waiting for worker");
  const worker = await navigator.serviceWorker.ready;
  console.debug("Got worker");
  // This should not happen as we awaited ready
  if (worker.active === null) throw new Error("No active service worker");
  worker.active.postMessage(message);
}
