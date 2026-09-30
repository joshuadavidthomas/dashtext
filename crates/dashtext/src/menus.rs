//! The application menus: native on macOS, drawn in the title bar elsewhere.

use dashtext_core::Folder;
use gpui_kit::App;
use gpui_kit::Entity;
use gpui_kit::Menu;
use gpui_kit::MenuItem;
use gpui_kit::component::GlobalState;
use gpui_kit::component::input;
use gpui_kit::component::menu::AppMenuBar;

use crate::commands;

/// What the menus need to know to label and enable draft commands.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MenuState {
    /// The folder and flag of the open draft, if it is saved.
    pub draft: Option<(Folder, bool)>,
    pub trash_is_empty: bool,
}

/// Installs the menus for `state` and refreshes `menu_bar` to show them.
pub fn install(state: MenuState, menu_bar: &Entity<AppMenuBar>, cx: &mut App) {
    // Bindings must exist before `set_menus` so each item shows its shortcut.
    cx.set_menus(build(state));
    GlobalState::global_mut(cx).set_app_menus(build(state).into_iter().map(Menu::owned).collect());
    menu_bar.update(cx, AppMenuBar::reload);
}

fn build(state: MenuState) -> Vec<Menu> {
    let folder = state.draft.map(|(folder, _)| folder);
    let flagged = state.draft.is_some_and(|(_, flagged)| flagged);
    let no_draft = state.draft.is_none();
    let in_trash = folder == Some(Folder::Trash);

    vec![
        Menu::new("File").items([
            MenuItem::action("New Draft", commands::NewDraft),
            MenuItem::action("Quick Capture…", commands::QuickCapture),
            MenuItem::separator(),
            MenuItem::action("Close Window", commands::CloseWindow),
            MenuItem::action("Quit", commands::Quit),
        ]),
        Menu::new("Edit").items([
            MenuItem::action("Undo", input::Undo),
            MenuItem::action("Redo", input::Redo),
            MenuItem::separator(),
            MenuItem::action("Cut", input::Cut),
            MenuItem::action("Copy", input::Copy),
            MenuItem::action("Paste", input::Paste),
            MenuItem::separator(),
            MenuItem::action("Select All", input::SelectAll),
        ]),
        Menu::new("Draft").items([
            MenuItem::action(
                if flagged { "Unflag" } else { "Flag" },
                commands::ToggleFlag,
            )
            .disabled(no_draft || in_trash),
            MenuItem::action(
                if folder == Some(Folder::Inbox) {
                    "Archive"
                } else {
                    "Move to Inbox"
                },
                commands::ToggleArchive,
            )
            .disabled(no_draft || in_trash),
            MenuItem::action(
                if in_trash { "Restore" } else { "Move to Trash" },
                commands::ToggleTrash,
            )
            .disabled(no_draft),
            MenuItem::separator(),
            MenuItem::action("Delete…", commands::DeleteDraft).disabled(!in_trash),
            MenuItem::action("Empty Trash…", commands::EmptyTrash).disabled(state.trash_is_empty),
        ]),
        Menu::new("View").items([
            MenuItem::action("Inbox", commands::ShowInbox),
            MenuItem::action("Flagged", commands::ShowFlagged),
            MenuItem::action("Archive", commands::ShowArchive),
            MenuItem::action("All", commands::ShowAll),
            MenuItem::action("Trash", commands::ShowTrash),
            MenuItem::separator(),
            MenuItem::action("Search", commands::FocusSearch),
        ]),
    ]
}
