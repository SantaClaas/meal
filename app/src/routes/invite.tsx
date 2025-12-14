import { useNavigate } from "@solidjs/router";
import {
  createEffect,
  createResource,
  createSignal,
  JSX,
  onCleanup,
  Show,
  VoidProps,
} from "solid-js";
//TODO replace with SVG QR-Code solution. Maybe with something custom
import QRCode from "qrcode";
import { setupCrackle } from "../useCrackle";
import { useBroadcast } from "../broadcast";
import { ROUTES } from ".";
import { getConfiguration } from "../database";
/** @import { JSX, VoidProps, Signal } from "solid-js" */

const FormElement = /** @type {const} */ {
  IncludeName: "includeName",
  DisplayName: "displayName",
  AutomaticAccept: "automaticAccept",
  CopyButton: "copyButton",
  ShareButton: "shareButton",
};

function InviteSettingsDialog({
  userName,
  ref,
}: VoidProps<{
  userName: string | undefined;
  ref: HTMLDialogElement | undefined;
}>) {
  /** @param {Parameters<JSX.EventHandler<HTMLFormElement, SubmitEvent>>[0]} event*/
  function saveSettings(
    event: Parameters<JSX.EventHandler<HTMLFormElement, SubmitEvent>>[0]
  ) {
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

async function createInviteUrl() {
  const crackle = await setupCrackle;
  const invite = await crackle.createInvite();

  return invite;
}

async function createInviteQrCode(createInvite: Promise<string>) {
  const url = await createInvite;
  const canvas = new OffscreenCanvas(512, 512);
  QRCode.toCanvas(canvas, url);

  const FILE_TYPE = "image/png";
  const blob = await canvas.convertToBlob({ type: FILE_TYPE });

  return new File([blob], "invite-qr-code.png", { type: FILE_TYPE });
}

async function getShareData(
  createInviteUrl: Promise<string>,
  createQrCode: Promise<File>
): Promise<
  | { isEnabled: true; shareData: ShareData }
  | { isEnabled: false; shareDate?: never }
> {
  if (!("share" in navigator) || !("canShare" in navigator))
    return { isEnabled: false };

  const [inviteUrl, qrCode] = await Promise.allSettled([
    createInviteUrl,
    createQrCode,
  ]);
  if (inviteUrl.status === "rejected") throw inviteUrl.reason;

  const shareData = {
    title: "Join me on melt",
    files: qrCode.status === "fulfilled" ? [qrCode.value] : [],
    text: "\nScan the QR code or follow the link to chat with me on melt",
    url: inviteUrl.value,
  };

  if (navigator.canShare(shareData)) return { isEnabled: true, shareData };
  return { isEnabled: false };
}

function useDataUrl(file: File) {
  const dataUrl = URL.createObjectURL(file);

  onCleanup(() => URL.revokeObjectURL(dataUrl));
  return dataUrl;
}

export default function Invite() {
  const navigate = useNavigate();

  // Navigate to new group chat when invite is accepted (when it is added from processing incoming welcome)
  useBroadcast((event) => {
    if (event.data.type !== "Group created") return;

    //TODO check if user has automatic accept enabled
    //TODO check if this welcome is for this key package
    console.debug("Group created. Navigating to chat", event.data.group.id);
    navigate(ROUTES.chat(event.data.group.id));
    console.debug("Navigated to chat");
  });

  const [configuration] = createResource(getConfiguration);

  let settingsDialog: HTMLDialogElement | undefined;
  let snackbar: HTMLDivElement | undefined;

  const createInvite = createInviteUrl();
  const createQrCode = createInviteQrCode(createInvite);
  const getShareDataPromise = getShareData(createInvite, createQrCode);

  const [inviteUrl] = createResource(() => createInvite);
  const [qrCode] = createResource(() => createQrCode);
  const [shareData] = createResource(() => getShareDataPromise);

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

      {/* TODO show prevent layout shifts */}
      <Show when={qrCode()}>
        {(qrCode) => {
          return (
            <img
              alt="QR code to scan to open invite on another device"
              src={useDataUrl(qrCode())}
              class="rounded-2xl"
            />
          );
        }}
      </Show>

      {/* Needs to be on click otherwise the pointer down opens it and the pointer up closes it immediately */}
      <Show when={inviteUrl()}>
        {(inviteUrl) => {
          /**
           * Has to be used with click event handler as pointer down opens the popover and pointer up closes it immediately
           */
          function copyInvite() {
            const url = inviteUrl();
            navigator.clipboard.writeText(url);
            snackbar?.showPopover();

            // "4-10 seconds based on platform" https://m3.material.io/components/snackbar/guidelines#12145fa5-ada2-4c3b-b2ae-9cdf8ee54ca1
            const HIDE_DELAY = 4_000;
            setTimeout(() => snackbar?.hidePopover(), HIDE_DELAY);
          }

          return (
            <>
              <div popover="manual" ref={snackbar}>
                Link copied
              </div>
              <button onClick={copyInvite}>Copy</button>
            </>
          );
        }}
      </Show>
      <Show when={shareData()}>
        {(share) => {
          // Progressively enhance the invite form
          async function shareInvite() {
            const sharing = share();

            console.debug("SHARING", sharing);
            if (!sharing.isEnabled)
              throw new Error(
                "Sharing is not enabled. Expected this function to not be callable"
              );

            await navigator.share(sharing.shareData);
          }

          return (
            <>
              <div popover="manual" ref={snackbar}>
                Link copied
              </div>
              <button onClick={shareInvite}>Share</button>
            </>
          );
        }}
      </Show>
      <InviteSettingsDialog
        userName={configuration()?.defaultUser?.name}
        ref={settingsDialog}
      />
    </>
  );
}
