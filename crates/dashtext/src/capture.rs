//! The quick capture window: type, save to the inbox, get back to work.

use std::time::Duration;

use gpui_kit::Action as _;
use gpui_kit::AppContext as _;
use gpui_kit::Context;
use gpui_kit::Entity;
use gpui_kit::FocusHandle;
use gpui_kit::Focusable;
use gpui_kit::InteractiveElement as _;
use gpui_kit::IntoElement;
use gpui_kit::MouseButton;
use gpui_kit::ParentElement as _;
use gpui_kit::Render;
use gpui_kit::Styled as _;
use gpui_kit::Subscription;
use gpui_kit::Task;
use gpui_kit::Window;
use gpui_kit::WindowControlArea;
use gpui_kit::assets::IconName;
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::Disableable as _;
use gpui_kit::component::Selectable as _;
use gpui_kit::component::Sizable as _;
use gpui_kit::component::Theme;
use gpui_kit::component::WindowExt as _;
use gpui_kit::component::button::Button;
use gpui_kit::component::button::ButtonVariants as _;
use gpui_kit::component::h_flex;
use gpui_kit::component::input::InputEvent;
use gpui_kit::component::input::Textarea;
use gpui_kit::component::input::TextareaState;
use gpui_kit::component::notification::Notification;
use gpui_kit::component::v_flex;
use gpui_kit::div;

use crate::commands;
use crate::commands::CAPTURE_CONTEXT;
use crate::library::Library;

/// How long typing must pause before unsaved text is stashed.
const STASH_DELAY: Duration = Duration::from_millis(500);

/// A small window for getting text into the inbox quickly.
///
/// Unsaved text survives closing the window and restarting the app, so a
/// capture is only ever lost by clearing it deliberately.
pub struct CaptureWindow {
    library: Entity<Library>,
    input: Entity<TextareaState>,
    flagged: bool,
    stash_task: Task<()>,
    focus_handle: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

impl CaptureWindow {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let library = Library::global(cx);
        let stashed = library.read(cx).capture_buffer().unwrap_or_else(|error| {
            log::warn!("could not restore unsaved capture text: {error:#}");
            String::new()
        });
        let input = cx.new(|cx| {
            let mut input = TextareaState::new(window, cx)
                .soft_wrap(true)
                .placeholder("Capture a thought…");
            let end = stashed.len();
            input.set_value(stashed, window, cx);
            input.set_selected_range(end..end, cx);
            input
        });
        let subscriptions = vec![
            cx.subscribe_in(&input, window, |this, _, event, window, cx| {
                if matches!(event, InputEvent::Change) {
                    this.schedule_stash(window, cx);
                    cx.notify();
                }
            }),
            cx.observe_window_appearance(window, |_, window, cx| {
                Theme::sync_system_appearance(Some(window), cx);
            }),
            cx.on_app_quit(|this, cx| {
                this.stash(cx);
                async {}
            }),
        ];
        input.update(cx, |input, cx| input.focus(window, cx));

        // Closing from the compositor (rather than Esc or the title bar)
        // must keep the text too.
        let view = cx.weak_entity();
        window.on_window_should_close(cx, move |_, cx| {
            if let Err(error) = view.update(cx, Self::stash) {
                log::debug!("capture view released before closing: {error}");
            }
            true
        });

        Self {
            library,
            input,
            flagged: false,
            stash_task: Task::ready(()),
            focus_handle: cx.focus_handle(),
            _subscriptions: subscriptions,
        }
    }

    /// Focuses the text, for when the window is shown again.
    pub fn focus_input(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.input.update(cx, |input, cx| input.focus(window, cx));
    }

    fn text(&self, cx: &gpui_kit::App) -> String {
        self.input.read(cx).value().to_string()
    }

    fn save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let text = self.text(cx);
        if text.trim().is_empty() {
            return;
        }
        let flagged = self.flagged;
        let result = self.library.update(cx, |library, cx| {
            library.create(text.trim_end(), flagged, cx)
        });
        match result {
            Ok(_) => {
                self.flagged = false;
                self.input
                    .update(cx, |input, cx| input.set_value("", window, cx));
                self.stash(cx);
                window.remove_window();
            }
            Err(error) => {
                log::error!("could not save capture: {error:#}");
                window.push_notification(
                    Notification::error("Couldn’t save to the inbox. Your text is kept here."),
                    cx,
                );
            }
        }
    }

    fn dismiss(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.stash(cx);
        window.remove_window();
    }

    fn schedule_stash(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.stash_task = cx.spawn_in(window, async move |this, cx| {
            cx.background_executor().timer(STASH_DELAY).await;
            if let Err(error) = this.update(cx, Self::stash) {
                log::debug!("capture window closed before stashing: {error}");
            }
        });
    }

    fn stash(&mut self, cx: &mut Context<Self>) {
        self.stash_task = Task::ready(());
        let text = self.text(cx);
        if let Err(error) = self.library.read(cx).set_capture_buffer(&text) {
            log::warn!("could not stash capture text: {error:#}");
        }
    }
}

