import { Navigate, useNavigate } from "@solidjs/router";
import { useAppContext } from "../components/AppContext";
import { createSignal, Show } from "solid-js";
/**
 * @import { JSX, Signal, Accessor } from "solid-js";
 */

export default function Invite() {
  const { app } = useAppContext();
  const navigate = useNavigate();
  if (app() === undefined) {
    return <Navigate href="/" />;
  }

  function getInvite() {
    const state = app();
    if (state === undefined) {
      console.error("Expected state to be defined");
      navigate("/");
      return;
    }

    const keyPackage = state.generate_key_package();
    return keyPackage;
  }
  /** @type {Signal<string | undefined>} */
  const [welcome, setWelcome] = createSignal();

  /** @type {Signal<string|undefined>} */
  const [groupId, setGroupId] = createSignal();

  /**
   * @param {Parameters<JSX.EventHandler<HTMLFormElement, SubmitEvent>>[0]} event
   */
  function handleKeyInput(event) {
    event.preventDefault();
    const state = app();
    if (state === undefined) {
      console.error("Expected state to be defined");
      navigate("/");
      return;
    }

    /** @type {HTMLInputElement} */
    const input = event.currentTarget.keypackage;
    const value = input.value;
    if (!value) return;

    // Create group to establish chat
    const invite = state.establish_contact(value);
    // Send welcome
    setWelcome(invite.welcome);
    setGroupId(invite.group_id);
  }

  /** @param {Accessor<string|undefined>} get */
  function copy(get) {
    return () => {
      const value = get();
      if (!(value && typeof value === "string")) return;

      navigator.clipboard.writeText(value);
    };
  }

  /**
   * @param {Parameters<JSX.EventHandler<HTMLFormElement, SubmitEvent>>[0]} event
   */
  function handleWelcomeIn(event) {
    event.preventDefault();
    const state = app();
    if (state === undefined) {
      console.error("Expected state to be defined");
      navigate("/");
      return;
    }

    const welcomeIn = /** @type {HTMLInputElement}*/ (
      event.currentTarget.welcome
    ).value;

    if (!welcomeIn) return;

    const groupId = state.join_group(welcomeIn);
    console.debug("Joined group", groupId);
    setGroupId(groupId);
  }

  /** @type {Signal<string|undefined>} */
  const [messageOut, setMessageOut] = createSignal();
  /**
   * @param {Parameters<JSX.EventHandler<HTMLFormElement, SubmitEvent>>[0]} event
   */
  function handleSendMessage(event) {
    event.preventDefault();

    const state = app();
    if (state === undefined) {
      console.error("Expected state to be defined");
      navigate("/");
      return;
    }

    const groupId = /** @type {HTMLInputElement}*/ (event.currentTarget.groupid)
      .value;

    if (!groupId) return;

    const message = /** @type {HTMLInputElement}*/ (event.currentTarget.message)
      .value;

    if (!message) return;

    const encodedMessage = state.send_message(groupId, message);
    setMessageOut(encodedMessage);
  }

  /** @type {Signal<string|undefined>} */
  const [messageIn, setMessageIn] = createSignal();
  /**
   * @param {Parameters<JSX.EventHandler<HTMLFormElement, SubmitEvent>>[0]} event
   */
  function handleReceiveMessage(event) {
    event.preventDefault();

    const state = app();
    if (state === undefined) {
      console.error("Expected state to be defined");
      navigate("/");
      return;
    }

    const groupId = /** @type {HTMLInputElement}*/ (event.currentTarget.groupid)
      .value;

    console.debug("Group id", groupId);
    if (!groupId) return;

    const message = /** @type {HTMLInputElement}*/ (event.currentTarget.message)
      .value;

    if (!message) return;

    const plainTextMessage = state.receive_message(groupId, message);
    setMessageIn(plainTextMessage);
  }

  return (
    <>
      <h1>Invite to chat</h1>
      <button onClick={copy(getInvite)}>Copy Invite</button>
      <form onSubmit={handleKeyInput}>
        <fieldset>
          <legend>Key Package in</legend>
          <label for="keypackage">Key Package</label>
          <input type="text" name="keypackage" id="keypackage" required />
        </fieldset>
        <button>Submit</button>
      </form>
      <form onSubmit={handleWelcomeIn}>
        <fieldset>
          <legend>Welcome in</legend>
          <label for="welcome">Welcome</label>
          <input type="text" name="welcome" id="welcome" required />
        </fieldset>
        <button>Submit</button>
      </form>
      <Show when={welcome()}>
        <h2>Welcome to send to partner</h2>
        <pre>{welcome()}</pre>
        <button onClick={copy(welcome)}>Copy</button>
      </Show>
      <Show when={groupId()}>
        <form onSubmit={handleSendMessage}>
          <fieldset>
            <input type="hidden" name="groupid" value={groupId()} />
            <legend>Send Message</legend>
            <label for="message">Message</label>
            <input type="text" id="message" name="message" required />
          </fieldset>
          <button type="submit">Send</button>
        </form>

        <form onSubmit={handleReceiveMessage}>
          <fieldset>
            <input type="hidden" name="groupid" value={groupId()} />
            <legend>Receive Message</legend>
            <label for="message">Message</label>
            <input type="text" id="message" name="message" required />
          </fieldset>
          <button type="submit">Receive</button>
        </form>
      </Show>
      <Show when={messageOut()}>
        <h2>Message out</h2>
        <pre>{messageOut()}</pre>

        <button onClick={copy(messageOut)}>Copy</button>
      </Show>
      <Show when={messageIn()}>
        <h2>Received Message!</h2>
        <p>{messageIn()}</p>
      </Show>
    </>
  );
}