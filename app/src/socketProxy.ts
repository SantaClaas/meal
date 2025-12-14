/**
 * The web socket connection needs to be managed by a tab because a service worker will be terminated when it is idle.
 * The service worker is best suited for event driven communication. A shared worker would be ideal but it is not yet
 * supported by Chrome on Android. But central state management and decryption is still done on the service worker
 * because it needs to be able to decrypt push notifications and there should only be one single instance that manages
 * the client state to avoid synchronization issues.
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
  socket.binaryType = "arraybuffer";

  socket.addEventListener("message", async (event) => {
    if (!(event.data instanceof ArrayBuffer))
      throw new Error("Expected ArrayBuffer");

    await handle.receiveMessage(new Uint8Array(event.data));
  });

  return socket;
}

const channel = new BroadcastChannel("websocket-tab-coordination");

type InitializedMessage = {
  type: "initializing socket";
};

type Message =
  | {
      type: "is socket alive?";
    }
  | { type: "socket is alive"; }
  | InitializedMessage;

type TimedOut = {
  type: "timed out";
  value?: never;
};

type TimeoutResult<T> =
  | TimedOut
  | {
      type: "success";
      value: T;
    };


function timeout(milliseconds: number) {
  return new Promise<void>((resolve) =>
    setTimeout(() => resolve(), milliseconds)
  );
}

function withTimeout<T>(
  promise: Promise<T>,
  milliseconds: number
): Promise<TimeoutResult<T>> {
  const awaitPromise = async () => {
    const result = await promise;
    return { type: "success", value: result } as const;
  };

  return Promise.race([
    awaitPromise(),
    timeout(milliseconds).then(() => ({ type: "timed out" } as const)),
  ]);
}

async function isMainTabAlive() {
  channel.postMessage({
    type: "is socket alive?"
  } satisfies Message);

  const keepAliveResponse = new Promise<void>((resolve) => {
    channel.addEventListener("message", (message: MessageEvent<Message>) => {
      if (message.data.type !== "socket is alive") return;
      resolve();
    })
  })

  const initializedMessage = new Promise<InitializedMessage>((resolve) => {
    channel.addEventListener("message", (message: MessageEvent<Message>) => {
      if (message.data.type !== "initializing socket") return;
      resolve(message.data);
    })
  })

  const result = await Promise.race([initializedMessage, withTimeout(keepAliveResponse, 10_000)])

  // Is either initialized or got a keep alive response
  return result.type === "initializing socket" || result.type === "success";
}


function handleKeepAlive(signal: AbortSignal){
  channel.addEventListener(
    "message",
    (event: MessageEvent<Message>) => {
      switch (event.data.type) {
        case "is socket alive?": {
          console.debug("[Websocket] Keep alive received", event.data);
          channel.postMessage({
            type: "socket is alive",
          } satisfies Message);
          return;
        }
      }
    },
    {
      signal,
    }
  );
}

async function runMainTab(){
  // Notify other tabs that we are taking over as main tab and they should not take over
  channel.postMessage({ type: "initializing socket" } satisfies Message);

  const closeController = new AbortController();
  // Respond to keep alive messages from other tabs
  handleKeepAlive(closeController.signal);
  const socket = await setUpWebsocket();

  socket.addEventListener("error", (event) => {
    console.error("[Websocket] Closing after error:", event);
    socket.close();
  }, { once: true, signal: closeController.signal });

  // Don't close on window unload because it is not reliable and the websocket will be closed by the browser anyway
  return await new Promise<void>((resolve) => {
    socket.addEventListener(
      "close",
      () => {
        closeController.abort();
        resolve()
      },
      { once: true }
    )
  });

}

export async function runSocketProxy() {
  while (true) {
    const isAlive = await isMainTabAlive();
    console.debug("[Websocket] Main tab alive:", isAlive);
    if (isAlive) {
      await timeout(10_000);
      continue;
    }

    // Take over main tab responsiblities
    await runMainTab();
  }
}
