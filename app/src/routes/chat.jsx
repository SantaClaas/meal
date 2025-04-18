import { Navigate, useParams } from "@solidjs/router";
import { createMemo, For, Show } from "solid-js";
import { useAppContext } from "../components/AppContext";
import { postMessage } from "../sendMessage";
//@ts-expect-error TS6192 Can not handle new JSDoc syntax (yet?)
/** @import { JSX } from "solid-js" */
//@ts-expect-error TS6192 Can not handle new JSDoc syntax (yet?)
/** @import { Message } from "../components/AppContext" */

// TODO take this inspiration https://firebasestorage.googleapis.com/v0/b/design-spec/o/projects%2Fgoogle-material-3%2Fimages%2Fly7219l1-1.png?alt=media&token=67ff316b-7515-4e9f-9971-4e580290b1f2
// from https://m3.material.io/foundations/layout/applying-layout/compact#283b4432-e3ee-46df-aa66-9ec87965c6ef
export default function Chat() {
  const parameters = useParams();

  if (parameters.groupId === undefined) {
    return <Navigate href="/" />;
  }

  const [app, setApp] = useAppContext();
  //TODO this will not work when group indices change due to change of ordering and new groups being added
  const groupIndex = createMemo(() =>
    app.groups.findIndex((group) => group.id === parameters.groupId)
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
    const messageText = /** @type {HTMLInputElement} */ (
      event.currentTarget.message
    ).value;

    event.currentTarget.reset();

    /** @type {Message} */
    const message = {
      sent: new Date(),
      text: messageText,
    };

    setApp("groups", groupIndex(), "messages", messages().length, message);
    console.debug("Posting message", message);
    await postMessage({
      type: "sendMessage",
      groupId: group().id,
      friendId: group().friend.id,
      sent: new Date(),
      text: messageText,
    });
  }

  return (
    <>
      <a href="/">Back</a>
      <h1>Chat</h1>
      <table>
        <tbody>
          <tr>
            <td>Group id</td>
            <td>{parameters.groupId}</td>
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
                <p>{message.text}</p>
                {/*TODO format date correctly for datetime attribute*/}
                <time>{message.sent.toLocaleTimeString()}</time>
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
