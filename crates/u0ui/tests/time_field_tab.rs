//! TimeField's hours and minutes are two tab stops, like the reference's two
//! `<input>`s: Tab from hours goes to minutes, Shift+Tab from minutes back to
//! hours (design review FG1). The same tracked-handle rule makes the
//! SteppedSlider's range a tab stop, and a pointer press on that range
//! focuses it without the `:focus-visible` ring.
use gpui_kit::test::{TestEventExt as _, TestWindowExt as _};
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, Context, Entity, FocusHandle,
    InteractiveElement as _, IntoElement, MouseButton, ParentElement as _, Pixels, Point, Render,
    SharedString, Styled as _, TestAppContext, Window, WindowBounds, WindowOptions, canvas, div,
    point, px, size,
};
use std::{cell::Cell, time::Duration};

thread_local! {
    /// The slider's bounds from the last paint.
    static SLIDER: Cell<Option<Bounds<Pixels>>> = const { Cell::new(None) };
}
use u0ui::u0::primitives::{SteppedSlider, TimeField};

struct Page {
    before: FocusHandle,
    after: FocusHandle,
    time: Entity<SharedString>,
    slider: Entity<usize>,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let time = self.time.clone();
        let slider = self.slider.clone();
        div()
            .p(px(20.))
            .w(px(420.))
            .flex()
            .flex_col()
            .gap(px(20.))
            .child(
                div()
                    .id("before")
                    .track_focus(&self.before.clone().tab_index(0).tab_stop(true))
                    .size(px(10.)),
            )
            .child(
                TimeField::new("tf")
                    .value(self.time.read(cx).clone())
                    .on_value_change(move |v, _, cx| time.update(cx, |t, _| *t = v)),
            )
            .child(
                div()
                    .relative()
                    .child(
                        SteppedSlider::new("ss", ["a", "b", "c"])
                            .value(*self.slider.read(cx))
                            .on_value_change(move |v, _, cx| slider.update(cx, |s, _| *s = v)),
                    )
                    .child(
                        canvas(|b, _, _| SLIDER.with(|s| s.set(Some(b))), |_, _, _, _| {})
                            .absolute()
                            .top_0()
                            .left_0()
                            .size_full(),
                    ),
            )
            .child(
                div()
                    .id("after")
                    .track_focus(&self.after.clone().tab_index(0).tab_stop(true))
                    .size(px(10.)),
            )
    }
}

fn open(
    cx: &mut TestAppContext,
) -> (
    AnyWindowHandle,
    FocusHandle,
    FocusHandle,
    Entity<SharedString>,
    Entity<usize>,
) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let (before, after, time, slider) = cx.update(|cx| {
        (
            cx.focus_handle(),
            cx.focus_handle(),
            cx.new(|_| SharedString::from("09:00")),
            cx.new(|_| 0usize),
        )
    });
    let page = Page {
        before: before.clone(),
        after: after.clone(),
        time: time.clone(),
        slider: slider.clone(),
    };
    let window = cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(600.), px(400.)),
                })),
                ..Default::default()
            },
            cx,
            move |_, cx| cx.new(|_| page),
        )
        .expect("open test window")
        .0
    });
    (window, before, after, time, slider)
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

fn focused(handle: &FocusHandle, window: AnyWindowHandle, cx: &mut TestAppContext) -> bool {
    cx.update_window(window, |_, window, _| handle.is_focused(window))
        .unwrap()
}

#[gpui_kit::test]
fn tab_walks_hours_minutes_then_the_slider(cx: &mut TestAppContext) {
    let (window, before, after, time, slider) = open(cx);
    settle(window, cx);
    cx.update_window(window, |_, window, cx| before.focus(window, cx))
        .unwrap();
    settle(window, cx);

    // Tab → hours: ↑ adds an hour.
    press("tab", window, cx);
    press("up", window, cx);
    assert_eq!(time.read_with(cx, |t, _| t.clone()), "10:00");
    // Tab → minutes: ↑ adds the 5-minute step.
    press("tab", window, cx);
    press("up", window, cx);
    assert_eq!(time.read_with(cx, |t, _| t.clone()), "10:05");
    // Shift+Tab → back to hours.
    press("shift-tab", window, cx);
    press("down", window, cx);
    assert_eq!(time.read_with(cx, |t, _| t.clone()), "09:05");

    // Tab, Tab → the slider's range: → moves one stop.
    press("tab", window, cx);
    press("tab", window, cx);
    press("right", window, cx);
    assert_eq!(slider.read_with(cx, |s, _| *s), 1);

    // Tab → past the slider to the trailing stop.
    press("tab", window, cx);
    assert!(focused(&after, window, cx));
}

#[gpui_kit::test]
fn a_press_focuses_the_slider_without_the_keyboard_ring(cx: &mut TestAppContext) {
    let (window, before, after, _, slider) = open(cx);
    settle(window, cx);
    let b = SLIDER.with(|s| s.get()).expect("slider painted");
    // The range is the slider's first 16px row; press near its right end.
    let at = point(b.origin.x + b.size.width - px(4.), b.origin.y + px(8.));
    cx.update_window(window, |_, window, cx| {
        window.pointer_down(at, MouseButton::Left, cx);
        window.pointer_up(at, MouseButton::Left, cx);
    })
    .unwrap();
    settle(window, cx);
    assert_eq!(
        slider.read_with(cx, |s, _| *s),
        2,
        "the press picks the last level"
    );
    assert!(!focused(&before, window, cx) && !focused(&after, window, cx));
    // `:focus-visible` follows the input modality: a press is not keyboard.
    let keyboard = cx
        .update_window(window, |_, window, _| window.last_input_was_keyboard())
        .unwrap();
    assert!(!keyboard, "no ring after a pointer press");
    // The range holds focus: ← steps it.
    press("left", window, cx);
    assert_eq!(slider.read_with(cx, |s, _| *s), 1);
}
