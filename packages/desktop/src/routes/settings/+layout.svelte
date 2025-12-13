<script lang="ts">
	import '../../app.css';
	import { createSettingsContext } from '@dashtext/lib';
	import { onMount } from 'svelte';
	import { showWindowWhenReady } from '$lib/window';

	let { children } = $props();

	createSettingsContext();

	onMount(() => {
		showWindowWhenReady();
	});
</script>

{@render children()}
