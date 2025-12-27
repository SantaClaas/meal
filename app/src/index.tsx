// /* @refresh reload */
import { Route, Router } from "@solidjs/router";
import { Show } from "solid-js";
import { render } from "solid-js/web";
import Index from "./routes/index.tsx";

import { version } from "../../package.json";
import { AppProvider } from "./components/AppContextProvider.tsx";
import Camera from "./components/Camera";
import Preview from "./components/Preview";
import "./index.css";
import { ROUTES } from "./routes";
import Chat from "./routes/chat";
import Debug from "./routes/debug";
import Invite from "./routes/invite";
import Join from "./routes/join";
import { runSocketProxy } from "./socketProxy.ts";
import { register as registerServiceWorker } from "./service-worker/register";

/**
 * Defined in the vite configuration vite.config.ts
 */
declare const __GIT_COMMIT_HASH__: string;
registerServiceWorker();

void runSocketProxy();

function App() {
  return (
    <AppProvider>
      <Router>
        <Route path={ROUTES.INDEX} component={Index}>
          <Route
            component={() => (
              <Show when={window.isSecureContext}>
                <Camera />
              </Show>
            )}
          />
          <Route path={ROUTES.PREVIEW} component={Preview} />
        </Route>
        <Route path={ROUTES.INVITE} component={Invite} />
        <Route path={ROUTES.join()} component={Join} />
        <Route path={ROUTES.chat()} component={Chat} />
        <Route path={ROUTES.DEBUG} component={Debug} />
      </Router>

      <span class="absolute bottom-4 left-4 text-xs text-reduced-contrast-on-surface-variant pointer-events-none">
        {version}+{__GIT_COMMIT_HASH__}
      </span>
    </AppProvider>
  );
}

render(App, document.body);
