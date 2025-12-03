<script lang="ts">
	import { cn, type WithElementRef } from "$lib/utils.js";
	import type { HTMLAttributes } from "svelte/elements";

	let {
		ref = $bindable(null),
		as = "div",
		class: className,
		children,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLElement>> & {
		as?: "div" | "header" | "footer" | "nav" | "aside";
	} = $props();
</script>

<svelte:element
	this={as}
	bind:this={ref}
	data-slot="app-bar"
	class={cn(
		"flex cursor-default select-none items-center justify-between",
		className
	)}
	{...restProps}
>
	{@render children?.()}
</svelte:element>