impl Focusable for CaptureWindow {
    fn focus_handle(&self, _: &gpui_kit::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for CaptureWindow {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let empty = self.text(cx).trim().is_empty();
        let flagged = self.flagged;

        v_flex()
            .key_context(CAPTURE_CONTEXT)
            .track_focus(&self.focus_handle)
            .size_full()
            .text_color(cx.theme().foreground)
            .on_action(
                cx.listener(|this, _: &commands::SaveCapture, window, cx| this.save(window, cx)),
            )
            .on_action(
                cx.listener(|this, _: &commands::DismissCapture, window, cx| {
                    this.dismiss(window, cx);
                }),
            )
            .on_action(cx.listener(|this, _: &commands::CloseWindow, window, cx| {
                this.dismiss(window, cx);
            }))
            .on_action(cx.listener(|this, _: &commands::ToggleFlag, _, cx| {
                this.flagged = !this.flagged;
                cx.notify();
            }))
            .child(
                div().flex_1().min_h_0().child(
                    Textarea::new(&self.input)
                        .appearance(false)
                        .h_full()
                        .px_4()
                        .py_3()
                        .text_base(),
                ),
            )
            .child(
                h_flex()
                    .flex_none()
                    .px_3()
                    .py_2()
                    .gap_2()
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .child(
                        Button::new("capture-flag")
                            .ghost()
                            .small()
                            .icon(IconName::Flag)
                            .selected(flagged)
                            .toggled(flagged)
                            .tooltip_with_action(
                                if flagged { "Unflag" } else { "Flag" },
                                &commands::ToggleFlag,
                                Some(CAPTURE_CONTEXT),
                            )
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.flagged = !this.flagged;
                                cx.notify();
                            })),
                    )
                    // No title bar: the empty stretch of the footer moves the window.
                    .child(
                        div()
                            .id("capture-drag-area")
                            .flex_1()
                            .h_full()
                            .window_control_area(WindowControlArea::Drag)
                            .on_mouse_down(MouseButton::Left, |_, window, _| {
                                window.start_window_move();
                            }),
                    )
                    .child(
                        Button::new("capture-cancel")
                            .ghost()
                            .small()
                            .label("Close")
                            .tooltip("Close and keep the text for later")
                            .on_click(|_, window, cx| {
                                window.dispatch_action(commands::DismissCapture.boxed_clone(), cx);
                            }),
                    )
                    .child(
                        Button::new("capture-save")
                            .primary()
                            .small()
                            .label("Save to Inbox")
                            .disabled(empty)
                            .tooltip_with_action(
                                "Save to Inbox",
                                &commands::SaveCapture,
                                Some(CAPTURE_CONTEXT),
                            )
                            .on_click(|_, window, cx| {
                                window.dispatch_action(commands::SaveCapture.boxed_clone(), cx);
                            }),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use dashtext_core::Scope;
    use gpui_kit::AppContext as _;
    use gpui_kit::TestAppContext;
    use gpui_kit::test::TestWindowExt as _;

    use super::CaptureWindow;
    use crate::test_support;

    fn open_capture(
        cx: &mut TestAppContext,
    ) -> (
        gpui_kit::WindowHandle<gpui_kit::base::Root>,
        gpui_kit::Entity<CaptureWindow>,
    ) {
        test_support::open(cx, |window, cx| cx.new(|cx| CaptureWindow::new(window, cx)))
    }

    #[gpui_kit::test]
    fn save_shortcut_adds_the_text_to_the_inbox_and_closes(cx: &mut TestAppContext) {
        let _library = test_support::init(cx, &[]);
        let (window, _) = open_capture(cx);

        cx.update_window(window.into(), |_, window, cx| {
            window.input("Buy oat milk", cx);
            window.press("secondary-enter", cx);
        })
        .expect("capture window is open");
        cx.run_until_parked();

        assert_eq!(test_support::contents(cx, Scope::Inbox), ["Buy oat milk"]);
        assert!(
            cx.update(|cx| cx.windows().is_empty()),
            "the window closes after saving"
        );
    }

    #[gpui_kit::test]
    fn escape_keeps_the_text_for_next_time(cx: &mut TestAppContext) {
        let _library = test_support::init(cx, &[]);
        let (window, _) = open_capture(cx);

        cx.update_window(window.into(), |_, window, cx| {
            window.input("Half a thought", cx);
            window.press("escape", cx);
        })
        .expect("capture window is open");
        cx.run_until_parked();

        assert!(
            test_support::contents(cx, Scope::All).is_empty(),
            "nothing is saved"
        );
        let (_, reopened) = open_capture(cx);
        assert_eq!(
            cx.update(|cx| reopened.read(cx).text(cx)),
            "Half a thought",
            "the text is restored"
        );
    }

    #[gpui_kit::test]
    fn blank_text_is_not_saved(cx: &mut TestAppContext) {
        let _library = test_support::init(cx, &[]);
        let (window, _) = open_capture(cx);

        cx.update_window(window.into(), |_, window, cx| {
            window.input("   ", cx);
            window.press("secondary-enter", cx);
        })
        .expect("capture window is open");
        cx.run_until_parked();

        assert!(test_support::contents(cx, Scope::All).is_empty());
        assert_eq!(
            cx.update(|cx| cx.windows().len()),
            1,
            "the window stays open"
        );
    }
}
