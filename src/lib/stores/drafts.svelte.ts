import { createContext } from 'svelte';
import { useDebounce } from 'runed';
import { createDraft, saveDraft } from '$lib/api';
import { goto, invalidateAll } from '$app/navigation';

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
 * DraftState - manages current draft and autosave
 * Set in context by route pages, consumed by Editor and other components
 */
export class DraftState {
	draft = $state<Draft | null>(null);
	private draftId: number | null;
	private pendingContent: string | null = null;
	private debouncedSave: ReturnType<typeof useDebounce>;

	constructor(draft: Draft | null = null) {
		this.draft = draft;
		this.draftId = draft?.id ?? null;
		this.debouncedSave = useDebounce(() => this.performSave(), () => 500);
	}

	updateContent(content: string) {
		if (this.draft) {
			this.draft.content = content;
		}
		this.pendingContent = content;
		this.debouncedSave();
	}

	private async performSave() {
		if (this.pendingContent === null) return;

		if (this.draftId === null && this.pendingContent.trim()) {
			// Create new draft on first save
			const newDraft = await createDraft();
			this.draftId = newDraft.id;
			this.draft = newDraft;
			this.draft.content = this.pendingContent;
			await saveDraft(this.draftId, this.pendingContent);
			await invalidateAll();
			goto(`/drafts/${this.draftId}`, { replaceState: true });
		} else if (this.draftId !== null) {
			await saveDraft(this.draftId, this.pendingContent);
		}

		this.pendingContent = null;
	}

	flushPendingSave() {
		this.debouncedSave.runScheduledNow();
	}
}

export const [getDraftState, setDraftState] = createContext<DraftState>();
