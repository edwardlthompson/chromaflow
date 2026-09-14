import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  plugins: [
    svelte(),
    {
      name: "strip-crossorigin",
      transformIndexHtml(html) {
        return html
          .replaceAll(" crossorigin", "")
          .replaceAll(' type="module"', "")
          .replaceAll("<script src=", "<script defer src=");
      },
    },
  ],
  base: "./",
  build: {
    cssCodeSplit: false,
    modulePreload: false,
    rollupOptions: {
      output: {
        format: "iife",
        name: "chromaflow",
        inlineDynamicImports: true,
        entryFileNames: "assets/[name].js",
      },
    },
  },
  server: { host: "127.0.0.1", port: 1420 },
  preview: { host: "127.0.0.1", port: 4173 },
});
