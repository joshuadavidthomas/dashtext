<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import { createDraftsState } from '$lib/stores/drafts.svelte';

	let { children } = $props();

	// Create drafts context at layout level
	const draftsState = createDraftsState();

	// Initialize on mount
	onMount(() => {
		draftsState.init();
	});
</script>

<div data-layout="root">
	{@render children()}
</div>
