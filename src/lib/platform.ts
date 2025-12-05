/**
 * Platform detection utilities for runtime environment checks.
 * These help conditionally render UI and select appropriate backends.
 */

/**
 * Check if running in Tauri desktop environment
 */
export function isTauri(): boolean {
	return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

/**
 * Check if running in a web browser (not Tauri)
 */
export function isWeb(): boolean {
	return typeof window !== 'undefined' && !isTauri();
}

/**
 * Check if running on server (SSR)
 */
export function isServer(): boolean {
	return typeof window === 'undefined';
}
