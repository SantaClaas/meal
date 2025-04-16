import { createEffect, createSignal, For, onMount, Show } from "solid-js";
import Onboarding from "../components/Onboarding";
import { useAppContext } from "../components/AppContext";
import TopAppBar from "../components/TopAppBar";
//@ts-expect-error TS6192 Can not handle new JSDoc syntax (yet?)
// https://devblogs.microsoft.com/typescript/announcing-typescript-5-5/#the-jsdoc-@import-tag
/** @import { Signal, JSX, Accessor, ParentProps } from "solid-js" */
function getPane1Width() {
  let storedWidth = localStorage.getItem("--pane-1-width");
  if (storedWidth !== null) {
    const value = parseInt(storedWidth);
    if (!isNaN(value)) {
      return value;
    }
  }

  // Get from CSS as default is different for different screen sizes
  storedWidth = getComputedStyle(document.body).getPropertyValue(
    "--pane-1-width"
  );
  if (storedWidth === null) return null;

  // parseInt ignores the trailing 'px' in a string like "360px"
  const value = parseInt(storedWidth);
  localStorage.setItem("--pane-1-width", value.toString());
  return value;
}

function ChatList() {
  const [_, setApp] = useAppContext();
  return (
    <aside class="grid grid-rows-subgrid row-span-2 isolate overscroll-contain">
      <TopAppBar
        header="Melt"
        trailingAction={
          <button
            onClick={() => {
              setApp("name", null);
              setApp("isOnboarded", false);
            }}
            class="p-3 text-light-on-surface-variant dark:text-dark-on-surface-variant"
          >
            <span class="sr-only">sign out</span>
            <svg
              xmlns="http://www.w3.org/2000/svg"
              height="24px"
              viewBox="0 -960 960 960"
              width="24px"
              aria-hidden="true"
              fill="currentColor"
            >
              <path d="M200-120q-33 0-56.5-23.5T120-200v-560q0-33 23.5-56.5T200-840h280v80H200v560h280v80H200Zm440-160-55-58 102-102H360v-80h327L585-622l55-58 200 200-200 200Z" />
            </svg>
          </button>
        }
      />
      <ol class="col-start-1 grid grid-cols-[auto_1fr_auto] sm:rounded-medium scrollbar-none overflow-y-scroll">
        <For each={new Array(100)}>
          {() => (
            <li class="contents">
              <a
                href="#"
                draggable="false"
                class="ps-4 pe-6 grid grid-cols-subgrid col-span-3 gap-x-4 py-2 items-center bg-light-surface dark:bg-dark-surface sm:bg-light-surface-container sm:dark:bg-dark-surface-container group
          hover:bg-[color-mix(in_srgb,theme(colors.light.surface),theme(colors.light.on-surface)_8%)]
          hover:dark:bg-[color-mix(in_srgb,theme(colors.dark.surface),theme(colors.dark.on-surface)_8%)]
          focus-visible:outline-[3px] focus-visible:z-[1] focus-visible:outline-offset-0 focus-visible:outline-light-secondary dark:focus-visible:outline-dark-secondary
          focus-visible:bg-[color-mix(in_srgb,theme(colors.light.surface),theme(colors.light.on-surface)_10%)]
          focus-visible:dark:bg-[color-mix(in_srgb,theme(colors.dark.surface),theme(colors.dark.on-surface)_10%)]
          active:bg-[color-mix(in_srgb,theme(colors.light.surface),theme(colors.light.on-surface)_10%)]
          active:dark:bg-[color-mix(in_srgb,theme(colors.dark.surface),theme(colors.dark.on-surface)_10%)]"
              >
                <span class="size-10 bg-light-surface-container-high dark:bg-dark-surface-container-high rounded-full text-center content-center text-title-md text-light-on-surface dark:text-dark-on-surface">
                  C
                </span>
                <hgroup class="min-h-14 content-center">
                  <h2 class="text-light-on-surface dark:text-dark-on-surface text-body-lg">
                    Headline
                  </h2>
                  <p
                    class="text-light-on-surface-variant dark:text-dark-on-surface-variant line-clamp-1 text-ellipsis text-body-md group-hover:text-light-on-surface dark:group-hover:text-dark-on-surface
              group-focus-visible:text-light-on-surface dark:group-focus-visible:text-dark-on-surface
              group-active:text-light-on-surface dark:group-active:text-dark-on-surface
 "
                  >
                    Supporting text that is long enough to fill up multiple
                    lines
                    {/* Supporting text */}
                  </p>
                </hgroup>
                <p class="text-light-on-surface-variant dark:text-dark-on-surface-variant text-label-sm">
                  Now
                </p>
              </a>
            </li>
          )}
        </For>
      </ol>
    </aside>
  );
}
/** @param {ParentProps} properties */
export default function Index(properties) {
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
        {/* <nav class="row-start-3 sm:row-start-1 sm:col-start-1"></nav>
        <main class="grid sm:col-span-3 sm:row-span-3 sm:grid-rows-subgrid sm:grid-cols-subgrid overflow-hidden sm:pb-6">
          <article class="hidden content-center sm:mt-4 isolate row-span-2 sm:col-start-3 sm:row-start-1 sm:block bg-light-surface dark:bg-dark-surface rounded-extra-large p-6">
            {properties.children}
          </article>
        </main> */}
        <div
          data-name="mobile-app-shell"
          class="sm:aspect-[9/16] sm:max-h-170 h-full bg-red-400 overflow-scroll border rounded-3xl border-outline"
        >
          <main class="bg-red-400 grid">
            <ChatList />
          </main>
        </div>
      </>
    </Show>
  );
}
