/**
 * A worker to persist files on the origin private file system in Safari
 * Safari does not support the FileSystemHandle.createWritable() method.
 * This is used to cache photos for preview and to avoid crashes
 */
addEventListener("message", async (event) => {
  if (
    !(event.data instanceof SharedArrayBuffer) &&
    !(event.data instanceof ArrayBuffer)
  ) {
    console.error("Expected SharedArrayBuffer");
    return;
  }

  const directory = await navigator.storage.getDirectory();

  const fileHandle = await directory.getFileHandle("preview", {
    create: true,
  });

  const handle = await fileHandle.createSyncAccessHandle();
  // Clear file
  handle.truncate(0);
  // Write new data
  handle.write(event.data);
  handle.flush();
  handle.close();

  postMessage(undefined);
});
