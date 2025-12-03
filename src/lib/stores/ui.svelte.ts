import { createContext } from 'svelte';

/**
 * UIState - manages UI visibility toggles (sidebar, aside, etc.)
 */
export class UIState {
	sidebarVisible = $state(true);

	toggleSidebar() {
		this.sidebarVisible = !this.sidebarVisible;
	}

	showSidebar() {
		this.sidebarVisible = true;
	}

	hideSidebar() {
		this.sidebarVisible = false;
	}
}

// Svelte 5 createContext returns [get, set] tuple
export const [getUIState, setUIState] = createContext<UIState>();

/**
 * Create UIState and set it in context
 * Call this in a parent component (e.g., +layout.svelte)
 */
export const createUIState = () => {
	const ui = new UIState();
	setUIState(ui);
	return ui;
};
