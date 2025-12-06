import { precacheAndRoute } from "workbox-precaching";
import { openDB } from "idb";
import init, { create_client } from "meal-core";
import { expose } from "../crackle";
import { Schema } from "./schema";

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

const openDatabase = /** @type {typeof openDB<Schema>} */ openDB("meal", 1, {
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

  store.add({ isOnboarded: false });
});

async function getConfiguration(): Promise<Schema["configuration"]> {
  // Get configuration
  const database = await openDatabase;
  const transaction = database.transaction("configuration", "readonly");
  const store = transaction.objectStore("configuration");
  // Get first item
  const cursor = await store.openCursor();
  const configuration = /** @type {Schema["configuration"]} */ cursor?.value;
  return configuration;
}

/**
 *
 * @returns {Promise<ArrayBuffer>}
 */
async function initializeClient() {
  const configuration = await getConfiguration();
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

const handler = {
  async getIsOnboarded() {
    console.debug("[Service worker] Getting isOnboarded");
    const configuration = await getConfiguration();
    return configuration.isOnboarded;
  },

  async completeOnboarding(name: string) {
    console.debug("[Service worker]: completing onboarding", name);
    const database = await openDatabase;
    const transaction = database.transaction("configuration", "readwrite");
    const store = transaction.objectStore("configuration");
    const cursor = await store.openCursor();
    cursor?.update({ isOnboarded: true, name });
  },
};

export type Handler = typeof handler;

async function handleMessage(
  event: MessageEvent<{ type: "initializeCrackle" }>
) {
  console.debug(`[Service worker] Handling message "${event.data.type}"`);

  switch (event.data.type) {
    case "initializeCrackle": {
      const port = event.ports[0];
      expose(handler, port);
      break;
    }
  }
}

self.addEventListener("message", handleMessage);
self.addEventListener("install", () => {
  console.debug("Forcing service worker to become active");
  return self.skipWaiting();
});
// Take over the message channels from the previous service worker
self.addEventListener("activate", () => {
  console.debug("Taking over message channels");
  return self.clients.claim();
});
