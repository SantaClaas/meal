type SendMessageRequest = {
  type: "sendMessage";
  groupId: string;
  friendId: string;
  sent: Date;
  text: string;
};

type CreateInviteRequest = {
  type: "createInvite";
};

type InitializePortRequest = {
  type: "initializePort";
};

type InitializePortResponse = {
  type: "portInitialized";
};

type CreateInviteResponse = {
  type: "inviteUrl";
  inviteUrl: string;
};

/**
 * A message sent to the service worker from a browsing context.
 * Expects a response. (synchronous)
 */
type ServiceWorkerRequest =
  | InitializePortRequest
  | SendMessageRequest
  | CreateInviteRequest;

type ServiceWorkerResponse = InitializePortResponse | CreateInviteResponse;

type InviteFromPackage = {
  type: "inviteFromPackage";
  user: {
    name: string;
  };
  package: string;
};

/** A message that does not expect a response. Fire and forget/async */
type ServiceWorkerMessage = InviteFromPackage;
