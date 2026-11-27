//! Chromium keeps the sequential focus navigation starting point at an
//! element that loses focus, so Tab continues from there instead of from the
//! top of the page (design review FG1B): after Stepper's Enter commits (and
//! blurs the input), and after KeyRecorder's focused clear button disappears.
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, Context, Entity, FocusHandle,
    InteractiveElement as _, IntoElement, ParentElement as _, Point, Render, SharedString,
    Styled as _, TestAppContext, Window, WindowBounds, WindowOptions, div, px, size,
};
use std::time::Duration;
use u0ui::u0::primitives::{KeyRecorder, Stepper};

struct Page {
    stops: [FocusHandle; 3],
    value: Entity<f64>,
    keys: Entity<SharedString>,
}

fn stop(handle: &FocusHandle, id: &'static str) -> impl IntoElement {
    div()
        .id(id)
        .track_focus(&handle.clone().tab_index(0).tab_stop(true))
        .size(px(10.))
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let value = self.value.clone();
        let keys = self.keys.clone();
        div()
            .p(px(20.))
            .w(px(420.))
            .flex()
            .flex_col()
            .gap(px(20.))
            .child(stop(&self.stops[0], "before"))
            .child(
                Stepper::new("st")
                    .label("Parallel")
                    .value(*self.value.read(cx))
                    .on_value_change(move |v, _, cx| value.update(cx, |s, _| *s = v)),
            )
            .child(stop(&self.stops[1], "mid"))
            .child(
                KeyRecorder::new("kr")
                    .label("Shortcut")
                    .value(self.keys.read(cx).clone())
                    .on_value_change(move |v, _, cx| keys.update(cx, |s, _| *s = v)),
            )
            .child(stop(&self.stops[2], "after"))
    }
}

struct Harness {
    window: AnyWindowHandle,
    stops: [FocusHandle; 3],
    value: Entity<f64>,
    keys: Entity<SharedString>,
}

fn open(cx: &mut TestAppContext) -> Harness {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let (stops, value, keys) = cx.update(|cx| {
        (
            [cx.focus_handle(), cx.focus_handle(), cx.focus_handle()],
            cx.new(|_| 3.),
            cx.new(|_| SharedString::from("⌘+K")),
        )
    });
    let page = Page {
        stops: stops.clone(),
        value: value.clone(),
        keys: keys.clone(),
    };
    let window = cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(600.), px(500.)),
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
    Harness {
        window,
        stops,
        value,
        keys,
    }
}

fn settle(window: AnyWindowHandle, cx: &mut TestAppContext) {
    cx.executor().advance_clock(Duration::from_millis(300));
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| window.render_frame(cx))
        .unwrap();
}

fn press(key: &str, window: AnyWindowHandle, cx: &mut TestAppContext) {
    cx.update_window(window, |_, window, cx| window.press(key, cx))
        .unwrap();
    settle(window, cx);
}

fn focus(handle: &FocusHandle, window: AnyWindowHandle, cx: &mut TestAppContext) {
    cx.update_window(window, |_, window, cx| handle.focus(window, cx))
        .unwrap();
    settle(window, cx);
}

fn focused(window: AnyWindowHandle, cx: &mut TestAppContext) -> Option<FocusHandle> {
    cx.update_window(window, |_, window, cx| window.focused(cx))
        .unwrap()
}

/// Tab into the stepper (caret before the 3), type 7, Enter: 73 commits.
fn commit_typed(h: &Harness, cx: &mut TestAppContext) {
    focus(&h.stops[0], h.window, cx);
    press("tab", h.window, cx);
    press("7", h.window, cx);
    press("enter", h.window, cx);
    assert_eq!(h.value.read_with(cx, |v, _| *v), 73.);
}

#[gpui_kit::test]
fn tab_after_stepper_enter_continues_past_the_stepper(cx: &mut TestAppContext) {
    let h = open(cx);
    commit_typed(&h, cx);
    // The input lost focus (no outline), but the next Tab is the stop after it.
    press("tab", h.window, cx);
    assert_eq!(focused(h.window, cx), Some(h.stops[1].clone()));
}

#[gpui_kit::test]
fn shift_tab_after_stepper_enter_goes_before_the_stepper(cx: &mut TestAppContext) {
    let h = open(cx);
    commit_typed(&h, cx);
    press("shift-tab", h.window, cx);
    assert_eq!(focused(h.window, cx), Some(h.stops[0].clone()));
}

/// Tab to the clear button and press it: the shortcut clears and the
/// button disappears with focus on it.
fn clear_by_keyboard(h: &Harness, cx: &mut TestAppContext) {
    focus(&h.stops[1], h.window, cx);
    press("tab", h.window, cx); // the record button
    press("tab", h.window, cx); // the clear button
    press("space", h.window, cx);
    assert_eq!(h.keys.read_with(cx, |k, _| k.clone()), "");
}

#[gpui_kit::test]
fn tab_after_clearing_a_shortcut_continues_past_the_recorder(cx: &mut TestAppContext) {
    let h = open(cx);
    clear_by_keyboard(&h, cx);
    press("tab", h.window, cx);
    assert_eq!(focused(h.window, cx), Some(h.stops[2].clone()));
}

#[gpui_kit::test]
fn shift_tab_after_clearing_a_shortcut_returns_to_the_record_button(cx: &mut TestAppContext) {
    let h = open(cx);
    clear_by_keyboard(&h, cx);
    // The record button sat right before the clear button.
    press("shift-tab", h.window, cx);
    let on = focused(h.window, cx);
    assert!(on.is_some() && !h.stops.contains(on.as_ref().unwrap()));
    press("shift-tab", h.window, cx);
    assert_eq!(focused(h.window, cx), Some(h.stops[1].clone()));
}
