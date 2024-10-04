import { For, Show } from "solid-js";
import Onboarding from "../components/Onboarding";
import { useAppContext } from "../components/AppContext";
import TopAppBar from "../components/TopAppBar";
//@ts-expect-error TS6192 Can not handle new JSDoc syntax (yet?)
// https://devblogs.microsoft.com/typescript/announcing-typescript-5-5/#the-jsdoc-@import-tag
/** @import { Signal, JSX, Accessor } from "solid-js" */

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
        {/* <main class="bg-light-surface dark:bg-dark-surface h-full mx-4 sm:mx-6 mb-4 sm:mb-6 rounded-"> */}
        {/* <h1>Welcome {app.name}</h1> */}
        {/* <a href="/invite">Invite</a> */}
        {/* </main> */}
        {/* Medium: 50/50 */}
        {/* Expanded: Fixed pane should be 360dp by default */}
        {/* Large & Extra large: Fixed pane should be 412dp by default */}
        <main class="grid grid-cols-1 sm:grid-cols-2 sm:gap-x-6 md:grid-cols-[360px_1fr] lg:grid-cols-[412px_1fr] sm:px-6 pb-4 sm:pb-6">
          <ol class="grid grid-cols-[auto_1fr_auto] rounded-medium overflow-hidden">
            <For each={new Array(100)}>
              {() => (
                <li class="contents">
                  <a
                    href="#"
                    class="ps-4 pe-6 grid grid-cols-subgrid col-span-3 gap-x-4 py-3 bg-light-surface dark:bg-dark-surface group
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
                    <hgroup>
                      <h2 class="text-light-on-surface dark:text-dark-on-surface text-body-lg ">
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
        </main>
      </>
    </Show>
  );
}
