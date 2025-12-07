import { onCleanup } from "solid-js";
import { Group } from "./service-worker/schema";

export const BROADCAST_NAME = "meal";
const broadcast = new BroadcastChannel(BROADCAST_NAME);

export type BroadcaseMessage = {
  type: "Group created";
  group: Group;
};

export function broadcastMessage(message: BroadcaseMessage) {
  broadcast.postMessage(message);
}

export function useBroadcast(
  callback: (event: MessageEvent<BroadcaseMessage>) => void
) {
  const listening = new AbortController();

  const channel = new BroadcastChannel(BROADCAST_NAME);
  channel.addEventListener("message", callback, {
    signal: listening.signal,
  });

  onCleanup(listening.abort);
}
