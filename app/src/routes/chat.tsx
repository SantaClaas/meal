import { Navigate, useParams } from "@solidjs/router";
import { For, JSX, Show, Suspense } from "solid-js";
import { setupCrackle } from "../useCrackle";
import { useApp } from "../components/AppContextProvider";

// TODO take this inspiration https://firebasestorage.googleapis.com/v0/b/design-spec/o/projects%2Fgoogle-material-3%2Fimages%2Fly7219l1-1.png?alt=media&token=67ff316b-7515-4e9f-9971-4e580290b1f2
// from https://m3.material.io/foundations/layout/applying-layout/compact#283b4432-e3ee-46df-aa66-9ec87965c6ef
export default function Chat() {
  const parameters = useParams();

  if (parameters.groupId === undefined) {
    return <Navigate href="/" />;
  }

  const groupId: string = parameters.groupId;

  const app = useApp();

  const group = () => {
    if (app.status !== "ready") return;
    return app.getGroup(groupId);
  };

  async function handleSend(
    event: Parameters<JSX.EventHandler<HTMLFormElement, SubmitEvent>>[0]
  ) {
    event.preventDefault();
    const text = (
      event.currentTarget.elements.namedItem("message") as HTMLInputElement
    ).value;
    event.currentTarget.reset();

    const currentGroup = group();
    if (currentGroup === undefined) throw new Error("Expected group");

    const handle = await setupCrackle;

    await handle.sendMessage({
      friendId: currentGroup.friend.id,
      groupId,
      sentAt: new Date(),
      text,
    });
  }

  return (
    <>
      <Suspense fallback={<p>Loading chat</p>}>
        <Show
          when={group()}
          fallback={
            <>
              <h1>Chat not found</h1>
              <p>
                This chat does not seem to exist <a href="/">Go back</a>
              </p>
            </>
          }
        >
          {(group) => (
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
                    <td>My name</td>
                    <td>{group().user?.name}</td>
                  </tr>
                </tbody>
              </table>
              <h2>Messages</h2>
              <Show
                when={group().messages.length > 0}
                fallback={<p>No messages</p>}
              >
                <ol>
                  <For each={group().messages}>
                    {(message) => (
                      <li>
                        <p>{message.text}</p>
                        {/*TODO format date correctly for datetime attribute*/}
                        <time datetime={message.sentAt.toISOString()}>
                          {message.sentAt.toLocaleTimeString()}
                        </time>
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
          )}
        </Show>
      </Suspense>
    </>
  );
}
