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
          header="Meal"
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

        <main>
          <ol class="grid grid-cols-[auto_1fr_auto]">
            <For each={new Array(100)}>
              {() => (
                <li class="ps-4 pe-6 grid grid-cols-subgrid col-span-3 gap-x-4 py-3 bg-light-surface dark:bg-dark-surface">
                  <span class="size-10 bg-light-surface-container-high dark:bg-dark-surface-container-high rounded-full text-center content-center text-title-md text-light-on-surface dark:text-dark-on-surface">
                    C
                  </span>
                  <hgroup>
                    <h2 class="text-light-on-surface dark:text-dark-on-surface-variant text-body-lg ">
                      Headline
                    </h2>
                    <p class="text-light-on-surface-variant dark:text-dark-on-surface-variant line-clamp-1 text-ellipsis text-body-md">
                      Supporting text that is long enough to fill up multiple
                      lines
                      {/* Supporting text */}
                    </p>
                  </hgroup>
                  <p class="text-light-on-surface-variant dark:text-dark-on-surface-variant text-label-sm">
                    Now
                  </p>
                </li>
              )}
            </For>
          </ol>
        </main>
      </>
    </Show>
  );
}
