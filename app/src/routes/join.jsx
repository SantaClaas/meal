import { Navigate, useNavigate, useParams } from "@solidjs/router";
import { createResource, Match, Switch } from "solid-js";
import { useAppContext, messagesUrl } from "../components/AppContext";
import { decode_key_package } from "../../../core/pkg";

//@ts-expect-error TS6192 Can not handle new JSDoc syntax (yet?)
// https://devblogs.microsoft.com/typescript/announcing-typescript-5-5/#the-jsdoc-@import-tag
/**@import { JSX, Accessor } from "solid-js" */
//@ts-expect-error TS6192 See above
/** @import { Group } from "../components/AppContext" */

/**
 * @returns {JSX.Element}
 */
export default function Join() {
  const parameters = useParams();

  if (!parameters.package) {
    return <Navigate href="/" />;
  }

  const [app, setApp] = useAppContext();

  const [keyPackage] = createResource(parameters.package, decode_key_package);

  const navigate = useNavigate();
  /** @param {Parameters<JSX.EventHandler<HTMLFormElement, SubmitEvent>>[0]} event */
  async function handleDecision(event) {
    event.preventDefault();

    const name = /** @type {HTMLInputElement} */ (event.currentTarget.username)
      .value;

    // Complete onboarding if used for the first time
    if (name && name !== app.name) {
      setApp("name", name);
    }

    const keys = keyPackage();
    if (keys === undefined) {
      console.error("Expected key package to be defined");
      return;
    }

    const groupId = app.client.create_group();
    /** @type {Group} */
    const group = {
      id: groupId,
      friend: keys.friend,
      messages: [],
    };

    // Add group to store https://docs.solidjs.com/concepts/stores#appending-new-values
    setApp("groups", app.groups.length, group);

    // Need to extract id before key package is consumed or it will error
    const url = new URL(group.friend.id, messagesUrl);

    const welcomePackage = app.client.invite(groupId, keys);
    // Send welcome to peer
    const request = new Request(url, {
      method: "post",
      headers: {
        //https://www.rfc-editor.org/rfc/rfc9420.html#name-the-message-mls-media-type
        "Content-Type": "message/mls",
      },
      body: welcomePackage,
    });

    //TODO error handling
    await fetch(request);

    // Navigate to the chat
    navigate(`/chat/${groupId}`);
  }

  return (
    <>
      <h1>Invitation</h1>

      <Switch>
        <Match when={keyPackage.loading}>
          {/* TODO loading shimmer/skeleton */}
          <p>Loading...</p>
        </Match>

        <Match when={keyPackage()?.friend.name}>
          {
            /** @type {(item: Accessor<NonNullable<string>>) => JSX.Element} */
            (
              (friendName) => (
                <p>{friendName()} has invited you to chat with them</p>
              )
            )
          }
        </Match>
        <Match when={!keyPackage()?.friend.name}>
          {/* TODO add additional information that the person that has sent the invite chose to not include their name
          publicly in the invite. Make it clear that this is only for the invite and the name might be revealed when they accepted your request */}
          <p>You are invited to a chat</p>
        </Match>
        <Match when={keyPackage.error}>
          <p>
            Sorry an error occurred trying to read your invite:
            {keyPackage.error}
          </p>
        </Match>
      </Switch>

      <form onSubmit={handleDecision}>
        <label for="username">Your Name</label>
        <input
          type="text"
          name="username"
          id="username"
          autocomplete="username"
          required
          value={app.name ?? ""}
        />

        <button type="submit" name="accept">
          Accept
        </button>
      </form>
    </>
  );
}
