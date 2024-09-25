import { Navigate, useNavigate, useParams } from "@solidjs/router";
import {
  createEffect,
  createResource,
  createSignal,
  Match,
  Switch,
} from "solid-js";
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

  const [currentState] = useAppContext();

  const [resource] = createResource(paramters.package, decode_key_package);

  /** @type {Signal<ReturnType<typeof setTimeout> | undefined>} */
  const [selfDestructTimerId, setSelfDestructTimerId] = createSignal();

  // Self close after a couple of seconds
  function selfDestruct() {
    const id = setTimeout(window.close, 3_000);
    setSelfDestructTimerId(id);
  }

  function cancelSelfDestruct() {
    clearTimeout(selfDestructTimerId());
    setSelfDestructTimerId(undefined);
  }

  return (
    <Switch>
      <Match when={!selfDestructTimerId()}>
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

        <form>
          <label for="name">Your Name</label>
          <input
            type="text"
            name="name"
            id="name"
            required
            value={currentState()?.name ?? ""}
          />

          <button type="submit">Accept</button>
          <button onClick={selfDestruct}>Decline</button>
        </form>
      </Match>
      <Match when={selfDestructTimerId()}>
        <p>No worries, come back any time</p>
        {/* TODO progress that shrinks to indicate when tab closes using CSS transition */}
        <button onClick={cancelSelfDestruct}>Cancel</button>
      </Match>
    </Switch>
  );
}
