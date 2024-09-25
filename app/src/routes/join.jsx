import { Navigate, useParams } from "@solidjs/router";
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

  const [currentState, { initialize }] = useAppContext();

  const [resource] = createResource(paramters.package, decode_key_package);

  /** @param {Parameters<JSX.EventHandler<HTMLFormElement, SubmitEvent>>[0]} event */
  function handleDecision(event) {
    event.preventDefault();

    const name = /** @type {HTMLInputElement} */ (event.currentTarget.username)
      .value;

    // Complete onboarding if used for the first time
    let state = currentState();
    if (state === undefined) {
      state = initialize(name);
    }

    state.app;
  }

  return (
    <>
      <h1>Invitation</h1>

      <Switch>
        <Match when={resource.loading}>
          {/* TODO loading shimmer/sceleton */}
          <p>Loading...</p>
        </Match>

        <Match when={resource()?.friend_name}>
          {
            /** @type {(item: Accessor<NonNullable<string>>) => JSX.Element} */
            (
              (friendName) => (
                <p>{friendName()} has invited you to chat with them</p>
              )
            )
          }
        </Match>
        <Match when={!resource()?.friend_name}>
          {/* TODO add additional information that the person that has sent the invite chose to not include their name
          publicly in the invite. Make it clear that this is only for the invite and the name might be revealed when they accepted your request */}
          <p>You are invited to a chat</p>
        </Match>
        <Match when={resource.error}>
          <p>
            Sorry an error occurred trying to read your invite:
            {resource.error}
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
          value={currentState()?.name ?? ""}
        />

        <button type="submit" name="accept">
          Accept
        </button>
      </form>
    </>
  );
}
