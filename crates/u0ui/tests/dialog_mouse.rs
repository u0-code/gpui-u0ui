//! Every activation path of a Dialog's trigger and footer Buttons runs
//! exactly once: a mouse click, Enter, Space, and an accessibility Press
//! (design review FG3C: an accessibility press on "Open live dialog" never
//! opened it, because a Button without a handler exposed no Press action).
//! GPUI performs a Press on an element with a click listener as a pointer
//! press and release at its centre, dispatched back to back. The clicked
//! trigger takes focus, so closing returns focus to it.
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, Context, ElementId, Entity, FocusHandle, IntoElement,
    ParentElement as _, Point, Render, Styled as _, TestAppContext, Window, WindowBounds,
    WindowOptions, div, px, size,
};
use gpui_kit::{InputEvent as _, Modifiers, MouseButton, MouseDownEvent, MouseUpEvent};
use std::time::Duration;
use u0ui::components::{Button, Dialog};

struct State {
    trigger: FocusHandle,
    open: bool,
    log: Vec<&'static str>,
    opens: usize,
}

struct Page {
    state: Entity<State>,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.clone();
        let log = |text: &'static str| {
            let state = state.clone();
            move |_: &mut Window, cx: &mut gpui_kit::App| {
                state.update(cx, |s, cx| {
                    s.log.push(text);
                    if text != "Primary" {
                        s.open = false;
                    }
                    cx.notify();
                })
            }
        };
        let on_open = {
            let state = state.clone();
            move |open: bool, _: &mut Window, cx: &mut gpui_kit::App| {
                state.update(cx, |s, cx| {
                    s.open = open;
                    if open {
                        s.opens += 1;
                    }
                    cx.notify();
                })
            }
        };
        div().p(px(40.)).w(px(900.)).h(px(700.)).child(
            Dialog::new("dlg")
                .open(self.state.read(cx).open)
                .on_open_change(on_open)
                .trigger(
                    Button::new("open", "Open live dialog")
                        .track_focus(&self.state.read(cx).trigger),
                )
                .title("Rename project")
                .primary_text("Save")
                .secondary_text("Cancel")
                .on_primary(log("Primary"))
                .on_secondary(log("Secondary"))
                .child(div().h(px(40.))),
        )
    }
}

fn open(cx: &mut TestAppContext) -> (AnyWindowHandle, Entity<State>) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let state = cx.update(|cx| {
        let trigger = cx.focus_handle();
        cx.new(|_| State {
            trigger,
            open: false,
            log: Vec::new(),
            opens: 0,
        })
    });
    let window = cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(1000.), px(800.)),
                })),
                ..Default::default()
            },
            cx,
            {
                let state = state.clone();
                move |_, cx| cx.new(|_| Page { state })
            },
        )
        .expect("open test window")
        .0
    });
    (window, state)
}

fn settle(window: AnyWindowHandle, cx: &mut TestAppContext) {
    cx.executor().advance_clock(Duration::from_millis(600));
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| window.render_frame(cx))
        .unwrap();
}

fn trigger_focused(
    window: AnyWindowHandle,
    state: &Entity<State>,
    cx: &mut TestAppContext,
) -> bool {
    let trigger = cx.update(|cx| state.read(cx).trigger.clone());
    cx.update_window(window, |_, window, _| trigger.is_focused(window))
        .unwrap()
}

fn secondary() -> ElementId {
    ElementId::from((ElementId::from("dlg"), "secondary"))
}

fn primary() -> ElementId {
    ElementId::from((ElementId::from("dlg"), "primary"))
}

#[gpui_kit::test]
fn mouse_click_opens_the_dialog_and_cancel_runs_once(cx: &mut TestAppContext) {
    let (window, state) = open(cx);
    settle(window, cx);
    cx.update_window(window, |_, window, cx| window.click("open", cx))
        .unwrap();
    settle(window, cx);
    assert!(cx.update(|cx| state.read(cx).open), "a click opens it");
    assert_eq!(cx.update(|cx| state.read(cx).opens), 1);

    cx.update_window(window, |_, window, cx| window.click(secondary(), cx))
        .unwrap();
    settle(window, cx);
    assert_eq!(
        cx.update(|cx| state.read(cx).log.clone()),
        vec!["Secondary"]
    );
    assert!(!cx.update(|cx| state.read(cx).open));

    // Open again; Save runs once and keeps it open.
    cx.update_window(window, |_, window, cx| window.click("open", cx))
        .unwrap();
    settle(window, cx);
    assert!(
        cx.update(|cx| state.read(cx).open),
        "a second click opens it"
    );
    cx.update_window(window, |_, window, cx| window.click(primary(), cx))
        .unwrap();
    settle(window, cx);
    assert_eq!(
        cx.update(|cx| state.read(cx).log.clone()),
        vec!["Secondary", "Primary"]
    );
}

#[gpui_kit::test]
fn keyboard_activation_runs_once(cx: &mut TestAppContext) {
    let (window, state) = open(cx);
    settle(window, cx);
    for key in ["enter", "space"] {
        cx.update_window(window, |_, window, cx| window.press("tab", cx))
            .unwrap();
        settle(window, cx);
        cx.update_window(window, |_, window, cx| window.press(key, cx))
            .unwrap();
        settle(window, cx);
        assert!(cx.update(|cx| state.read(cx).open), "{key} opens it");
        // Danger off: focus starts on Save; Shift+Tab to Cancel.
        cx.update_window(window, |_, window, cx| window.press("shift-tab", cx))
            .unwrap();
        settle(window, cx);
        cx.update_window(window, |_, window, cx| window.press(key, cx))
            .unwrap();
        settle(window, cx);
        assert!(
            !cx.update(|cx| state.read(cx).open),
            "{key} on Cancel closes"
        );
    }
    assert_eq!(
        cx.update(|cx| state.read(cx).log.clone()),
        vec!["Secondary", "Secondary"]
    );
    assert_eq!(cx.update(|cx| state.read(cx).opens), 2);
}

/// GPUI's fallback for an accessibility Press: down and up at the node's
/// centre, with no frame in between.
fn ax_press(window: AnyWindowHandle, id: ElementId, cx: &mut TestAppContext) {
    cx.update_window(window, |_, window, cx| {
        let center = window.find(id).bounds().center();
        let down = MouseDownEvent {
            button: MouseButton::Left,
            position: center,
            modifiers: Modifiers::default(),
            click_count: 1,
            first_mouse: false,
        };
        let up = MouseUpEvent {
            button: MouseButton::Left,
            position: center,
            modifiers: Modifiers::default(),
            click_count: 1,
        };
        window.dispatch_event(down.to_platform_input(), cx);
        window.dispatch_event(up.to_platform_input(), cx);
    })
    .unwrap();
    settle(window, cx);
}

#[gpui_kit::test]
fn accessibility_press_opens_and_cancels_once(cx: &mut TestAppContext) {
    let (window, state) = open(cx);
    settle(window, cx);
    ax_press(window, "open".into(), cx);
    assert!(cx.update(|cx| state.read(cx).open), "a Press opens it");
    assert_eq!(cx.update(|cx| state.read(cx).opens), 1);
    ax_press(window, secondary(), cx);
    assert_eq!(
        cx.update(|cx| state.read(cx).log.clone()),
        vec!["Secondary"]
    );
    assert!(!cx.update(|cx| state.read(cx).open));
    assert!(trigger_focused(window, &state, cx));
}
