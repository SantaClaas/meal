import { createContext, createEffect, createMemo, useContext } from "solid-js";
//TODO register rust wasm pack package as package in workspace
import { Client } from "../../../core/pkg";
import { createStore } from "solid-js/store";

/**
 * @import { ParentProps, Context, EffectFunction } from "solid-js";
 * @import { Friend } from "../../../core/pkg";
 * @typedef {{id: string, friend: Friend, messages: string[]}} Group
 */

/**
 * A wrapper around the client to make using it easier
 */
class App {
  client;

  groups;

  #setGroups;
  /**
   * @param {Client} client
   */
  constructor(client) {
    this.client = client;
    const [groups, setGroups] = createStore([]);
    this.groups = groups;
    this.#setGroups = setGroups;
  }

  /**
   *
   * @param {string} toClientId
   * @param {string} groupId
   * @param {string} message
   */
  async sendMessage(toClientId, groupId, message) {
    const body = this.client.send_message(groupId, message);
    const request = new Request(
      `http://127.0.0.1:3000/messages/${toClientId}`,
      {
        method: "post",
        headers: {
          //https://www.rfc-editor.org/rfc/rfc9420.html#name-the-message-mls-media-type
          "Content-Type": "message/mls",
        },
        body,
      }
    );

    //TODO error handling
    //TODO retry
    await fetch(request);
  }
}

const id = localStorage.getItem("id");
const name = localStorage.getItem("name");
const isOnboarded = localStorage.getItem("isOnboarded") !== null;
const client = new Client(id ?? undefined, name ?? undefined);
const [app, setApp] = createStore({
  name,
  // Client creates an id if there is none provided
  get id() {
    return client.id;
  },
  client,
  /**
   * @type {Group[]}
   */
  groups: [],
  isOnboarded,
});

// When name changes, update client and local storage
createEffect(
  /** @type {EffectFunction<ReturnType<typeof name>>}*/ (previousName) => {
    console.debug("Updating name from", previousName, "to", app.name);
    const newName = app.name;

    if (newName === previousName) return newName;

    if (newName === null) {
      localStorage.removeItem("name");
      app.client.set_name(undefined);
      return newName;
    }

    localStorage.setItem("name", newName);
    app.client.set_name(newName);
    return newName;
  },
  app.name
);

// Update local storage when isOnboarded changes
createEffect(
  /** @type {EffectFunction<typeof isOnboarded | undefined>}*/ (
    previousIsOnboarded
  ) => {
    console.debug(
      "Updating isOnboarded from",
      previousIsOnboarded,
      "to",
      app.isOnboarded
    );
    if (previousIsOnboarded === app.isOnboarded) return app.isOnboarded;

    // We only check if it exists to check if user is onboarded
    if (app.isOnboarded) {
      localStorage.setItem("isOnboarded", "");
    } else {
      localStorage.removeItem("isOnboarded");
    }

    return app.isOnboarded;
  }
);

/**
 * @param {MessageEvent} event
 */
function receiveMessage(event) {
  if (!(event.data instanceof ArrayBuffer)) {
    console.error("Expected websocket data to be binary");
    return;
  }

  console.debug("Processing message");
  const message = /** @type {Message} */ (
    app.client.process_message(new Uint8Array(event.data))
  );
  console.debug("Processed message", message);
  switch (message.type) {
    case "Welcome":
      console.debug("Processed welcome", message);
      // Add group to store https://docs.solidjs.com/concepts/stores#appending-new-values
      setApp("groups", app.groups.length, {
        id: message.group_id,
        friend: message.friend,
        messages: [],
      });
      break;
    case "Private":
      console.debug("Received message", message.group_id, message.message);

      // Get group
      //TODO sort groups by last message time
      // Using linear search because:
      // 1. Groups that often receive messages are displayed at the top of the list which reduces search time
      // 2. An average user has few active groups that reguarly receive messages

      // Assuming groups that often receive messages are reguarly used
      // this means they are at the top of the list.

      const groupIndex = app.groups.findIndex(
        (group) => group.id === message.group_id
      );

      // First time store user. This stuff is whack
      setApp("groups", groupIndex, "messages", (messages) => [
        ...messages,
        message.message,
      ]);
      break;
  }
}

// Create new socket when id changes
const socket = createMemo(
  /** @type {EffectFunction<WebSocket | undefined>}*/ (previousSocket) => {
    console.debug("Previous socket", previousSocket);
    if (previousSocket) previousSocket.close();

    //TODO ensure secure connection (WSS/HTTPS) in production
    const socket = new WebSocket(`ws://127.0.0.1:3000/messages/${app.id}`);
    socket.binaryType = "arraybuffer";
    socket.addEventListener("message", receiveMessage);

    socket.addEventListener("close", (event) => {
      console.warn("Socket closed. Not implemented.", event);
    });

    return socket;
  }
);

/**
 * TODO these need to be derivated from the rust types which should be automated
 * @typedef {{type: "Welcome", group_id: string, friend: Friend}} Welcome
 * @typedef {{type: "Private", group_id: string, message: string}} Private
 * @typedef {Welcome | Private} Message
 */

/**
 * @type {Context<[typeof app, typeof setApp]>}
 */
const AppContext = createContext([app, setApp]);

/**
 *
 * @param {ParentProps} properties
 */
export function AppContextProvider(properties) {
  return (
    <AppContext.Provider value={[app, setApp]}>
      {properties.children}
    </AppContext.Provider>
  );
}
/**
 *
 * @returns {[typeof app, typeof setApp]}
 */
export function useAppContext() {
  return useContext(AppContext);
}
