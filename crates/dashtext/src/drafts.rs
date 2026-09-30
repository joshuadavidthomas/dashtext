//! The drafts window: the library's scopes, the draft list and the editor.

mod editor;
mod list;

use dashtext_core::Draft;
use dashtext_core::DraftId;
use dashtext_core::Folder;
use dashtext_core::Scope;
use dashtext_core::Workspace;
use gpui_kit::Action;
use gpui_kit::AppContext as _;
use gpui_kit::Context;
use gpui_kit::Entity;
use gpui_kit::FocusHandle;
use gpui_kit::Focusable;
use gpui_kit::FontWeight;
use gpui_kit::InteractiveElement as _;
use gpui_kit::IntoElement;
use gpui_kit::ParentElement as _;
use gpui_kit::Render;
use gpui_kit::SharedString;
use gpui_kit::StyleRefinement;
use gpui_kit::Styled as _;
use gpui_kit::Subscription;
use gpui_kit::Window;
use gpui_kit::assets::IconName;
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::Disableable as _;
use gpui_kit::component::IconName as ComponentIconName;
use gpui_kit::component::IndexPath;
use gpui_kit::component::Selectable as _;
use gpui_kit::component::Sizable as _;
use gpui_kit::component::Theme;
use gpui_kit::component::TitleBar;
use gpui_kit::component::WindowExt as _;
use gpui_kit::component::button::Button;
use gpui_kit::component::button::ButtonVariant;
use gpui_kit::component::button::ButtonVariants as _;
use gpui_kit::component::h_flex;
use gpui_kit::component::list::ListEvent;
use gpui_kit::component::list::ListState;
use gpui_kit::component::menu::AppMenuBar;
use gpui_kit::component::notification::Notification;
use gpui_kit::component::resizable::h_resizable;
use gpui_kit::component::resizable::resizable_panel;
use gpui_kit::component::sidebar::Sidebar;
use gpui_kit::component::sidebar::SidebarFooter;
use gpui_kit::component::sidebar::SidebarGroup;
use gpui_kit::component::sidebar::SidebarMenu;
use gpui_kit::component::sidebar::SidebarMenuItem;
use gpui_kit::component::v_flex;
use gpui_kit::div;
use gpui_kit::prelude::FluentBuilder as _;

use self::editor::DraftEditor;
use self::editor::DraftEditorEvent;
use self::list::DraftList;
use self::list::DraftListView;
use crate::commands;
use crate::commands::DRAFTS_CONTEXT;
use crate::library::Library;
use crate::library::LibraryEvent;
use crate::menus;
use crate::menus::MenuState;
use crate::time_format;

pub struct DraftsWindow {
    library: Entity<Library>,
    workspace: Workspace,
    list: Entity<ListState<DraftList>>,
    list_view: Entity<DraftListView>,
    editor: Entity<DraftEditor>,
    app_menu_bar: Entity<AppMenuBar>,
    menu_state: Option<MenuState>,
    focus_handle: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

impl DraftsWindow {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let library = Library::global(cx);
        let workspace = library
            .read(cx)
            .default_workspace()
            .unwrap_or_else(|error| {
                log::error!("could not load the workspace: {error:#}");
                Workspace::new("Drafts")
            });
        let drafts = load_drafts(&library, &workspace, cx);
        let list = cx.new(|cx| {
            ListState::new(DraftList::new(workspace.scope(), drafts), window, cx).searchable(true)
        });
        let list_view = cx.new(|_| DraftListView::new(list.clone()));
        let editor = cx.new(|cx| DraftEditor::new(library.clone(), window, cx));

        let subscriptions = vec![
            cx.subscribe_in(&library, window, |this, _, event, window, cx| match event {
                LibraryEvent::Saved(draft) => this.apply_saved(draft, window, cx),
                LibraryEvent::Deleted(id) => this.apply_deleted(*id, window, cx),
                LibraryEvent::Reloaded => this.reload(window, cx),
            }),
            cx.subscribe_in(&list, window, Self::on_list_event),
            cx.subscribe_in(&editor, window, |this, _, event, window, cx| match event {
                DraftEditorEvent::Created(id) => this.select_in_list(*id, window, cx),
            }),
            cx.observe_window_appearance(window, |_, window, cx| {
                Theme::sync_system_appearance(Some(window), cx);
            }),
        ];

        let closing_editor = editor.clone();
        window.on_window_should_close(cx, move |_, cx| {
            closing_editor.update(cx, DraftEditor::finish);
            true
        });

