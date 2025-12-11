/**
 * It is recommended to place context in its own file for Hot-Module Replacement.
 * https://docs.solidjs.com/reference/component-apis/create-context#usage
 */

import { createContext } from "solid-js";
import { Group } from "./database/schema";

export type App =
  | {
      status: "initializing";
      groups?: never;
    }
  | {
      status: "ready";
      groups: Group[];
      getGroup(groupId: string): Group | undefined;
    };

const INITIAL_CONTEXT: App = {
  status: "initializing",
};

export const AppContext = createContext<App>(INITIAL_CONTEXT);
