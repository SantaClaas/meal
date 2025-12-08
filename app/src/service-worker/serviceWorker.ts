import { precacheAndRoute } from "workbox-precaching";
import init, { Client, DecodedPackage, Friend } from "meal-core";
import { expose } from "../crackle";
import { Group, IncomingMessage } from "../database/schema";
import { broadcastMessage } from "../broadcast";
import {
  deleteDatabase,
  getConfiguration,
  getGroup,
  insertGroup,
  openDatabase,
  updateGroup,
} from "../database";

// Reduce noise
// @ts-expect-error
self.__WB_DISABLE_DEV_LOGS = true;

console.debug("Service worker: environment", process.env.NODE_ENV);
// @ts-expect-error This variable is replaced by workbox through vite pwa plugin
precacheAndRoute(self.__WB_MANIFEST);

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

async function deleteClient() {
  const directory = await navigator.storage.getDirectory();
  try {
    await directory.removeEntry(FILE_NAME, {
      recursive: false,
    });
  } catch (error) {
    if (error instanceof DOMException && error.name === "NotFoundError") {
      console.debug("No client file found");
      return;
    }

    throw error;
  }
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
async function postMessage(friendId: string, body: Uint8Array<ArrayBuffer>) {
  const url = new URL(friendId, messagesUrl);
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
}

function assertNotShared(
  data: Uint8Array<ArrayBufferLike>
): asserts data is Uint8Array<ArrayBuffer> {
  if (data.buffer instanceof SharedArrayBuffer)
    throw new Error("Uint8Array uses a SharedArrayBuffer");
}

export type OutgoingMessage = {
  groupId: string;
  sentAt: Date;
  text: string;
};

const handler = {
  async getIsOnboarded() {
    console.debug("[Service worker] Getting isOnboarded");
    const configuration = await getConfiguration();
    return configuration.isOnboarded;
  },

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
    await postMessage(keyPackage.friend.id, welcomePackage);
  },

  async sendMessage(message: OutgoingMessage) {
    const group = await getGroup(message.groupId);
    if (group === undefined) throw new Error("Group not found");

    const client = await getClient;
    const body = client.send_message(group.id, {
      sent_at: message.sentAt,
      text: message.text,
    });

    assertNotShared(body);
    await postMessage(group.friend.id, body);
  },

  async wipe() {
    // Wipe client
    await Promise.all([
      getSocket.then((socket) => socket.close()),
      //TODO test what happens if clients have open connections to the database
      deleteDatabase(),
      deleteClient(),
    ]);
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
      broadcastMessage({ type: "Group created", group });
      return;
    }
    case "Private":
      {
        console.debug("Processed private message", message);
        const messageEntry: IncomingMessage = {
          receivedAt: new Date(),
          sentAt: new Date(message.content.sent_at),
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

//TODO why did I decide to use a websocket instead of SSE? Replace with SSE
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
