import { proxy } from "./crackle";
import { Handler } from "./service-worker/serviceWorker";

async function initializeCrackle() {
  const { port1, port2 } = new MessageChannel();
  const message = {
    type: "initializeCrackle",
  };

  const worker = await navigator.serviceWorker.ready;
  // Should not happen as we just awaited the ready promise
  if (worker.active === null) throw new Error("No active service worker");
  worker.active.postMessage(message, [port1]);
  const wrapper = await proxy<Handler>(port2);
  return wrapper;
}

export const setupCrackle = initializeCrackle();
