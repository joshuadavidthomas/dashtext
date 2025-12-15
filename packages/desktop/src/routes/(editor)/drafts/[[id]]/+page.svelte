<script lang="ts">
	import { Editor } from '@dashtext/lib/editor';
	import { getDraftsState } from '@dashtext/lib/stores';
	import { beforeNavigate } from '$app/navigation';
	import { getContext } from 'svelte';

	let { data } = $props();

	const draftsState = getDraftsState();

	// Sync route data to context
	$effect.pre(() => {
		draftsState.setCurrentDraft(data.draft);
	});

	// Flush pending save before navigating away
	beforeNavigate(async () => {
		const navigationHandlers = getContext<{handleDraftSave: () => Promise<void>, handleDraftDelete: () => Promise<void>, handleDraftArchive: () => Promise<void>}>('navigationHandlers');
		await navigationHandlers.handleDraftSave();
	});
</script>

<Editor />