        let mut this = Self {
            library,
            workspace,
            list,
            list_view,
            editor,
            app_menu_bar: AppMenuBar::new(cx),
            menu_state: None,
            focus_handle: cx.focus_handle(),
            _subscriptions: subscriptions,
        };
        this.open_first(window, cx);
        this.sync_menus(cx);

        // Start with the cursor in the editor, ready to type.
        let editor_focus = this.editor.focus_handle(cx);
        window.defer(cx, move |window, cx| editor_focus.focus(window, cx));
        this
    }

    fn close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editor.update(cx, DraftEditor::finish);
        window.remove_window();
    }

    fn scope(&self) -> Scope {
        self.workspace.scope()
    }

    fn current_draft(&self, cx: &gpui_kit::App) -> Option<Draft> {
        self.editor.read(cx).draft().cloned()
    }

    /// Brings the menus in line with the open draft; rebuilt only on change.
    fn sync_menus(&mut self, cx: &mut Context<Self>) {
        let state = MenuState {
            draft: self
                .current_draft(cx)
                .map(|draft| (draft.folder(), draft.is_flagged())),
            trash_is_empty: self.library.read(cx).counts().get(Scope::Trash) == 0,
        };
        if self.menu_state != Some(state) {
            self.menu_state = Some(state);
            menus::install(state, &self.app_menu_bar, cx);
        }
    }

    /// Re-reads the whole scope. Used when many drafts may have changed.
    fn reload(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let drafts = load_drafts(&self.library, &self.workspace, cx);
        let scope = self.scope();
        self.editor
            .update(cx, |editor, cx| editor.refresh(window, cx));
        self.update_list(window, cx, |list| list.set_drafts(scope, drafts));
    }

    /// Patches the list for one saved draft; this runs after every autosave,
    /// so it must not re-read the library.
    fn apply_saved(&mut self, draft: &Draft, window: &mut Window, cx: &mut Context<Self>) {
        let sort = self.workspace.sort();
        let row = draft.clone();
        self.editor.update(cx, |editor, cx| editor.adopt(draft, cx));
        self.update_list(window, cx, |list| list.apply(row, sort));
    }

    fn apply_deleted(&mut self, id: DraftId, window: &mut Window, cx: &mut Context<Self>) {
        if self.editor.read(cx).draft_id() == Some(id) {
            self.editor
                .update(cx, |editor, cx| editor.refresh(window, cx));
        }
        self.update_list(window, cx, |list| list.remove(id));
    }

    /// Changes the list's rows, keeps the open draft selected, and refreshes
    /// the counts and menus that depend on the library.
    fn update_list(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
        change: impl FnOnce(&mut DraftList),
    ) {
        let selected = self.editor.read(cx).draft_id();
        self.list.update(cx, |list, cx| {
            change(list.delegate_mut());
            let ix = selected.and_then(|id| list.delegate().position_of(id));
            list.set_selected_index(ix, window, cx);
            cx.notify();
        });
        self.sync_menus(cx);
        cx.notify();
    }

    fn set_scope(&mut self, scope: Scope, window: &mut Window, cx: &mut Context<Self>) {
        if scope == self.scope() {
            return;
        }
        self.editor.update(cx, DraftEditor::save);
        self.workspace.set_scope(scope);
        if let Err(error) = self.library.read(cx).save_workspace(&self.workspace) {
            log::warn!("could not remember the selected scope: {error:#}");
        }
        self.reload(window, cx);
        self.open_first(window, cx);
    }

    /// Opens the first draft in the list, or a new draft when it is empty.
    fn open_first(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let first = self
            .list
            .read(cx)
            .delegate()
            .draft_at(IndexPath::new(0))
            .cloned();
        match first {
            Some(draft) => self.open(draft, window, cx),
            None => self.new_draft(window, cx),
        }
    }

    fn open(&mut self, draft: Draft, window: &mut Window, cx: &mut Context<Self>) {
        let id = draft.id();
        self.editor
            .update(cx, |editor, cx| editor.open(draft, window, cx));
        self.select_in_list(id, window, cx);
        self.sync_menus(cx);
    }

    fn select_in_list(&mut self, id: DraftId, window: &mut Window, cx: &mut Context<Self>) {
        self.list.update(cx, |list, cx| {
            let ix = list.delegate().position_of(id);
            list.set_selected_index(ix, window, cx);
            if ix.is_some() {
                list.scroll_to_selected_item(window, cx);
            }
            // Selecting does not notify; the list view is cached.
            cx.notify();
        });
    }

    /// The New Draft command: new drafts land in the inbox, so show it when
    /// the current view could not list the draft, then put the cursor in it.
    fn start_new_draft(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if matches!(self.scope(), Scope::Archive | Scope::Trash) {
            self.set_scope(Scope::Inbox, window, cx);
        }
        self.new_draft(window, cx);
        self.editor.focus_handle(cx).focus(window, cx);
    }

    fn new_draft(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editor
            .update(cx, |editor, cx| editor.new_draft(window, cx));
        self.list.update(cx, |list, cx| {
            list.set_selected_index(None, window, cx);
            // Selecting does not notify; the list view is cached.
            cx.notify();
        });
        self.sync_menus(cx);
        cx.notify();
    }

    fn on_list_event(
        &mut self,
        list: &Entity<ListState<DraftList>>,
        event: &ListEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let (ix, focus_editor) = match event {
            ListEvent::Select(ix) => (*ix, false),
            ListEvent::Confirm(ix) => (*ix, true),
            ListEvent::Cancel => return,
        };
        let Some(draft) = list.read(cx).delegate().draft_at(ix).cloned() else {
            return;
        };
        self.editor
            .update(cx, |editor, cx| editor.open(draft, window, cx));
        self.sync_menus(cx);
        if focus_editor {
            self.editor.focus_handle(cx).focus(window, cx);
        }
        cx.notify();
    }

    /// Picks the draft to show after the current one leaves the list: the
    /// next one, or the previous one at the end.
    fn neighbor_of(&self, id: DraftId, cx: &gpui_kit::App) -> Option<Draft> {
        let list = self.list.read(cx).delegate();
        let ix = list.position_of(id)?;
        list.draft_at(IndexPath::new(ix.row + 1))
            .or_else(|| {
                ix.row
                    .checked_sub(1)
                    .and_then(|row| list.draft_at(IndexPath::new(row)))
            })
            .cloned()
    }

    fn toggle_flag(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Trashed drafts are readonly; restore one to change it.
        let Some(draft) = self
            .current_draft(cx)
            .filter(|d| d.folder() != Folder::Trash)
        else {
            return;
        };
        let neighbor = self.neighbor_of(draft.id(), cx);
        let result = self.library.update(cx, |library, cx| {
            library.set_flagged(draft.id(), !draft.is_flagged(), cx)
        });
        match result {
            // Unflagging in the Flagged view takes the draft out of the list.
            Ok(saved) => self.move_on_if_gone(&saved, neighbor, window, cx),
            Err(error) => report(window, cx, "Couldn’t change the flag.", &error),
        }
    }

    /// Shows `neighbor` (or a new draft) when `draft` no longer belongs to
    /// the current scope.
    fn move_on_if_gone(
        &mut self,
        draft: &Draft,
        neighbor: Option<Draft>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Library events arrive after this update; the editor must see the
        // new folder and flag before it decides whether to keep the draft.
        self.editor.update(cx, |editor, cx| editor.adopt(draft, cx));
        if self.scope().contains(draft) {
            return;
        }
        match neighbor {
            Some(next) => self.open(next, window, cx),
            None => self.new_draft(window, cx),
        }
    }

    /// Moves the current draft to `folder`, then shows its neighbor if it
    /// left the current scope. Offers to undo the move.
    fn move_current(&mut self, folder: Folder, window: &mut Window, cx: &mut Context<Self>) {
        self.editor.update(cx, DraftEditor::save);
        let Some(draft) = self.current_draft(cx) else {
            return;
        };
        let from = draft.folder();
        let neighbor = self.neighbor_of(draft.id(), cx);

        let moved = match self
            .library
            .update(cx, |library, cx| library.move_to(draft.id(), folder, cx))
        {
            Ok(moved) => moved,
            Err(error) => {
                report(window, cx, "Couldn’t move the draft.", &error);
                return;
            }
        };

        self.move_on_if_gone(&moved, neighbor, window, cx);

        let message = match folder {
            Folder::Inbox => "Moved to Inbox",
            Folder::Archive => "Archived",
            Folder::Trash => "Moved to Trash",
        };
        let id = moved.id();
        let library = self.library.clone();
        window.push_notification(
            Notification::new()
                .message(message)
                .action(move |_, _, cx| {
                    let library = library.clone();
                    Button::new("undo-move")
                        .label("Undo")
                        .small()
                        .on_click(cx.listener(move |notification, _, window, cx| {
                            if let Err(error) =
                                library.update(cx, |library, cx| library.move_to(id, from, cx))
                            {
                                log::error!("could not undo move of {id}: {error:#}");
                            }
                            notification.dismiss(window, cx);
                        }))
                })
                // After `action`, which turns autohide off.
                .autohide(true),
            cx,
        );
    }

    fn toggle_archive(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(draft) = self
            .current_draft(cx)
            .filter(|d| d.folder() != Folder::Trash)
        else {
            return;
        };
        let target = if draft.folder() == Folder::Inbox {
            Folder::Archive
        } else {
            Folder::Inbox
        };
        self.move_current(target, window, cx);
    }

    fn toggle_trash(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(draft) = self.current_draft(cx) else {
            return;
        };
        let target = if draft.folder() == Folder::Trash {
            Folder::Inbox
        } else {
            Folder::Trash
        };
        self.move_current(target, window, cx);
    }

    fn confirm_delete(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(draft) = self.current_draft(cx) else {
            return;
        };
        if draft.folder() != Folder::Trash {
            return;
        }
        let title = match draft.title() {
            "" => "Delete this draft?".to_owned(),
            title => format!("Delete “{}”?", truncate(title, 40)),
        };
        let this = cx.entity().downgrade();
        window.open_alert_dialog(cx, move |dialog, _, _| {
            let this = this.clone();
            dialog
                .title(title.clone())
                .description("This can’t be undone.")
                .ok_text("Delete")
                .ok_variant(ButtonVariant::Danger)
                .show_cancel(true)
                .on_ok(move |_, window, cx| {
                    if let Err(error) = this.update(cx, |this, cx| this.delete_current(window, cx))
                    {
                        log::debug!("drafts window closed before deleting: {error}");
                    }
                    true
                })
        });
    }

    fn delete_current(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(draft) = self.current_draft(cx) else {
            return;
        };
        let neighbor = self.neighbor_of(draft.id(), cx);
        if let Err(error) = self
            .library
            .update(cx, |library, cx| library.delete(draft.id(), cx))
        {
            report(window, cx, "Couldn’t delete the draft.", &error);
            return;
        }
        match neighbor {
            Some(next) => self.open(next, window, cx),
            None => self.new_draft(window, cx),
        }
    }

    fn confirm_empty_trash(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let count = self.library.read(cx).counts().get(Scope::Trash);
        if count == 0 {
            return;
        }
        let this = cx.entity().downgrade();
        let title = if count == 1 {
            "Delete 1 draft in the trash?".to_owned()
        } else {
            format!("Delete {count} drafts in the trash?")
        };
        window.open_alert_dialog(cx, move |dialog, _, _| {
            let this = this.clone();
            dialog
                .title(title.clone())
                .description("This can’t be undone.")
                .ok_text("Empty Trash")
                .ok_variant(ButtonVariant::Danger)
                .show_cancel(true)
                .on_ok(move |_, window, cx| {
                    let result =
                        this.update(cx, |this, cx| this.library.update(cx, Library::empty_trash));
                    match result {
                        Ok(Ok(_)) => {
                            if let Err(error) = this.update(cx, |this, cx| {
                                if this.scope() == Scope::Trash {
                                    this.new_draft(window, cx);
                                }
                            }) {
                                log::debug!("drafts window closed: {error}");
                            }
                        }
                        Ok(Err(error)) => report(window, cx, "Couldn’t empty the trash.", &error),
                        Err(error) => log::debug!("drafts window closed: {error}"),
                    }
                    true
                })
        });
    }

    fn render_sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let counts = self.library.read(cx).counts();
        let current = self.scope();
        let items = Scope::ALL.map(|scope| {
            let count = counts.get(scope);
            SidebarMenuItem::new(scope_label(scope))
                .icon(scope_icon(scope))
                .active(scope == current)
                .suffix(move |_, cx| {
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .when(count > 0, |this| this.child(count.to_string()))
                })
                .on_click(cx.listener(move |this, _, window, cx| this.set_scope(scope, window, cx)))
        });

        Sidebar::new("scopes")
            .collapsible(false)
            .w_56()
            .child(
                SidebarGroup::new(self.workspace.name().to_owned())
                    .child(SidebarMenu::new().children(items)),
            )
            .footer(
                SidebarFooter::new().child(
                    Button::new("quick-capture")
                        .ghost()
                        .small()
                        .w_full()
                        .icon(IconName::Zap)
                        .label("Quick Capture…")
                        .tooltip_with_action("Open quick capture", &commands::QuickCapture, None)
                        .on_click(|_, window, cx| {
                            window.dispatch_action(commands::QuickCapture.boxed_clone(), cx);
                        }),
                ),
            )
    }

    fn render_list_pane(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let scope = self.scope();
        let count = self.library.read(cx).counts().get(scope);

        v_flex()
            .size_full()
            .child(
                h_flex()
                    .h_12()
                    .flex_none()
                    .px_4()
                    .gap_2()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(scope_label(scope)),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(count.to_string()),
                    )
                    .child(div().flex_1())
                    .when(scope == Scope::Trash, |this| {
                        this.child(
                            Button::new("empty-trash")
                                .ghost()
                                .small()
                                .label("Empty Trash…")
                                .disabled(count == 0)
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.confirm_empty_trash(window, cx);
                                })),
                        )
                    })
                    .child(
                        Button::new("new-draft")
                            .ghost()
                            .small()
                            .icon(IconName::SquarePen)
                            .tooltip_with_action(
                                "New draft",
                                &commands::NewDraft,
                                Some(DRAFTS_CONTEXT),
                            )
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.start_new_draft(window, cx);
                            })),
                    ),
            )
            .child(
                div().flex_1().min_h_0().child(
                    self.list_view
                        .clone()
                        .cached(StyleRefinement::default().size_full()),
                ),
            )
    }

    fn render_toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let draft = self.current_draft(cx);
        let exists = draft.is_some();
        let flagged = draft.as_ref().is_some_and(Draft::is_flagged);
        let folder = draft.as_ref().map(Draft::folder);
        let in_trash = folder == Some(Folder::Trash);
        // Views spanning folders say where the draft lives; elsewhere the
        // list header already does.
        let location = match (self.scope(), folder) {
            (Scope::Flagged | Scope::All, Some(Folder::Inbox)) => Some("In Inbox"),
            (Scope::Flagged | Scope::All, Some(Folder::Archive)) => Some("In Archive"),
            (_, None) => Some("New draft"),
            _ => None,
        };

        h_flex()
            .h_12()
            .flex_none()
            .px_4()
            .gap_1()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(
                div()
                    .flex_1()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .children(location),
            )
            .map(|this| {
                if in_trash {
                    this.child(Self::trash_buttons(cx))
                } else {
                    let archived = folder == Some(Folder::Archive);
                    this.child(Self::draft_buttons(exists, flagged, archived, cx))
                }
            })
    }

    /// Flag, archive and trash commands for a draft outside the trash.
    fn draft_buttons(
        exists: bool,
        flagged: bool,
        archived: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        h_flex()
            .gap_1()
            .child(
                Button::new("toggle-flag")
                    .ghost()
                    .small()
                    .icon(IconName::Flag)
                    .selected(flagged)
                    .toggled(flagged)
                    .disabled(!exists)
                    .tooltip_with_action(
                        if flagged { "Unflag" } else { "Flag" },
                        &commands::ToggleFlag,
                        Some(DRAFTS_CONTEXT),
                    )
                    .on_click(cx.listener(|this, _, window, cx| this.toggle_flag(window, cx))),
            )
            .child(
                Button::new("toggle-archive")
                    .ghost()
                    .small()
                    .icon(if archived {
                        IconName::ArchiveRestore
                    } else {
                        IconName::Archive
                    })
                    .disabled(!exists)
                    .tooltip_with_action(
                        if archived { "Move to Inbox" } else { "Archive" },
                        &commands::ToggleArchive,
                        Some(DRAFTS_CONTEXT),
                    )
                    .on_click(cx.listener(|this, _, window, cx| this.toggle_archive(window, cx))),
            )
            .child(
                Button::new("move-to-trash")
                    .ghost()
                    .small()
                    .icon(IconName::Trash)
                    .disabled(!exists)
                    .tooltip_with_action(
                        "Move to Trash",
                        &commands::ToggleTrash,
                        Some(DRAFTS_CONTEXT),
                    )
                    .on_click(cx.listener(|this, _, window, cx| this.toggle_trash(window, cx))),
            )
    }

    /// Restore and delete commands for a draft in the trash.
    fn trash_buttons(cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .gap_1()
            .child(
                Button::new("restore")
                    .ghost()
                    .small()
                    .icon(ComponentIconName::Undo2)
                    .label("Restore")
                    .tooltip_with_action(
                        "Move to Inbox",
                        &commands::ToggleTrash,
                        Some(DRAFTS_CONTEXT),
                    )
                    .on_click(cx.listener(|this, _, window, cx| this.toggle_trash(window, cx))),
            )
            .child(
                Button::new("delete")
                    .ghost()
                    .small()
                    .label("Delete…")
                    .tooltip_with_action(
                        "Delete permanently",
                        &commands::DeleteDraft,
                        Some(DRAFTS_CONTEXT),
                    )
                    .on_click(cx.listener(|this, _, window, cx| this.confirm_delete(window, cx))),
            )
    }

    fn render_status_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let editor = self.editor.read(cx);
        let stats = editor.stats();
        let words = plural(stats.words(), "word", "words");
        let characters = plural(stats.characters(), "character", "characters");
        let modified: Option<SharedString> = editor
            .draft()
            .map(|draft| format!("Modified {}", time_format::relative(draft.modified_at())).into());
        let error = editor.error().map(ToOwned::to_owned);

        h_flex()
            .h_8()
            .flex_none()
            .px_4()
            .gap_4()
            .border_t_1()
            .border_color(cx.theme().border)
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .child(format!("{words} · {characters}"))
            .child(div().flex_1())
            .map(|this| match error {
                Some(error) => this.child(div().text_color(cx.theme().danger).child(error)),
                None => this.children(modified),
            })
    }
}

