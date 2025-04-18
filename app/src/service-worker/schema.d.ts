import { DBSchema } from "idb";

interface Schema extends DBSchema {
  configuration: {
    user?: {
      name?: string;
    };
    clientId?: string;
  };
}
