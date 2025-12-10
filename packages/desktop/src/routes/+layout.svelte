<script lang="ts">
	import '../app.css';
	import { setPlatformContext } from '@dashtext/lib/platform';
	import { desktopPlatform } from '$lib/platform';

	let { children } = $props();

	setPlatformContext(desktopPlatform);
</script>

{@render children()}
