//! Regression: a press on a Tooltip's trigger inside the 100ms enter delay
//! cancels the pending show, and the tooltip's text follows the trigger's
//! state (design review A3: the password toggle's tooltip appeared after the
//! press and kept saying "Show password" while the text was visible).
//! Hiding on press is `close_on_activation` (Carbon `closeOnActivation`): on
//! for icon buttons (the password toggle, `Button` icon-only tooltips), off
//! for a standalone `Tooltip`; Escape hides either.
use gpui_kit::test::{TestEventExt as _, TestQueryExt as _, TestWindowExt as _};
use gpui_kit::{
    AppContext as _, Bounds, Context, ElementId, Entity, IntoElement, MouseButton,
    ParentElement as _, Point, Render, Role, Styled as _, TestAppContext, Window, WindowBounds,
    WindowOptions, div, point, px, size,
};
use std::time::Duration;
use u0ui::components::{Button, Input, InputState, Tooltip};
use u0ui::icons::CarbonIcon;

struct Page {
    state: Entity<InputState>,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().p(px(40.)).w(px(400.)).child(
            Input::new("key", &self.state)
                .password(true)
                .password_labels("Show password", "Hide password"),
        )
    }
}

/// The toggle's tooltip trigger: `(("key", "visibility-tip"), "trigger")`.
fn trigger() -> ElementId {
    let tip = ElementId::from((ElementId::from("key"), "visibility-tip"));
    ElementId::from((tip, "trigger"))
}

fn tooltips(window: &Window) -> Vec<String> {
    window
        .find_all_by_role(Role::Tooltip)
        .into_iter()
        .filter(|tip| tip.visible())
        .filter_map(|tip| tip.label().map(str::to_string))
        .collect()
}

#[gpui_kit::test]
fn press_inside_the_enter_delay_cancels_the_show(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let (window, page) = cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(800.), px(600.)),
                })),
                ..Default::default()
            },
            cx,
            |window, cx| {
                let state = cx.new(|cx| InputState::new(window, cx));
                cx.new(|_| Page { state })
            },
        )
        .expect("open test window")
    });
    let wait = |ms: u64, cx: &mut TestAppContext| {
        cx.executor().advance_clock(Duration::from_millis(ms));
        cx.run_until_parked();
        cx.update_window(window, |_, window, cx| window.render_frame(cx))
            .unwrap();
    };
    let masked = |cx: &mut TestAppContext| {
        cx.update(|cx| page.read(cx).state.read(cx).presentation().is_masked())
    };
    wait(10, cx);
    assert!(masked(cx), "a password field starts masked");

    // Rest the pointer on the toggle and press it 50ms later, before the
    // 100ms enter delay has run out.
    cx.update_window(window, |_, window, cx| window.hover(trigger(), cx))
        .unwrap();
    wait(50, cx);
    cx.update_window(window, |_, window, cx| window.click(trigger(), cx))
        .unwrap();
    assert!(!masked(cx), "the press reveals the text");
    // Long past the enter delay: the pending show was cancelled.
    wait(1000, cx);
    let shown = cx
        .update_window(window, |_, window, _| tooltips(window))
        .unwrap();
    assert_eq!(
        shown,
        Vec::<String>::new(),
        "the tooltip showed after the press"
    );

    // Leave and come back: the tooltip names the current action.
    cx.update_window(window, |_, window, cx| {
        window.pointer_move(point(px(780.), px(580.)), None, cx)
    })
    .unwrap();
    wait(500, cx);
    cx.update_window(window, |_, window, cx| window.hover(trigger(), cx))
        .unwrap();
    wait(200, cx);
    let shown = cx
        .update_window(window, |_, window, _| tooltips(window))
        .unwrap();
    assert_eq!(shown, vec!["Hide password".to_string()]);

    // Pressed again while it shows: it hides, the text is masked again, and
    // the next arrival reads "Show password".
    cx.update_window(window, |_, window, cx| window.click(trigger(), cx))
        .unwrap();
    wait(500, cx);
    assert!(masked(cx));
    let shown = cx
        .update_window(window, |_, window, _| tooltips(window))
        .unwrap();
    assert_eq!(shown, Vec::<String>::new());
    cx.update_window(window, |_, window, cx| {
        window.pointer_move(point(px(780.), px(580.)), None, cx)
    })
    .unwrap();
    wait(500, cx);
    cx.update_window(window, |_, window, cx| window.hover(trigger(), cx))
        .unwrap();
    wait(200, cx);
    let shown = cx
        .update_window(window, |_, window, _| tooltips(window))
        .unwrap();
    assert_eq!(shown, vec!["Show password".to_string()]);

    // A press whose hover report arrives late (down/up, then the move, as a
    // warped pointer delivers them) does not bring the tooltip back either.
    cx.update_window(window, |_, window, cx| {
        window.pointer_move(point(px(780.), px(580.)), None, cx)
    })
    .unwrap();
    wait(500, cx);
    let center = cx
        .update_window(window, |_, window, _| {
            window.find(trigger()).bounds().center()
        })
        .unwrap();
    cx.update_window(window, |_, window, cx| {
        window.pointer_down(center, MouseButton::Left, cx);
        window.pointer_up(center, MouseButton::Left, cx);
        window.pointer_move(center, None, cx);
    })
    .unwrap();
    assert!(!masked(cx), "the press reveals the text");
    wait(1000, cx);
    let shown = cx
        .update_window(window, |_, window, _| tooltips(window))
        .unwrap();
    assert_eq!(shown, Vec::<String>::new(), "a late hover report showed it");
}

