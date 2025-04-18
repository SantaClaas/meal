type SendMessage = {
  type: "sendMessage";
  groupId: string;
  friendId: string;
  sent: Date;
  text: string;
};
/**
 * A message sent to the service worker from a browsing context.
 */
type ServiceWorkerRequest = SendMessage;
