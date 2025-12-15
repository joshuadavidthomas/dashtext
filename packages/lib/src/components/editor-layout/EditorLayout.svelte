<script lang="ts">
  import type { Snippet } from 'svelte';
  import { createEditorContext } from '../editor';
  import { createDraftsState, Draft, type Draft } from '../../stores';
  import { getPlatform } from '../../platform';
  import * as Sidebar from '../ui/sidebar';
  import * as Tooltip from '../ui/tooltip';
  import DraftsSidebar from './DraftsSidebar.svelte';
  import BaseWinBar from './BaseWinBar.svelte';
  import BaseStatusLine from './BaseStatusLine.svelte';

  type Props = {
    /** Initial drafts data from load function */
    drafts: Draft[];
    /** Platform-specific status items (e.g., VersionIndicator on desktop) */
    statusExtra?: Snippet;
    /** Content rendered after layout (e.g., UpdateDialog on desktop) */
    afterLayout?: Snippet;
    /** Main content slot */
    children: Snippet;
  };

  let { drafts, statusExtra, afterLayout, children }: Props = $props();

  const platform = getPlatform();

  // Initialize editor context
  createEditorContext();

  // Create API adapter that maps new DraftAPI to old DraftsAPI interface
  const apiAdapter = {
    async createDraft() {
      const draftData = await platform.draftsAPI.create();
      return new Draft({ ...draftData, id: 0 }); // Internal ID not exposed
    },
    async saveDraft(uuid: string, content: string) {
      return platform.draftsAPI.save(uuid, content);
    },
    async deleteDraft(uuid: string) {
      return platform.draftsAPI.delete(uuid);
    },
    async archiveDraft(uuid: string) {
      return platform.draftsAPI.archive(uuid);
    },
    async unarchiveDraft(uuid: string) {
      return platform.draftsAPI.unarchive(uuid);
    },
    async pinDraft(uuid: string) {
      return platform.draftsAPI.pin(uuid);
    },
    async unpinDraft(uuid: string) {
      return platform.draftsAPI.unpin(uuid);
    },
    async restoreDraft(uuid: string) {
      return platform.draftsAPI.restore(uuid);
    },
    async hardDeleteDraft(uuid: string) {
      return platform.draftsAPI.hardDelete(uuid);
    },
    replaceUrl: platform.replaceUrl,
    navigateTo: platform.navigateTo,
  };

  // Initialize drafts state with adapted API
  const draftsState = createDraftsState(() => drafts, apiAdapter);

  // Platform-agnostic focus refresh
  $effect(() => {
    return platform.onFocusChange(() => {
      platform.refreshDrafts().then((fresh) => {
        draftsState.drafts = fresh;
      });
    });
  });
</script>

<Tooltip.Provider delayDuration={300}>
  <Sidebar.Provider>
    <div data-layout="root">
      <BaseWinBar />

      <Sidebar.Root collapsible="offcanvas">
        <DraftsSidebar />
      </Sidebar.Root>

      <main data-layout="main">
        {@render children()}
      </main>

      <aside data-layout="aside">
        <!-- Future: aside content -->
      </aside>

      <BaseStatusLine>
        {#snippet statusExtra()}
          {#if statusExtra}{@render statusExtra()}{/if}
        {/snippet}
      </BaseStatusLine>
    </div>

    {#if afterLayout}{@render afterLayout()}{/if}
  </Sidebar.Provider>
</Tooltip.Provider>
