import { createStore } from "solid-js/store";
import { createEffect, ParentProps, useContext } from "solid-js";
import { App, AppContext } from "../appContext";
import { useBroadcast } from "../broadcast";
import { streamGroups } from "../database";
import { Group } from "../database/schema";

async function loadGroups(pushGroup: (group: Group) => void) {
  const stream = streamGroups();
  for await (const group of stream) {
    pushGroup(group);
  }
}
/**
 * A central place and reactive store to derive sub page state from and get automatic updates when events from
 * the service worker are received.
 */
export function AppProvider(properties: ParentProps) {
  const [app, setApp] = createStore<App>({
    status: "ready",
    groups: [],
    getGroup(groupId: string): Group | undefined {
      console.debug("Getting group", groupId, "from groups", app.groups);
      // Often used groups are at the top so this should be fine
      return app.groups?.find((group) => group.id === groupId);
    },
  });

  useBroadcast((event) => {
    if (app.status !== "ready") return;

    switch (event.data.type) {
      case "Group created": {
        console.debug("Group created. Adding to groups", event.data.group);
        const group = event.data.group;
        setApp("groups", app.groups.length, group);
        return;
      }
      case "Message received": {
        const { groupId, message } = event.data;
        setApp(
          "groups",
          (group) => group.id === groupId,
          "messages",
          // Any way to not clone the messages array?
          (messages) => [...messages, message]
        );
        break;
      }
    }
  });

  // Start off loading groups
  createEffect(() => {
    if (app.status !== "ready") return;

    //TODO
    console.debug("Loading groups");
    void loadGroups((group) => setApp("groups", app.groups.length, group));
  });

  return (
    <AppContext.Provider value={app}>{properties.children}</AppContext.Provider>
  );
}

export function useApp() {
  return useContext(AppContext);
}
