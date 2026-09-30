use std::time::Duration;

use dashtext_core::Draft;
use dashtext_core::DraftId;
use dashtext_core::Folder;
use dashtext_core::TextStats;
use gpui_kit::AppContext as _;
use gpui_kit::Context;
use gpui_kit::Entity;
use gpui_kit::EventEmitter;
use gpui_kit::FocusHandle;
use gpui_kit::Focusable;
use gpui_kit::InteractiveElement as _;
use gpui_kit::IntoElement;
use gpui_kit::ParentElement as _;
use gpui_kit::Render;
use gpui_kit::Styled as _;
use gpui_kit::Subscription;
use gpui_kit::Task;
use gpui_kit::Window;
use gpui_kit::component::input::InputEvent;
use gpui_kit::component::input::Textarea;
use gpui_kit::component::input::TextareaState;
use gpui_kit::component::v_flex;

use crate::commands::EDITOR_CONTEXT;
use crate::library::Library;

/// How long typing must pause before the draft is written to the library.
const SAVE_DELAY: Duration = Duration::from_millis(400);

/// Edits one draft at a time and keeps it saved.
///
/// The editor always holds a draft: either one from the library or a new,
/// unsaved one. A new draft is added to the library when its first
/// non-blank text is typed, and a draft left blank is discarded when the
/// editor moves on, so empty drafts never pile up.
pub struct DraftEditor {
    library: Entity<Library>,
    input: Entity<TextareaState>,
    /// The library's copy of the open draft; `None` for a new, unsaved draft.
    draft: Option<Draft>,
    /// Whether the text has changes not yet written to the library.
    dirty: bool,
    /// Ignores the change event produced by loading text programmatically.
    loading: bool,
    save_task: Task<()>,
    /// Counts for the status bar, updated on edit rather than each frame.
    stats: TextStats,
    error: Option<String>,
    _subscriptions: Vec<Subscription>,
}

#[derive(Clone, Debug)]
pub enum DraftEditorEvent {
    /// A new draft was added to the library.
    Created(DraftId),
}

impl EventEmitter<DraftEditorEvent> for DraftEditor {}

impl DraftEditor {
    pub fn new(library: Entity<Library>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| {
            TextareaState::new(window, cx)
                .soft_wrap(true)
                .placeholder("Start typing…")
        });
        let subscriptions = vec![
            cx.subscribe_in(&input, window, Self::on_input_event),
            cx.on_app_quit(|this, cx| {
                this.finish(cx);
                async {}
            }),
        ];

