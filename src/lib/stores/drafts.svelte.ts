import { createContext } from 'svelte';
import { useDebounce } from 'runed';
import { replaceState } from '$app/navigation';
import { createDraft, saveDraft } from '$lib/api';

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
 * DraftsState - unified state for all drafts and current draft with autosave
 * Set in context by root layout, consumed by Editor, Sidebar, and route pages
 */
export class DraftsState {
	drafts = $state<Draft[]>([]);
	currentDraft = $state<Draft | null>(null);

	private pendingContent: string | null = null;
	private debouncedSave: ReturnType<typeof useDebounce>;

	constructor(initialDrafts: Draft[]) {
		this.drafts = initialDrafts;
		this.debouncedSave = useDebounce(() => this.performSave(), () => 500);
	}

	setCurrentDraft(draft: Draft | null) {
		if (draft) {
			// Use existing draft from our array to maintain object identity for reactivity
			this.currentDraft = this.drafts.find((d) => d.id === draft.id) ?? draft;
		} else {
			this.currentDraft = null;
		}
	}

	updateContent(content: string) {
		if (this.currentDraft) {
			this.currentDraft.content = content;
		}
		this.pendingContent = content;
		this.debouncedSave();
	}

	flushPendingSave() {
		this.debouncedSave.runScheduledNow();
	}

	private async performSave() {
		if (this.pendingContent === null) return;

		if (this.currentDraft === null && this.pendingContent.trim()) {
			// NEW DRAFT: create in DB, update local state, replaceState URL
			const newDraft = await createDraft();
			newDraft.content = this.pendingContent;
			await saveDraft(newDraft.id, this.pendingContent);

			// Update local state (triggers reactive updates)
			this.drafts = [newDraft, ...this.drafts];
			this.currentDraft = newDraft;

			// Update URL without navigation (preserves focus)
			replaceState(`/drafts/${newDraft.id}`, {});
		} else if (this.currentDraft !== null) {
			// EXISTING DRAFT: just save
			const updated = await saveDraft(this.currentDraft.id, this.pendingContent);
			this.currentDraft.modified_at = updated.modified_at;
		}

		this.pendingContent = null;
	}
}

export const [getDraftsState, setDraftsState] = createContext<DraftsState>();
