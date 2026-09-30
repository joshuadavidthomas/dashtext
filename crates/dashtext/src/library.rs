use std::time::Duration;

use dashtext_core::Draft;
use dashtext_core::DraftId;
use dashtext_core::Folder;
use dashtext_core::Scope;
use dashtext_core::ScopeCounts;
use dashtext_core::Sort;
use dashtext_core::Store;
use dashtext_core::Timestamp;
use dashtext_core::Workspace;
use gpui_kit::App;
use gpui_kit::AppContext as _;
use gpui_kit::Context;
use gpui_kit::Entity;
use gpui_kit::EventEmitter;
use gpui_kit::Global;

/// Settings key holding unsaved quick capture text across restarts.
const CAPTURE_BUFFER_KEY: &str = "capture.buffer";

/// Drafts in the trash longer than this are deleted for good.
const TRASH_RETENTION: Duration = Duration::from_hours(30 * 24);

/// How often a running app looks for drafts past the trash retention.
const TRASH_PURGE_INTERVAL: Duration = Duration::from_hours(1);

/// The draft library as seen by the UI.
///
/// Owns the [`Store`] and is the single place drafts are changed, so every
/// window observes the same state. Each mutation emits a [`LibraryEvent`]
/// naming what changed, so views can patch what they show instead of
/// re-reading the library.
pub struct Library {
    store: Store,
    counts: ScopeCounts,
}

#[derive(Clone, Debug)]
pub enum LibraryEvent {
    /// One draft was created or changed; this is its new state.
    Saved(Draft),
    /// One draft was permanently deleted.
    Deleted(DraftId),
    /// Anything may have changed (many drafts at once, or another process).
    Reloaded,
}

impl EventEmitter<LibraryEvent> for Library {}

struct GlobalLibrary(Entity<Library>);

impl Global for GlobalLibrary {}

impl Library {
    pub fn init(store: Store, cx: &mut App) {
        let library = cx.new(|cx| {
            let mut library = Self {
                store,
                counts: ScopeCounts::default(),
            };
            library.purge_old_trash(cx);
            library.refresh_counts();
            // The app may run for weeks; keep emptying the trash while it does.
            cx.spawn(async move |this, cx| {
                loop {
                    cx.background_executor().timer(TRASH_PURGE_INTERVAL).await;
                    if this.update(cx, Self::purge_old_trash).is_err() {
                        break;
                    }
                }
            })
            .detach();
            library
        });
        cx.set_global(GlobalLibrary(library));
    }

    /// Deletes drafts that have been in the trash past the retention period.
    fn purge_old_trash(&mut self, cx: &mut Context<Self>) {
        let retention = i64::try_from(TRASH_RETENTION.as_millis()).unwrap_or(i64::MAX);
        match self
            .store
            .purge_trash(Timestamp::now().minus_millis(retention))
        {
            Ok(0) => {}
            Ok(purged) => {
                log::info!("deleted {purged} drafts that were in the trash for 30 days");
                self.changed(LibraryEvent::Reloaded, cx);
            }
            Err(error) => log::warn!("could not purge old drafts from the trash: {error}"),
        }
    }

    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalLibrary>().0.clone()
    }

    pub fn counts(&self) -> ScopeCounts {
        self.counts
    }

    pub fn drafts(&self, scope: Scope, sort: Sort) -> anyhow::Result<Vec<Draft>> {
        Ok(self.store.drafts(scope, sort)?)
    }

    pub fn draft(&self, id: DraftId) -> anyhow::Result<Option<Draft>> {
        Ok(self.store.draft(id)?)
    }

    pub fn default_workspace(&self) -> anyhow::Result<Workspace> {
        Ok(self.store.default_workspace()?)
    }

    pub fn save_workspace(&self, workspace: &Workspace) -> anyhow::Result<()> {
        Ok(self.store.save_workspace(workspace)?)
    }

    pub fn create(
        &mut self,
        content: &str,
        flagged: bool,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<Draft> {
        let draft = self.store.create_draft(content, Folder::Inbox, flagged)?;
        self.changed(LibraryEvent::Saved(draft.clone()), cx);
        Ok(draft)
    }

    pub fn update_content(
        &mut self,
        id: DraftId,
        content: &str,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<Draft> {
        let draft = self.store.update_content(id, content)?;
        self.changed(LibraryEvent::Saved(draft.clone()), cx);
        Ok(draft)
    }

    pub fn set_flagged(
        &mut self,
        id: DraftId,
        flagged: bool,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<Draft> {
        let draft = self.store.set_flagged(id, flagged)?;
        self.changed(LibraryEvent::Saved(draft.clone()), cx);
        Ok(draft)
    }

    pub fn move_to(
        &mut self,
        id: DraftId,
        folder: Folder,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<Draft> {
        let draft = self.store.move_to(id, folder)?;
        self.changed(LibraryEvent::Saved(draft.clone()), cx);
        Ok(draft)
    }

    /// Records that a draft was opened. Access time only affects sorting by
    /// "accessed", so this does not announce a change.
    pub fn mark_accessed(&self, id: DraftId) -> anyhow::Result<()> {
        Ok(self.store.mark_accessed(id)?)
    }

    pub fn delete(&mut self, id: DraftId, cx: &mut Context<Self>) -> anyhow::Result<()> {
        self.store.delete(id)?;
        self.changed(LibraryEvent::Deleted(id), cx);
        Ok(())
    }

    pub fn empty_trash(&mut self, cx: &mut Context<Self>) -> anyhow::Result<usize> {
        let removed = self.store.empty_trash()?;
        self.changed(LibraryEvent::Reloaded, cx);
        Ok(removed)
    }

    /// Unsaved quick capture text, kept so closing the capture window (or
    /// quitting) never loses what was typed.
    pub fn capture_buffer(&self) -> anyhow::Result<String> {
        Ok(self.store.setting(CAPTURE_BUFFER_KEY)?.unwrap_or_default())
    }

    pub fn set_capture_buffer(&self, text: &str) -> anyhow::Result<()> {
        if text.is_empty() {
            self.store.remove_setting(CAPTURE_BUFFER_KEY)?;
        } else {
            self.store.set_setting(CAPTURE_BUFFER_KEY, text)?;
        }
        Ok(())
    }

    /// Re-reads the library after another process changed it.
    pub fn reload(&mut self, cx: &mut Context<Self>) {
        self.changed(LibraryEvent::Reloaded, cx);
    }

    fn changed(&mut self, event: LibraryEvent, cx: &mut Context<Self>) {
        self.refresh_counts();
        cx.emit(event);
        cx.notify();
    }

    fn refresh_counts(&mut self) {
        match self.store.counts() {
            Ok(counts) => self.counts = counts,
            Err(error) => log::error!("could not count drafts: {error}"),
        }
    }
}
