//! PdfViewer's page field (design review FG3B): the reference renders
//! `value={pd ?? p}`, so once the user edits, the draft wins until Enter or
//! blur. Emptying "2" and typing "3" shows "3", and Enter goes to page 3.
//! After Enter (`e.currentTarget.blur()`), Tab continues after the field, as
//! Chromium keeps the focus navigation starting point at the blurred input.
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, Context, Entity, FocusHandle,
    InteractiveElement as _, IntoElement, ParentElement as _, Point, Render, Styled as _,
    TestAppContext, Window, WindowBounds, WindowOptions, div, px, size,
};
use std::time::Duration;
use u0ui::u0::panels::PdfViewer;

struct Page {
    stops: [FocusHandle; 2],
    moved: Entity<Vec<u32>>,
}

fn stop(handle: &FocusHandle, id: &'static str) -> impl IntoElement {
    div()
        .id(id)
        .track_focus(&handle.clone().tab_index(0).tab_stop(true))
        .size(px(10.))
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let moved = self.moved.clone();
        div()
            .w(px(640.))
            .flex()
            .flex_col()
            .child(stop(&self.stops[0], "before"))
            .child(
                PdfViewer::new("pdf", "design-review.pdf", 4)
                    .default_page(2)
                    .on_page_change(move |n, _, cx| moved.update(cx, |m, _| m.push(n))),
            )
            .child(stop(&self.stops[1], "after"))
    }
}

struct Harness {
    window: AnyWindowHandle,
    stops: [FocusHandle; 2],
    moved: Entity<Vec<u32>>,
}

fn open(cx: &mut TestAppContext) -> Harness {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let (stops, moved) =
        cx.update(|cx| ([cx.focus_handle(), cx.focus_handle()], cx.new(|_| vec![])));
    let page = Page {
        stops: stops.clone(),
        moved: moved.clone(),
    };
    let window = cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(700.), px(500.)),
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
        moved,
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

fn focused(window: AnyWindowHandle, cx: &mut TestAppContext) -> Option<FocusHandle> {
    cx.update_window(window, |_, window, cx| window.focused(cx))
        .unwrap()
}

/// From the stop before the viewer: Tab to "Previous page", Tab to the field.
fn into_field(h: &Harness, cx: &mut TestAppContext) {
    let first = h.stops[0].clone();
    cx.update_window(h.window, |_, window, cx| first.focus(window, cx))
        .unwrap();
    settle(h.window, cx);
    press("tab", h.window, cx);
    press("tab", h.window, cx);
}

/// Empty the field ("2"), type 3, press Enter.
fn retype(h: &Harness, cx: &mut TestAppContext) {
    into_field(h, cx);
    press("end", h.window, cx);
    press("backspace", h.window, cx);
    press("3", h.window, cx);
    press("enter", h.window, cx);
}

#[gpui_kit::test]
fn the_draft_wins_until_enter(cx: &mut TestAppContext) {
    let h = open(cx);
    retype(&h, cx);
    assert_eq!(h.moved.read_with(cx, |m, _| m.clone()), vec![3]);
}

#[gpui_kit::test]
fn tab_after_enter_continues_past_the_field(cx: &mut TestAppContext) {
    let h = open(cx);
    retype(&h, cx);
    // Not the window's first control: the next stop after the field is
    // "Next page"; Shift+Tab from there returns to the field.
    let after_enter = focused(h.window, cx);
    assert!(after_enter.is_some(), "focus rests beside the field");
    assert!(!h.stops.contains(after_enter.as_ref().unwrap()));
    press("tab", h.window, cx);
    let next = focused(h.window, cx);
    assert!(next.is_some() && !h.stops.contains(next.as_ref().unwrap()));
    assert_ne!(next, after_enter);
    // Back twice from "Next page" (field, "Previous page"), then the stop before.
    press("shift-tab", h.window, cx);
    press("shift-tab", h.window, cx);
    press("shift-tab", h.window, cx);
    assert_eq!(focused(h.window, cx), Some(h.stops[0].clone()));
}
