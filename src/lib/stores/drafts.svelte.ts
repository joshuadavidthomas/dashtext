import { createContext } from 'svelte';
import { useDebounce } from 'runed';
import { listDrafts, createDraft, saveDraft, deleteDraft, type Draft } from '$lib/api/drafts';

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

		// Update local state immediately
		this.currentDraft = { ...this.currentDraft, content };

		// Update in drafts list too
		this.drafts = this.drafts.map((d) =>
			d.id === this.currentDraft!.id ? this.currentDraft! : d
		);

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
			// Update local state with server response (includes new modified_at)
			this.currentDraft = updated;
			this.drafts = this.drafts.map((d) => (d.id === updated.id ? updated : d));
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
