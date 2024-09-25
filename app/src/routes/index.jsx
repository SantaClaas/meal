import { createEffect, createMemo, createSignal, For, Show } from "solid-js";
import Onboarding from "../components/Onboarding";
import { useAppContext } from "../components/AppContext";
/**
 * @import { Signal, JSX, Accessor } from "solid-js"
 * @import { State } from "../components/AppContext";}
 */

export default function () {
  const [state, {initialize}]= useAppContext();

  return (
    <>
     <Show when={state()} fallback={<Onboarding setName={initialize} />}>
        {/** @type {(item: Accessor<State>) => JSX.Element } */(
          (state) =>
          (<>
            <h1>Welcome {state().name}</h1>
            <a href="/invite">Invite</a>
          </>))}
    </Show>
    </>
  );
}
