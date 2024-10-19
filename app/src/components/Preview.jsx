import { useNavigate } from "@solidjs/router";
import { createEffect, createResource, createSignal } from "solid-js";

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

  const [url, setUrl] = createSignal();

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
        console.debug("Loaded preview");
        context.drawImage(newImage, 0, 0);
        canvas.width = newImage.width;
        canvas.height = newImage.height;
        // URL.revokeObjectURL(url);
      },
      { once: true }
    );

    newImage.src = url;

    setUrl(url);

    console.debug("Url; " + url);

    // context?.drawImage(preview., 0, 0);
  });

  return (
    <div>
      Preview {image()?.name}
      <canvas ref={canvas}></canvas>
      <img src={url()} />
    </div>
  );
}
