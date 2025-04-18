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
 */
type ServiceWorkerRequest =
  | InitializePortRequest
  | SendMessageRequest
  | CreateInviteRequest;

type ServiceWorkerResponse = InitializePortResponse | CreateInviteResponse;
