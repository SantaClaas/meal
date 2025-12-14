/**
 * The web socket connection needs to be managed by a tab because a service worker will be terminated when it is idle.
 * The service worker is best suited for event driven communication. A shared worker would be ideal but it is not yet
 * supported by Chrome on Android. But central state management and decryption is still done on the service worker
 * because it needs to be able to decrypt push notifications and there should only be one single instance that manages
 * the client state to avoid synchronization issues.
 * See more [Leader election](https://en.wikipedia.org/wiki/Leader_election)
 */

import { messagesUrl } from "./messagesUrl";
import { setupCrackle } from "./useCrackle";

/**
 * Using a websocket instead of SSE because websocket supports binary messages while SSE only supports text and requires
 * base64 encoding.
 */
async function setUpWebsocket() {
  const handle = await setupCrackle;
  const clientId = await handle.getClientId();

  console.debug("[Websocket] Connecting to socket", clientId);
  // https:// is automatically replaced with wss://
  const socketUrl = new URL(clientId, messagesUrl);
  const socket = new WebSocket(socketUrl);
  const { promise: open, resolve } = Promise.withResolvers<void>();
  socket.addEventListener("open", () => resolve());

  socket.binaryType = "arraybuffer";

  socket.addEventListener("message", async (event) => {
    if (!(event.data instanceof ArrayBuffer))
      throw new Error("Expected ArrayBuffer");

    await handle.receiveMessage(new Uint8Array(event.data));
  });

  await open;
  console.debug("[Websocket] open");
  return socket;
}

async function runSocket() {
  const closeController = new AbortController();
  // Respond to keep alive messages from other tabs
  const socket = await setUpWebsocket();

  socket.addEventListener(
    "error",
    (event) => {
      console.error("[Websocket] Closing after error:", event);
      socket.close();
    },
    { once: true, signal: closeController.signal }
  );

  // Don't close on window unload because it is not reliable and the websocket will be closed by the browser anyway
  const { promise, resolve } = Promise.withResolvers<void>();
  socket.addEventListener(
    "close",
    () => {
      console.debug("[Websocket] Closing socket");
      closeController.abort();
      resolve();
    },
    { once: true }
  );

  return promise;
}

export async function runSocketProxy() {
  while (true) {
    console.debug("[Websocket] requesting lock");
    await navigator.locks.request("socket-proxy", async () => {
      console.debug("[Websocket] acquired lock");
      await runSocket();
      console.debug("[Websocket] socket closed. Releasing lock");
    });
  }
}
