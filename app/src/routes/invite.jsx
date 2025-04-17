import { useNavigate } from "@solidjs/router";
import { useAppContext } from "../components/AppContext";
import { createEffect, Show } from "solid-js";
//TODO replace with SVG QR-Code solution. Maybe with something custom
import QRCode from "qrcode";
/** @import { JSX, EffectFunction, VoidProps } from "solid-js" */

const FormElement = /** @type {const} */ ({
  IncludeName: "includeName",
  DisplayName: "displayName",
  AutomaticAccept: "automaticAccept",
  CopyButton: "copyButton",
  ShareButton: "shareButton",
});

/**
 *
 * @param {VoidProps<{userName: string, ref: HTMLDialogElement | undefined}>} properties
 * @returns
 */
function InviteSettingsDialog({ userName, ref }) {
  /** @param {Parameters<JSX.EventHandler<HTMLFormElement, SubmitEvent>>[0]} event*/
  function saveSettings(event) {
    event.preventDefault();
  }
  return (
    <dialog ref={ref}>
      <form onSubmit={saveSettings}>
        <details>
          <summary>Advanced</summary>
          <fieldset>
            <legend>Name</legend>
            <input
              type="checkbox"
              id={FormElement.IncludeName}
              name={FormElement.IncludeName}
              checked
              required
            />
            <label for={FormElement.IncludeName}>Include in invite</label>
            {/* TODO disable in css when show name is not checked */}
            <label for={FormElement.DisplayName}>Display name</label>
            <input
              type="text"
              name={FormElement.DisplayName}
              id={FormElement.DisplayName}
              required
              value={userName}
            />
          </fieldset>
          {/* TODO wire up */}
          {/* This setting is to reduce the steps needed to establish a new chat but might be uncomfortable for some users */}
          <input
            type="checkbox"
            name={FormElement.AutomaticAccept}
            id={FormElement.AutomaticAccept}
            checked
            disabled
          />
          <label for={FormElement.AutomaticAccept}>
            Automatically accept requests from this invite
          </label>
        </details>
        {/* <a href="whatsapp://send?text=Join me on melt">Whatsapp</a> */}
      </form>
    </dialog>
  );
}

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

  const [app, setApp] = useAppContext();
  /** @type {HTMLDialogElement | undefined} */
  let settingsDialog;
  /** @type { HTMLDivElement | undefined} */
  let snackbar;

  /** @param {string|undefined} name */
  function createInviteUrl(name) {
    const encodedInvite = app.client.create_invite(name);
    return new URL(`/join/${encodedInvite}`, location.origin);
  }

  const inviteUrl = () => createInviteUrl(app.name ?? undefined);

  /** @type {File | undefined} */
  let qrCodeFile;

  /**
   * @param {string} [url]
   * @returns {ShareData}
   */
  const shareTemplate = (url) => ({
    title: "Join me on melt",
    //TODO generate QR code
    files: qrCodeFile ? [qrCodeFile] : [],
    text: "\nScan the QR code or follow the link to chat with me on melt",
    url,
  });

  // Progressively enhance the invite form
  const isShareEnabled =
    "share" in navigator &&
    "canShare" in navigator &&
    navigator.canShare(shareTemplate());

  async function shareInvite() {
    if (!isShareEnabled)
      throw new Error(
        "Sharing is not enabled. Expected this function to not be callable"
      );

    await navigator.share(shareTemplate(inviteUrl().href));
  }

  /**
   * Has to be used with click event handler as pointer down opens the popover and pointer up closes it immediately
   */
  function copyInvite() {
    navigator.clipboard.writeText(inviteUrl().href);
    snackbar?.showPopover();

    // "4-10 seconds based on platform" https://m3.material.io/components/snackbar/guidelines#12145fa5-ada2-4c3b-b2ae-9cdf8ee54ca1
    const HIDE_DELAY = 4_000;
    setTimeout(() => snackbar?.hidePopover(), HIDE_DELAY);
  }

  /** @type {HTMLCanvasElement | undefined} */
  let canvas;

  createEffect(async () => {
    if (canvas === undefined) return;

    await QRCode.toCanvas(canvas, inviteUrl().href);

    const FILE_TYPE = "image/png";
    canvas.toBlob((blob) => {
      if (blob === null) return;
      return (qrCodeFile = new File([blob], "invite-qr-code.png", {
        type: FILE_TYPE,
      }));
    }, FILE_TYPE);
  });
  return (
    <>
      <a href="/">Back</a>
      <h1>Invite to chat</h1>
      <button onPointerDown={() => settingsDialog?.showModal()}>
        <span class="sr-only">Open settings</span>
        <svg
          xmlns="http://www.w3.org/2000/svg"
          height="24px"
          viewBox="0 -960 960 960"
          width="24px"
          fill="currentColor"
        >
          <path d="m370-80-16-128q-13-5-24.5-12T307-235l-119 50L78-375l103-78q-1-7-1-13.5v-27q0-6.5 1-13.5L78-585l110-190 119 50q11-8 23-15t24-12l16-128h220l16 128q13 5 24.5 12t22.5 15l119-50 110 190-103 78q1 7 1 13.5v27q0 6.5-2 13.5l103 78-110 190-118-50q-11 8-23 15t-24 12L590-80H370Zm70-80h79l14-106q31-8 57.5-23.5T639-327l99 41 39-68-86-65q5-14 7-29.5t2-31.5q0-16-2-31.5t-7-29.5l86-65-39-68-99 42q-22-23-48.5-38.5T533-694l-13-106h-79l-14 106q-31 8-57.5 23.5T321-633l-99-41-39 68 86 64q-5 15-7 30t-2 32q0 16 2 31t7 30l-86 65 39 68 99-42q22 23 48.5 38.5T427-266l13 106Zm42-180q58 0 99-41t41-99q0-58-41-99t-99-41q-59 0-99.5 41T342-480q0 58 40.5 99t99.5 41Zm-2-140Z" />
        </svg>
      </button>

      <canvas width={200} height={200} ref={canvas} />

      {/* Needs to be on click otherwise the pointer down opens it and the pointer up closes it immediately */}
      <button onPointerDown={copyInvite}>Copy</button>
      <Show when={isShareEnabled}>
        <button onPointerDown={shareInvite}>Share</button>
      </Show>
      <div popover="manual" ref={snackbar}>
        Link copied
      </div>
      <InviteSettingsDialog userName={app.name ?? ""} ref={settingsDialog} />
    </>
  );
}
