//@ts-expect-error TS6192 Can not handle new JSDoc syntax (yet?)
// https://devblogs.microsoft.com/typescript/announcing-typescript-5-5/#the-jsdoc-@import-tag
/** @import { JSX, VoidProps } from "solid-js" */

/**
 *
 * @param {VoidProps<{setName: (name:string)=>void}>} properties
 * @returns {JSX.Element}
 */
export default function ({ setName }) {
  /**
   * @param {Parameters<JSX.EventHandler<HTMLFormElement, SubmitEvent>>[0]} event
   */
  function handleSubmit(event) {
    event.preventDefault();

    /** @type {HTMLInputElement} */
    const input = event.currentTarget.username;
    const value = input.value;
    if (!value) return;

    setName(value);
  }

  return (
    <main class="h-dvh grid content-end sm:content-center px-4 sm:px-6 pb-4 sm:pb-6">
      <hgroup class="sm:mx-auto sm:w-full sm:max-w-sm text-center">
        <img
          class="mx-auto h-10 w-auto"
          src="/logo-04.svg"
          alt="Melt logo showing melting orange goop"
        />
        <h1 class="mt-10 text-display-lg font-bold leading-9 tracking-tight text-light-on-surface dark:text-dark-on-surface">
          Welcome
        </h1>
        <p class="mt-5 text-title-lg text-pretty">
          Enter a name to help others recognize you
        </p>
      </hgroup>

      <form
        onSubmit={handleSubmit}
        class="space-y-6 sm:mx-auto sm:w-full sm:max-w-sm mt-10"
      >
        <label
          for="username"
          class="block group text-light-on-surface-variant dark:text-dark-on-surface-variant text-body-lg font-body-lg leading-body-lg
              hover:text-light-on-surface has-[:hover]:text-light-on-surface dark:has-[:hover]:text-dark-on-surface dark:hover:text-dark-on-surface
            focus-within:text-light-primary dark:focus-within:text-dark-primary"
        >
          Name
          {/* Height should be 56 but that looks too big */}
          <input
            id="username"
            name="username"
            type="username"
            autocomplete="username"
            required
            class="block px-4 py-4 mt-2 w-full bg-inherit rounded-small ring-1 ring-light-outline dark:ring-dark-outline
               group-hover:ring-light-on-surface dark:group-hover:ring-dark-on-surface
               focus:ring-2 focus:ring-light-primary dark:focus:ring-dark-primary focus:text-light-on-surface dark:focus:text-dark-on-surface"
          />
        </label>

        {/* Overengineered buttons styles is just fun. Some of this should probably be done in CSS or added Tailwind CSS utilitties. Follows Material You Filled Button specification*/}
        <button
          type="submit"
          class="w-full rounded-full bg-light-primary dark:bg-dark-primary px-6 py-2.5 text-label-lg font-label-lg leading-label-lg
            text-light-on-primary dark:text-dark-on-primary
           hover:bg-[color-mix(in_srgb,theme(colors.light.primary),theme(colors.light.on-primary)_8%)]
           hover:dark:bg-[color-mix(in_srgb,theme(colors.dark.primary),theme(colors.dark.on-primary)_8%)]
           focus-visible:outline focus-visible:outline-[3px] focus-visible:outline-offset-2 focus-visible:outline-light-secondary focus-visible:dark:outline-dark-secondary
           focus-visible:bg-[color-mix(in_srgb,theme(colors.light.primary),theme(colors.light.on-primary)_10%)]
           focus-visible:dark:bg-[color-mix(in_srgb,theme(colors.dark.primary),theme(colors.dark.on-primary)_10%)]
           active:bg-[color-mix(in_srgb,theme(colors.light.primary),theme(colors.light.on-primary)_10%)]
           active:dark:bg-[color-mix(in_srgb,theme(colors.dark.primary),theme(colors.dark.on-primary)_10%)]
           hover:shadow-elevation-light-level-1 hover:dark:shadow-elevation-dark-level-1"
        >
          Submit
        </button>
      </form>
    </main>
  );
}
