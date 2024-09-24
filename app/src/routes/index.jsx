import { createEffect, createMemo, createSignal, For, Show } from "solid-js";
import Onboarding from "../components/Onboarding";
import { useAppContext } from "../components/AppContext";
/**
 * @import { Signal, JSX } from "solid-js"
 */

export default function () {
  const { app, initialize } = useAppContext();

  const username = () => app()?.get_name();
  return (
    <>
      <Show when={app()} fallback={<Onboarding setName={initialize} />}>
        <p>welcome {username()}</p>
        <a href="/invite">Invite</a>
      </Show>
    </>
  );
}
