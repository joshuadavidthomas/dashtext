<script lang="ts">
	import { createDraftsState } from '$lib/stores/drafts.svelte';
	import { onMount } from 'svelte';
	import '../app.css';

	let { children } = $props();

	const draftsState = createDraftsState();

	onMount(() => {
		draftsState.init();
	});
</script>

{@render children()}