impl Focusable for DraftsWindow {
    fn focus_handle(&self, _: &gpui_kit::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for DraftsWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let list_width = window.rem_size() * 22.;

        v_flex()
            .key_context(DRAFTS_CONTEXT)
            .track_focus(&self.focus_handle)
            .size_full()
            .text_color(cx.theme().foreground)
            .on_action(cx.listener(|this, _: &commands::NewDraft, window, cx| {
                this.start_new_draft(window, cx);
            }))
            .on_action(cx.listener(|this, _: &commands::FocusSearch, window, cx| {
                this.list.focus_handle(cx).focus(window, cx);
            }))
            .on_action(cx.listener(|this, _: &commands::FocusList, window, cx| {
                this.editor.update(cx, DraftEditor::save);
                this.list.focus_handle(cx).focus(window, cx);
            }))
            .on_action(cx.listener(|this, _: &commands::ToggleFlag, window, cx| {
                this.toggle_flag(window, cx);
            }))
            .on_action(
                cx.listener(|this, _: &commands::ToggleArchive, window, cx| {
                    this.toggle_archive(window, cx);
                }),
            )
            .on_action(cx.listener(|this, _: &commands::ToggleTrash, window, cx| {
                this.toggle_trash(window, cx);
            }))
            .on_action(cx.listener(|this, _: &commands::DeleteDraft, window, cx| {
                this.confirm_delete(window, cx);
            }))
            .on_action(cx.listener(|this, _: &commands::EmptyTrash, window, cx| {
                this.confirm_empty_trash(window, cx);
            }))
            .on_action(cx.listener(|this, _: &commands::ShowInbox, window, cx| {
                this.set_scope(Scope::Inbox, window, cx);
            }))
            .on_action(cx.listener(|this, _: &commands::ShowFlagged, window, cx| {
                this.set_scope(Scope::Flagged, window, cx);
            }))
            .on_action(cx.listener(|this, _: &commands::ShowArchive, window, cx| {
                this.set_scope(Scope::Archive, window, cx);
            }))
            .on_action(cx.listener(|this, _: &commands::ShowAll, window, cx| {
                this.set_scope(Scope::All, window, cx);
            }))
            .on_action(cx.listener(|this, _: &commands::ShowTrash, window, cx| {
                this.set_scope(Scope::Trash, window, cx);
            }))
            .on_action(
                cx.listener(|this, _: &commands::CloseWindow, window, cx| this.close(window, cx)),
            )
            .child(
                TitleBar::new()
                    .on_close_window(cx.listener(|this, _, window, cx| this.close(window, cx)))
                    .map(|this| {
                        // macOS shows the menus in the system menu bar.
                        if cfg!(target_os = "macos") {
                            this.child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .child("Dashtext"),
                            )
                        } else {
                            this.child(self.app_menu_bar.clone())
                        }
                    }),
            )
            .child(
                h_flex()
                    .flex_1()
                    .min_h_0()
                    .items_stretch()
                    .child(self.render_sidebar(cx))
                    .child(
                        div().flex_1().min_w_0().h_full().child(
                            h_resizable("drafts-split")
                                .child(
                                    resizable_panel()
                                        .size(list_width)
                                        .size_range(
                                            window.rem_size() * 16.0..window.rem_size() * 36.0,
                                        )
                                        .child(self.render_list_pane(cx)),
                                )
                                .child(
                                    resizable_panel().child(
                                        v_flex()
                                            .size_full()
                                            .min_w_0()
                                            .child(self.render_toolbar(cx))
                                            .child(self.editor.clone().cached(
                                                StyleRefinement::default().flex_1().min_h_0(),
                                            ))
                                            .child(self.render_status_bar(cx)),
                                    ),
                                ),
                        ),
                    ),
            )
    }
}

