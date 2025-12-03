<script lang="ts">
	import { cn, type WithElementRef } from "$lib/utils.js";
	import type { HTMLAttributes } from "svelte/elements";

	let {
		ref = $bindable(null),
		class: className,
		children,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLDivElement>> = $props();
</script>

<div
	bind:this={ref}
	data-slot="app-bar"
	class={cn(
		"flex cursor-default select-none items-stretch justify-between gap-1 p-1",
		className
	)}
	{...restProps}
>
	{@render children?.()}
</div>
