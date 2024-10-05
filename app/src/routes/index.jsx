import { createEffect, createSignal, For, Show } from "solid-js";
import Onboarding from "../components/Onboarding";
import { useAppContext } from "../components/AppContext";
import TopAppBar from "../components/TopAppBar";
//@ts-expect-error TS6192 Can not handle new JSDoc syntax (yet?)
// https://devblogs.microsoft.com/typescript/announcing-typescript-5-5/#the-jsdoc-@import-tag
/** @import { Signal, JSX, Accessor } from "solid-js" */
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
export default function Index() {
  const [app, setApp] = useAppContext();

  /** @type {HTMLDivElement | undefined} */
  let dragHandle;
  /** @type {HTMLOListElement | undefined} */
  let pane1;
  /** @type {HTMLElement | undefined} */
  let pane2;
  // Get pane 1 width
  const initialPane1Width = getPane1Width() ?? 0;
  const [pane1Width, setPane1Width] = createSignal(initialPane1Width);

  const clampedWidth = () => {
    if (pane1Width() < 0) {
      return 0;
    }

    if (pane2 !== undefined) {
      const styles = getComputedStyle(pane2);
      let width = pane2?.clientWidth;
      width -= parseFloat(styles.paddingLeft) + parseFloat(styles.paddingRight);
      console.debug("Pane 2 width", width);

      // if (pane2?.clientWidth === 0) {
      if (width === 0) {
        //TODO
        return pane1Width();
      }
    }

    return Math.max(0, pane1Width());
  };

  createEffect(() => {
    // console.debug(pane1?.offsetWidth, pane2?.offsetWidth);

    // if (pane1?.offsetWidth === 0 || pane2?.offsetWidth === 0) return;
    // if (dragHandle === undefined) return;
    // console.debug(clampedWidth());
    // const rectanble = dragHandle.getBoundingClientRect();
    // if (rectanble.right >= window.innerWidth) {
    //   console.debug("Not enough space to show drag handle");
    //   return;
    // }
    // console.debug(
    //   "Bounding client rect",
    //   rectanble.x,
    //   rectanble.right,
    //   rectanble.width
    // );

    localStorage.setItem("--pane-1-width", clampedWidth().toString());
    document.body.style.setProperty("--pane-1-width", clampedWidth() + "px");
  });

  /** @param {MouseEvent} event */
  function handleMouseMove(event) {
    setPane1Width((width) => width + event.movementX);
  }

  // Drag based on https://jsfiddle.net/thatOneGuy/u5bvwh8m/16/
  function handleMouseDown() {
    const controller = new AbortController();

    document.addEventListener("mousemove", handleMouseMove, {
      signal: controller.signal,
    });
    // Stop listening to mousemove when mouse is released somewhere on the document. Not necessarily the button
    document.addEventListener("mouseup", () => controller.abort(), {
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
        <nav class="row-start-3 sm:row-start-1 sm:col-start-1">NAV</nav>
        {/* <main class="bg-light-surface dark:bg-dark-surface h-full mx-4 sm:mx-6 mb-4 sm:mb-6 rounded-"> */}
        {/* <h1>Welcome {app.name}</h1> */}
        {/* <a href="/invite">Invite</a> */}
        {/* </main> */}
        {/* Medium: 50/50 */}
        {/* Expanded: Fixed pane should be 360dp by default */}
        {/* Large & Extra large: Fixed pane should be 412dp by default */}
        <main class="grid sm:col-span-3 sm:row-span-3 sm:grid-rows-subgrid sm:grid-cols-subgrid overflow-hidden sm:pb-6">
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
            ref={pane1}
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
          {/* Drag handle specification: https://m3.material.io/foundations/layout/understanding-layout/parts-of-layout */}
          {/* "The drag handle container is wider than than the handle, and stretches to fill vertical space." */}
          {/* Draggable area has to be at least 24dp wide touch target */}
          {/* Is a drag handle a button?*/}
          <div id="drag-handle" ref={dragHandle} class="w-6 content-center">
            {/* TODO check what the transtion duration should be */}
            <button
              onMouseDown={handleMouseDown}
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
            class="hidden row-span-2 sm:col-start-3 sm:row-start-1 sm:block bg-light-surface dark:bg-dark-surface rounded-extra-large p-6"
          >
            Chat window
          </article>
        </main>
      </>
    </Show>
  );
}