fn load_drafts(library: &Entity<Library>, workspace: &Workspace, cx: &gpui_kit::App) -> Vec<Draft> {
    library
        .read(cx)
        .drafts(workspace.scope(), workspace.sort())
        .unwrap_or_else(|error| {
            log::error!("could not load drafts: {error:#}");
            Vec::new()
        })
}

fn scope_label(scope: Scope) -> &'static str {
    match scope {
        Scope::Inbox => "Inbox",
        Scope::Flagged => "Flagged",
        Scope::Archive => "Archive",
        Scope::All => "All",
        Scope::Trash => "Trash",
    }
}

fn scope_icon(scope: Scope) -> IconName {
    match scope {
        Scope::Inbox => IconName::Inbox,
        Scope::Flagged => IconName::Flag,
        Scope::Archive => IconName::Archive,
        Scope::All => IconName::FileText,
        Scope::Trash => IconName::Trash,
    }
}

fn plural(count: usize, one: &str, many: &str) -> String {
    format!("{count} {}", if count == 1 { one } else { many })
}

fn truncate(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        text.to_owned()
    } else {
        let mut truncated: String = text.chars().take(max_chars - 1).collect();
        truncated.push('…');
        truncated
    }
}

/// Shows a recoverable error to the user and logs the details.
fn report(window: &mut Window, cx: &mut gpui_kit::App, message: &str, error: &anyhow::Error) {
    log::error!("{message} {error:#}");
    window.push_notification(Notification::error(message.to_owned()), cx);
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use dashtext_core::Scope;
    use gpui_kit::AppContext as _;
    use gpui_kit::Entity;
    use gpui_kit::TestAppContext;
    use gpui_kit::WindowHandle;
    use gpui_kit::base::Root;
    use gpui_kit::test::TestWindowExt as _;

    use super::DraftsWindow;
    use crate::test_support;

    fn open_drafts(cx: &mut TestAppContext) -> (WindowHandle<Root>, Entity<DraftsWindow>) {
        test_support::open(cx, |window, cx| cx.new(|cx| DraftsWindow::new(window, cx)))
    }

    fn open_content(cx: &mut TestAppContext, drafts: &Entity<DraftsWindow>) -> Option<String> {
        cx.update(|cx| {
            drafts
                .read(cx)
                .current_draft(cx)
                .map(|draft| draft.content().to_owned())
        })
    }

    fn type_keys(cx: &mut TestAppContext, window: WindowHandle<Root>, keys: &[&str]) {
        cx.update_window(window.into(), |_, window, cx| {
            for key in keys {
                match key.strip_prefix("text:") {
                    Some(text) => window.input(text, cx),
                    None => window.press(key, cx),
                }
            }
        })
        .expect("drafts window is open");
        // Let the autosave debounce elapse.
        cx.executor().advance_clock(Duration::from_secs(1));
        cx.run_until_parked();
    }

    #[gpui_kit::test]
    fn typing_in_a_new_draft_adds_it_to_the_inbox(cx: &mut TestAppContext) {
        let _library = test_support::init(cx, &[]);
        let (window, drafts) = open_drafts(cx);
        assert_eq!(
            open_content(cx, &drafts),
            None,
            "an empty library opens a new draft"
        );

        type_keys(cx, window, &["text:Groceries", "enter", "text:oat milk"]);

        assert_eq!(
            test_support::contents(cx, Scope::Inbox),
            ["Groceries\noat milk"]
        );
        assert_eq!(
            open_content(cx, &drafts).as_deref(),
            Some("Groceries\noat milk")
        );
    }

    #[gpui_kit::test]
    fn drafts_left_blank_are_discarded(cx: &mut TestAppContext) {
        let _library = test_support::init(cx, &[]);
        let (window, _) = open_drafts(cx);

        type_keys(cx, window, &["text:x"]);
        assert_eq!(test_support::contents(cx, Scope::Inbox), ["x"]);

        type_keys(cx, window, &["backspace", "secondary-n"]);
        assert!(test_support::contents(cx, Scope::All).is_empty());
    }

    #[gpui_kit::test]
    fn archiving_moves_on_to_the_next_draft(cx: &mut TestAppContext) {
        let _library = test_support::init(cx, &["Older", "Newer"]);
        let (window, drafts) = open_drafts(cx);
        assert_eq!(open_content(cx, &drafts).as_deref(), Some("Newer"));

        type_keys(cx, window, &["secondary-shift-a"]);

        assert_eq!(test_support::contents(cx, Scope::Inbox), ["Older"]);
        assert_eq!(test_support::contents(cx, Scope::Archive), ["Newer"]);
        assert_eq!(open_content(cx, &drafts).as_deref(), Some("Older"));
    }

    #[gpui_kit::test]
    fn trash_and_restore(cx: &mut TestAppContext) {
        let _library = test_support::init(cx, &["Keep me"]);
        let (window, drafts) = open_drafts(cx);

        type_keys(cx, window, &["secondary-shift-backspace", "secondary-5"]);
        assert_eq!(test_support::contents(cx, Scope::Trash), ["Keep me"]);
        assert_eq!(open_content(cx, &drafts).as_deref(), Some("Keep me"));

        type_keys(cx, window, &["secondary-shift-backspace"]);
        assert_eq!(test_support::contents(cx, Scope::Inbox), ["Keep me"]);
    }

    #[gpui_kit::test]
    fn unflagging_in_flagged_moves_on_to_the_next_draft(cx: &mut TestAppContext) {
        let _library = test_support::init(cx, &["Keep", "Unflag me"]);
        let (window, drafts) = open_drafts(cx);

        // Flag both, then view only flagged drafts.
        type_keys(cx, window, &["secondary-shift-l"]);
        type_keys(cx, window, &["escape"]);
        type_keys(cx, window, &["down"]);
        type_keys(cx, window, &["secondary-shift-l"]);
        type_keys(cx, window, &["secondary-2"]);
        assert_eq!(
            test_support::contents(cx, Scope::Flagged),
            ["Unflag me", "Keep"]
        );
        assert_eq!(open_content(cx, &drafts).as_deref(), Some("Unflag me"));

        type_keys(cx, window, &["secondary-shift-l"]);

        assert_eq!(test_support::contents(cx, Scope::Flagged), ["Keep"]);
        assert_eq!(open_content(cx, &drafts).as_deref(), Some("Keep"));
    }

    #[gpui_kit::test]
    fn a_blank_draft_moved_to_the_trash_is_kept(cx: &mut TestAppContext) {
        let _library = test_support::init(cx, &["Other"]);
        let (window, _) = open_drafts(cx);

        // One step at a time: each waits for focus changes and autosave.
        type_keys(cx, window, &["secondary-n"]);
        type_keys(cx, window, &["text:x"]);
        type_keys(cx, window, &["backspace"]);
        type_keys(cx, window, &["secondary-shift-backspace"]);

        assert_eq!(test_support::contents(cx, Scope::Trash), [""]);
    }

    #[gpui_kit::test]
    fn search_filters_the_list(cx: &mut TestAppContext) {
        let _library =
            test_support::init(cx, &["Buy oat milk", "Call the dentist", "Milk the cow"]);
        let (window, drafts) = open_drafts(cx);

        type_keys(cx, window, &["secondary-f", "text:milk -cow"]);

        let matches = cx.update(|cx| {
            let list = drafts.read(cx).list.read(cx).delegate();
            (0..list.len())
                .filter_map(|row| list.draft_at(gpui_kit::component::IndexPath::new(row)))
                .map(|draft| draft.content().to_owned())
                .collect::<Vec<_>>()
        });
        assert_eq!(matches, ["Buy oat milk"]);
    }

    #[gpui_kit::test]
    #[ignore = "benchmark: cargo test -p dashtext bench_ -- --ignored --nocapture"]
    fn bench_typing_and_navigation(cx: &mut TestAppContext) {
        use std::time::Instant;

        let seed: Vec<String> = (0..300)
            .map(|ix| format!("Draft {ix}\n{}", "lorem ipsum dolor sit amet ".repeat(20)))
            .collect();
        let seed: Vec<&str> = seed.iter().map(String::as_str).collect();
        let _library = test_support::init(cx, &seed);
        let (window, _) = open_drafts(cx);

        let text = "The quick brown fox jumps over";
        let started = Instant::now();
        cx.update_window(window.into(), |_, window, cx| {
            for ch in text.chars() {
                window.input(&ch.to_string(), cx);
            }
        })
        .expect("drafts window is open");
        let typing = started.elapsed() / u32::try_from(text.len()).expect("short text");

        let started = Instant::now();
        cx.executor().advance_clock(Duration::from_secs(1));
        cx.run_until_parked();
        let autosave = started.elapsed();

        cx.update_window(window.into(), |_, window, cx| window.press("escape", cx))
            .expect("drafts window is open");
        let started = Instant::now();
        cx.update_window(window.into(), |_, window, cx| {
            for _ in 0..25 {
                window.press("down", cx);
            }
        })
        .expect("drafts window is open");
        let navigation = started.elapsed() / 25;

        eprintln!("BENCH typing/key={typing:?} autosave={autosave:?} list-step={navigation:?}");
    }
}
