<script lang="ts">
	import '../../app.css';
	import { createSettingsContext } from '@dashtext/lib';

	let { children } = $props();

	createSettingsContext();
</script>

{@render children()}
