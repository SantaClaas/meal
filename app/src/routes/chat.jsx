import { useParams } from "@solidjs/router";
import { For, Show } from "solid-js";
/** @import { JSX } from "solid-js" */

export default function Chat() {
  const paramters = useParams();

  const messages = () => [];

  /** @param {Parameters<JSX.EventHandler<HTMLFormElement, SubmitEvent>>[0]} event*/
  function handleSend(event) {
    event.preventDefault();
    const message = /** @type {HTMLInputElement} */ (
      event.currentTarget.message
    ).value;

    console.debug("Send message", message);
  }

  return (
    <>
      <hgroup>
        <h1>Chat</h1>
        <p>Group ID: {paramters.groupId}</p>
      </hgroup>
      <h2>Messages</h2>
      <Show when={messages().length > 0} fallback={<p>No messages</p>}>
        <ol>
          <For each={messages()}>
            {(message) => (
              <li>
                <p>{message}</p>
              </li>
            )}
          </For>
        </ol>
      </Show>
      <form onSubmit={handleSend}>
        <fieldset>
          <legend>New message</legend>
          <label for="message">Message</label>
          <input type="text" name="message" id="message" />
          <button type="submit">Send</button>
        </fieldset>
      </form>
    </>
  );
}
