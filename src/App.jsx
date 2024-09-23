import { createSignal, Show } from "solid-js";
import { AppState } from "../rust/pkg/meal";
import "./App.css";
/**
 * @import { Signal, JSX } from "solid-js"
 */

function App() {
  /** @type {Signal<string | undefined>} */
  const [name, setName] = createSignal();

  const app = () => {
    const userName = name();
    if (userName === undefined) return undefined;

    console.debug(userName);
    return new AppState(userName);
  };

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
      <Show
        when={app()}
        fallback={
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
        }
      >
        <p>welcome {name()}</p>
      </Show>
    </>
  );
}

export default App;
