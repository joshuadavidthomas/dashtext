import { test } from "@playwright/test";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = fileURLToPath(new URL(".", import.meta.url));
const assetsDir = path.resolve(__dirname, "../../assets");

test.describe("README Screenshots", () => {
  test("empty editor", async ({ page }) => {
    await page.goto("/drafts");

    // Wait for CodeMirror editor to be ready
    await page.waitForSelector(".cm-editor", { timeout: 10000 });
    await page.waitForTimeout(500);

    await page.screenshot({
      path: path.join(assetsDir, "empty-editor.png"),
    });
  });

  test("editor with content", async ({ page }) => {
    await page.goto("/drafts");

    // Wait for editor
    await page.waitForSelector(".cm-editor", { timeout: 10000 });
    const editor = page.locator(".cm-content");
    await editor.click();

    // Enter insert mode
    await page.keyboard.press("i");
    await page.waitForTimeout(100);

    // Paste content via clipboard to avoid markdown auto-continuation
    const sampleContent = `# Quick Capture

Dashtext is a minimal text editor for quick capture.

## Features

- Vim mode always enabled
- Fast startup
- Markdown support

Just start typing and your thoughts are saved.`;

    // Set clipboard and paste
    await page.evaluate((text) => navigator.clipboard.writeText(text), sampleContent);
    await page.keyboard.press("Control+v");

    // Return to normal mode
    await page.keyboard.press("Escape");
    await page.waitForTimeout(500);

    await page.screenshot({
      path: path.join(assetsDir, "editor-with-content.png"),
    });
  });

  test("drafts list", async ({ page }) => {
    await page.goto("/drafts");

    // Wait for app to be ready
    await page.waitForSelector(".cm-editor", { timeout: 10000 });

    // Open the sidebar
    const sidebarTrigger = page.locator("[data-sidebar='trigger']");
    if (await sidebarTrigger.isVisible().catch(() => false)) {
      await sidebarTrigger.click();
      await page.waitForTimeout(300);
    }

    await page.waitForTimeout(300);

    await page.screenshot({
      path: path.join(assetsDir, "drafts-list.png"),
    });
  });

  test("capture window", async ({ page }) => {
    // Match the actual Tauri capture window dimensions (600x300)
    await page.setViewportSize({ width: 600, height: 300 });

    await page.goto("/capture");

    // Wait for capture layout to be ready
    await page.waitForSelector("[data-layout='capture-root']", { timeout: 10000 });
    await page.waitForTimeout(500);

    await page.screenshot({
      path: path.join(assetsDir, "capture-window.png"),
    });
  });
});
