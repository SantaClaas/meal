import { precacheAndRoute } from "workbox-precaching";
import { openDB } from "idb";
import init, { Client as CoreClient } from "meal-core";

/** @import { Schema } from "./schema" */

// @ts-expect-error
self.__WB_DISABLE_DEV_LOGS = true;

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
  const client = new CoreClient(
    configuration?.clientId,
    configuration?.user?.name
  );

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
 * Map of ports to browsing contexts
 * @type {Map<string, WeakRef<MessagePort>>}
 */
const ports = new Map();
// There is no close event to clean up message channels
// https://github.com/fergald/explainer-messageport-close#proposal
setTimeout(() => {
  for (const [id, port] of ports) {
    if (port.deref() !== undefined) continue;

    console.debug("Cleaning up port", id);
    ports.delete(id);
  }
}, 100_000);

/**
 * @param {MessageEvent<ServiceWorkerRequest>} event
 */
async function handleMessage(event) {
  console.debug("Service worker: received message", event.data.type);

  switch (event.data.type) {
    case "initializePort": {
      console.debug("Received initializePort");
      if (event.source === null)
        throw new Error("Expected message event source to be not null");

      if (!(event.source instanceof Client))
        throw new Error("Expected message event source to be a client");

      const port = event.ports[0];
      port.addEventListener("message", handleMessage);
      // ports.set(id, new WeakRef(port));
      // port.start();
      port.postMessage(
        /** @type {InitializePortResponse} */ ({ type: "portInitialized" })
      );

      console.debug("Posted port initialized");

      return;
    }
    case "sendMessage": {
      const client = await setupClient;
      const body = client.send_message(event.data.groupId, {
        sent: event.data.sent.toISOString(),
        text: event.data.text,
      });
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
    case "createInvite": {
      const client = await setupClient;
      const encodedInvite = client.create_invite(client.get_name());
      const inviteUrl = new URL(`/join/${encodedInvite}`, location.origin);

      if (event.source === null)
        throw new Error(
          "Expected browser context source to send invite url back to"
        );

      if (!(event.source instanceof Client))
        throw new Error("Expected message event source to be a client");

      /** @type {ServiceWorkerResponse} */
      const response = {
        type: "inviteUrl",
        inviteUrl: inviteUrl.href,
      };
      event.ports[0].postMessage(response);
      return;
    }
  }
}

self.addEventListener("message", handleMessage);
