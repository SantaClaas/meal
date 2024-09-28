import { useNavigate } from "@solidjs/router";
import { useAppContext } from "../components/AppContext";
import { createEffect, Show } from "solid-js";
//@ts-expect-error TS6192 Can not handle new JSDoc syntax (yet?)
// https://devblogs.microsoft.com/typescript/announcing-typescript-5-5/#the-jsdoc-@import-tag
/** @import { JSX, Signal, Accessor, EffectFunction } from "solid-js" */

export default function Invite() {
  const navigate = useNavigate();

  // Navigate to new group chat when invite is accepted (when it is added from processing incoming welcome)
  createEffect(
    // Using the groups (instead of length) as previous does not work because the reference does not change
    /** @type {EffectFunction<number | undefined>}*/ (previousLength) => {
      //TODO check if user has automatic accept enabled
      //TODO check if this welcome is for this key package
      if (previousLength === undefined) return app.groups.length;

      // Just take the last one added for now 🥴
      // I need to think more about this and then rework it
      if (previousLength >= app.groups.length) return app.groups.length;
      // Just assume the last one is the last one added 🥴
      const group = app.groups.at(-1);
      if (group !== undefined) navigate(`/chat/${group.id}`);

      return app.groups.length;
    }
  );

  const [app] = useAppContext();

  /** @param {string|undefined} name */
  function createInviteUrl(name) {
    const encodedInvite = app.client.create_invite(name);
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

    /** @type {string | undefined} */
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
              value={app.name ?? ""}
            />
          </fieldset>
          {/* TODO wire up */}
          {/* This setting is to reduce the steps needed to establish a new chat but might be uncomfortable for some users */}
          <input type="checkbox" name="accept" id="accept" checked disabled />
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
