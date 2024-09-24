import { createContext, createSignal, useContext } from "solid-js";
//TODO register rust wasm pack package as package in workspace
import { AppState } from "../../../core/pkg";

/**
 * @import { ParentProps, Context, Accessor } from "solid-js";
 * @typedef {{app: Accessor<AppState|undefined>, initialize(name:string):void}} AppContext
 */

const [app, setApp] = createSignal();
/**
 * @param {string} name
 */
function initialize(name) {
  const app = new AppState(name);
  setApp(app);
}
const state = { app, initialize };
/**
 * @type {Context<AppContext>}
 */
const AppContext = createContext(state);

/**
 *
 * @param {ParentProps} properties
 */
export function AppContextProvider(properties) {
  return (
    <AppContext.Provider value={state}>
      {properties.children}
    </AppContext.Provider>
  );
}
export function useAppContext() {
  return useContext(AppContext);
}