<script lang="ts">
	import { Editor } from '$lib/components/editor';
	import { setCurrentDraft, Draft } from '$lib/stores/drafts.svelte';
	import { createDraft, saveDraft } from '$lib/api';
	import { goto, invalidateAll, beforeNavigate } from '$app/navigation';
	import { useDebounce } from 'runed';

	let draft: Draft | null = $state(null);
	let pendingContent: string | null = null;

	const debouncedSave = useDebounce(
		async () => {
			if (draft && pendingContent !== null) {
				await saveDraft(draft.id, pendingContent);
				pendingContent = null;
			}
		},
		() => 500
	);

	async function updateContent(content: string) {
		if (draft === null && content.trim()) {
			draft = await createDraft();
			draft.content = content;
			await saveDraft(draft.id, content);
			await invalidateAll();
			goto(`/drafts/${draft.id}`, { replaceState: true });
			return;
		}

		if (draft) {
			draft.content = content;
			pendingContent = content;
			debouncedSave();
		}
	}

	setCurrentDraft({
		draft: () => draft,
		updateContent
	});

	beforeNavigate(() => {
		debouncedSave.runScheduledNow();
	});
</script>

<Editor />
