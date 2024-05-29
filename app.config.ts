import { defineConfig } from "@solidjs/start/config";
import wasm from "vite-plugin-wasm";

export default defineConfig({
  ssr: false,
  server: {
    baseURL: process.env.BASE_PATH,
    preset: "static",
  },
  vite: {
    plugins: [wasm()],
  },
});
