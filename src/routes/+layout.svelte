<script lang="ts">
	import './layout.css';

	let { children } = $props();
</script>

<div data-layout="root">
	{@render children()}
</div>
