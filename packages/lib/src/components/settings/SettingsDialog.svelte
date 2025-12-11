<script lang="ts">
	import * as Dialog from '../ui/dialog';
	import SettingsForm from './SettingsForm.svelte';

	let { open = $bindable(false) }: { open?: boolean } = $props();
</script>

<Dialog.Root bind:open>
	<Dialog.Portal>
		<Dialog.Overlay />
		<Dialog.Content class="bg-[var(--cm-background-dark)] text-[var(--cm-foreground)] border-[var(--cm-gutter-foreground)]">
			<Dialog.Header>
				<Dialog.Title class="text-[var(--cm-foreground)]">Settings</Dialog.Title>
				<Dialog.Description class="text-[var(--cm-foreground-dark)]">
					Configure your Dashtext preferences
				</Dialog.Description>
			</Dialog.Header>
			<SettingsForm />
		</Dialog.Content>
	</Dialog.Portal>
</Dialog.Root>
