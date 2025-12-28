import { defineConfig } from "vite";
import solid from "vite-plugin-solid";
import wasm from "vite-plugin-wasm";
import tailwindcss from "@tailwindcss/vite";
import { VitePWA as vitePwa } from "vite-plugin-pwa";
import { execSync } from "node:child_process";
/** @import { defineConfig } from "vite"; */

const gitHash = execSync("git rev-parse --short HEAD").toString().trim();

// Source code is public so source maps can be included
const IS_SOURCE_MAPS_ENABLED = true;
export default defineConfig({
  define: {
    __GIT_COMMIT_HASH__: JSON.stringify(gitHash),
  },
  plugins: [
    tailwindcss(),
    solid(),
    wasm(),
    vitePwa({
      /**
       * Plugin does not support module registration in production which is baseline beginning 13.01.2026
       * https://developer.mozilla.org/en-US/docs/Web/API/ServiceWorker#browser_compatibility
       */
      injectRegister: null,
      strategies: "injectManifest",
      srcDir: "./src/service-worker",
      filename: "serviceWorker.ts",
      // Has to be kept or service worker will not work in development even if we register it manually
      devOptions: {
        enabled: true,
        type: "module",
      },
      pwaAssets: {
        image: "./public/logo-04.svg",
      },
      workbox: {
        sourcemap: IS_SOURCE_MAPS_ENABLED,
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
      // Needed for SQLite wasm
      "Cross-Origin-Opener-Policy": "same-origin",
      "Cross-Origin-Embedder-Policy": "require-corp",
    },
  },
  build: {
    // Fixes top level await build error and support should be fine, right?...right?
    // Needs to be not ESNEXT to compile using statement down as it is not supported in Safari yet
    // Has to stay the same as tsconfig.json target but can't import that because importing JSON does not support comments
    target: "ES2024",
    sourcemap: IS_SOURCE_MAPS_ENABLED,
    minify: false,
  },
  optimizeDeps: {
    exclude: ["@sqlite.org/sqlite-wasm"],
  },
});
