//! A message draws an action only when it has a handler to call: a side
//! chat's lines (u0's SplitSideChat board) carry none, so they show none.
//! `AssistantMessage::mark` leads the meta line with the app's own element,
//! in the 16px box of `ProviderIcon size="xs"`.
use gpui_kit::test::TestQueryExt as _;
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AnyElement, AnyWindowHandle, AppContext as _, Bounds, Context, InteractiveElement as _,
    IntoElement, ParentElement as _, Point, Render, Styled as _, TestAppContext,
    TestSupportExt as _, Window, WindowBounds, WindowOptions, div, px, size,
};
use std::time::Duration;
use u0ui::t3::timeline::{AssistantMessage, Prose, UserMessage};

struct Page {
    make: fn() -> AnyElement,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().w(px(400.)).child((self.make)())
    }
}

fn open(cx: &mut TestAppContext, make: fn() -> AnyElement) -> AnyWindowHandle {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
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
            move |_, cx| cx.new(|_| Page { make }),
        )
        .expect("open test window")
        .0
    });
    cx.executor().advance_clock(Duration::from_millis(300));
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| window.render_frame(cx))
        .unwrap();
    window
}

/// Which of `labels` the window draws.
fn drawn(cx: &mut TestAppContext, window: AnyWindowHandle, labels: &[&str]) -> Vec<String> {
    cx.update_window(window, |_, window, _| {
        labels
            .iter()
            .filter(|l| !window.find_all_by_label(l).is_empty())
            .map(|l| l.to_string())
            .collect()
    })
    .unwrap()
}

const ASSISTANT: [&str; 3] = ["Copy", "Reply to this response", "Fork from this response"];
const USER: [&str; 2] = ["Copy", "Edit and resend"];

#[gpui_kit::test]
fn an_assistant_message_without_handlers_draws_no_actions(cx: &mut TestAppContext) {
    let w = open(cx, || {
        AssistantMessage::new("a")
            .provider("Claude")
            .model("Sonnet")
            .child(Prose::markdown("Level 3 adds conditional UI."))
            .into_any_element()
    });
    assert!(drawn(cx, w, &ASSISTANT).is_empty());
}

#[gpui_kit::test]
fn an_assistant_message_draws_only_the_actions_it_can_call(cx: &mut TestAppContext) {
    let w = open(cx, || {
        AssistantMessage::new("a")
            .on_copy(|_, _| {})
            .on_fork(|_, _| {})
            .into_any_element()
    });
    assert_eq!(
        drawn(cx, w, &ASSISTANT),
        ["Copy", "Fork from this response"]
    );
}

#[gpui_kit::test]
fn a_user_message_without_handlers_draws_no_actions(cx: &mut TestAppContext) {
    let w = open(cx, || {
        UserMessage::new("u", "Why level 3?")
            .time("10:23")
            .into_any_element()
    });
    assert!(drawn(cx, w, &USER).is_empty());
}

#[gpui_kit::test]
fn a_user_message_draws_only_the_actions_it_can_call(cx: &mut TestAppContext) {
    let w = open(cx, || {
        UserMessage::new("u", "Why level 3?")
            .on_edit(|_, _| {})
            .into_any_element()
    });
    assert_eq!(drawn(cx, w, &USER), ["Edit and resend"]);
}

#[gpui_kit::test]
fn the_apps_mark_leads_the_meta_line_in_a_16px_box(cx: &mut TestAppContext) {
    let w = open(cx, || {
        AssistantMessage::new("a")
            .mark(div().id("mark").test_support().size(px(40.)))
            .model("Sonnet")
            .time("10:23")
            .into_any_element()
    });
    let mark = cx
        .update_window(w, |_, window, _| window.find("mark").bounds())
        .unwrap();
    // The box is 16px; the mark is centred in it, at the line's start.
    let centre = mark.center();
    assert!((f32::from(centre.x) - 8.).abs() <= 0.5, "{mark:?}");
    // `.t3-amsg-meta { min-height: 1.5rem }`: the 16px box is centred in 24px.
    assert!((f32::from(centre.y) - 12.).abs() <= 0.5, "{mark:?}");
}
