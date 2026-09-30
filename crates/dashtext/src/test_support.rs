//! Shared setup for UI integration tests.

use dashtext_core::Scope;
use dashtext_core::Sort;
use dashtext_core::Store;
use gpui_kit::App;
use gpui_kit::Bounds;
use gpui_kit::Entity;
use gpui_kit::Point;
use gpui_kit::Render;
use gpui_kit::TestAppContext;
use gpui_kit::Window;
use gpui_kit::WindowBounds;
use gpui_kit::WindowHandle;
use gpui_kit::WindowOptions;
use gpui_kit::base::Root;
use gpui_kit::px;
use gpui_kit::size;
use tempfile::TempDir;

use crate::commands;
use crate::library::Library;

/// Initializes the app against a library in a temporary directory, adding
/// `seed` drafts to the inbox (oldest first). Keep the directory alive for
/// the test's duration.
pub fn init(cx: &mut TestAppContext, seed: &[&str]) -> TempDir {
    let dir = tempfile::tempdir().expect("create temp dir");
    let store = Store::open(&dir.path().join("library.db")).expect("open library");
    for content in seed {
        store
            .create_draft(content, dashtext_core::Folder::Inbox, false)
            .expect("seed draft");
        // Distinct modification times keep the list order deterministic.
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    cx.update(|cx| {
        gpui_kit::init(cx);
        commands::bind_keys(cx);
        Library::init(store, cx);
    });
    dir
}

/// Opens `build`'s view in a production-style window (with `Root`).
pub fn open<V: Render>(
    cx: &mut TestAppContext,
    build: impl FnOnce(&mut Window, &mut App) -> Entity<V>,
) -> (WindowHandle<Root>, Entity<V>) {
    let (window, view) = cx.update(|cx| {
        let bounds = Bounds {
            origin: Point::default(),
            size: size(px(1100.), px(720.)),
        };
        let (window, view) = gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            cx,
            build,
        )
        .expect("open test window");
        (window.downcast::<Root>().expect("Base Root"), view)
    });
    cx.run_until_parked();
    (window, view)
}

/// The contents of the drafts in `scope`, newest first.
pub fn contents(cx: &mut TestAppContext, scope: Scope) -> Vec<String> {
    cx.update(|cx| {
        Library::global(cx)
            .read(cx)
            .drafts(scope, Sort::default())
            .expect("list drafts")
            .iter()
            .map(|draft| draft.content().to_owned())
            .collect()
    })
}
