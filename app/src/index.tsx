// /* @refresh reload */
import { render } from "solid-js/web";
import { Show } from "solid-js";
import { Route, Router } from "@solidjs/router";
import Index from "./routes/index";

import "./index.css";
import { AppContextProvider } from "./components/AppContext";
import Invite from "./routes/invite";
import Join from "./routes/join";
import Chat from "./routes/chat";
import Camera from "./components/Camera";
import Preview from "./components/Preview";
import Debug from "./routes/debug";
import { version } from "../../package.json";
import { ROUTES } from "./routes";

declare const __GIT_COMMIT_HASH__: string;
render(
  () => (
    <AppContextProvider>
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
    </AppContextProvider>
  ),
  document.body
);
