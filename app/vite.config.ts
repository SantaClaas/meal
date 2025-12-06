import { defineConfig } from "vite";
import solid from "vite-plugin-solid";
import wasm from "vite-plugin-wasm";
import tailwindcss from "@tailwindcss/vite";
import { VitePWA as vitePwa } from "vite-plugin-pwa";
import { execSync } from "node:child_process";
/** @import { defineConfig } from "vite"; */

const gitHash = execSync("git rev-parse --short HEAD").toString().trim();
export default defineConfig({
  define: {
    __GIT_COMMIT_HASH__: JSON.stringify(gitHash),
  },
  plugins: [
    tailwindcss(),
    solid(),
    wasm(),
    vitePwa({
      registerType: "autoUpdate",
      devOptions: {
        enabled: true,
        type: "module",
      },
      strategies: "injectManifest",
      srcDir: "./src/service-worker",
      filename: "serviceWorker.ts",
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
      // Needed for SQLite wasm
      "Cross-Origin-Opener-Policy": "same-origin",
      "Cross-Origin-Embedder-Policy": "require-corp",
    },
  },
  build: {
    // Fixes top level await build error and support should be fine, right?...right?
    target: "esnext",
  },
  optimizeDeps: {
    exclude: ["@sqlite.org/sqlite-wasm"],
  },
});
