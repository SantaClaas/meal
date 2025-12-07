import { DBSchema } from "idb";

interface Schema extends DBSchema {
  configuration: {
    user?: {
      /**
       * The user's name. Is optional to appear as unknown if users want to keep their identity private.
       * Additionally this is used as the default name. The user can choose a different name per group.
       */
      name?: string;
    };
    /**
     * A user can be onboarded but not have a name to allow for anonymous usage.
     * That is why the name can not be used to indicate if the user is onboarded.
     */
    isOnboarded: boolean;
  };
}
