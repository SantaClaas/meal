/** @import { JSX, VoidProps } from "solid-js" */

/**
 *
 * @param {VoidProps<{header: string, leadingAction?: JSX.Element, trailingAction?: JSX.Element}>} properties
 * @returns {JSX.Element}
 */
export default function TopAppBar({ header, leadingAction, trailingAction }) {
  return (
    <header class="px-1 min-h-16 py-2 grid grid-cols-[3rem,1fr,3rem] gap-1 bg-light-surface-container dark:bg-dark-surface-container">
      {leadingAction}
      <h1 class="text-xl col-start-2 font-normal content-center text-center">
        {header}
      </h1>
      {trailingAction}
    </header>
  );
}
