// Build target determines adapter:
// - Tauri (default): adapter-static with SPA fallback
// - Web: adapter-cloudflare for Workers deployment
// See: https://svelte.dev/docs/kit/single-page-apps
// See: https://v2.tauri.app/start/frontend/sveltekit/
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

const isWeb = process.env.BUILD_TARGET === "web";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: isWeb
      ? (await import("@sveltejs/adapter-cloudflare")).default()
      : (await import("@sveltejs/adapter-static")).default({
          fallback: "index.html",
        }),
  },
};

export default config;
