/**
 * @import { Signal, JSX, VoidProps, Setter } from "solid-js"
 */

/**
 *
 * @param {VoidProps<{setName: (name:string)=>void}>} properties
 * @returns {JSX.Element}
 */
export default function ({ setName }) {
  /**
   * @param {Parameters<JSX.EventHandler<HTMLFormElement, SubmitEvent>>[0]} event
   */
  function handleSubmit(event) {
    event.preventDefault();

    /** @type {HTMLInputElement} */
    const input = event.currentTarget.username;
    const value = input.value;
    if (!value) return;

    setName(value);
  }

  return (
    <>
      <hgroup>
        <h1>Your name</h1>
        <p>How do you want others to recognize you</p>
      </hgroup>
      <form onSubmit={handleSubmit}>
        <label for="name">Name</label>
        <input
          name="name"
          id="username"
          type="text"
          autocomplete="username"
          required
        />
        <button type="submit">Submit</button>
      </form>
    </>
  );
}
