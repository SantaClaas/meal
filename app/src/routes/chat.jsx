import { Navigate, useParams } from "@solidjs/router";
import { createMemo, For, Show } from "solid-js";
import { useAppContext } from "../components/AppContext";
/** @import { JSX } from "solid-js" */

export default function Chat() {
  const paramters = useParams();

  if (paramters.groupId === undefined) {
    return <Navigate href="/" />;
  }

  const [app, setApp] = useAppContext();
  //TODO this will not work when group indices change due to change of ordering and new groups being added
  const groupIndex = createMemo(() =>
    app.groups.findIndex((group) => group.id === paramters.groupId)
  );
  const group = () => app.groups[groupIndex()];

  const messages = () => group().messages;

  /** @param {Parameters<JSX.EventHandler<HTMLFormElement, SubmitEvent>>[0]} event*/
  function handleSend(event) {
    event.preventDefault();
    const message = /** @type {HTMLInputElement} */ (
      event.currentTarget.message
    ).value;

    event.currentTarget.reset();

    console.debug("Send message", message, groupIndex(), messages());
    setApp("groups", groupIndex(), "messages", messages().length, message);
    console.debug("Updated messages", messages(), app.groups[groupIndex()]);
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
