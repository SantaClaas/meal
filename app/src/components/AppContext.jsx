import { createContext, createEffect, useContext } from "solid-js";
//TODO register rust wasm pack package as package in workspace
import { Client } from "../../../core/pkg";
import { createStore } from "solid-js/store";

/**
 * @import { ParentProps, Context, EffectFunction } from "solid-js"
 * @import { Friend } from "../../../core/pkg"
 * @typedef {{sent: Date, text: string}} Message
 * @typedef {{id: string, friend: Friend, messages: Message[]}} Group
 * TODO these need to be derived from the rust types which should be automated
 * @typedef {{type: "Welcome", group_id: string, friend: Friend}} Welcome
 * @typedef {{sent: string, text: string}} MessageContent
 * @typedef {{type: "Private", group_id: string, content: MessageContent}} Private
 * @typedef {Welcome | Private} ApplicationMessage
 */

const id = localStorage.getItem("id");
const name = localStorage.getItem("name");
const isOnboarded = localStorage.getItem("isOnboarded") !== null;
const client = new Client(id ?? undefined, name ?? undefined);

// const isLocalhost =
//   window.location.hostname === "localhost" ||
//   window.location.hostname === "127.0.0.1";
// const isSecureRequired = window.isSecureContext && !isLocalhost;
/** The url for messages endpoint. Don't forget the trailing slash. */
export const messagesUrl = new URL("/messages/", window.location.origin);

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
  const message = /** @type {ApplicationMessage} */ (
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
      /** @type {Message} */
      const messageContent = {
        sent: new Date(message.content.sent),
        text: message.content.text,
      };
      console.debug("Received message", message.group_id, messageContent);

      // Get group
      //TODO sort groups by last message time
      // Using linear search because:
      // 1. Groups that often receive messages are displayed at the top of the list which reduces search time
      // 2. An average user has few active groups that regularly receive messages

      // Assuming groups that often receive messages are regularly used
      // this means they are at the top of the list.

      const groupIndex = app.groups.findIndex(
        (group) => group.id === message.group_id
      );

      //TODO avoid new array creation
      setApp("groups", groupIndex, "messages", (messages) => [
        ...messages,
        messageContent,
      ]);
      break;
  }
}

// Create new socket when id changes
createEffect(
  /** @type {EffectFunction<WebSocket | undefined>}*/ (previousSocket) => {
    console.debug("Previous socket", previousSocket);
    if (previousSocket) previousSocket.close();

    // https:// is automatically replaced with wss://
    const socketUrl = new URL(app.id, messagesUrl);
    const socket = new WebSocket(socketUrl);
    socket.binaryType = "arraybuffer";
    socket.addEventListener("message", receiveMessage);

    socket.addEventListener("close", (event) => {
      console.warn("Socket closed. Not implemented.", event);
    });

    return socket;
  }
);

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
