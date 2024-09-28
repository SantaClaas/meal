import { Navigate, useParams } from "@solidjs/router";
import { createMemo, For, Show } from "solid-js";
import { useAppContext, messagesUrl } from "../components/AppContext";
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

  if (groupIndex() < 0) {
    return (
      <>
        <h1>Chat not found</h1>
        <p>
          This chat does not seem to exist <a href="/">Go back</a>
        </p>
      </>
    );
  }

  const group = () => app.groups[groupIndex()];

  const messages = () => group().messages;

  /** @param {Parameters<JSX.EventHandler<HTMLFormElement, SubmitEvent>>[0]} event*/
  async function handleSend(event) {
    event.preventDefault();
    const message = /** @type {HTMLInputElement} */ (
      event.currentTarget.message
    ).value;

    event.currentTarget.reset();

    setApp("groups", groupIndex(), "messages", messages().length, message);
    //TODO Should use SolidJS signal system to make sending messages an effect of adding messages to the chat
    const body = app.client.send_message(group().id, message);

    const url = new URL(group().friend.id, messagesUrl);
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

  return (
    <>
      <h1>Chat</h1>
      <table>
        <tbody>
          <tr>
            <td>Group id</td>
            <td>{paramters.groupId}</td>
          </tr>
          <tr>
            <td>Friend client id</td>
            <td>{group().friend.id}</td>
          </tr>
          <tr>
            <td>Friend name</td>
            <td>{group().friend.name}</td>
          </tr>
          <tr>
            <td>My client id</td>
            <td>{app.id}</td>
          </tr>
          <tr>
            <td>My name</td>
            <td>{app.name}</td>
          </tr>
        </tbody>
      </table>
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
