// Stub for web platform - capture shortcut is desktop-only.
// This file exists to satisfy Vite's SSR import resolution.
// When web needs persistent settings, this will get a real implementation.

export interface AppSettings {
  captureShortcut: string;
}

export async function loadSettings(): Promise<AppSettings> {
  throw new Error('Settings API is desktop-only');
}

export async function saveCaptureShortcut(_shortcut: string): Promise<void> {
  throw new Error('Settings API is desktop-only');
}

export async function initializeCaptureShortcut(): Promise<void> {
  throw new Error('Settings API is desktop-only');
}
