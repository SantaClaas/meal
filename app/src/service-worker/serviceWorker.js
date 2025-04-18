import { precacheAndRoute } from "workbox-precaching";

// @ts-expect-error This variable is replaced by workbox through vite pwa plugin
precacheAndRoute(self.__WB_MANIFEST);
