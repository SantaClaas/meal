import { createEffect, createSignal, For, onMount, Show } from "solid-js";
import Onboarding from "../components/Onboarding";
import { useAppContext } from "../components/AppContext";
import TopAppBar from "../components/TopAppBar";
import Camera from "../components/Camera";
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

/** @param {ParentProps} properties */
export default function Index(properties) {
  const [app, setApp] = useAppContext();

  /** @type {HTMLDivElement | undefined} */
  let dragHandle;
  /** @type {number | undefined} */
  let dragHandleCenterOffset;
  onMount(() => {
    if (dragHandle === undefined)
      throw new Error("Drag handle should be defined");

    dragHandleCenterOffset = dragHandle.clientWidth / 2;
    console.debug("Drag handle center offset", dragHandleCenterOffset);
  });

  /** @type {Signal<HTMLOListElement | undefined>} */
  let [pane1, setPane1] = createSignal();
  /** @type {HTMLElement | undefined} */
  let pane2;
  // Get pane 1 width
  const initialHandleX = getPane1Width() ?? 0;

  const [handleX, setHandleX] = createSignal(initialHandleX);

  const clampedWidth = () => {
    if (handleX() < 0) {
      return 0;
    }

    const maxWidth = document.documentElement.clientWidth;

    if (dragHandleCenterOffset === undefined)
      throw new Error("Drag handle center offset should be defined");

    return Math.max(
      0,
      Math.min(maxWidth - 48 - dragHandleCenterOffset, handleX())
    );
  };

  createEffect(() => {
    localStorage.setItem("--pane-1-width", clampedWidth().toString());
    document.body.style.setProperty("--pane-1-width", clampedWidth() + "px");
  });

  // const pane1X = () => {
  //   console.debug("Getting pane 1 x", pane1()?.getBoundingClientRect());
  //   return pane1()?.getBoundingClientRect().x;
  // };

  /** @type {number | undefined} */
  let pane1X;
  // Need to run on mount because getting boundling client rectangle will return 0 for x when set and updated through
  // memorization
  onMount(() => {
    pane1X = pane1()?.getBoundingClientRect().x;
    console.assert(pane1X !== undefined, "Pane 1 should be defined");
  });

  /** @param {PointerEvent} event */
  function handlePointerMove(event) {
    console.debug("Pointer move");
    setHandleX((_x) => {
      // Pane 1 X is set on mount and user should not be able to move drag handle before mount
      if (pane1X === undefined) throw new Error("Pane 1 X should be defined");
      if (dragHandleCenterOffset === undefined)
        throw new Error("Drag handle center offset should be defined");

      return event.clientX - pane1X - dragHandleCenterOffset;
    });
  }

  // Drag based on https://jsfiddle.net/thatOneGuy/u5bvwh8m/16/
  function handlePointerDown() {
    console.debug("Pointer down");
    const controller = new AbortController();

    document.addEventListener("pointermove", handlePointerMove, {
      signal: controller.signal,
    });
    // Stop listening to mousemove when mouse is released somewhere on the document. Not necessarily the button
    document.addEventListener("pointerup", () => controller.abort(), {
      signal: controller.signal,
    });
  }

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
        <nav class="row-start-3 sm:row-start-1 sm:col-start-1"></nav>
        {/* <main class="bg-light-surface dark:bg-dark-surface h-full mx-4 sm:mx-6 mb-4 sm:mb-6 rounded-"> */}
        {/* <h1>Welcome {app.name}</h1> */}
        {/* <a href="/invite">Invite</a> */}
        {/* </main> */}
        {/* Medium: 50/50 */}
        {/* Expanded: Fixed pane should be 360dp by default */}
        {/* Large & Extra large: Fixed pane should be 412dp by default */}
        <main class="grid sm:col-span-3 sm:row-span-3 sm:grid-rows-subgrid sm:grid-cols-subgrid overflow-hidden sm:pb-6">
          <aside class="grid grid-rows-subgrid row-span-2 isolate">
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
            <ol
              ref={setPane1}
              class="col-start-1 grid grid-cols-[auto_1fr_auto] sm:rounded-medium scrollbar-none overflow-y-scroll"
            >
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
                          Supporting text that is long enough to fill up
                          multiple lines
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

          {/* Drag handle specification: https://m3.material.io/foundations/layout/understanding-layout/parts-of-layout */}
          {/* "The drag handle container is wider than than the handle, and stretches to fill vertical space." */}
          {/* Draggable area has to be at least 24dp wide touch target */}
          {/* Is a drag handle a button?*/}
          <div
            id="drag-handle"
            ref={dragHandle}
            class="w-6 invisible md:visible select-none content-center isolate row-span-2 bg-light-surface-container dark:bg-dark-surface-container"
          >
            {/* TODO check what the transtion duration should be */}
            <button
              onPointerDown={handlePointerDown}
              class="block bg-light-outline dark:bg-dark-outline w-1 h-12 rounded-full mx-auto hover:bg-[color-mix(in_srgb,theme(colors.light.outline),theme(colors.light.inverse-on-surface)_8%)] dark:hover:bg-[color-mix(in_srgb,theme(colors.dark.outline),theme(colors.dark.inverse-on-surface)_8%)] focus-visible:outline-none focus-visible:bg-[color-mix(in_srgb,theme(colors.light.outline),theme(colors.light.inverse-on-surface)_10%)] dark:focus-visible:bg-[color-mix(in_srgb,theme(colors.dark.outline),theme(colors.dark.inverse-on-surface)_10%)] active:bg-light-on-surface dark:active:bg-dark-on-surface active:rounded-medium active:w-3 active:h-[3.25rem] transition-all duration-short-3 relative cursor-ew-resize"
            >
              {/* Touch target as the tailwind master would do it himself https://youtu.be/MrzrSFbxW7M?t=1869 */}
              {/* TODO change -translate-x and y to -translate-1/2 in tailwind css 4 */}
              <span class="absolute top-1/2 left-1/2 w-[max(100%,24px)] h-12 -translate-x-1/2 -translate-y-1/2 [@media(pointer:fine)]:hidden"></span>
              <span class="sr-only">drag handle</span>
            </button>
          </div>
          <article
            ref={pane2}
            class="hidden content-center sm:mt-4 isolate row-span-2 sm:col-start-3 sm:row-start-1 sm:block bg-light-surface dark:bg-dark-surface rounded-extra-large p-6"
          >
            {properties.children}
          </article>
        </main>
      </>
    </Show>
  );
}
