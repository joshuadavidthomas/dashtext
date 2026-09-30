//! Keyboard commands and their default bindings.
//!
//! Buttons, menus and key bindings dispatch these actions so each command is
//! implemented once, by the view that owns it.

use gpui_kit::App;
use gpui_kit::KeyBinding;

/// Key context of the drafts window.
pub const DRAFTS_CONTEXT: &str = "DraftsWindow";
/// Key context of the quick capture window.
pub const CAPTURE_CONTEXT: &str = "CaptureWindow";
/// Key context of the draft editor inside the drafts window.
pub const EDITOR_CONTEXT: &str = "DraftEditor";

gpui_kit::actions!(
    dashtext,
    [
        /// Quit the application.
        Quit,
        /// Close the focused window.
        CloseWindow,
        /// Open the quick capture window.
        QuickCapture,
        /// Start a new draft in the drafts window.
        NewDraft,
        /// Move focus to the search field of the draft list.
        FocusSearch,
        /// Move focus to the draft list.
        FocusList,
        /// Flag or unflag the current draft.
        ToggleFlag,
        /// Archive the current draft, or return an archived draft to the inbox.
        ToggleArchive,
        /// Move the current draft to the trash, or restore it from the trash.
        ToggleTrash,
        /// Permanently delete the current draft (trash only).
        DeleteDraft,
        /// Permanently delete every draft in the trash.
        EmptyTrash,
        /// Show the inbox.
        ShowInbox,
        /// Show flagged drafts.
        ShowFlagged,
        /// Show the archive.
        ShowArchive,
        /// Show every draft outside the trash.
        ShowAll,
        /// Show the trash.
        ShowTrash,
        /// Save the quick capture text to the inbox.
        SaveCapture,
        /// Hide the quick capture window, keeping its text.
        DismissCapture,
    ]
);

pub fn bind_keys(cx: &mut App) {
    let drafts = Some(DRAFTS_CONTEXT);
    let capture = Some(CAPTURE_CONTEXT);
    let editor = Some(EDITOR_CONTEXT);

    cx.bind_keys([
        KeyBinding::new("secondary-q", Quit, None),
        KeyBinding::new("secondary-w", CloseWindow, None),
        KeyBinding::new("secondary-shift-n", QuickCapture, None),
        KeyBinding::new("secondary-n", NewDraft, drafts),
        KeyBinding::new("secondary-f", FocusSearch, drafts),
        KeyBinding::new("secondary-shift-l", ToggleFlag, drafts),
        KeyBinding::new("secondary-shift-a", ToggleArchive, drafts),
        KeyBinding::new("secondary-shift-backspace", ToggleTrash, drafts),
        KeyBinding::new("secondary-shift-delete", DeleteDraft, drafts),
        KeyBinding::new("secondary-1", ShowInbox, drafts),
        KeyBinding::new("secondary-2", ShowFlagged, drafts),
        KeyBinding::new("secondary-3", ShowArchive, drafts),
        KeyBinding::new("secondary-4", ShowAll, drafts),
        KeyBinding::new("secondary-5", ShowTrash, drafts),
        KeyBinding::new("escape", FocusList, editor),
        KeyBinding::new("secondary-shift-l", ToggleFlag, capture),
        KeyBinding::new("escape", DismissCapture, capture),
        // The text field binds `secondary-enter` itself (to insert a line and
        // report it). Binding in the same context but later takes precedence;
        // where no view handles `SaveCapture`, dispatch falls through to the
        // field's own binding.
        KeyBinding::new("secondary-enter", SaveCapture, Some("Input")),
        KeyBinding::new("secondary-enter", SaveCapture, capture),
    ]);
}
