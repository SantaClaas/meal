import { createEffect, createMemo, createSignal, For, Show } from "solid-js";
import { AppState, GroupId } from "../../rust/pkg/meal";
import Onboarding from "../components/Onboarding";
import { useAppContext } from "../components/AppContext";
/**
 * @import { Signal, JSX } from "solid-js"
 */

export default function () {
  const { app, initialize } = useAppContext();

  const [groups, setGroups] = createSignal(/** @type {GroupId[]}*/ ([]));

  function createGroup() {
    const state = app();
    if (state === undefined) return;

    const groupId = state.create_group();
    setGroups((groups) => [...groups, groupId]);
    console.debug("Created group");
  }

  const username = () => app()?.get_name();
  return (
    <>
      <Show when={app()} fallback={<Onboarding setName={initialize} />}>
        <p>welcome {username()}</p>
        <button onPointerDown={createGroup}>Create Group</button>
        <a href="/invite">Invite</a>
        <Show when={() => groups().length > 0}>
          <ol>
            <For each={groups()}>{(item, index) => <li>Group</li>}</For>
          </ol>
        </Show>
      </Show>
    </>
  );
}
