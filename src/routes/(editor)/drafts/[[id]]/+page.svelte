<script lang="ts">
	import { Editor } from '$lib/components/editor';
	import { getDraftsState } from '$lib/stores/drafts.svelte';
	import { beforeNavigate } from '$app/navigation';

	let { data } = $props();

	const draftsState = getDraftsState();

	// Sync route data to context
	$effect.pre(() => {
		draftsState.setCurrentDraft(data.draft);
	});

	// Flush pending save before navigating away
	beforeNavigate(() => draftsState.flushPendingSave());
</script>

<Editor />