/// A standalone Tooltip and an icon-only Button's tooltip side by side.
struct Pair;

impl Render for Pair {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .p(px(80.))
            .flex()
            .gap(px(200.))
            .child(
                Tooltip::new(
                    "plain",
                    Button::new("plain-b", "Copy")
                        .icon(CarbonIcon::Copy)
                        .icon_only(true)
                        .tooltip(false),
                )
                .content("Copy to clipboard"),
            )
            .child(
                Button::new("icon", "Delete")
                    .icon(CarbonIcon::TrashCan)
                    .icon_only(true),
            )
    }
}

fn plain_trigger() -> ElementId {
    ElementId::from((ElementId::from("plain"), "trigger"))
}

fn icon_trigger() -> ElementId {
    let tip = ElementId::from((ElementId::from("icon"), "tooltip"));
    ElementId::from((tip, "trigger"))
}

/// Carbon `closeOnActivation`: false for a standalone `Tooltip` (it stays
/// up when its trigger is pressed), true for `IconButton` (it hides).
/// Escape hides either.
#[gpui_kit::test]
fn only_icon_button_tooltips_close_on_activation(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let (window, _pair) = cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(800.), px(600.)),
                })),
                ..Default::default()
            },
            cx,
            |_, cx| cx.new(|_| Pair),
        )
        .expect("open test window")
    });
    let wait = |ms: u64, cx: &mut TestAppContext| {
        cx.executor().advance_clock(Duration::from_millis(ms));
        cx.run_until_parked();
        cx.update_window(window, |_, window, cx| window.render_frame(cx))
            .unwrap();
    };
    let shown = |cx: &mut TestAppContext| {
        cx.update_window(window, |_, window, _| tooltips(window))
            .unwrap()
    };
    let away = |cx: &mut TestAppContext| {
        cx.update_window(window, |_, window, cx| {
            window.pointer_move(point(px(780.), px(580.)), None, cx)
        })
        .unwrap();
    };
    wait(10, cx);

    // Standalone: shown on hover, still shown after a press.
    cx.update_window(window, |_, window, cx| window.hover(plain_trigger(), cx))
        .unwrap();
    wait(200, cx);
    assert_eq!(shown(cx), vec!["Copy to clipboard".to_string()]);
    cx.update_window(window, |_, window, cx| window.click(plain_trigger(), cx))
        .unwrap();
    wait(500, cx);
    assert_eq!(
        shown(cx),
        vec!["Copy to clipboard".to_string()],
        "a standalone tooltip hid on press"
    );
    // Escape still hides it.
    cx.update_window(window, |_, window, cx| window.press("escape", cx))
        .unwrap();
    wait(500, cx);
    assert_eq!(shown(cx), Vec::<String>::new(), "Escape did not hide it");
    away(cx);
    wait(500, cx);

    // Icon button: a press hides it.
    cx.update_window(window, |_, window, cx| window.hover(icon_trigger(), cx))
        .unwrap();
    wait(200, cx);
    assert_eq!(shown(cx), vec!["Delete".to_string()]);
    cx.update_window(window, |_, window, cx| window.click(icon_trigger(), cx))
        .unwrap();
    wait(500, cx);
    assert_eq!(
        shown(cx),
        Vec::<String>::new(),
        "an icon button's tooltip stayed up on press"
    );
}
