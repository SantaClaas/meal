/**
 * @type {HTMLInputElement}
 */
const keyInput = document.getElementById("key");
const valueInput = document.getElementById("value");

/**
 * @param {ClipboardEvent} event
 */
function handlePaste(event) {
  const data = event.clipboardData.getData("text");
  const splitIndex = data.indexOf("=");
  if (splitIndex === -1) return;

  event.preventDefault();
  const key = data.substring(0, splitIndex);
  const value = data.substring(splitIndex + 1);
  keyInput.value = key;
  valueInput.value = value;
}

keyInput.addEventListener("paste", handlePaste);
valueInput.addEventListener("paste", handlePaste);
