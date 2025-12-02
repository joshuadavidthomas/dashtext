<script lang="ts">
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { Minus, Square, X } from '@lucide/svelte';

	const appWindow = getCurrentWindow();

	async function minimize() {
		await appWindow.minimize();
	}

	async function toggleMaximize() {
		await appWindow.toggleMaximize();
	}

	async function close() {
		await appWindow.close();
	}
</script>

<header
	data-layout="menu-bar"
	data-tauri-drag-region
	class="flex h-[var(--layout-menu-h)] items-center justify-end bg-[var(--cm-background-dark)] px-2"
>
	<div class="flex items-center gap-1" style="-webkit-app-region: no-drag;">
		<button
			onclick={minimize}
			class="rounded p-1.5 text-[var(--cm-comment)] transition-colors hover:bg-[var(--cm-background-highlight)] hover:text-[var(--cm-foreground)]"
			aria-label="Minimize"
		>
			<Minus class="size-3.5" />
		</button>
		<button
			onclick={toggleMaximize}
			class="rounded p-1.5 text-[var(--cm-comment)] transition-colors hover:bg-[var(--cm-background-highlight)] hover:text-[var(--cm-foreground)]"
			aria-label="Maximize"
		>
			<Square class="size-3" />
		</button>
		<button
			onclick={close}
			class="rounded p-1.5 text-[var(--cm-comment)] transition-colors hover:bg-[var(--cm-background-highlight)] hover:text-[var(--cm-error)]"
			aria-label="Close"
		>
			<X class="size-3.5" />
		</button>
	</div>
</header>
