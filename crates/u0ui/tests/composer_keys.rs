//! The T3 Composer's keys (design review FT2, items 1 and 2): Enter sends,
//! Shift+Enter is a new line, Tab and Shift+Tab move focus as in a plain
//! `<textarea>`, and Tab accepts a suggestion only while one is showing.
use gpui_kit::component::input::TextareaState;
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, Context, Entity, Focusable as _, IntoElement,
    ParentElement as _, Point, Render, SharedString, Styled as _, TestAppContext, Window,
    WindowBounds, WindowOptions, div, px, size,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use u0ui::t3::composer::Composer;

struct Page {
    input: Entity<TextareaState>,
    sent: Rc<RefCell<Vec<SharedString>>>,
    suggestion: Option<SharedString>,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let sent = self.sent.clone();
        let mut composer = Composer::new("composer")
            .input(self.input.clone())
            .on_send(move |text, _, _| sent.borrow_mut().push(text));
        if let Some(s) = self.suggestion.clone() {
            composer = composer.suggestion(s);
        }
        div().size_full().p(px(16.)).child(composer)
    }
}

struct Fixture {
    window: AnyWindowHandle,
    page: Entity<Page>,
    sent: Rc<RefCell<Vec<SharedString>>>,
}

fn open(cx: &mut TestAppContext, suggestion: Option<&str>) -> Fixture {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let sent = Rc::new(RefCell::new(Vec::new()));
    let suggestion = suggestion.map(SharedString::from);
    let page_sent = sent.clone();
    let (window, page) = cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(900.), px(400.)),
                })),
                ..Default::default()
            },
            cx,
            move |window, cx| {
                let input = cx.new(|cx| TextareaState::new(window, cx));
                cx.new(|_| Page {
                    input,
                    sent: page_sent,
                    suggestion,
                })
            },
        )
        .expect("open test window")
    });
    let f = Fixture { window, page, sent };
    settle(cx, &f);
    f
}

fn settle(cx: &mut TestAppContext, f: &Fixture) {
    cx.executor().advance_clock(Duration::from_millis(300));
    cx.run_until_parked();
    cx.update_window(f.window, |_, window, cx| window.render_frame(cx))
        .unwrap();
}

fn focus_input(cx: &mut TestAppContext, f: &Fixture) {
    let page = f.page.clone();
    cx.update_window(f.window, |_, window, cx| {
        let focus = page.read(cx).input.read(cx).focus_handle(cx);
        focus.focus(window, cx);
    })
    .unwrap();
    settle(cx, f);
}

fn press(cx: &mut TestAppContext, f: &Fixture, key: &str) {
    cx.update_window(f.window, |_, window, cx| window.press(key, cx))
        .unwrap();
    settle(cx, f);
}

fn type_text(cx: &mut TestAppContext, f: &Fixture, text: &str) {
    cx.update_window(f.window, |_, window, cx| window.input(text, cx))
        .unwrap();
    settle(cx, f);
}

fn value(cx: &mut TestAppContext, f: &Fixture) -> String {
    cx.update(|cx| f.page.read(cx).input.read(cx).value().to_string())
}

fn input_focused(cx: &mut TestAppContext, f: &Fixture) -> bool {
    let page = f.page.clone();
    cx.update_window(f.window, |_, window, cx| {
        page.read(cx)
            .input
            .read(cx)
            .focus_handle(cx)
            .is_focused(window)
    })
    .unwrap()
}

#[gpui_kit::test]
fn enter_sends_and_shift_enter_inserts_a_new_line(cx: &mut TestAppContext) {
    let f = open(cx, None);
    focus_input(cx, &f);
    type_text(cx, &f, "first");
    press(cx, &f, "shift-enter");
    assert!(f.sent.borrow().is_empty(), "Shift+Enter sent the message");
    assert_eq!(value(cx, &f), "first\n", "Shift+Enter is a new line");
    type_text(cx, &f, "second");
    press(cx, &f, "enter");
    assert_eq!(
        f.sent.borrow().as_slice(),
        &[SharedString::from("first\nsecond")],
        "Enter sends the whole text"
    );
    assert_eq!(value(cx, &f), "", "sending clears the input");
    // A blank input sends nothing and gains no new line.
    press(cx, &f, "enter");
    assert_eq!(f.sent.borrow().len(), 1);
    assert_eq!(value(cx, &f), "");
}

#[gpui_kit::test]
fn tab_moves_focus_instead_of_indenting(cx: &mut TestAppContext) {
    let f = open(cx, None);
    focus_input(cx, &f);
    type_text(cx, &f, "text");
    press(cx, &f, "tab");
    assert_eq!(value(cx, &f), "text", "Tab indented the text");
    assert!(!input_focused(cx, &f), "Tab kept focus in the input");
    focus_input(cx, &f);
    press(cx, &f, "shift-tab");
    assert_eq!(value(cx, &f), "text", "Shift+Tab dedented the text");
    assert!(!input_focused(cx, &f), "Shift+Tab kept focus in the input");
}

#[gpui_kit::test]
fn tab_accepts_only_a_showing_suggestion(cx: &mut TestAppContext) {
    let f = open(cx, Some("Run the tests"));
    focus_input(cx, &f);
    press(cx, &f, "tab");
    assert_eq!(value(cx, &f), "Run the tests", "Tab accepts the suggestion");
    assert!(input_focused(cx, &f), "accepting keeps focus in the input");
    // The caret ends after the accepted text.
    type_text(cx, &f, "!");
    assert_eq!(value(cx, &f), "Run the tests!");
    // Once text is there the suggestion is gone, and Tab moves focus.
    press(cx, &f, "tab");
    assert_eq!(value(cx, &f), "Run the tests!");
    assert!(
        !input_focused(cx, &f),
        "Tab without a suggestion moves focus"
    );
}
