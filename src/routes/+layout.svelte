<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import { createDraftsState } from '$lib/stores/drafts.svelte';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';

	let { children } = $props();

	// Create drafts context at layout level
	const draftsState = createDraftsState();

	// Initialize on mount
	onMount(() => {
		draftsState.init();
	});
</script>

<Sidebar.Provider>
	<div data-layout="root">
		{@render children()}
	</div>
</Sidebar.Provider>
