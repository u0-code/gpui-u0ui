//! `:focus-visible` follows Chromium's modality, not GPUI's hover modality
//! (design review FINAL-CORE, AlertDialog): programmatic focus before any
//! pointer press shows the ring, a pointer press hides it, pointer movement
//! leaves it alone, and the next key brings it back.
use gpui_kit::test::{TestEventExt as _, TestWindowExt as _};
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, Context, FocusHandle, InteractiveElement as _,
    IntoElement, MouseButton, ParentElement as _, Point, Render, Styled as _, TestAppContext,
    Window, WindowBounds, WindowOptions, div, point, px, size,
};
use std::time::Duration;
use u0ui::components::Button;
use u0ui::u0::primitives::parts;

struct Page {
    first: FocusHandle,
    second: FocusHandle,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .p(px(20.))
            .flex()
            .flex_col()
            .gap(px(20.))
            .child(
                div()
                    .id("first")
                    .track_focus(&self.first.clone().tab_index(0).tab_stop(true))
                    .size(px(40.)),
            )
            .child(
                div()
                    .id("second")
                    .track_focus(&self.second.clone().tab_index(0).tab_stop(true))
                    .size(px(40.)),
            )
            .child(Button::new("button", "Cancel"))
    }
}

fn open(cx: &mut TestAppContext) -> (AnyWindowHandle, FocusHandle, FocusHandle) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let (first, second) = cx.update(|cx| (cx.focus_handle(), cx.focus_handle()));
    let page = Page {
        first: first.clone(),
        second: second.clone(),
    };
    let window = cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(400.), px(300.)),
                })),
                ..Default::default()
            },
            cx,
            move |_, cx| cx.new(|_| page),
        )
        .expect("open test window")
        .0
    });
    settle(window, cx);
    (window, first, second)
}

fn settle(window: AnyWindowHandle, cx: &mut TestAppContext) {
    cx.executor().advance_clock(Duration::from_millis(300));
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| window.render_frame(cx))
        .unwrap();
}

fn ring(handle: &FocusHandle, window: AnyWindowHandle, cx: &mut TestAppContext) -> bool {
    cx.update_window(window, |_, window, _| parts::focus_visible(handle, window))
        .unwrap()
}

fn mode(window: AnyWindowHandle, cx: &mut TestAppContext) -> bool {
    cx.update_window(window, |_, window, _| {
        u0ui::focus::focus_visible_mode(window)
    })
    .unwrap()
}

/// Press and release at `at`, as a click does.
fn press_at(
    at: gpui_kit::Point<gpui_kit::Pixels>,
    window: AnyWindowHandle,
    cx: &mut TestAppContext,
) {
    cx.update_window(window, |_, window, cx| {
        window.pointer_down(at, MouseButton::Left, cx);
        window.pointer_up(at, MouseButton::Left, cx);
    })
    .unwrap();
    settle(window, cx);
}

fn key(k: &str, window: AnyWindowHandle, cx: &mut TestAppContext) {
    cx.update_window(window, |_, window, cx| window.press(k, cx))
        .unwrap();
    settle(window, cx);
}

#[gpui_kit::test]
fn programmatic_focus_before_any_press_shows_the_ring(cx: &mut TestAppContext) {
    let (window, first, _) = open(cx);
    // A pointer that only moves (it entered the window) changes nothing.
    cx.update_window(window, |_, window, cx| {
        window.pointer_move(point(px(300.), px(250.)), None, cx)
    })
    .unwrap();
    cx.update_window(window, |_, window, cx| first.focus(window, cx))
        .unwrap();
    settle(window, cx);
    assert!(ring(&first, window, cx), "initial focus is :focus-visible");
}

#[gpui_kit::test]
fn a_pointer_press_hides_the_ring(cx: &mut TestAppContext) {
    let (window, first, second) = open(cx);
    // Press on the second box (20px padding + 40px box + 20px gap).
    press_at(point(px(40.), px(100.)), window, cx);
    assert!(
        cx.update_window(window, |_, window, _| second.is_focused(window))
            .unwrap(),
        "the press focuses the box"
    );
    assert!(!ring(&second, window, cx), "no ring after a pointer press");
    // Programmatic focus after a press is not focus-visible either.
    cx.update_window(window, |_, window, cx| first.focus(window, cx))
        .unwrap();
    settle(window, cx);
    assert!(!ring(&first, window, cx));
    // A click on a u0ui Button keeps PR #21's rule: no ring.
    cx.update_window(window, |_, window, cx| window.click("button", cx))
        .unwrap();
    settle(window, cx);
    assert!(!mode(window, cx), "a Button click is a pointer press");
}

#[gpui_kit::test]
fn tab_ring_survives_pointer_movement(cx: &mut TestAppContext) {
    let (window, first, _) = open(cx);
    press_at(point(px(300.), px(250.)), window, cx);
    assert!(!mode(window, cx));
    key("tab", window, cx);
    assert!(ring(&first, window, cx), "Tab shows the ring");
    cx.update_window(window, |_, window, cx| {
        window.pointer_move(point(px(200.), px(200.)), None, cx);
        window.pointer_move(point(px(210.), px(220.)), None, cx);
    })
    .unwrap();
    settle(window, cx);
    // GPUI's own modality flips to mouse on a move (it suppresses hover);
    // `:focus-visible` does not.
    assert!(
        !cx.update_window(window, |_, window, _| window.last_input_was_keyboard())
            .unwrap()
    );
    assert!(
        ring(&first, window, cx),
        "moving the pointer keeps the ring"
    );
}

#[gpui_kit::test]
fn a_key_after_a_press_brings_the_ring_back(cx: &mut TestAppContext) {
    let (window, first, _) = open(cx);
    press_at(point(px(40.), px(40.)), window, cx);
    assert!(!ring(&first, window, cx));
    // A lone modifier is not a key press.
    cx.update_window(window, |_, window, cx| {
        window.change_modifiers(
            gpui_kit::Modifiers {
                shift: true,
                ..Default::default()
            },
            cx,
        );
        window.change_modifiers(gpui_kit::Modifiers::default(), cx);
    })
    .unwrap();
    settle(window, cx);
    assert!(
        !ring(&first, window, cx),
        "shift alone keeps pointer modality"
    );
    key("a", window, cx);
    assert!(ring(&first, window, cx), "a key press is keyboard modality");
}
