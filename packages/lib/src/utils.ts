import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

// biome-ignore lint/suspicious/noExplicitAny: Generic type constraint needs any
export type WithoutChild<T> = T extends { child?: any } ? Omit<T, "child"> : T;
// biome-ignore lint/suspicious/noExplicitAny: Generic type constraint needs any
export type WithoutChildren<T> = T extends { children?: any }
  ? Omit<T, "children">
  : T;
export type WithoutChildrenOrChild<T> = WithoutChildren<WithoutChild<T>>;
export type WithElementRef<T, U extends HTMLElement = HTMLElement> = T & {
  ref?: U | null;
};

/**
 * Generate a UUID v4 using the built-in crypto.randomUUID() API
 * Available in modern browsers (Chrome 92+, Firefox 95+, Safari 15+) and Node.js 15.6+
 */
export function generateUUID(): string {
  return crypto.randomUUID();
}
