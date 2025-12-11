import { readFileSync } from "fs";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";

const pkg = JSON.parse(readFileSync("./package.json", "utf-8"));

const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  plugins: [tailwindcss(), sveltekit()],
  define: { __APP_VERSION__: JSON.stringify(pkg.version) },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: {
      ignored: ["**/src-tauri/**"]
    },
    fs: {
      allow: ["../.."]
    }
  }
}));
