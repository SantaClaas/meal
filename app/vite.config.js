import { defineConfig } from "vite";
import solid from "vite-plugin-solid";
import wasm from "vite-plugin-wasm";
/** @import { defineConfig } from "vite"; */

export default defineConfig({
  plugins: [solid(), wasm()],
  server: {
    // Using a proxy during development to use vite and the rust delivery service while vite is not needed in production
    proxy: {
      "/messages": {
        target: "http://127.0.0.1:3000/",
        changeOrigin: true,
        ws: true,
      },
    },
  },
  build: {
    // Fixes top level await build error and support should be fine, right?...right?
    target: "esnext",
  },
});
