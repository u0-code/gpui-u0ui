//! The CodeSnippet's code area is a tab stop before the copy button, as in
//! Carbon's DOM (`.cds--snippet-container` `tabIndex={0}` for single, the
//! multi `pre` `tabIndex={0}`): a press focuses it, Left/Right scroll it, a
//! vertical wheel leaves it alone (the page scrolls), and the multi "Show
//! more" button after the copy button toggles from Enter and Space (design
//! review FC).
use gpui_kit::test::{TestEventExt as _, TestWindowExt as _};
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, Context, FocusHandle, InputEvent as _, IntoElement,
    MouseButton, ParentElement as _, Point, Render, ScrollDelta, ScrollHandle, Styled as _,
    TestAppContext, Window, WindowBounds, WindowOptions, div, point, px, size,
};
use std::time::Duration;
use u0ui::components::{CodeSnippet, SnippetType};

struct Page {
    single: FocusHandle,
    single_scroll: ScrollHandle,
    multi: FocusHandle,
    multi_scroll: ScrollHandle,
}

const LONG: &str = "npx carbon-ui init --theme g100 --density compact --with-icons \
                    --with-motion --with-gallery --force --verbose --dry-run";

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let rows = (1..=20)
            .map(|i| format!("line {i}"))
            .collect::<Vec<_>>()
            .join("\n");
        div()
            .p(px(20.))
            .w(px(420.))
            .flex()
            .flex_col()
            .gap(px(20.))
            .child(
                CodeSnippet::new("single", SnippetType::Single, LONG)
                    .copy_texts("Copy", "Copied!")
                    .track_focus(&self.single)
                    .track_scroll(&self.single_scroll),
            )
            .child(
                CodeSnippet::new("multi", SnippetType::Multi, rows)
                    .copy_texts("Copy", "Copied!")
                    .expand_texts("Show more", "Show less")
                    .track_focus(&self.multi)
                    .track_scroll(&self.multi_scroll),
            )
    }
}

fn open(cx: &mut TestAppContext) -> (AnyWindowHandle, Page) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let handles = cx.update(|cx| Page {
        single: cx.focus_handle(),
        single_scroll: ScrollHandle::default(),
        multi: cx.focus_handle(),
        multi_scroll: ScrollHandle::default(),
    });
    let page = Page {
        single: handles.single.clone(),
        single_scroll: handles.single_scroll.clone(),
        multi: handles.multi.clone(),
        multi_scroll: handles.multi_scroll.clone(),
    };
    let window = cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(800.), px(700.)),
                })),
                ..Default::default()
            },
            cx,
            move |_, cx| cx.new(|_| handles),
        )
        .expect("open test window")
        .0
    });
    (window, page)
}

fn settle(window: AnyWindowHandle, cx: &mut TestAppContext) {
    cx.executor().advance_clock(Duration::from_millis(600));
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| window.render_frame(cx))
        .unwrap();
}

fn focused(handle: &FocusHandle, window: AnyWindowHandle, cx: &mut TestAppContext) -> bool {
    cx.update_window(window, |_, window, _| handle.is_focused(window))
        .unwrap()
}

fn press(key: &str, window: AnyWindowHandle, cx: &mut TestAppContext) {
    cx.update_window(window, |_, window, cx| window.press(key, cx))
        .unwrap();
    settle(window, cx);
}

#[gpui_kit::test]
fn single_code_area_takes_focus_and_scrolls_from_the_keyboard(cx: &mut TestAppContext) {
    let (window, page) = open(cx);
    settle(window, cx);
    let viewport = page.single_scroll.bounds();
    assert!(
        f32::from(page.single_scroll.max_offset().x) > 40.,
        "the long command overflows: {viewport:?}"
    );

    // A press on the code focuses it (`tabIndex={0}`).
    let at = point(viewport.origin.x + px(20.), viewport.center().y);
    cx.update_window(window, |_, window, cx| {
        window.pointer_down(at, MouseButton::Left, cx);
        window.pointer_up(at, MouseButton::Left, cx);
    })
    .unwrap();
    settle(window, cx);
    assert!(
        focused(&page.single, window, cx),
        "a press focuses the code"
    );

    // Right / Left scroll it by Chromium's 40px line step.
    press("right", window, cx);
    assert_eq!(page.single_scroll.offset().x, px(-40.));
    press("left", window, cx);
    assert_eq!(page.single_scroll.offset().x, px(0.));

    // A vertical wheel does not scroll it sideways (it goes to the page).
    cx.update_window(window, |_, window, cx| {
        window.pointer_move(at, None, cx);
        window.dispatch_event(
            gpui_kit::ScrollEvent {
                position: at,
                delta: ScrollDelta::Pixels(point(px(0.), px(-30.))),
                ..Default::default()
            }
            .to_platform_input(),
            cx,
        );
    })
    .unwrap();
    settle(window, cx);
    assert_eq!(page.single_scroll.offset().x, px(0.));

    // Tab moves on to the copy button (after the code in DOM order), and
    // Shift+Tab comes back.
    press("tab", window, cx);
    assert!(!focused(&page.single, window, cx), "Tab leaves the code");
    assert!(
        !focused(&page.multi, window, cx),
        "the copy button is next, not the multi code"
    );
    press("shift-tab", window, cx);
    assert!(
        focused(&page.single, window, cx),
        "Shift+Tab returns to the code"
    );
}

#[gpui_kit::test]
fn multi_tab_order_is_code_copy_expand(cx: &mut TestAppContext) {
    let (window, page) = open(cx);
    settle(window, cx);
    cx.update_window(window, |_, window, cx| page.multi.focus(window, cx))
        .unwrap();
    settle(window, cx);
    let collapsed = f32::from(page.multi_scroll.bounds().size.height);
    // 15 rows × 16px: the collapsed `max-height`.
    assert!((collapsed - 240.).abs() < 0.5, "{collapsed}");

    // Code → copy → Show more; Enter expands, Space collapses.
    press("tab", window, cx);
    assert!(!focused(&page.multi, window, cx));
    press("tab", window, cx);
    press("enter", window, cx);
    let expanded = f32::from(page.multi_scroll.bounds().size.height);
    // 20 rows + `pre` 1.5rem + expanded 1rem.
    assert!((expanded - 360.).abs() < 0.5, "Enter expands: {expanded}");
    press("space", window, cx);
    let again = f32::from(page.multi_scroll.bounds().size.height);
    assert!((again - 240.).abs() < 0.5, "Space collapses: {again}");

    // Back to the code: Shift+Tab twice.
    press("shift-tab", window, cx);
    press("shift-tab", window, cx);
    assert!(focused(&page.multi, window, cx));
}
