<script lang="ts">
	import { Switch } from '../ui/switch';
	import { getSettingsState } from '../../stores';

	const settings = getSettingsState();
</script>

<div class="space-y-6">
	<div class="space-y-4">
		<h3 class="text-sm font-medium text-[var(--cm-foreground)]">Editor</h3>

		<div class="flex items-center justify-between">
			<div class="space-y-0.5">
				<label for="vim-switch" class="text-sm font-medium text-[var(--cm-foreground)]">
					Vim Mode
				</label>
				<p class="text-xs text-[var(--cm-comment)]">Enable vim keybindings in the editor</p>
			</div>
			<Switch
				id="vim-switch"
				checked={settings.vimEnabled}
				onCheckedChange={(v: boolean) => settings.setVimEnabled(v)}
				class="data-[state=checked]:bg-[var(--cm-accent)]"
			/>
		</div>
	</div>
</div>
