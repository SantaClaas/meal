import { createEffect, createResource, For, Show } from "solid-js";
import Onboarding from "../components/Onboarding";
import { useAppContext } from "../components/AppContext";
import TopAppBar from "../components/TopAppBar";
import { proxy } from "../crackle";
import { type Handler } from "../service-worker/serviceWorker";
// https://devblogs.microsoft.com/typescript/announcing-typescript-5-5/#the-jsdoc-@import-tag
/** @import { Signal, JSX, Accessor, ParentProps } from "solid-js" */
/** @import { Message } from "../components/AppContext" */

function FloatingActionButton() {
  return (
    <a
      href="/invite"
      class="size-14 content-center justify-items-center rounded-medium bg-primary text-on-primary outline-offset-4 focus-within:outline-primary focus:outline-none right-4 bottom-4 absolute"
    >
      <span class="sr-only">New chat</span>
      <svg
        xmlns="http://www.w3.org/2000/svg"
        height="24px"
        viewBox="0 -960 960 960"
        width="24px"
        fill="#e8eaed"
      >
        <path d="M240-400h320v-80H240v80Zm0-120h480v-80H240v80Zm0-120h480v-80H240v80ZM80-80v-720q0-33 23.5-56.5T160-880h640q33 0 56.5 23.5T880-800v480q0 33-23.5 56.5T800-240H240L80-80Zm126-240h594v-480H160v525l46-45Zm-46 0v-480 480Z" />
      </svg>
    </a>
  );
}

function ChatList() {
  const [app, setApp] = useAppContext();

  return (
    <section class="grid isolate overscroll-contain">
      <TopAppBar
        header="Melt"
        trailingAction={
          <button
            onClick={() => {
              setApp("name", null);
              setApp("isOnboarded", false);
            }}
            class="p-3 text-on-surface-variant"
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
      <ol class="col-start-1 grid grid-cols-[auto_1fr_auto] scrollbar-none overflow-y-scroll">
        <For each={app.groups}>
          {(group) => {
            const lastMessage = () =>
              group.messages.length > 0
                ? group.messages[group.messages.length - 1]
                : undefined;

            return (
              <li class="contents">
                <a
                  href={`/chat/${group.id}`}
                  draggable="false"
                  class="ps-4 pe-6 grid grid-cols-subgrid col-span-3 gap-x-4 py-2 items-center bg-surface group
          hover:bg-[color-mix(in_srgb,theme(colors.light.surface),theme(colors.light.on-surface)_8%)]
          hover:dark:bg-[color-mix(in_srgb,theme(colors.dark.surface),theme(colors.dark.on-surface)_8%)]
          focus-visible:outline-[3px] focus-visible:z-[1] focus-visible:outline-offset-0 focus-visible:outline-secondary
          focus-visible:bg-[color-mix(in_srgb,theme(colors.light.surface),theme(colors.light.on-surface)_10%)]
          focus-visible:dark:bg-[color-mix(in_srgb,theme(colors.dark.surface),theme(colors.dark.on-surface)_10%)]
          active:bg-[color-mix(in_srgb,theme(colors.light.surface),theme(colors.light.on-surface)_10%)]
          active:dark:bg-[color-mix(in_srgb,theme(colors.dark.surface),theme(colors.dark.on-surface)_10%)]"
                >
                  <span class="size-10 bg-surface-container-high rounded-full text-center content-center text-title-md text-on-surface">
                    {group.friend.name?.[0].toUpperCase() ?? "🥷"}
                  </span>
                  <hgroup class="min-h-14 content-center">
                    <h2 class="text-on-surface text-body-lg">
                      {/* TODO add info button that they hid their name */}
                      {group.friend.name ?? "Unknown"}
                    </h2>
                    <Show when={lastMessage()}>
                      {
                        /** @type {(item: Accessor<Message>) => JSX.Element} */ (
                          message
                        ) => (
                          <p
                            class="text-on-surface-variant line-clamp-1 text-ellipsis text-body-md group-hover:text-on-surface
            group-focus-visible:text-on-surface group-active:text-on-surface
"
                          >
                            {message().text}
                          </p>
                        )
                      }
                    </Show>
                  </hgroup>
                  <Show when={lastMessage()}>
                    {
                      /** @type {(item: Accessor<Message>) => JSX.Element} */ (
                        message
                      ) => {
                        const dateString = message().sent.toLocaleDateString();
                        const isToday =
                          dateString === new Date().toLocaleDateString();

                        console.debug(
                          "Is today?",
                          isToday,
                          dateString,
                          message().sent.toLocaleTimeString()
                        );

                        return (
                          //TODO format date correctly for datetime attribute
                          <time class="text-on-surface-variant text-label-sm">
                            {isToday
                              ? message().sent.toLocaleTimeString(undefined, {
                                  timeStyle: "short",
                                })
                              : dateString}
                          </time>
                        );
                      }
                    }
                  </Show>
                </a>
              </li>
            );
          }}
        </For>
      </ol>
    </section>
  );
}

/**
 * @import { ComlinkExposed } from "../service-worker/serviceWorker"
 */
// async function initializeComlink() {
//   const { port1, port2 } = new MessageChannel();
//   const message = {
//     type: "initializeComlink",
//   };

//   const worker = await navigator.serviceWorker.ready;
//   if (worker.active === null) throw new Error("No active service worker");
//   worker.active.postMessage(message, [port1]);
//   const comlink = /** @type {Comlink.Remote<ComlinkExposed>} */ (
//     Comlink.wrap(port2)
//   );
//   console.debug("Comlink", comlink.getIsOnboarded);
//   const isOnboarded = await comlink.getIsOnboarded();
//   console.debug("Is onboarded?", isOnboarded);
// }

async function initializeCrackle() {
  const { port1, port2 } = new MessageChannel();
  const message = {
    type: "initializeCrackle",
  };

  const worker = await navigator.serviceWorker.ready;
  // Should not happen as we just awaited the ready promise
  if (worker.active === null) throw new Error("No active service worker");
  worker.active.postMessage(message, [port1]);
  const wrapper = await proxy<Handler>(port2);
  console.debug("Crackle", wrapper.getIsOnboarded);
  return wrapper;
}

const crackled = initializeCrackle();

export default function Index() {
  const [isOnboarded, { mutate: setIsOnboarded }] = createResource(async () => {
    const crackle = await crackled;
    return await crackle.getIsOnboarded();
  });

  async function setName(name: string) {
    console.debug("Setting name", name, crackled);
    const crackle = await crackled;
    console.debug("Setting name", crackled);
    await crackle.completeOnboarding(name);
    console.debug("Name set");
    setIsOnboarded(true);
  }

  createEffect(() => console.debug("Is onboarded?", isOnboarded()));
  return (
    <Show
      when={isOnboarded.state === "ready" && isOnboarded()}
      fallback={<Onboarding setName={setName} />}
    >
      <>
        {/* <nav class="row-start-3 sm:row-start-1 sm:col-start-1"></nav>
        <main class="grid sm:col-span-3 sm:row-span-3 sm:grid-rows-subgrid sm:grid-cols-subgrid overflow-hidden sm:pb-6">
          <article class="hidden content-center sm:mt-4 isolate row-span-2 sm:col-start-3 sm:row-start-1 sm:block bg-light-surface dark:bg-dark-surface rounded-extra-large p-6">
            {properties.children}
          </article>
        </main> */}

        <main class="grid grid-rows-[auto_1fr] h-full w-full bg-surface">
          <ChatList />
          <FloatingActionButton />
        </main>
      </>
    </Show>
  );
}
