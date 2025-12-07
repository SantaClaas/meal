import { openDB } from "idb";
import { Group, Schema } from "./schema";

export const openDatabase = openDB<Schema>("meal", 1, {
  upgrade(database) {
    console.debug("Service worker: upgrading database");
    if (!database.objectStoreNames.contains("configuration")) {
      database.createObjectStore("configuration", { autoIncrement: true });
    }
    if (!database.objectStoreNames.contains("groups")) {
      database.createObjectStore("groups", {
        autoIncrement: true,
        keyPath: "id",
      });
    }
  },
});

openDatabase.then((database) => {
  const transaction = database.transaction("configuration", "readwrite");
  const store = transaction.objectStore("configuration");
  if (store.add === undefined) throw new Error("Store is undefined");

  store.add({ isOnboarded: false });
});

export async function getConfiguration(): Promise<Schema["configuration"]> {
  // Get configuration
  const database = await openDatabase;
  const transaction = database.transaction("configuration", "readonly");
  const store = transaction.objectStore("configuration");
  // Get first item
  const cursor = await store.openCursor();
  const configuration = cursor?.value as Schema["configuration"];
  return configuration;
}

export async function getGroup(groupId: string): Promise<Group | undefined> {
  const database = await openDatabase;
  const transaction = database.transaction("groups", "readonly");
  const store = transaction.objectStore("groups");
  return (await store.get(groupId)) as Group;
}

export async function updateGroup(group: Group) {
  const database = await openDatabase;
  const transaction = database.transaction("groups", "readwrite");
  const store = transaction.objectStore("groups");
  store.put(group);
}

export async function insertGroup(group: Group) {
  const database = await openDatabase;
  const transaction = database.transaction("groups", "readwrite");
  const store = transaction.objectStore("groups");
  store.add(group);
}
