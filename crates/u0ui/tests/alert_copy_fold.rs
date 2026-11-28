//! An alert's words are copied whole, folded ones too, and long words fold
//! behind a link that shows them all and then fewer: Tab reaches the link,
//! then the Copy button; Enter presses each.
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, Context, InteractiveElement as _, IntoElement,
    ParentElement as _, Point, Render, ScrollHandle, StatefulInteractiveElement as _, Styled as _,
    TestAppContext, Window, WindowBounds, WindowOptions, div, px, size,
};
use std::time::Duration;
use u0ui::components::alert::{Alert, AlertVariant};

struct Page {
    words: String,
    scroll: ScrollHandle,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().p(px(20.)).w(px(760.)).child(
            div()
                .id("wrap")
                .overflow_y_scroll()
                .track_scroll(&self.scroll)
                .child(
                    Alert::new("stopped")
                        .variant(AlertVariant::Error)
                        .title("The agent stopped")
                        .words(self.words.clone())
                        .copy("Copy", "Copied")
                        .fold("Show all", "Show less"),
                ),
        )
    }
}

fn open(cx: &mut TestAppContext) -> (AnyWindowHandle, ScrollHandle, String) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let words: String = (1..=20)
        .map(|i| format!("    at frame{i} (node:internal/streams:{i}:10)"))
        .collect::<Vec<_>>()
        .join("\n");
    let scroll = ScrollHandle::default();
    let page = Page {
        words: words.clone(),
        scroll: scroll.clone(),
    };
    let window = cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(900.), px(800.)),
                })),
                ..Default::default()
            },
            cx,
            move |_, cx| cx.new(|_| page),
        )
        .expect("open test window")
        .0
    });
    (window, scroll, words)
}

fn settle(window: AnyWindowHandle, cx: &mut TestAppContext) {
    cx.executor().advance_clock(Duration::from_millis(600));
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| window.render_frame(cx))
        .unwrap();
}

fn press(key: &str, window: AnyWindowHandle, cx: &mut TestAppContext) {
    cx.update_window(window, |_, window, cx| window.press(key, cx))
        .unwrap();
    settle(window, cx);
}

#[gpui_kit::test]
fn long_words_fold_and_copy_copies_all_of_them(cx: &mut TestAppContext) {
    let (window, scroll, words) = open(cx);
    settle(window, cx);
    let folded = f32::from(scroll.bounds().size.height);

    // Tab: the Show all link; Enter shows every line.
    press("tab", window, cx);
    press("enter", window, cx);
    let all = f32::from(scroll.bounds().size.height);
    assert!(
        all > folded + 100.,
        "Show all shows the other 14 lines: {folded} -> {all}"
    );
    // Enter again: Show less.
    press("enter", window, cx);
    let again = f32::from(scroll.bounds().size.height);
    assert!((again - folded).abs() < 0.5, "{again} vs {folded}");

    // Tab: the Copy button; Enter copies the title and every line.
    press("tab", window, cx);
    press("enter", window, cx);
    let copied = cx.read_from_clipboard().and_then(|c| c.text());
    assert_eq!(copied, Some(format!("The agent stopped\n{words}")));
}
