//! Tab into a single-line text field selects all of its text, as a browser
//! does for `<input>`; a click into it keeps the caret where it lands, and
//! focus moved by code does not select either (design review FP2, item 12).
use gpui_kit::Focusable as _;
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, Context, Entity, FocusHandle,
    InteractiveElement as _, IntoElement, ParentElement as _, Point, Render, Styled as _,
    TestAppContext, TestSupportExt as _, Window, WindowBounds, WindowOptions,
    component::input::InputState, div, px, size,
};
use std::time::Duration;
use u0ui::components::Input;

const VALUE: &str = "carbon-ui";

struct Page {
    sentinel: FocusHandle,
    state: Entity<InputState>,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("page")
            .test_support()
            .w(px(400.))
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(
                div()
                    .id("sentinel")
                    .track_focus(&self.sentinel.clone().tab_index(0).tab_stop(true))
                    .size(px(4.)),
            )
            .child(
                div()
                    .id("field-box")
                    .test_support()
                    .child(Input::new("field", &self.state).label("Project name")),
            )
    }
}

fn open(cx: &mut TestAppContext) -> (AnyWindowHandle, Entity<Page>) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let handle = cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(600.), px(400.)),
                })),
                ..Default::default()
            },
            cx,
            |window, cx| {
                let sentinel = cx.focus_handle();
                let state = cx.new(|cx| InputState::new(window, cx).default_value(VALUE));
                cx.new(|_| Page { sentinel, state })
            },
        )
        .expect("open test window")
    });
    let page = handle.1.clone();
    (handle.0, page)
}

fn settle(window: AnyWindowHandle, cx: &mut TestAppContext) {
    cx.executor().advance_clock(Duration::from_millis(300));
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| window.render_frame(cx))
        .unwrap();
}

fn selection(page: &Entity<Page>, cx: &mut TestAppContext) -> std::ops::Range<usize> {
    cx.update(|cx| page.read(cx).state.read(cx).selected_range())
}

fn field_focused(window: AnyWindowHandle, page: &Entity<Page>, cx: &mut TestAppContext) -> bool {
    cx.update_window(window, |_, window, cx| {
        page.read(cx)
            .state
            .read(cx)
            .focus_handle(cx)
            .is_focused(window)
    })
    .unwrap()
}

fn focus_sentinel(window: AnyWindowHandle, page: &Entity<Page>, cx: &mut TestAppContext) {
    cx.update_window(window, |_, window, cx| {
        let sentinel = page.read(cx).sentinel.clone();
        sentinel.focus(window, cx);
    })
    .unwrap();
    settle(window, cx);
}

#[gpui_kit::test]
fn tab_into_a_field_selects_all_of_it(cx: &mut TestAppContext) {
    let (window, page) = open(cx);
    settle(window, cx);
    focus_sentinel(window, &page, cx);
    cx.update_window(window, |_, window, cx| window.press("tab", cx))
        .unwrap();
    settle(window, cx);
    assert!(field_focused(window, &page, cx), "Tab reaches the field");
    assert_eq!(selection(&page, cx), 0..VALUE.len());
}

#[gpui_kit::test]
fn a_click_or_code_keeps_the_caret(cx: &mut TestAppContext) {
    let (window, page) = open(cx);
    settle(window, cx);
    // A click into the field: focus by pointer, no selection.
    cx.update_window(window, |_, window, cx| window.click("field-box", cx))
        .unwrap();
    settle(window, cx);
    assert!(
        field_focused(window, &page, cx),
        "the click focuses the field"
    );
    assert!(selection(&page, cx).is_empty());
    // Focus moved by code after a Tab elsewhere: still no selection.
    focus_sentinel(window, &page, cx);
    cx.update_window(window, |_, window, cx| {
        let focus = page.read(cx).state.read(cx).focus_handle(cx);
        focus.focus(window, cx);
    })
    .unwrap();
    settle(window, cx);
    assert!(field_focused(window, &page, cx));
    assert!(selection(&page, cx).is_empty());
}
