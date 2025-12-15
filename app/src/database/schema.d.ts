import { DBSchema } from "idb";
import { Friend } from "meal-core";

//TODO maybe rename user into alias as there can be multiple
export type User = {
  /**
   * The user's name. Is optional to appear as unknown if users want to keep their identity private.
   * Additionally this is used as the default name. The user can choose a different name per group.
   */
  name?: string;
};

export type IncomingMessage = {
  type: "incoming";
  receivedAt: Date;
  sentAt: Date;
  text: string;
};

export type OutgoingMessage = {
  type: "outgoing";
  sentAt: Date;
  text: string;
};

export type Message = IncomingMessage | OutgoingMessage;

export type Group = {
  id: string;
  /**
   * The users appearance in the group. The user can chose a different appearance per group.
   */
  user?: User;
  friend: Friend;
  messages: Message[];
};

interface Schema extends DBSchema {
  configuration: {
    /**
     * The default user used for creating groups. The user can chose a different appearance per group.
     */
    defaultUser?: User;
    /**
     * A user can be onboarded but not have a name to allow for anonymous usage.
     * That is why the name can not be used to indicate if the user is onboarded.
     */
    isOnboarded: boolean;
  };

  groups: Group;
}