        Self {
            library,
            input,
            draft: None,
            dirty: false,
            loading: false,
            save_task: Task::ready(()),
            stats: TextStats::default(),
            error: None,
            _subscriptions: subscriptions,
        }
    }

    pub fn draft(&self) -> Option<&Draft> {
        self.draft.as_ref()
    }

    pub fn draft_id(&self) -> Option<DraftId> {
        self.draft.as_ref().map(Draft::id)
    }

    /// Opens `draft`, saving (or discarding) the one being edited.
    pub fn open(&mut self, draft: Draft, window: &mut Window, cx: &mut Context<Self>) {
        if self.draft_id() == Some(draft.id()) {
            return;
        }
        self.finish(cx);
        if let Err(error) = self.library.read(cx).mark_accessed(draft.id()) {
            log::warn!("could not record access to {}: {error}", draft.id());
        }
        self.load(Some(draft), window, cx);
    }

    /// Starts a new, unsaved draft.
    pub fn new_draft(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.finish(cx);
        self.load(None, window, cx);
    }

    /// Takes `draft` as the library's latest copy if it is the open draft.
    pub fn adopt(&mut self, draft: &Draft, cx: &mut Context<Self>) {
        if self.draft_id() == Some(draft.id()) {
            self.draft = Some(draft.clone());
            cx.notify();
        }
    }

    /// Adopts the library's latest copy of the open draft (after it was
    /// flagged or moved, for example) without touching the text being edited.
    pub fn refresh(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(id) = self.draft_id() else {
            return;
        };
        match self.library.read(cx).draft(id) {
            Ok(Some(latest)) => {
                if let Some(draft) = &mut self.draft {
                    *draft = latest;
                }
            }
            // Deleted on purpose elsewhere (another window, a script). Honor
            // that: saving the text again would bring the draft back as new.
            Ok(None) => {
                self.save_task = Task::ready(());
                self.load(None, window, cx);
            }
            Err(error) => log::warn!("could not refresh draft {id}: {error}"),
        }
        cx.notify();
    }

    /// Writes pending changes to the library now.
    pub fn save(&mut self, cx: &mut Context<Self>) {
        self.save_task = Task::ready(());
        if !self.dirty {
            return;
        }
        let text = self.input.read(cx).value();
        let result = match self.draft_id() {
            Some(id) => self
                .library
                .update(cx, |library, cx| library.update_content(id, &text, cx)),
            None if text.trim().is_empty() => return,
            None => {
                let created = self
                    .library
                    .update(cx, |library, cx| library.create(&text, false, cx));
                if let Ok(draft) = &created {
                    cx.emit(DraftEditorEvent::Created(draft.id()));
                }
                created
            }
        };

        match result {
            Ok(draft) => {
                self.draft = Some(draft);
                self.dirty = false;
                self.error = None;
            }
            Err(error) => {
                log::error!("could not save draft: {error:#}");
                self.error = Some("Couldn’t save this draft. Your text is kept here.".into());
            }
        }
        cx.notify();
    }

    /// Saves the open draft, or discards it if it is blank. Call before the
    /// editor goes away.
    ///
    /// Blank drafts in the trash are kept: they are there on purpose, and
    /// the move to the trash may still be undone.
    pub fn finish(&mut self, cx: &mut Context<Self>) {
        let blank = self.input.read(cx).value().trim().is_empty();
        let trashed = self
            .draft
            .as_ref()
            .is_some_and(|draft| draft.folder() == Folder::Trash);
        match self.draft_id() {
            Some(id) if blank && !trashed => {
                self.save_task = Task::ready(());
                if let Err(error) = self
                    .library
                    .update(cx, |library, cx| library.delete(id, cx))
                {
                    log::warn!("could not discard empty draft {id}: {error:#}");
                }
            }
            _ => self.save(cx),
        }
    }

    fn load(&mut self, draft: Option<Draft>, window: &mut Window, cx: &mut Context<Self>) {
        let text = draft
            .as_ref()
            .map(|draft| draft.content().to_owned())
            .unwrap_or_default();
        let end = text.len();
        self.stats = TextStats::of(&text);
        self.loading = true;
        self.input.update(cx, |input, cx| {
            input.set_value(text, window, cx);
            // Continue where the text ends, the usual next step for a draft.
            input.set_selected_range(end..end, cx);
        });
        self.loading = false;
        self.draft = draft;
        self.dirty = false;
        self.error = None;
        cx.notify();
    }

    fn on_input_event(
        &mut self,
        _: &Entity<TextareaState>,
        event: &InputEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            InputEvent::Change if !self.loading => {
                self.dirty = true;
                self.stats = TextStats::of(&self.input.read(cx).value());
                if self.draft.is_none() {
                    // Create immediately so the draft shows up in the list as it is typed.
                    self.save(cx);
                } else {
                    self.schedule_save(window, cx);
                }
                cx.notify();
            }
            InputEvent::Blur => self.save(cx),
            InputEvent::Change | InputEvent::Focus | InputEvent::PressEnter { .. } => {}
        }
    }

    fn schedule_save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.save_task = cx.spawn_in(window, async move |this, cx| {
            cx.background_executor().timer(SAVE_DELAY).await;
            // The editor may have closed meanwhile; then there is nothing to save.
            if let Err(error) = this.update(cx, Self::save) {
                log::debug!("editor closed before saving: {error}");
            }
        });
    }

    pub fn stats(&self) -> TextStats {
        self.stats
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
}

impl Focusable for DraftEditor {
    fn focus_handle(&self, cx: &gpui_kit::App) -> FocusHandle {
        self.input.focus_handle(cx)
    }
}

impl Render for DraftEditor {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        // Drafts in the trash are kept for reference; restore one to edit it.
        let readonly = self
            .draft
            .as_ref()
            .is_some_and(|draft| draft.folder() == Folder::Trash);

        v_flex().key_context(EDITOR_CONTEXT).size_full().child(
            Textarea::new(&self.input)
                .appearance(false)
                .readonly(readonly)
                .h_full()
                .px_6()
                .py_4()
                .text_base(),
        )
    }
}
