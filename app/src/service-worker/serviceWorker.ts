import { precacheAndRoute } from "workbox-precaching";
import { openDB } from "idb";
import init, { Client, DecodedPackage, Friend } from "meal-core";
import { expose } from "../crackle";
import { Group, Message, Schema } from "./schema";

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
    if (!database.objectStoreNames.contains("groups")) {
      database.createObjectStore("groups", {
        autoIncrement: true,
        keyPath: "id",
      });
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
 * Convenience function to make get a file from a directory more ergonomic without failing if the file does not exist.
 */
async function getFile(
  directory: FileSystemDirectoryHandle,
  fileName: string
): Promise<FileSystemFileHandle | undefined> {
  try {
    return await directory.getFileHandle(fileName);
  } catch (error) {
    if (!(error instanceof DOMException) || error.name !== "NotFoundError")
      throw error;
  }
}

const FILE_NAME = "client.meal";
const fileSizeFormatter = new Intl.NumberFormat(undefined, {
  unit: "megabyte",
});

async function persistClient(client: Client) {
  // Persist client state
  const directory = await navigator.storage.getDirectory();

  const fileHandle = await directory.getFileHandle(FILE_NAME, {
    create: true,
  });

  const writeStream = await fileHandle.createWritable();

  const serializedClient = client.serialize();
  assertNotShared(serializedClient);

  try {
    await writeStream.write(serializedClient.buffer);
  } finally {
    writeStream.close();
  }

  console.debug(
    `[Service worker]: Stored client with length ${fileSizeFormatter.format(
      serializedClient.length
    )}`
  );

  return client;
}

async function initializeClient(): Promise<Client> {
  // Have to use wasm-pack --target web to build the wasm package to get the init function because with the bundler target
  // it is included as a top level await which is not supported by service workers according to the web spec
  // Has to be initialized before we do anything with the Rust code. It also needs to be initialized if a client exists
  await init();

  // Try to load existing client
  const directory = await navigator.storage.getDirectory();
  const fileHandle = await getFile(directory, FILE_NAME);

  if (fileHandle !== undefined) {
    const file = await fileHandle.getFile();
    const buffer = await file.arrayBuffer();
    return Client.from_serialized(new Uint8Array(buffer));
  }

  // Create new client
  const client = new Client();

  return await persistClient(client);
}

let getClient = initializeClient();

async function updateClient(client: Client) {
  getClient = persistClient(client);
  await getClient;
}

/** The url for messages endpoint. Don't forget the trailing slash. */
const messagesUrl = new URL("/messages/", self.location.origin);
async function sendGroupInvite(
  friendId: string,
  welcomePackage: Uint8Array<ArrayBuffer>
) {
  const url = new URL(friendId, messagesUrl);
  // Send welcome to peer
  const request = new Request(url, {
    method: "post",
    headers: {
      // https://www.rfc-editor.org/rfc/rfc9420.html#name-the-message-mls-media-type
      "Content-Type": "message/mls",
    },
    body: welcomePackage,
  });

  //TODO error handling
  await fetch(request);
}
function assertNotShared(
  data: Uint8Array<ArrayBufferLike>
): asserts data is Uint8Array<ArrayBuffer> {
  if (data.buffer instanceof SharedArrayBuffer)
    throw new Error("Uint8Array uses a SharedArrayBuffer");
}

async function getGroup(groupId: string): Promise<Group | undefined> {
  const database = await openDatabase;
  const transaction = database.transaction("groups", "readonly");
  const store = transaction.objectStore("groups");
  return await store.get(groupId);
}

async function updateGroup(group: Group) {
  const database = await openDatabase;
  const transaction = database.transaction("groups", "readwrite");
  const store = transaction.objectStore("groups");
  store.put(group);
}

async function insertGroup(group: Group) {
  const database = await openDatabase;
  const transaction = database.transaction("groups", "readwrite");
  const store = transaction.objectStore("groups");
  store.add(group);
}

const handler = {
  async getIsOnboarded() {
    console.debug("[Service worker] Getting isOnboarded");
    const configuration = await getConfiguration();
    return configuration.isOnboarded;
  },

  getConfiguration,
  async setName(name: string) {
    const database = await openDatabase;
    const transaction = database.transaction("configuration", "readwrite");
    const store = transaction.objectStore("configuration");
    const cursor = await store.openCursor();
    if (cursor === null) throw new Error("Expected query to return a cursor");
    cursor.update({ user: { name } });
  },

  async completeOnboarding(name: string) {
    console.debug("[Service worker]: completing onboarding", name);
    const database = await openDatabase;
    const transaction = database.transaction("configuration", "readwrite");
    const store = transaction.objectStore("configuration");
    const cursor = await store.openCursor();
    if (cursor === null) throw new Error("Expected query to return a cursor");
    cursor.update({ isOnboarded: true, name });
  },

  //TODO make providing name optional for invite
  async createInvite() {
    const [client, configuration] = await Promise.all([
      getClient,
      getConfiguration(),
    ]);

    //TODO fix invite storage/memory leak from creating and adding new key packages without removing them
    const invite_payload = client.create_invite(
      configuration.defaultUser?.name
    );

    // Persisting the client does not block us from responding
    void updateClient(client);

    const inviteUrl = new URL(`/join/${invite_payload}`, location.origin);
    return inviteUrl.href;
  },

  async decodeKeyPackage(encodedInvite: string) {
    const client = await getClient;
    //TODO make it more ergonomic to not needing to know when the client is mutated and needs to be persisted
    return client.decode_key_package(encodedInvite);
  },

  /**
   * @param friend The friend to chat with in the group
   * @param name The name the user wants to appear as in the group
   */
  async createGroup(friend: Friend, name: string) {
    const client = await getClient;
    // const { client: newClient, group_id } = create_group(client);
    const groupId = client.create_group();
    void updateClient(client);
    const group: Group = {
      id: groupId,
      user: { name },
      friend,
      messages: [],
    };

    await insertGroup(group);
    return group;
  },

  //TODO there might be overhead in passing the key package between contexts
  async inviteToGroup(groupId: string, keyPackage: DecodedPackage) {
    const client = await getClient;
    const welcomePackage = client.invite(groupId, keyPackage);
    // Persist client change even if the buffer is shared
    void updateClient(client);

    assertNotShared(welcomePackage);
    await sendGroupInvite(keyPackage.friend.id, welcomePackage);
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

async function receiveMessage(event: MessageEvent<ArrayBuffer>) {
  const client = await getClient;
  const message = client.process_message(new Uint8Array(event.data));
  void updateClient(client);
  switch (message.type) {
    case "Welcome": {
      console.debug("Processed welcome", message);
      const configuration = await getConfiguration();
      const group: Group = {
        id: message.group_id,
        friend: message.friend,
        messages: [],
        // TODO load user identity that theuser chose to be associated with the key package
        user: configuration.defaultUser,
      };
      await insertGroup(group);
      return;
    }
    case "Private":
      {
        console.debug("Processed private message", message);
        const messageEntry: Message = {
          receivedAt: new Date(),
          sentAt: new Date(message.content.sent),
          text: message.content.text,
        };

        const group = await getGroup(message.group_id);
        if (group === undefined)
          //TODO are we able to reconstruct the group?
          throw new Error("Got message for group that does not exist");

        // Assume it is sorted by time
        group.messages.push(messageEntry);
        await updateGroup(group);
      }
      return;
  }
}

async function setupWebsocket() {
  // Assume client id does not change
  const client = await getClient;

  // https:// is automatically replaced with wss://
  const socketUrl = new URL(client.id, messagesUrl);
  const socket = new WebSocket(socketUrl);
  socket.binaryType = "arraybuffer";
  socket.addEventListener("message", receiveMessage);
  socket.addEventListener("close", (event) => {
    console.warn("Socket closed. Not implemented.", event);
  });

  return socket;
}

const getSocket = setupWebsocket();
