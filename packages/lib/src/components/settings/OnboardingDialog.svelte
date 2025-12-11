<script lang="ts">
	import * as Dialog from '../ui/dialog';
	import { Button } from '../ui/button';
	import { getSettingsState } from '../../stores';

	const settings = getSettingsState();

	const open = $derived(!settings.onboardingCompleted);

	function handleChoice(enableVim: boolean) {
		settings.setVimEnabled(enableVim);
		settings.completeOnboarding();
	}
</script>

<Dialog.Root {open}>
	<Dialog.Portal>
		<Dialog.Overlay />
		<Dialog.Content class="bg-[var(--cm-background-dark)] text-[var(--cm-foreground)] border-[var(--cm-gutter-foreground)]">
			<Dialog.Header>
				<Dialog.Title class="text-[var(--cm-foreground)]">Welcome to Dashtext</Dialog.Title>
				<Dialog.Description class="text-[var(--cm-foreground-dark)]">
					Dashtext includes vim keybindings by default. Would you like to enable them?
				</Dialog.Description>
			</Dialog.Header>
			<Dialog.Footer class="gap-2">
				<Button variant="outline" class="bg-transparent border-[var(--cm-gutter-foreground)] text-[var(--cm-foreground)] hover:bg-[var(--cm-background-highlight)] hover:text-[var(--cm-foreground)]" onclick={() => handleChoice(false)}>
					No, use standard editing
				</Button>
				<Button class="bg-[var(--cm-accent)] text-[var(--cm-accent-foreground)] hover:bg-[var(--cm-accent)]/90" onclick={() => handleChoice(true)}>Yes, enable vim mode</Button>
			</Dialog.Footer>
		</Dialog.Content>
	</Dialog.Portal>
</Dialog.Root>
