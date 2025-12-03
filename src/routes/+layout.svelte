<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import { createDraftsState } from '$lib/stores/drafts.svelte';
	import { createUIState } from '$lib/stores/ui.svelte';

	let { children } = $props();

	// Create contexts at layout level
	const draftsState = createDraftsState();
	const uiState = createUIState();

	// Initialize on mount
	onMount(() => {
		draftsState.init();
	});
</script>

<div data-layout="root">
	{@render children()}
</div>
