<script lang="ts">
	import { Editor } from '$lib/components/editor';
	import { setCurrentDraft } from '$lib/stores/drafts.svelte';
	import { saveDraft } from '$lib/api';
	import { beforeNavigate } from '$app/navigation';
	import { useDebounce } from 'runed';

	let { data } = $props();

	const draft = data.draft;
	let pendingContent: string | null = null;

	const debouncedSave = useDebounce(
		async () => {
			if (pendingContent !== null) {
				await saveDraft(draft.id, pendingContent);
				pendingContent = null;
			}
		},
		() => 500
	);

	function updateContent(content: string) {
		draft.content = content;
		pendingContent = content;
		debouncedSave();
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
