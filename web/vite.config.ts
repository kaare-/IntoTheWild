import { defineConfig } from "vite";

export default defineConfig({
  server: {
    host: "127.0.0.1",
    port: 47321,
    strictPort: true,
    proxy: {
      "/v1": "http://127.0.0.1:18431",
      "/health": "http://127.0.0.1:18431",
    },
  },
  build: {
    outDir: "dist",
    emptyOutDir: true,
  },
});
