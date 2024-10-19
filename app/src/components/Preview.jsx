import { useNavigate } from "@solidjs/router";
import { createEffect, createResource } from "solid-js";

async function loadPreviewImage() {
  const directory = await navigator.storage.getDirectory();

  let fileHandle;
  try {
    fileHandle = await directory.getFileHandle("preview");
  } catch (error) {
    if (error instanceof DOMException && error.name === "NotFoundError") {
      console.debug("No preview file found");
      return;
    }
    throw error;
  }
  const file = await fileHandle.getFile();
  return file;
}

export default function Preview() {
  const [image] = createResource(loadPreviewImage);

  // Navigate away if there is no preview
  const navigate = useNavigate();
  createEffect(() => {
    if (image() === undefined && image.loading === false) {
      navigate("/");
      return;
    }
  });

  /** @type {HTMLCanvasElement | undefined} */
  let canvas;

  createEffect(() => {
    if (canvas === undefined) return;
    const preview = image();
    if (preview === undefined) return;
    const context = canvas.getContext("2d");
    if (context === null) {
      console.error("Expected to get context");
      return;
    }

    const url = URL.createObjectURL(preview);
    const newImage = new Image();
    newImage.addEventListener(
      "load",
      () => {
        // Needs to be set before drawing image
        canvas.width = newImage.width;
        canvas.height = newImage.height;
        context.drawImage(newImage, 0, 0, newImage.width, newImage.height);
        URL.revokeObjectURL(url);
      },
      { once: true }
    );

    newImage.src = url;
  });

  return (
    <div class="aspect-[9/16] max-w-2xl max-h-full min-w-80 rounded-large grid grid-cols-1 place-items-center grid-rows-[1fr_auto] overflow-hidden mx-auto">
      <canvas ref={canvas} class="w-full"></canvas>
      {/* <img src={url()} /> */}
    </div>
  );
}
