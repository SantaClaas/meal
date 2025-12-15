import { onCleanup } from "solid-js";
import { Group, Message } from "./database/schema";

export const BROADCAST_NAME = "meal";
export const broadcast = new BroadcastChannel(BROADCAST_NAME);

export type BroadcastMessage =
  | {
      type: "Group created";
      group: Group;
    }
  | {
      type: "Message added";
      groupId: string;
      message: Message;
    }
  | {
      type: "Wipe";
    };

export function broadcastMessage(message: BroadcastMessage) {
  broadcast.postMessage(message);
}

export function useBroadcast(
  callback: (event: MessageEvent<BroadcastMessage>) => void
) {
  const listening = new AbortController();

  const channel = new BroadcastChannel(BROADCAST_NAME);
  channel.addEventListener("message", callback, {
    signal: listening.signal,
  });

  onCleanup(() => listening.abort());
}
