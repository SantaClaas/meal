import { precacheAndRoute } from "workbox-precaching";
import { openDB } from "idb";
import init, { create_client } from "meal-core";

/**
 * @import { Schema } from "./schema"
 * @import { Operation } from "./operation"
 */

// Reduce noise
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

openDatabase.then((database) => {
  const transaction = database.transaction("configuration", "readwrite");
  const store = transaction.objectStore("configuration");
  if (store.add === undefined) throw new Error("Store is undefined");

  store.add({});
});

/**
 *
 * @returns {Promise<ArrayBuffer>}
 */
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

  // Persist client state
  const directory = await navigator.storage.getDirectory();

  const FILE_NAME = "client.meal";
  /** @type {FileSystemFileHandle | undefined} */
  let fileHandle;
  try {
    fileHandle = await directory.getFileHandle(FILE_NAME);
  } catch (error) {
    // Finding out if the file does not exist is a bit unergonomic
    if (!(error instanceof DOMException) || error.name !== "NotFoundError")
      throw error;

    console.debug("No client.meal file found");
  }

  if (fileHandle !== undefined) {
    const file = await fileHandle.getFile();
    return await file.arrayBuffer();
  }

  const client = create_client(
    configuration.clientId,
    configuration.user?.name
  );

  fileHandle = await directory.getFileHandle(FILE_NAME, {
    create: true,
  });

  const writeStream = await fileHandle.createWritable();
  if (client.buffer instanceof SharedArrayBuffer)
    throw new Error("Did not expect a SharedArrayBuffer");

  try {
    await writeStream.write(client.buffer);
  } finally {
    writeStream.close();
  }

  console.debug("Created new client", client);

  return client.buffer;
}

const setupClient = initializeClient();

/** The url for messages endpoint. Don't forget the trailing slash. */
const messagesUrl = new URL("/messages/", self.location.origin);

/**
 * @param {MessageEvent<ServiceWorkerRequest | ServiceWorkerMessage>} event
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
      // const body = client.send_message(event.data.groupId, {
      //   sent: event.data.sent.toISOString(),
      //   text: event.data.text,
      // });
      // const url = new URL(event.data.friendId, messagesUrl);
      // const request = new Request(url, {
      //   method: "post",
      //   headers: {
      //     //https://www.rfc-editor.org/rfc/rfc9420.html#name-the-message-mls-media-type
      //     "Content-Type": "message/mls",
      //   },
      //   body,
      // });

      // //TODO error handling
      // //TODO retry
      // await fetch(request);
      return;
    }
    case "createInvite": {
      const client = await setupClient;
      // const encodedInvite = client.create_invite(client.get_name());
      // const inviteUrl = new URL(`/join/${encodedInvite}`, location.origin);

      // if (event.source === null)
      //   throw new Error(
      //     "Expected browser context source to send invite url back to"
      //   );

      // if (!(event.source instanceof Client))
      //   throw new Error("Expected message event source to be a client");

      // /** @type {ServiceWorkerResponse} */
      // const response = {
      //   type: "inviteUrl",
      //   inviteUrl: inviteUrl.href,
      // };
      // event.ports[0].postMessage(response);
      return;
    }
    case "inviteFromPackage": {
      // Update name state
      // Update name in database
      // Broadcast name update to all contexts
      // Decode key package
      // Create group with core
      // Create group in database
      // Broadcast group creation to all contexts
      // Create invite welcome package with core
      // Send welcome package to friend
    }
  }
}

self.addEventListener("message", handleMessage);
