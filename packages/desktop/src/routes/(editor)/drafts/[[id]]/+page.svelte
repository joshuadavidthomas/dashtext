<script lang="ts">
	import { Editor } from '@dashtext/lib/editor';
	import { getDraftsState } from '@dashtext/lib/stores';
	import { beforeNavigate } from '$app/navigation';

	let { data } = $props();

	const draftsState = getDraftsState();

	// Sync route data to context
	$effect.pre(() => {
		draftsState.setCurrentDraft(data.draft);
	});

	// Flush pending save before navigating away
	beforeNavigate(async () => {
		await draftsState.flushPendingSave();
	});
</script>

<Editor />
