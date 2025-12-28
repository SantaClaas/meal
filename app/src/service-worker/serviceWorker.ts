import { cleanupOutdatedCaches, precacheAndRoute } from "workbox-precaching";
import init, { Client, DecodedPackage, Friend } from "meal-core";
import { expose } from "../crackle";
import { Group, IncomingMessage, OutgoingMessage } from "../database/schema";
import { broadcastMessage } from "../broadcast";
import {
  deleteDatabase,
  getConfiguration,
  insertGroup,
  openDatabase,
  pushMessage,
} from "../database";
import { messagesUrl } from "../messagesUrl";
import "./disposePolyfill";

const initialization = init();

// Reduce noise
// @ts-expect-error
self.__WB_DISABLE_DEV_LOGS = true;

console.debug("Service worker: environment", process.env.NODE_ENV);

cleanupOutdatedCaches();
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

/** Wraps a file handle in a nice disposable interface and ensures compatibility with Safari < 26 */
async function createWritable(fileHandle: FileSystemFileHandle): Promise<{
  write: (data: Uint8Array) => Promise<void>;
  [Symbol.dispose](): void;
}> {
  // Safari < 26 compatibility
  if (!("createWritable" in fileHandle)) {
    // Not sure how to solve this without casting as it is turned into never by the assertion in the if statement
    // but Safari does not have createWritable
    const accessHandle = await (
      fileHandle as FileSystemFileHandle
    ).createSyncAccessHandle();

    return {
      write(data: Uint8Array): Promise<void> {
        accessHandle.write(data);
        accessHandle.flush();
        return Promise.resolve();
      },

      [Symbol.dispose]() {
        accessHandle.close();
      },
    };
  }

  const writable = await fileHandle.createWritable();
  return {
    async write(data: Uint8Array) {
      assertNotShared(data);
      await writable.write(data);
    },
    [Symbol.dispose]() {
      writable.close();
    },
  };
}

async function persistClient(client: Client) {
  // Persist client state
  const directory = await navigator.storage.getDirectory();

  const fileHandle = await directory.getFileHandle(FILE_NAME, {
    create: true,
  });

  using writable = await createWritable(fileHandle);

  const serializedClient = client.serialize();
  writable.write(serializedClient);

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
  await initialization;

  console.debug("[Service worker/client initialization]: Initializing client");
  // Try to load existing client
  const directory = await navigator.storage.getDirectory();
  const fileHandle = await getFile(directory, FILE_NAME);

  console.debug("[Service worker/client initialization]: File handle acquired", fileHandle);

  if(fileHandle === undefined || fileHandle.kind !== "file"){
    console.debug("[Service worker/client initialization]: No file handle, creating new client");
    const client = new Client();
    return await persistClient(client);
  }

  const file = await fileHandle.getFile();

  if(file.size === 0) {
    console.debug("[Service worker/client initialization]: Empty file handle, creating new client");
    const client = new Client();
    return await persistClient(client);
  }

  console.debug("[Service worker/client initialization]: Deserializing client");
  const buffer = await file.arrayBuffer();
  return Client.from_serialized(new Uint8Array(buffer));
}

let getClient = initializeClient();

async function updateClient(client: Client) {
  getClient = persistClient(client);
  await getClient;
}

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
  return await fetch(request);
}

function assertNotShared(
  data: Uint8Array<ArrayBufferLike>
): asserts data is Uint8Array<ArrayBuffer> {
  // Need to check if SharedArrayBuffer is even defined for Safari
  if ("SharedArrayBuffer" in globalThis && data.buffer instanceof SharedArrayBuffer)
    throw new Error("Uint8Array uses a SharedArrayBuffer");
}

type SendMessageRequest = {
  friendId: string;
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
    broadcastMessage({ type: "Group created", group });
    return group;
  },

  //TODO there might be overhead in passing the key package between contexts
  async inviteToGroup(groupId: string, keyPackage: DecodedPackage) {
    const client = await getClient;
    const welcomePackage = client.invite(groupId, keyPackage);
    // Persist client change even if the buffer is shared
    void updateClient(client);

    assertNotShared(welcomePackage);
    const response = await postMessage(keyPackage.friend.id, welcomePackage);
    if (response.status !== 201)
      throw new Error(`Unexpected status code response ${response.status}`);
  },

  async sendMessage(request: SendMessageRequest) {
    const message: OutgoingMessage = {
      type: "outgoing",
      sentAt: request.sentAt,
      text: request.text,
    };

    broadcastMessage({
      type: "Message added",
      groupId: request.groupId,
      message,
    });

    // Storing the message should happen even if sending fails as that can be retried
    const storeMessage = pushMessage(request.groupId, message);

    const client = await getClient;
    const body = client.send_message(request.groupId, {
      sent_at: request.sentAt.toISOString(),
      text: request.text,
    });

    assertNotShared(body);
    await Promise.all([postMessage(request.friendId, body), storeMessage]);
  },

  async wipe() {
    // Wipe client
    await Promise.all([
      broadcastMessage({ type: "Wipe" }),
      //TODO test what happens if clients have open connections to the database
      deleteDatabase(),
      deleteClient(),
    ]);
  },

  async getClientId() {
    const client = await getClient;
    return client.id;
  },

  async receiveMessage(data: Uint8Array) {
    const client = await getClient;
    const message = client.process_message(data);
    console.debug("Processed message", message.type);
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
            type: "incoming",
            receivedAt: new Date(),
            sentAt: new Date(message.content.sent_at),
            text: message.content.text,
          };

          await pushMessage(message.group_id, messageEntry);

          // Need to store message before showing it
          broadcastMessage({
            type: "Message added",
            groupId: message.group_id,
            message: messageEntry,
          });
        }
        return;
    }
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
  console.debug("[Service worker] Forcing service worker to become active");
  return self.skipWaiting();
});
// Take over the message channels from the previous service worker
self.addEventListener("activate", () => {
  console.debug("[Service worker] Taking over message channels");
  return self.clients.claim();
});
