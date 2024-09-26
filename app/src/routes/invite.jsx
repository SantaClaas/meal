import { Navigate, useNavigate } from "@solidjs/router";
import { useAppContext } from "../components/AppContext";
import { createEffect, createSignal, Show } from "solid-js";
/**
 * @import { JSX, Signal, Accessor } from "solid-js";
 */

export default function Invite() {
  const navigate = useNavigate();

  /**
   * Go to chat when invite is accepted (welcome for the keypackage comes in)
   * @param {string} groupId
   */
  function handleWelcomeIn(groupId) {
    //TODO check if user has automatic accept enabled
    //TODO check if this welcome is for this key package
    navigate(`/chat/${groupId}`);
  }

  const [currentState] = useAppContext(handleWelcomeIn);
  if (currentState() === undefined) {
    return <Navigate href="/" />;
  }

  /** @param {string|undefined} name */
  function createInviteUrl(name) {
    const state = currentState();
    if (state === undefined) {
      console.error("Expected state to be defined");
      navigate("/");
      return;
    }

    const encodedInvite = state.app.create_invite(name);
    return new URL(`/join/${encodedInvite}`, location.origin);
  }

  /**
   * @param {string} [url]
   * @returns {ShareData}
   */
  const shareTemplate = (url) => ({
    title: "Join me on melt",
    //TODO generate QR code
    text: "\nScan the QR code or follow the link to chat with me on melt",
    url,
  });

  // Progressively enhance the invite form
  const isShareEnabled =
    "share" in navigator &&
    "canShare" in navigator &&
    navigator.canShare(shareTemplate());

  /** @param {Parameters<JSX.EventHandler<HTMLFormElement, SubmitEvent>>[0]} event*/
  async function handleShare(event) {
    event.preventDefault();

    const state = currentState();
    if (state === undefined) {
      console.error("Expected state to be defined");
      navigate("/");
      return;
    }

    // This should be a switch expression
    /** @type {"share"|"copy"} */
    let shareMedium;
    switch (event.submitter) {
      case event.currentTarget.share:
        shareMedium = "share";
        break;
      case event.currentTarget.copy:
        shareMedium = "copy";
        break;
      default:
        console.error("Unexpected submitter", event.submitter);
        return;
    }

    const isNameIncluded = /** @type {HTMLInputElement} */ (
      event.currentTarget.includename
    ).checked;

    let name = undefined;
    // const shareMedium = "share" in event.currentTarget ;
    if (isNameIncluded) {
      // Types get confused when the input name is "name"
      name = /** @type {HTMLInputElement} */ (event.currentTarget.displayname)
        .value;
    }

    const inviteUrl = createInviteUrl(name);
    if (!inviteUrl) return;

    if (shareMedium === "share" && isShareEnabled) {
      //TODO error handling as we expect at this point that share works
      await navigator.share(shareTemplate(inviteUrl.href));
      return;
    }

    // Fall back to copy

    //TODO show toast to show user that a copy was made
    navigator.clipboard.writeText(inviteUrl.href);
  }

  return (
    <>
      <h1>Invite to chat</h1>
      <form onSubmit={handleShare}>
        <details>
          <summary>Advanced</summary>
          <fieldset>
            <legend>Name</legend>
            <input
              type="checkbox"
              id="includename"
              name="includename"
              checked
              required
            />
            <label for="includename">Include in invite</label>
            {/* TODO disable in css when show name is not checked */}
            <label for="displayname">Display name</label>
            <input
              type="text"
              name="displayname"
              id="displayname"
              required
              value={currentState()?.name}
            />
          </fieldset>
          {/* TODO wire up */}
          {/* This setting is to reduce the steps needed to establish a new chat but might be uncomfortable for some users */}
          <input type="checkbox" name="accept" id="accept" />
          <label for="accept">
            Automatically accept requests from this invite
          </label>
        </details>
        {/* <a href="whatsapp://send?text=Join me on melt">Whatsapp</a> */}
        <button type="submit" name="copy">
          Copy
        </button>
        <Show when={isShareEnabled}>
          <button type="submit" name="share">
            Share
          </button>
        </Show>
      </form>
    </>
  );
}
