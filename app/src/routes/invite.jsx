import { Navigate, useNavigate } from "@solidjs/router";
import { useAppContext } from "../components/AppContext";
import { createSignal, Show } from "solid-js";
/**
 * @import { JSX, Signal, Accessor } from "solid-js";
 */

export default function Invite() {
  const [currentState, { sendMessage }] = useAppContext();
  const navigate = useNavigate();
  if (currentState() === undefined) {
    return <Navigate href="/" />;
  }

  function getInviteUrl() {
    const state = currentState();
    if (state === undefined) {
      console.error("Expected state to be defined");
      navigate("/");
      return;
    }

    const encodedPackage =  state.app.create_key_package();
    return new URL(`/join/${encodedPackage}`, location.origin);
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
    const state = currentState();
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
    const invite = state.app.establish_contact(value);
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
    const state = currentState();
    if (state === undefined) {
      console.error("Expected state to be defined");
      navigate("/");
      return;
    }

    const welcomeIn = /** @type {HTMLInputElement}*/ (
      event.currentTarget.welcome
    ).value;

    if (!welcomeIn) return;

    const groupId = state.app.join_group(welcomeIn);
    console.debug("Joined group", groupId);
    setGroupId(groupId);
  }

  /**
   * @param {Parameters<JSX.EventHandler<HTMLFormElement, SubmitEvent>>[0]} event
   */
  async function handleSendMessage(event) {
    event.preventDefault();

    const state = currentState();
    if (state === undefined) {
      console.error("Expected state to be defined");
      navigate("/");
      return;
    }

    const groupId = /** @type {HTMLInputElement}*/ (event.currentTarget.groupid)
      .value;

    if (!groupId) return;

    const to = /** @type {HTMLInputElement}*/ (event.currentTarget.to)
          .value;

    if (!to) return;

    const message = /** @type {HTMLInputElement}*/ (event.currentTarget.message)
      .value;

    if (!message) return;

    await sendMessage(to, groupId, message)
  }

  async function shareInvite(){
    const state = currentState();
    if (state === undefined) return;

    const inviteUrl = getInviteUrl();
    if (!inviteUrl) return;

    /** @type {ShareData} */
    const data = {
      title: "Join me on melt",
      //TODO generate QR code
      text: "\nScan the QR code or follow the link to chat with me on melt",
      url: inviteUrl.href,
    };
    if (!navigator.canShare(data)){
      console.debug("Can not share", data);
      return;
    }

   await navigator.share(data);
  }
  return (
    <>
      <h1>Invite to chat</h1>
      <button onClick={copy(() => getInviteUrl()?.href)}>Copy Invite</button>
      <button onClick={shareInvite}>Share Invite</button>
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
            <legend>Send Message</legend>

            <input type="hidden" name="groupid" value={groupId()} />
              <label for="to">To</label>
            <input type="text" id="to" name="to" required />
            <label for="message">Message</label>
            <input type="text" id="message" name="message" required />
          </fieldset>
          <button type="submit">Send</button>
        </form>
      </Show>
    </>
  );
}
