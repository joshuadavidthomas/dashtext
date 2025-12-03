<script lang="ts">
	import { Editor } from '$lib/components/editor';
	import { saveDraft } from '$lib/api';
	import { beforeNavigate } from '$app/navigation';
	import { useDebounce } from 'runed';

	let { data } = $props();

	// Track pending content for autosave
	let pendingContent: string | null = null;

	// Debounced save
	const debouncedSave = useDebounce(
		async () => {
			if (pendingContent !== null) {
				await saveDraft(data.draft.id, pendingContent);
				pendingContent = null;
			}
		},
		() => 500
	);

	function handleChange(content: string) {
		pendingContent = content;
		debouncedSave();
	}

	// Flush pending saves before navigating away
	beforeNavigate(() => {
		debouncedSave.runScheduledNow();
	});
</script>

<Editor initialContent={data.draft.content} onchange={handleChange} />
