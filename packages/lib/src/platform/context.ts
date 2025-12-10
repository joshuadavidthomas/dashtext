import { createContext } from 'svelte';
import type { PlatformCapabilities } from './types';

export const [getPlatform, setPlatformContext] = createContext<PlatformCapabilities>();
