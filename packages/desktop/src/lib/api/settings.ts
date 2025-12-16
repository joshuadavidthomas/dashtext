import { invoke } from '@tauri-apps/api/core';
import { eq } from 'drizzle-orm';
import { getDb, settings } from '$lib/db';

const DEFAULT_CAPTURE_SHORTCUT = 'CommandOrControl+Shift+C';

export interface AppSettings {
  captureShortcut: string;
}

function getNow(): string {
  return new Date().toISOString();
}

/**
 * Load settings from database
 */
export async function loadSettings(): Promise<AppSettings> {
  const db = await getDb();
  
  const rows = await db
    .select()
    .from(settings)
    .where(eq(settings.id, 1))
    .limit(1);

  if (rows.length > 0) {
    return {
      captureShortcut: rows[0].captureShortcut ?? DEFAULT_CAPTURE_SHORTCUT,
    };
  }

  // No settings row exists, create default
  const now = getNow();
  await db.insert(settings).values({
    id: 1,
    captureShortcut: DEFAULT_CAPTURE_SHORTCUT,
    createdAt: now,
    updatedAt: now,
  });

  return {
    captureShortcut: DEFAULT_CAPTURE_SHORTCUT,
  };
}

/**
 * Save capture shortcut to database and register with backend
 */
export async function saveCaptureShortcut(shortcut: string): Promise<void> {
  const db = await getDb();
  const now = getNow();

  // Save to database
  await db
    .insert(settings)
    .values({
      id: 1,
      captureShortcut: shortcut,
      createdAt: now,
      updatedAt: now,
    })
    .onConflictDoUpdate({
      target: settings.id,
      set: {
        captureShortcut: shortcut,
        updatedAt: now,
      },
    });

  // Register with Rust backend
  await invoke('register_capture_shortcut', { shortcut });
}

/**
 * Initialize hotkey registration from database on app startup
 */
export async function initializeCaptureShortcut(): Promise<void> {
  const { captureShortcut } = await loadSettings();
  await invoke('register_capture_shortcut', { shortcut: captureShortcut });
}
