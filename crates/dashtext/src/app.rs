//! Application-level window management and request handling.

use gpui_kit::App;
use gpui_kit::AppContext as _;
use gpui_kit::Bounds;
use gpui_kit::Focusable as _;
use gpui_kit::Global;
use gpui_kit::Pixels;
use gpui_kit::Size;
use gpui_kit::WeakEntity;
use gpui_kit::Window;
use gpui_kit::WindowBackgroundAppearance;
use gpui_kit::WindowBounds;
use gpui_kit::WindowDecorations;
use gpui_kit::WindowHandle;
use gpui_kit::WindowKind;
use gpui_kit::WindowOptions;
use gpui_kit::base::Root;
use gpui_kit::component::TitleBar;
use gpui_kit::component::WindowExt as _;
use gpui_kit::component::notification::Notification;
use gpui_kit::px;
use gpui_kit::size;

use crate::capture::CaptureWindow;
use crate::commands;
use crate::drafts::DraftsWindow;
use crate::instance::Request;
use crate::library::Library;

/// Application identifier: the Wayland app ID and the desktop entry name.
pub const APP_ID: &str = "app.dashtext.Dashtext";

/// The open windows. Each exists at most once; asking for it again brings
/// the existing one forward.
#[derive(Default)]
struct Windows {
    drafts: Option<WindowHandle<Root>>,
    capture: Option<(WindowHandle<Root>, WeakEntity<CaptureWindow>)>,
}

impl Global for Windows {}

pub fn init(cx: &mut App) {
    cx.set_global(Windows::default());

    cx.on_action(|_: &commands::Quit, cx| cx.quit());
    cx.on_action(|_: &commands::QuickCapture, cx| open_capture(cx));

    // Linux and Windows have no application that outlives its windows; on
    // macOS the app stays running without windows, as Mac apps do.
    #[cfg(not(target_os = "macos"))]
    cx.on_window_closed(|cx, _| {
        if cx.windows().is_empty() {
            cx.quit();
        }
    })
    .detach();
    // Clicking the Dock icon with no windows open brings the drafts back.
    #[cfg(target_os = "macos")]
    cx.on_reopen(|cx| {
        if cx.windows().is_empty() {
            open_drafts(cx);
        }
    });
}

pub fn handle(request: Request, cx: &mut App) {
    match request {
        Request::Open => open_drafts(cx),
        Request::Capture => open_capture(cx),
        Request::Reload => Library::global(cx).update(cx, Library::reload),
    }
}

pub fn open_drafts(cx: &mut App) {
    if let Some(handle) = cx.global::<Windows>().drafts
        && handle
            .update(cx, |_, window, _| window.activate_window())
            .is_ok()
    {
        return;
    }

    let bounds = Bounds::centered(None, fit_to_display(size(px(1100.), px(720.)), cx), cx);
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        window_min_size: Some(size(px(640.), px(400.))),
        ..window_options()
    };
    match gpui_kit::open_window(options, cx, |window, cx| {
        window.set_window_title("Dashtext");
        cx.new(|cx| DraftsWindow::new(window, cx))
    }) {
        Ok((handle, _)) => {
            if let Err(error) =
                handle.update(cx, |_, window, cx| warn_if_software_rendering(window, cx))
            {
                log::debug!("drafts window closed immediately: {error}");
            }
            cx.global_mut::<Windows>().drafts = handle.downcast::<Root>();
        }
        Err(error) => log::error!("could not open the drafts window: {error:#}"),
    }
}

pub fn open_capture(cx: &mut App) {
    if let Some((handle, view)) = cx.global::<Windows>().capture.clone()
        && let Some(view) = view.upgrade()
        && handle
            .update(cx, |_, window, cx| {
                window.activate_window();
                view.update(cx, |view, cx| view.focus_input(window, cx));
            })
            .is_ok()
    {
        return;
    }

    let bounds = Bounds::centered(None, size(px(560.), px(320.)), cx);
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        window_min_size: Some(size(px(360.), px(200.))),
        kind: capture_window_kind(),
        is_minimizable: false,
        // A bare panel: no title bar, closed with Esc or its Close button.
        titlebar: None,
        app_owns_titlebar_drag: false,
        ..window_options()
    };
    match gpui_kit::open_window(options, cx, |window, cx| {
        window.set_window_title("Quick Capture");
        let view = cx.new(|cx| CaptureWindow::new(window, cx));
        let focus = view.focus_handle(cx);
        window.defer(cx, move |window, cx| {
            if window.focused(cx).is_none() {
                focus.focus(window, cx);
            }
        });
        view
    }) {
        Ok((handle, view)) => {
            if let Some(handle) = handle.downcast::<Root>() {
                cx.global_mut::<Windows>().capture = Some((handle, view.downgrade()));
            }
        }
        Err(error) => log::error!("could not open the capture window: {error:#}"),
    }
}

/// On macOS a pop-up is a floating panel that takes key focus without
/// activating the app, which suits quick capture. On X11 a pop-up is an
/// override-redirect window that window managers do not focus or move, so
/// Linux (and Windows) use an ordinary window.
fn capture_window_kind() -> WindowKind {
    if cfg!(target_os = "macos") {
        WindowKind::PopUp
    } else {
        WindowKind::Normal
    }
}

/// Explains the sluggishness when rendering falls back to a CPU "GPU" such as
/// llvmpipe (no GPU driver, or a VM), as Zed does. Every frame is then drawn
/// on the CPU, which no amount of app-side work can hide.
fn warn_if_software_rendering(window: &mut Window, cx: &mut App) {
    let Some(specs) = window.gpu_specs() else {
        return;
    };
    if !specs.is_software_emulated || std::env::var_os("DASHTEXT_ALLOW_EMULATED_GPU").is_some() {
        return;
    }
    log::warn!("rendering on a software GPU: {}", specs.device_name);
    window.push_notification(
        Notification::warning(format!(
            "Dashtext is drawing on the CPU ({}), so it will feel slow. Install a GPU driver \
             with Vulkan support for smooth rendering. Set DASHTEXT_ALLOW_EMULATED_GPU=1 to \
             hide this message.",
            specs.device_name
        ))
        .title("No GPU acceleration")
        .autohide(false),
        cx,
    );
}

/// Shrinks a preferred window size to leave a margin on small displays.
fn fit_to_display(preferred: Size<Pixels>, cx: &App) -> Size<Pixels> {
    let Some(display) = cx.primary_display() else {
        return preferred;
    };
    let available = display.bounds().size;
    size(
        preferred.width.min(available.width * 0.9),
        preferred.height.min(available.height * 0.9),
    )
}

fn window_options() -> WindowOptions {
    WindowOptions {
        app_id: Some(APP_ID.to_owned()),
        window_background: WindowBackgroundAppearance::Opaque,
        // Draw the title bar ourselves on Linux so it matches the app on
        // every compositor, including those without server-side decorations.
        window_decorations: cfg!(target_os = "linux").then_some(WindowDecorations::Client),
        ..TitleBar::window_options()
    }
}
