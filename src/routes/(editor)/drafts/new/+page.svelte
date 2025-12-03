<script lang="ts">
	import { Editor } from '$lib/components/editor';
	import { createDraft, saveDraft } from '$lib/api';
	import { goto, invalidateAll, beforeNavigate } from '$app/navigation';
	import { useDebounce } from 'runed';

	// Track if we've created the draft yet
	let draftId: number | null = null;
	let pendingContent: string | null = null;

	// Debounced save (only runs after draft is created)
	const debouncedSave = useDebounce(
		async () => {
			if (draftId !== null && pendingContent !== null) {
				await saveDraft(draftId, pendingContent);
				pendingContent = null;
			}
		},
		() => 500
	);

	// Flush pending saves before navigating away
	beforeNavigate(() => {
		debouncedSave.runScheduledNow();
	});

	async function handleChange(content: string) {
		// First change: create the draft
		if (draftId === null && content.trim()) {
			const draft = await createDraft();
			draftId = draft.id;
			await saveDraft(draftId, content);
			await invalidateAll(); // Refresh sidebar
			// Replace URL without adding history entry
			goto(`/drafts/${draftId}`, { replaceState: true });
			return;
		}

		// Subsequent changes: debounced save
		if (draftId !== null) {
			pendingContent = content;
			debouncedSave();
		}
	}
</script>

<Editor initialContent="" onchange={handleChange} />
