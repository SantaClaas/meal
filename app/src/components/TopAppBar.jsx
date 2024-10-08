// @ts-expect-error TS6192 Can not handle new JSDoc syntax (yet?)
/** @import { JSX, VoidProps } from "solid-js" */

/**
 *
 * @param {VoidProps<{header: string, leadingAction?: JSX.Element, trailingAction?: JSX.Element}>} properties
 * @returns {JSX.Element}
 */
export default function TopAppBar({ header, leadingAction, trailingAction }) {
  return (
    // Sticky requires parent element to be height of all conent. Otherwise if parent element is scrolled off screen, it will take the top app bar with it
    // example: if body is parent it needs to have min height of fit content to not have it smaller than the content on the page
    <header class="sticky top-0 px-1 min-h-[--h] [--h:theme(spacing.16)] py-2 grid grid-cols-[3rem,1fr,3rem] gap-1 is-scrolled:bg-light-surface-container dark:is-scrolled:bg-dark-surface-container transition-colors ease-decelerate duration-medium-1 bg-light-surface dark:bg-dark-surface sm:bg-light-surface-container dark:sm:bg-dark-surface-container is-scrolled:z-elevation-level-2">
      {leadingAction}
      <h1 class="text-title-lg col-start-2 content-center text-center text-light-on-surface dark:text-dark-on-surface">
        {header}
      </h1>
      {trailingAction}
    </header>
  );
}
