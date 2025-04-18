import { precacheAndRoute } from "workbox-precaching";
import { openDB } from "idb";
import init, { Client } from "meal-core";

/** @import { Schema } from "./schema" */

console.debug("Service worker: environment", process.env.NODE_ENV);
// @ts-expect-error This variable is replaced by workbox through vite pwa plugin
precacheAndRoute(self.__WB_MANIFEST);

const openDatabase = /** @type {typeof openDB<Schema>} */ (openDB)("meal", 1, {
  upgrade(database) {
    console.debug("Service worker: upgrading database");
    if (!database.objectStoreNames.contains("configuration")) {
      database.createObjectStore("configuration", { autoIncrement: true });
    }
  },
});

async function initializeClient() {
  // Get configuration
  const database = await openDatabase;
  const transaction = database.transaction("configuration", "readonly");
  const store = transaction.objectStore("configuration");
  // Get first item
  const cursor = await store.openCursor();
  const configuration = /** @type {Schema["configuration"]} */ (cursor?.value);
  // Have to use wasm-pack --target web to build the wasm package to get the init function because with the bundler target
  // it is included as a top level await which is not supported by service workers according to the web spec
  await init();
  const client = new Client(configuration?.clientId, configuration?.user?.name);
  return client;
}

const setupClient = initializeClient();

// const id = localStorage.getItem("id");
// const name = localStorage.getItem("name");
// const isOnboarded = localStorage.getItem("isOnboarded") !== null;
// const client = new Client(id ?? undefined, name ?? undefined);

/** The url for messages endpoint. Don't forget the trailing slash. */
const messagesUrl = new URL("/messages/", self.location.origin);

/**
 * @param {MessageEvent<ServiceWorkerRequest>} event
 */
async function handleMessage(event) {
  console.debug("Service worker: received message", event.data);

  switch (event.data.type) {
    case "sendMessage":
      const client = await setupClient;
      const body = client.send_message(event.data.groupId, event.data);
      const url = new URL(event.data.friendId, messagesUrl);
      const request = new Request(url, {
        method: "post",
        headers: {
          //https://www.rfc-editor.org/rfc/rfc9420.html#name-the-message-mls-media-type
          "Content-Type": "message/mls",
        },
        body,
      });

      //TODO error handling
      //TODO retry
      await fetch(request);
      return;
  }
}

self.addEventListener("message", handleMessage);
