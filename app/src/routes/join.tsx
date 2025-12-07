import { Navigate, useNavigate, useParams } from "@solidjs/router";
import { createResource, JSX, Match, Switch } from "solid-js";
import { setupCrackle } from "../useCrackle";
import { ROUTES } from ".";

async function decodeKeyPackage(encodedInvite: string) {
  const handle = await setupCrackle;
  const result = await handle.decodeKeyPackage(encodedInvite);
  return result;
}

/**
 * This page opens after the user opens an invite link.
 * It can be the very first time they open the app
 */
export default function Join() {
  const parameters = useParams();

  if (!parameters.package) {
    return <Navigate href="/" />;
  }

  const [keyPackage] = createResource(parameters.package, decodeKeyPackage);

  const [configuration] = createResource(async () => {
    const handle = await setupCrackle;
    return await handle.getConfiguration();
  });

  const navigate = useNavigate();

  async function handleDecision(
    event: Parameters<JSX.EventHandler<HTMLFormElement, SubmitEvent>>[0]
  ) {
    event.preventDefault();
    const handle = await setupCrackle;

    const name =
      /** @type {HTMLInputElement} */ event.currentTarget.username.value;

    //TODO make this more elegant
    const currentConfiguration = configuration();
    if (currentConfiguration === undefined)
      throw new Error("Expected configuration to be loaded");

    if (name && !currentConfiguration.isOnboarded)
      await handle.completeOnboarding(name);
    if (name) {
      //TODO support name per group
      if (currentConfiguration.isOnboarded) await handle.setName(name);
      // Complete onboarding if used for the first time
      else await handle.completeOnboarding(name);
    }

    const keys = keyPackage();
    if (keys === undefined)
      throw new Error("Expected key package to be loaded");

    const group = await handle.createGroup(keys.friend, name);
    console.debug("Created group", group);

    await handle.inviteToGroup(group.id, keys);
    navigate(ROUTES.chat(group.id));
  }

  //TODO show confirm dialog if they want to stay anonymous
  return (
    <>
      <h1>Invitation</h1>

      <Switch>
        <Match when={keyPackage.loading}>
          {/* TODO loading shimmer/skeleton */}
          <p>Loading...</p>
        </Match>

        <Match when={keyPackage()?.friend.name}>
          {(friendName) => (
            <p>{friendName()} has invited you to chat with them</p>
          )}
        </Match>
        <Match when={!keyPackage()?.friend.name}>
          {/* TODO add additional information that the person that has sent the invite chose to not include their name
          publicly in the invite. Make it clear that this is only for the invite and the name might be revealed when they accepted your request */}
          <p>You are invited to a chat</p>
        </Match>
        <Match when={keyPackage.error}>
          <p>
            Sorry an error occurred trying to read your invite:
            {keyPackage.error}
          </p>
        </Match>
      </Switch>

      <form onSubmit={handleDecision}>
        <label for="username">Your Name</label>
        <input
          type="text"
          name="username"
          id="username"
          autocomplete="username"
          required
          disabled={configuration.loading}
          value={configuration()?.defaultUser?.name}
        />

        <button type="submit" name="accept">
          Accept
        </button>
      </form>
    </>
  );
}
