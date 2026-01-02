import { readFileSync } from "fs";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

const pkg = JSON.parse(readFileSync("./package.json", "utf-8"));

export default defineConfig(async () => ({
  plugins: [topLevelAwait(), wasm(), tailwindcss(), sveltekit()],
  define: { __APP_VERSION__: JSON.stringify(pkg.version) },
  
  optimizeDeps: {
    exclude: ["@automerge/automerge-wasm"],
  },
  
  server: {
    fs: {
      allow: ["../.."]
    }
  },
  assetsInclude: ['**/*.sql']
}));
