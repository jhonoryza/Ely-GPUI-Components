import { defineConfig } from "vite";

// The manifest lives beside the gallery, one folder up.
export default defineConfig({
  server: { fs: { allow: [".."] } },
  // three.js is a chunk of its own, for the home page.
  build: { chunkSizeWarningLimit: 600 },
});
