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

declare const __GIT_COMMIT_HASH__: string;
render(
  () => (
    <AppContextProvider>
      <Router>
        <Route path="/" component={Index}>
          <Route
            component={() => (
              <Show when={window.isSecureContext}>
                <Camera />
              </Show>
            )}
          />
          <Route path="preview" component={Preview} />
        </Route>
        <Route path="/invite" component={Invite} />
        <Route path="/join/:package" component={Join} />
        <Route path="/chat/:groupId" component={Chat} />
        <Route path="/debug" component={Debug} />
      </Router>

      <span class="absolute bottom-4 left-4 text-xs text-reduced-contrast-on-surface-variant">
        {version}+{__GIT_COMMIT_HASH__}
      </span>
    </AppContextProvider>
  ),
  document.body
);
