import { defineConfig } from "vite";
import solid from "vite-plugin-solid";
import wasm from "vite-plugin-wasm";
import tailwindcss from "@tailwindcss/vite";
import { VitePWA as vitePwa } from "vite-plugin-pwa";
/** @import { defineConfig } from "vite"; */

export default defineConfig({
  plugins: [
    tailwindcss(),
    solid(),
    wasm(),
    vitePwa({
      registerType: "autoUpdate",
      devOptions: {
        enabled: true,
      },
      pwaAssets: {
        image: "./public/logo-04.svg",
      },
    }),
  ],
  server: {
    // Using a proxy during development to use vite and the rust delivery service while vite is not needed in production
    proxy: {
      "/messages": {
        target: "http://127.0.0.1:3000/",
        changeOrigin: true,
        ws: true,
      },
    },

    headers: {
      // Support SharedArrayBuffer to send files to workers which is required for Safari to write files to the private origin file system
      // Also might be nice to increase security
      "Cross-Origin-Opener-Policy": "same-origin",
      "Cross-Origin-Embedder-Policy": "require-corp",
    },
  },
  build: {
    // Fixes top level await build error and support should be fine, right?...right?
    target: "esnext",
  },
});
