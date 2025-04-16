// @ts-expect-error TS6192 Can not handle new JSDoc syntax (yet?)
/** @import { JSX, VoidProps } from "solid-js" */

/**
 *
 * @param {VoidProps<{header: string, leadingAction?: JSX.Element, trailingAction?: JSX.Element}>} properties
 * @returns {JSX.Element}
 */
export default function TopAppBar({ header, leadingAction, trailingAction }) {
  return (
    <div
      data-name="stick-detection-container"
      class="sticky top-0 @container-[scroll-state]"
    >
      <header class="px-1 py-2 grid grid-cols-[3rem_1fr_3rem] gap-1 stuck-top-app-bar transition-colors ease-decelerate duration-medium-1 bg-surface">
        {leadingAction}
        <h1 class="text-title-lg col-start-2 content-center text-center text-on-surface">
          {header}
        </h1>
        {trailingAction}
      </header>
    </div>
  );
}
