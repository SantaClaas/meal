import { Navigate, useNavigate, useParams } from "@solidjs/router";
import { createResource, Match, Switch } from "solid-js";
import { useAppContext } from "../components/AppContext";
import { decode_key_package } from "../../../core/pkg/meal";

/** @import { JSX, Signal, Accessor } from "solid-js" */

/**
 * @returns {JSX.Element}
 */
export default function Join() {
  const paramters = useParams();

  if (!paramters.package) {
    return <Navigate href="/" />;
  }

  const [app, setApp] = useAppContext();

  const [keyPackage] = createResource(paramters.package, decode_key_package);

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
    // Need to extract id before key package is consumed or it will error
    const url = new URL(`http://127.0.0.1:3000/messages/${keys.client_id}`);

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
          {/* TODO loading shimmer/sceleton */}
          <p>Loading...</p>
        </Match>

        <Match when={keyPackage()?.friend_name}>
          {
            /** @type {(item: Accessor<NonNullable<string>>) => JSX.Element} */
            (
              (friendName) => (
                <p>{friendName()} has invited you to chat with them</p>
              )
            )
          }
        </Match>
        <Match when={!keyPackage()?.friend_name}>
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
