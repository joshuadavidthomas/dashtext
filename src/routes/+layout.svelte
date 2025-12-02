<script lang="ts">
	import '../app.css';

	let { children } = $props();
</script>

<div data-layout="root">
	{@render children()}
</div>
