import { createContext } from 'svelte';

const STORAGE_KEY = 'dashtext-settings';

interface SettingsData {
	vimEnabled: boolean;
	onboardingCompleted: boolean;
}

const DEFAULTS: SettingsData = {
	vimEnabled: true,
	onboardingCompleted: false,
};

export class SettingsState {
	vimEnabled = $state(DEFAULTS.vimEnabled);
	onboardingCompleted = $state(DEFAULTS.onboardingCompleted);

	constructor() {
		this.load();
	}

	private load() {
		if (typeof localStorage === 'undefined') return;
		try {
			const saved = localStorage.getItem(STORAGE_KEY);
			if (saved) {
				const data = JSON.parse(saved) as Partial<SettingsData>;
				this.vimEnabled = data.vimEnabled ?? DEFAULTS.vimEnabled;
				this.onboardingCompleted = data.onboardingCompleted ?? DEFAULTS.onboardingCompleted;
			}
		} catch {
			/* ignore parse errors */
		}
	}

	private persist() {
		if (typeof localStorage === 'undefined') return;
		localStorage.setItem(
			STORAGE_KEY,
			JSON.stringify({
				vimEnabled: this.vimEnabled,
				onboardingCompleted: this.onboardingCompleted,
			})
		);
	}

	setVimEnabled(enabled: boolean) {
		this.vimEnabled = enabled;
		this.persist();
	}

	completeOnboarding() {
		this.onboardingCompleted = true;
		this.persist();
	}
}

export const [getSettingsState, setSettingsState] = createContext<SettingsState>();

export function createSettingsContext(): SettingsState {
	const state = new SettingsState();
	setSettingsState(state);
	return state;
}
