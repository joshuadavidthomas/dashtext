/**
 * Type declarations for SvelteKit modules used in shared components.
 * These are resolved at build time by the consuming SvelteKit apps.
 */

declare module '$app/navigation' {
  export function goto(
    url: string | URL,
    opts?: {
      replaceState?: boolean;
      noScroll?: boolean;
      keepFocus?: boolean;
      invalidateAll?: boolean;
      state?: App.PageState;
    }
  ): Promise<void>;
  export function invalidate(url: string | URL | ((url: URL) => boolean)): Promise<void>;
  export function invalidateAll(): Promise<void>;
  export function preloadData(url: string | URL): Promise<void>;
  export function preloadCode(...urls: string[]): Promise<void>;
  export function beforeNavigate(callback: (navigation: any) => void): void;
  export function afterNavigate(callback: (navigation: any) => void): void;
  export function onNavigate(callback: (navigation: any) => void): void;
  export function pushState(url: string | URL, state: App.PageState): void;
  export function replaceState(url: string | URL, state: App.PageState): void;
}

declare module '$app/stores' {
  import type { Readable } from 'svelte/store';
  export const page: Readable<any>;
  export const navigating: Readable<any>;
  export const updated: Readable<any>;
}
