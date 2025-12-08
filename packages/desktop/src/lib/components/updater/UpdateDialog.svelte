<script lang="ts">
  import * as Dialog from '@dashtext/ui/dialog';
  import { Button } from '@dashtext/ui/button';
  import { getUpdaterState } from '$lib/stores/updater.svelte';

  const updater = getUpdaterState();
</script>

<Dialog.Root bind:open={updater.dialogOpen}>
  <Dialog.Content class="bg-[var(--cm-background-dark)] text-[var(--cm-foreground)] border-[var(--cm-gutter-foreground)]">
    <Dialog.Header>
      <Dialog.Title class="text-[var(--cm-foreground)]">Update Available</Dialog.Title>
      <Dialog.Description class="text-[var(--cm-foreground-dark)]">
        v{updater.currentVersion} → v{updater.newVersion}
      </Dialog.Description>
    </Dialog.Header>

    {#if updater.releaseNotes}
      <div class="max-h-60 overflow-y-auto rounded-md border border-[var(--cm-gutter-foreground)] bg-[var(--cm-background)] p-3 text-sm text-[var(--cm-foreground)]">
        {updater.releaseNotes}
      </div>
    {/if}

    {#if updater.canAutoUpdate}
      {#if updater.status === 'error' && updater.error}
        <p class="text-sm text-[var(--cm-error)]">{updater.error}</p>
      {/if}

      <Dialog.Footer>
        {#if updater.status === 'available'}
          <Button variant="outline" class="bg-transparent border-[var(--cm-gutter-foreground)] text-[var(--cm-foreground)] hover:bg-[var(--cm-background-highlight)] hover:text-[var(--cm-foreground)]" onclick={() => updater.closeDialog()}>
            Cancel
          </Button>
          <Button class="bg-[var(--cm-accent)] text-[var(--cm-accent-foreground)] hover:bg-[var(--cm-accent)]/90" onclick={() => updater.downloadAndInstall()}>
            Download & Install
          </Button>
        {:else if updater.status === 'downloading'}
          <div class="flex items-center gap-2 text-sm text-[var(--cm-foreground-dark)]">
            <span>Downloading... {updater.downloadProgress}%</span>
          </div>
        {:else if updater.status === 'ready'}
          <Button variant="outline" class="bg-transparent border-[var(--cm-gutter-foreground)] text-[var(--cm-foreground)] hover:bg-[var(--cm-background-highlight)] hover:text-[var(--cm-foreground)]" onclick={() => updater.closeDialog()}>
            Later
          </Button>
          <Button class="bg-[var(--cm-accent)] text-[var(--cm-accent-foreground)] hover:bg-[var(--cm-accent)]/90" onclick={() => updater.restart()}>
            Restart Now
          </Button>
        {:else}
          <Button variant="outline" class="bg-transparent border-[var(--cm-gutter-foreground)] text-[var(--cm-foreground)] hover:bg-[var(--cm-background-highlight)] hover:text-[var(--cm-foreground)]" onclick={() => updater.closeDialog()}>
            Close
          </Button>
        {/if}
      </Dialog.Footer>
    {:else}
      <p class="text-sm text-[var(--cm-foreground-dark)]">
        This installation doesn't support automatic updates. Please update using your package manager (apt, dnf, etc.).
      </p>

      <Dialog.Footer>
        <Button variant="outline" class="bg-transparent border-[var(--cm-gutter-foreground)] text-[var(--cm-foreground)] hover:bg-[var(--cm-background-highlight)] hover:text-[var(--cm-foreground)]" onclick={() => updater.closeDialog()}>
          Close
        </Button>
      </Dialog.Footer>
    {/if}
  </Dialog.Content>
</Dialog.Root>
