import { DBSchema } from "idb";

interface Schema extends DBSchema {
  configuration: {
    user?: {
      name?: string;
    };
    clientId?: string;
    /**
     * A user can be onboarded but not have a name to allow for anonymous usage.
     * That is why the name can not be used to indicate if the user is onboarded.
     */
    isOnboarded: boolean;
  };
}
