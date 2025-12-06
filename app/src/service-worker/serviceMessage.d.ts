type CompleteOnboardingRequest = {
  type: "completeOnboarding";
  name: string;
};

type CompleteOnboardingResponse = {
  type: "completeOnboarding";
};

type GetIsOnboardedRequest = {
  type: "getIsOnboarded";
};

type GetIsOnboardedResponse = {
  type: "isOnboarded";
  isOnboarded: boolean;
};

/**
 * A message sent to the service worker from a browsing context.
 * Expects a response. (synchronous)
 */
type ServiceWorkerRequest = GetIsOnboardedRequest | CompleteOnboardingRequest;

type ServiceWorkerResponse =
  | GetIsOnboardedResponse
  | CompleteOnboardingResponse;

type InviteFromPackage = {
  type: "inviteFromPackage";
  user: {
    name: string;
  };
  package: string;
};

/** A message that does not expect a response. Fire and forget/async */
type ServiceWorkerMessage = InviteFromPackage;
