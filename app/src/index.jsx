// /* @refresh reload */
import { render } from "solid-js/web";
import { Route, Router } from "@solidjs/router";
import Index from "./routes/index";

import "./index.css";
import { AppContextProvider } from "./components/AppContext";
import Invite from "./routes/invite";
import Join from "./routes/join";
import Chat from "./routes/chat";

render(
  () => (
    <AppContextProvider>
      <Router>
        <Route path="/" component={Index} />
        <Route path="/invite" component={Invite} />
        <Route path="/join/:package" component={Join} />
        <Route path="/chat/:groupId" component={Chat} />
      </Router>
    </AppContextProvider>
  ),
  document.body
);
