import { createEffect, createMemo, createSignal, For, Show } from "solid-js";
import Onboarding from "../components/Onboarding";
import { useAppContext } from "../components/AppContext";
/**
 * @import { Signal, JSX, Accessor } from "solid-js"
 */

export default function () {
  const [app, setApp] = useAppContext();

  return (
    <Show
      when={app.isOnboarded}
      fallback={
        <Onboarding
          setName={(name) => {
            setApp("name", name);
            setApp("isOnboarded", true);
          }}
        />
      }
    >
      <>
        <h1>Welcome {app.name}</h1>
        <a href="/invite">Invite</a>
      </>
    </Show>
  );
}
