import {
  createContext,
  createEffect,
  createSignal,
  useContext,
} from "solid-js";
//TODO register rust wasm pack package as package in workspace
import { AppState } from "../../../core/pkg";

/**
 * @import { ParentProps, Context, Signal, Accessor } from "solid-js";
 * @typedef {{name: string, app: AppState, groups: Accessor<string[]>}} State
 */

const [groups, setGroups] = createSignal(/** @type {string[]} */ ([]));

/** @type {Signal<State | undefined>} */
const [currentState, setState] = createSignal();

/**
 * TODO these need to be derivated from the rust types which should be automated
 * @typedef { {Welcome: { group_id: string}} | {Private: {group_id: string, message: string}} } Message
 */

/**
 *
 * @param {string} to
 * @param {string} groupId
 * @param {string} message
 */
async function sendMessage(to, groupId, message) {
  const state = currentState();
  if (state === undefined) {
    console.error("Need statet to be defined to send message");
    return;
  }

  const body = state.app.send_message(groupId, message);
  //TODO error handling
  //TODO retry
  const response = await fetch(`http://127.0.0.1:3000/messages/${to}`, {
    headers: {
      //https://www.rfc-editor.org/rfc/rfc9420.html#name-the-message-mls-media-type
      "Content-Type": "message/mls",
    },
    method: "post",
    body,
  });

  console.debug("Send message response", response);
}

/**
 * @param {string} name
 * @returns {NonNullable<ReturnType<typeof setState>>}
 */
function initialize(name) {
  const app = new AppState(name);

  return setState({
    groups,
    name,
    app,
  });
}

const accessors = { initialize, sendMessage };
/** @type {[state: typeof currentState, typeof accessors]} */
const state = [currentState, accessors];
/**
 * @type {Context<typeof state>}
 */
const AppContext = createContext(state);

/**
 *
 * @param {ParentProps} properties
 */
export function AppContextProvider(properties) {
  return (
    <AppContext.Provider value={state}>
      {properties.children}
    </AppContext.Provider>
  );
}
/**
 *
 * @param {(groupId: string) => void} [onWelcomeIn]
 * @returns {typeof state}
 */
export function useAppContext(onWelcomeIn) {
  const [currentState, ...accessors] = useContext(AppContext);
  /**
   * @param {MessageEvent} event
   */
  function receiveMessage(event) {
    console.debug("Received message", event.data, currentState());

    if (!(event.data instanceof ArrayBuffer)) {
      console.error("Expected websocket data to be binary");
      return;
    }

    const state = currentState();
    if (state === undefined) return;

    console.debug("Processing message");
    const message = /** @type {Message} */ (
      state.app.process_message(new Uint8Array(event.data))
    );
    console.debug("Processed message", message);
    if ("Welcome" in message) {
      setGroups((groups) => [...groups, message.Welcome.group_id]);
      onWelcomeIn?.(message.Welcome.group_id);
      return;
    }

    if ("Private" in message) {
      console.debug(
        "Received message",
        message.Private.group_id,
        message.Private.message
      );
    }
  }

  //TODO close socket when app state changes
  createEffect(() => {
    const state = currentState();
    if (state === undefined) return;

    //TODO ensure secure connection (WSS/HTTPS) in production
    const socket = new WebSocket(`ws://127.0.0.1:3000/messages/${state.name}`);
    socket.binaryType = "arraybuffer";
    socket.addEventListener("message", receiveMessage);

    socket.addEventListener("close", (event) => {
      console.warn("Socket closed. Not implemented.", event);
    });
  });

  return [currentState, ...accessors];
}
