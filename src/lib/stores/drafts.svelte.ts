import { createContext } from 'svelte';
import { useDebounce } from 'runed';
import { listDrafts, createDraft, saveDraft, deleteDraft } from '$lib/api/drafts';

/**
 * Raw draft data shape from Tauri API
 */
export type DraftData = {
	id: number;
	content: string;
	created_at: string;
	modified_at: string;
};

/**
 * Draft - reactive draft model with derived presentation properties
 */
export class Draft {
	id: number;
	content = $state('');
	created_at: string;
	modified_at = $state('');

	title = $derived(this.content.split('\n')[0].trim() || 'Untitled');

	previewLines = $derived.by(() => {
		const lines = this.content.split('\n').slice(1);
		return lines.filter((line) => line.trim()).slice(0, 3);
	});

	formattedModifiedAt = $derived.by(() => {
		const value = this.modified_at;
		const asNumber = parseInt(value);
		const date =
			!isNaN(asNumber) && value === String(asNumber)
				? new Date(asNumber * 1000) // Unix timestamp (seconds)
				: new Date(value); // ISO/RFC 3339 string

		if (isNaN(date.getTime())) return 'Unknown date';

		return date.toLocaleString(undefined, {
			month: 'short',
			day: 'numeric',
			year: 'numeric',
			hour: 'numeric',
			minute: '2-digit'
		});
	});

	constructor(data: DraftData) {
		this.id = data.id;
		this.content = data.content;
		this.created_at = data.created_at;
		this.modified_at = data.modified_at;
	}
}

/**
 * DraftsState - manages draft list and current draft with autosave
 */
export class DraftsState {
	drafts = $state<Draft[]>([]);
	currentDraft = $state<Draft | null>(null);
	isLoading = $state(false);
	error = $state('');

	// Debounced save using runed
	private debouncedSave = useDebounce(() => this.performSave(), () => 500);

	/**
	 * Initialize: load drafts, select most recent or create one if empty
	 */
	async init() {
		this.isLoading = true;
		this.error = '';

		try {
			this.drafts = await listDrafts();

			if (this.drafts.length === 0) {
				// Create initial draft if none exist
				const newDraft = await createDraft();
				this.drafts = [newDraft];
			}

			// Select most recent (first in list, since sorted by modified_at DESC)
			this.currentDraft = this.drafts[0];
		} catch (e) {
			this.error = e instanceof Error ? e.message : String(e);
		} finally {
			this.isLoading = false;
		}
	}

	/**
	 * Select a draft by ID
	 */
	async selectDraft(id: number) {
		// Flush any pending save first
		this.debouncedSave.runScheduledNow();

		const draft = this.drafts.find((d) => d.id === id);
		if (draft) {
			this.currentDraft = draft;
		}
	}

	/**
	 * Update current draft content (triggers debounced save)
	 */
	updateContent(content: string) {
		if (!this.currentDraft) return;

		// Update reactive state directly
		this.currentDraft.content = content;

		// Trigger debounced save
		this.debouncedSave();
	}

	/**
	 * Create a new draft and select it
	 */
	async newDraft() {
		// Flush any pending save first
		this.debouncedSave.runScheduledNow();

		try {
			const draft = await createDraft();
			this.drafts = [draft, ...this.drafts];
			this.currentDraft = draft;
		} catch (e) {
			this.error = e instanceof Error ? e.message : String(e);
		}
	}

	/**
	 * Delete a draft by ID
	 */
	async removeDraft(id: number) {
		try {
			await deleteDraft(id);
			this.drafts = this.drafts.filter((d) => d.id !== id);

			// If we deleted the current draft, select another or create new
			if (this.currentDraft?.id === id) {
				if (this.drafts.length > 0) {
					this.currentDraft = this.drafts[0];
				} else {
					const newDraft = await createDraft();
					this.drafts = [newDraft];
					this.currentDraft = newDraft;
				}
			}
		} catch (e) {
			this.error = e instanceof Error ? e.message : String(e);
		}
	}

	/**
	 * Actually save to database
	 */
	private async performSave() {
		if (!this.currentDraft) return;

		try {
			const updated = await saveDraft(this.currentDraft.id, this.currentDraft.content);
			// Update modified_at from server response
			this.currentDraft.modified_at = updated.modified_at;
		} catch (e) {
			this.error = e instanceof Error ? e.message : String(e);
		}
	}
}

// Svelte 5 createContext returns [get, set] tuple
export const [getDraftsState, setDraftsState] = createContext<DraftsState>();

/**
 * Create DraftsState and set it in context
 * Call this in a parent component (e.g., +layout.svelte)
 */
export const createDraftsState = () => {
	const drafts = new DraftsState();
	setDraftsState(drafts);
	return drafts;
};
