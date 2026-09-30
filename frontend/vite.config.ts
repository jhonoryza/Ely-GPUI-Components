import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

// The manifest lives beside the gallery, one folder up.
export default defineConfig({
  plugins: [react()],
  server: { fs: { allow: [".."] } },
});
