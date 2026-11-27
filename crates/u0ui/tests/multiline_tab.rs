//! Tab and Shift+Tab in a multi-line input move focus, as in an HTML
//! `<textarea>`, and never insert whitespace (design review FG3, S2).
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, Context, FocusHandle, InteractiveElement as _,
    IntoElement, ParentElement as _, Point, Render, Styled as _, TestAppContext, Window,
    WindowBounds, WindowOptions, div, px, size,
};
use std::time::Duration;
use u0ui::components::{InputGroup, InputGroupTextarea, Textarea, TextareaState};

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

/// Multi-line inputs: Tab and Shift+Tab move focus like an HTML `<textarea>`
/// and never insert whitespace (Carbon TextArea, InputGroup's textarea).
struct Areas {
    before: FocusHandle,
    after: FocusHandle,
    area: gpui_kit::Entity<TextareaState>,
    group: gpui_kit::Entity<TextareaState>,
}

impl Render for Areas {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .w(px(600.))
            .flex()
            .flex_col()
            .child(
                div()
                    .id("before")
                    .track_focus(&self.before.clone().tab_index(0).tab_stop(true))
                    .size(px(4.)),
            )
            .child(Textarea::new("area", &self.area).label("Description"))
            .child(InputGroup::new("group").textarea(InputGroupTextarea::new(&self.group)))
            .child(
                div()
                    .id("after")
                    .track_focus(&self.after.clone().tab_index(0).tab_stop(true))
                    .size(px(4.)),
            )
    }
}

#[gpui_kit::test]
fn tab_leaves_multi_line_inputs(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let (before, after) = cx.update(|cx| (cx.focus_handle(), cx.focus_handle()));
    let (b, a) = (before.clone(), after.clone());
    let (window, view) = cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(700.), px(600.)),
                })),
                ..Default::default()
            },
            cx,
            move |window, cx| {
                let area = cx.new(|cx| TextareaState::new(window, cx));
                let group = cx.new(|cx| TextareaState::new(window, cx));
                cx.new(|_| Areas {
                    before: b,
                    after: a,
                    area,
                    group,
                })
            },
        )
        .expect("open test window")
    });
    settle(window, cx);
    let (area, group) = view.read_with(cx, |v, _| (v.area.clone(), v.group.clone()));
    let focus_of = |s: &gpui_kit::Entity<TextareaState>, cx: &mut TestAppContext| {
        s.read_with(cx, gpui_kit::Focusable::focus_handle)
    };
    let (area_focus, group_focus) = (focus_of(&area, cx), focus_of(&group, cx));
    cx.update_window(window, |_, window, cx| before.focus(window, cx))
        .unwrap();
    settle(window, cx);

    press("tab", window, cx);
    assert_eq!(focused(window, cx), Some(area_focus.clone()));
    press("a", window, cx);
    press("tab", window, cx);
    assert_eq!(
        focused(window, cx),
        Some(group_focus.clone()),
        "Tab left the text area"
    );
    press("b", window, cx);
    press("tab", window, cx);
    assert_eq!(
        focused(window, cx),
        Some(after.clone()),
        "Tab left the group's text area"
    );
    press("shift-tab", window, cx);
    assert_eq!(focused(window, cx), Some(group_focus));
    press("shift-tab", window, cx);
    assert_eq!(focused(window, cx), Some(area_focus));
    let value = |s: &gpui_kit::Entity<TextareaState>, cx: &mut TestAppContext| {
        s.read_with(cx, |s, _| s.value().to_string())
    };
    assert_eq!(value(&area, cx), "a", "Tab inserted whitespace");
    assert_eq!(value(&group, cx), "b", "Tab inserted whitespace");
}
