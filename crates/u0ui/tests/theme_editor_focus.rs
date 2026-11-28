//! A ThemeEditor swatch gets focus back when its picker closes (design
//! review FINAL2, ThemeEditor grid): the reference's swatch is a `<label>`
//! around a native `<input type=color>`, so a click anywhere on the swatch
//! focuses the input, and the input keeps focus while its picker is open and
//! after it closes, by Escape or by a pick. The focus-visible ring then shows
//! only when the picker was closed from the keyboard (`u0ui::focus`).
use gpui_kit::test::{TestEventExt as _, TestQueryExt as _, TestWindowExt as _};
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, Context, ElementId, FocusHandle,
    InteractiveElement as _, IntoElement, MouseButton, ParentElement as _, Pixels, Point, Render,
    SharedString, Styled as _, TestAppContext, Window, WindowBounds, WindowOptions, div, px, size,
};
use std::time::Duration;
use u0ui::u0::settings::{ThemeEditor, ThemeToken};

const ID: &str = "te";

struct Page {
    before: FocusHandle,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .p(px(20.))
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(
                div()
                    .id("before")
                    .w(px(40.))
                    .h(px(20.))
                    .track_focus(&self.before.clone().tab_stop(true)),
            )
            .child(ThemeEditor::new(
                ID,
                [
                    ThemeToken::hex("background", "#161616").unwrap(),
                    ThemeToken::hex("text-primary", "#f4f4f4").unwrap(),
                    ThemeToken::hex("interactive", "#4589ff").unwrap(),
                ],
            ))
    }
}

fn open(cx: &mut TestAppContext) -> (AnyWindowHandle, FocusHandle) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let before = cx.update(|cx| cx.focus_handle());
    let page = Page {
        before: before.clone(),
    };
    let window = cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(900.), px(700.)),
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
    (window, before)
}

fn settle(window: AnyWindowHandle, cx: &mut TestAppContext) {
    cx.executor().advance_clock(Duration::from_millis(300));
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| window.render_frame(cx))
        .unwrap();
}

fn key(k: &str, window: AnyWindowHandle, cx: &mut TestAppContext) {
    cx.update_window(window, |_, window, cx| window.press(k, cx))
        .unwrap();
    settle(window, cx);
}

fn press_at(at: Point<Pixels>, window: AnyWindowHandle, cx: &mut TestAppContext) {
    cx.update_window(window, |_, window, cx| {
        window.pointer_down(at, MouseButton::Left, cx);
        window.pointer_up(at, MouseButton::Left, cx);
    })
    .unwrap();
    settle(window, cx);
}

/// The swatches' pickers, found as the tab stops after Import and Export.
fn swatches(
    before: &FocusHandle,
    window: AnyWindowHandle,
    cx: &mut TestAppContext,
) -> Vec<FocusHandle> {
    cx.update_window(window, |_, window, cx| before.focus(window, cx))
        .unwrap();
    settle(window, cx);
    let mut found = Vec::new();
    for _ in 0..5 {
        key("tab", window, cx);
        let now = cx
            .update_window(window, |_, window, cx| window.focused(cx))
            .unwrap()
            .expect("a tab stop has focus");
        found.push(now);
    }
    found.drain(..2);
    found
}

fn focused(handle: &FocusHandle, window: AnyWindowHandle, cx: &mut TestAppContext) -> bool {
    cx.update_window(window, |_, window, _| handle.is_focused(window))
        .unwrap()
}

fn tile(i: usize) -> ElementId {
    (ElementId::from(ID), SharedString::from(format!("sw-{i}"))).into()
}

/// Whether swatch `i`'s picker shows its popover (its Palette tab).
fn is_open(i: usize, window: AnyWindowHandle, cx: &mut TestAppContext) -> bool {
    let tile = tile(i);
    cx.update_window(window, |_, window, _| {
        window
            .find_all_by_label("Palette")
            .iter()
            .any(|e| e.path().contains(&tile))
    })
    .unwrap()
}

fn ring(window: AnyWindowHandle, cx: &mut TestAppContext) -> bool {
    cx.update_window(window, |_, window, _| {
        u0ui::focus::focus_visible_mode(window)
    })
    .unwrap()
}

fn text_id(i: usize) -> ElementId {
    (tile(i), "text").into()
}

/// Click swatch `i`'s name, as a click on the reference's `<label>`.
fn click_text(i: usize, before: &FocusHandle, window: AnyWindowHandle, cx: &mut TestAppContext) {
    cx.update_window(window, |_, window, cx| before.focus(window, cx))
        .unwrap();
    settle(window, cx);
    cx.update_window(window, |_, window, cx| window.click(text_id(i), cx))
        .unwrap();
    settle(window, cx);
    assert!(
        is_open(i, window, cx),
        "a click on the swatch opens its picker"
    );
    assert!(!ring(window, cx), "opened by the pointer: no ring");
}

#[gpui_kit::test]
fn escape_after_a_click_on_the_swatch_text_returns_focus(cx: &mut TestAppContext) {
    let (window, before) = open(cx);
    let sw = swatches(&before, window, cx);
    click_text(1, &before, window, cx);
    key("escape", window, cx);
    assert!(!is_open(1, window, cx), "Escape closes the picker");
    assert!(
        focused(&sw[1], window, cx),
        "the swatch that opened the picker has focus"
    );
    assert!(ring(window, cx), "closed from the keyboard: the ring shows");
    // Tab continues from the swatch.
    key("tab", window, cx);
    assert!(
        focused(&sw[2], window, cx),
        "Tab moves on to the next swatch"
    );
}

#[gpui_kit::test]
fn escape_after_keyboard_open_returns_focus(cx: &mut TestAppContext) {
    let (window, before) = open(cx);
    let sw = swatches(&before, window, cx);
    cx.update_window(window, |_, window, cx| sw[0].focus(window, cx))
        .unwrap();
    settle(window, cx);
    key("enter", window, cx);
    assert!(is_open(0, window, cx), "Enter opens the picker");
    key("escape", window, cx);
    assert!(!is_open(0, window, cx));
    assert!(focused(&sw[0], window, cx), "focus returns to the swatch");
    assert!(ring(window, cx));
}

#[gpui_kit::test]
fn a_pick_returns_focus_without_a_ring(cx: &mut TestAppContext) {
    let (window, before) = open(cx);
    let sw = swatches(&before, window, cx);
    click_text(2, &before, window, cx);
    // The first featured color, 20px square, 12px under the Palette/HSLA
    // tabs: a click there is Kit's `select_color`, which closes the picker
    // (a click elsewhere inside the popover would leave it open).
    let tabs = cx
        .update_window(window, |_, window, _| {
            window.find_by_label("Palette").bounds()
        })
        .unwrap();
    press_at(
        Point::new(tabs.left() + px(10.), tabs.bottom() + px(22.)),
        window,
        cx,
    );
    assert!(!is_open(2, window, cx), "a pick closes the picker");
    assert!(focused(&sw[2], window, cx), "focus returns to the swatch");
    assert!(!ring(window, cx), "closed by the pointer: no ring");
}
