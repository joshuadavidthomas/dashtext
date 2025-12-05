import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./test/specs",
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: 1,
  reporter: "list",

  use: {
    baseURL: "http://localhost:1420",
    trace: "on-first-retry",
    permissions: ["clipboard-read", "clipboard-write"],
    colorScheme: "dark",
  },

  projects: [
    {
      name: "screenshots",
      use: { ...devices["Desktop Chrome"] },
      testMatch: /screenshots\.e2e\.js/,
    },
  ],

  webServer: {
    command: "bun run dev",
    url: "http://localhost:1420",
    reuseExistingServer: !process.env.CI,
    timeout: 120000,
  },
});
