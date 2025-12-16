<script lang="ts">
	import { Kbd, KbdGroup } from '../ui/kbd';
	import { getPlatform } from '../../platform';

	interface Props {
		value: string;
		onchange?: (value: string) => void;
		error?: string | null;
	}

	let { value, onchange, error = null }: Props = $props();

	const platform = getPlatform();
	const isMac = $derived(platform.platform === 'desktop' && typeof navigator !== 'undefined' && navigator.platform.includes('Mac'));
	
	const DEFAULT_SHORTCUT = 'CommandOrControl+Shift+C';

	let mode = $state<'idle' | 'recording'>('idle');
	let previewKeys = $state<string[]>([]);
	let showSuccess = $state(false);
	let inputRef: HTMLButtonElement;

	// Parse shortcut string into display keys
	function parseShortcut(shortcut: string): string[] {
		if (!shortcut) return [];
		
		return shortcut.split('+').map((key) => {
			if (key === 'CommandOrControl') return isMac ? '⌘' : 'Ctrl';
			if (key === 'Alt') return isMac ? '⌥' : 'Alt';
			if (key === 'Shift') return isMac ? '⇧' : 'Shift';
			return key;
		});
	}

	const displayKeys = $derived(parseShortcut(value));

	function startRecording() {
		mode = 'recording';
		previewKeys = [];
	}

	function cancelRecording() {
		mode = 'idle';
		previewKeys = [];
	}

	function clearShortcut(event: MouseEvent) {
		event.stopPropagation();
		onchange?.(DEFAULT_SHORTCUT);
		showSuccess = true;
		setTimeout(() => (showSuccess = false), 1000);
	}

	function buildModifierPreview(event: KeyboardEvent): string[] {
		const keys: string[] = [];
		if (event.ctrlKey || event.metaKey) keys.push(isMac ? '⌘' : 'Ctrl');
		if (event.shiftKey) keys.push(isMac ? '⇧' : 'Shift');
		if (event.altKey) keys.push(isMac ? '⌥' : 'Alt');
		return keys;
	}

	function handleKeydown(event: KeyboardEvent) {
		if (mode !== 'recording') return;

		event.preventDefault();

		// Escape cancels recording
		if (event.key === 'Escape') {
			cancelRecording();
			return;
		}

		// Skip modifier-only keys, but update preview
		if (['Control', 'Shift', 'Alt', 'Meta'].includes(event.key)) {
			previewKeys = buildModifierPreview(event);
			return;
		}

		// Validate: require at least one modifier
		if (!event.ctrlKey && !event.metaKey && !event.shiftKey && !event.altKey) {
			// Silently ignore non-modified keys
			return;
		}

		// Build shortcut string
		const parts: string[] = [];
		if (event.ctrlKey || event.metaKey) parts.push('CommandOrControl');
		if (event.shiftKey) parts.push('Shift');
		if (event.altKey) parts.push('Alt');
		parts.push(event.key.toUpperCase());

		const newValue = parts.join('+');
		onchange?.(newValue);

		// Show success feedback
		showSuccess = true;
		mode = 'idle';
		previewKeys = [];
		setTimeout(() => (showSuccess = false), 1000);
		
		// Return focus to the button
		inputRef?.focus();
	}

	function handleBlur() {
		if (mode === 'recording') {
			cancelRecording();
		}
	}

	function handleButtonKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter' && mode === 'idle') {
			event.preventDefault();
			startRecording();
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="space-y-2">
	<div class="flex w-full items-stretch gap-2">
		<button
			bind:this={inputRef}
			type="button"
			class="flex flex-1 items-center gap-2 rounded border border-[var(--cm-selection)] bg-[var(--cm-background)] px-3 py-2 text-sm text-[var(--cm-foreground)] transition-colors hover:border-[var(--cm-accent)] focus:outline-none focus:ring-1 focus:ring-[var(--cm-accent)]"
			onclick={startRecording}
			onkeydown={handleButtonKeydown}
			onblur={handleBlur}
			aria-label={mode === 'recording' ? 'Recording keyboard shortcut' : 'Click to change keyboard shortcut'}
		>
			{#if mode === 'idle'}
				<KbdGroup>
					{#each displayKeys as key}
						<Kbd class="border border-[var(--cm-selection)] bg-[var(--cm-background-highlight)] text-[var(--cm-foreground)]">{key}</Kbd>
					{/each}
				</KbdGroup>
				{#if showSuccess}
					<span class="text-green-500" aria-label="Success">✓</span>
				{/if}
			{:else if previewKeys.length > 0}
				<KbdGroup>
					{#each previewKeys as key}
						<Kbd class="border border-[var(--cm-selection)] bg-[var(--cm-background-highlight)] text-[var(--cm-foreground)]">{key}</Kbd>
					{/each}
				</KbdGroup>
				<span class="text-[var(--cm-comment)]">+ ...</span>
			{:else}
				<span class="text-[var(--cm-comment)]">⌨️ Press new shortcut...</span>
			{/if}
		</button>

		{#if mode === 'idle'}
			<button
				type="button"
				class="flex items-center justify-center rounded border border-[var(--cm-selection)] bg-[var(--cm-background)] px-3 py-2 text-[var(--cm-comment)] transition-colors hover:border-[var(--cm-accent)] hover:text-[var(--cm-foreground)]"
				onclick={clearShortcut}
				aria-label="Reset to default shortcut"
			>
				×
			</button>
		{/if}
	</div>

	{#if error}
		<p class="text-xs text-red-500">{error}</p>
	{/if}
</div>
