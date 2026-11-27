//! A contained (in-page) Sheet is not modal: the reference `Panel`
//! (`src/index.jsx`) has no focus trap, so Tab leaves the sheet's last
//! control and goes on to the rest of the page (design review FB4 #7).
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, Context, FocusHandle, InteractiveElement as _,
    IntoElement, ParentElement as _, Point, Render, Styled as _, TestAppContext, Window,
    WindowBounds, WindowOptions, div, px, size,
};
use std::time::Duration;
use u0ui::components::{Button, Sheet};

struct Page {
    after: FocusHandle,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(
                div().relative().h(px(400.)).w(px(500.)).child(
                    Sheet::new("sheet")
                        .contained(true)
                        .open(true)
                        .title("Edit instance")
                        .footer(Button::new("save", "Save")),
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

fn settle(window: AnyWindowHandle, cx: &mut TestAppContext) {
    cx.executor().advance_clock(Duration::from_millis(800));
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| window.render_frame(cx))
        .unwrap();
}

#[gpui_kit::test]
fn tab_leaves_a_contained_sheet(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let after = cx.update(|cx| cx.focus_handle());
    let page_after = after.clone();
    let window = cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(700.), px(600.)),
                })),
                ..Default::default()
            },
            cx,
            move |_, cx| cx.new(|_| Page { after: page_after }),
        )
        .expect("open test window")
        .0
    });
    settle(window, cx);
    // Close, Save, then out of the sheet: well within six presses.
    let mut reached = false;
    for _ in 0..6 {
        cx.update_window(window, |_, window, cx| window.press("tab", cx))
            .unwrap();
        settle(window, cx);
        reached |= cx
            .update_window(window, |_, window, _| after.is_focused(window))
            .unwrap();
    }
    assert!(reached, "Tab never left the contained sheet");
}
