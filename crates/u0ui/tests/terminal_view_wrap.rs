//! `TerminalView` lines wrap like `.g-term-screen { white-space: pre-wrap;
//! word-break: break-all }` (ghost.css): a row fills to the column edge and
//! breaks mid-token, and the cursor (`.g-term-cursor`, an inline-block after
//! the last character) stays on the last text row.
//!
//! `harness = false`: the platform text system (CoreText) must be created on
//! the main thread, so `main` runs the checks in order.
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AppContext as _, Bounds, Context, ElementId, IntoElement, ParentElement as _, Pixels, Point,
    Render, SharedString, Styled as _, TestAppContext, TestDispatcher, Window, WindowBounds,
    WindowOptions, div, px, size,
};
use std::time::Duration;
use u0ui::u0::panels::TerminalView;

/// IBM Plex Mono's advance at 0.75rem: 0.6em.
const COL: f32 = 7.2;
/// `.g-term { line-height: 1.33333 }` at 0.75rem.
const LINE: f32 = 16.;
/// `.g-term-screen { padding: … var(--spacing-04) }`, both sides.
const PAD_X: f32 = 24.;

struct Page {
    width: f32,
    lines: Vec<String>,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .w(px(self.width))
            .child(TerminalView::new("term").lines(self.lines.clone()))
    }
}

/// A test app on the platform's real text system (CoreText on macOS): the
/// default test text system gives every character, U+200B included, a full
/// column, so it cannot show where the real wrapper breaks.
fn real_text() -> TestAppContext {
    TestAppContext::build_with_text_system(
        TestDispatcher::new(0),
        None,
        gpui_kit::platform::current_platform(true).text_system(),
    )
}

/// Lay out `lines` in a `cols`-column terminal; the bounds of each line.
fn layout(cols: usize, lines: &[&str]) -> Vec<Bounds<Pixels>> {
    let cx = &mut real_text();
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    // A little slack past `cols` columns, short of another one.
    let width = cols as f32 * COL + PAD_X + COL / 2.;
    let owned: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
    let window = cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(800.), px(400.)),
                })),
                ..Default::default()
            },
            cx,
            move |_, cx| {
                cx.new(|_| Page {
                    width,
                    lines: owned,
                })
            },
        )
        .expect("open test window")
        .0
    });
    cx.executor().advance_clock(Duration::from_millis(100));
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| window.render_frame(cx))
        .unwrap();
    cx.update_window(window, |_, window, _| {
        (0..lines.len())
            .map(|i| {
                let id: ElementId = (
                    ElementId::from("term"),
                    SharedString::from(format!("line-{i}")),
                )
                    .into();
                window.find(id).bounds()
            })
            .collect()
    })
    .unwrap()
}

fn rows(b: &Bounds<Pixels>) -> f32 {
    (f32::from(b.size.height) / LINE).round()
}

fn break_all_fills_each_row_before_breaking() {
    // 39 characters in 20 columns: "ab " and 17 x's on the first row (with
    // `word-break: normal` the token would move down whole: 3 rows), the
    // other 19 x's and then the cursor on the second.
    let line = format!("ab {}", "x".repeat(36));
    let b = layout(20, &[&line]);
    assert_eq!(rows(&b[0]), 2., "{b:?}");
}

fn the_cursor_stays_on_the_last_text_row() {
    // A wrapped last line: 30 characters, then the cursor, in 20 columns.
    let long = "y".repeat(30);
    let b = layout(20, &["$ ls", &long]);
    assert_eq!(rows(&b[0]), 1.);
    assert_eq!(rows(&b[1]), 2., "{b:?}");
}

fn a_full_row_pushes_the_cursor_down() {
    // Like an inline-block at the edge: 20 characters fill the row.
    let full = "z".repeat(20);
    let b = layout(20, &[&full]);
    assert_eq!(rows(&b[0]), 2., "{b:?}");
}

fn short_lines_keep_one_row_each() {
    let b = layout(60, &["$ pnpm dev", "", "\x1b[90m10:42:03\x1b[0m hmr", "$ "]);
    assert!(b.iter().all(|b| rows(b) == 1.), "{b:?}");
}

fn main() {
    let checks: [(&str, fn()); 4] = [
        (
            "break_all_fills_each_row_before_breaking",
            break_all_fills_each_row_before_breaking,
        ),
        (
            "the_cursor_stays_on_the_last_text_row",
            the_cursor_stays_on_the_last_text_row,
        ),
        (
            "a_full_row_pushes_the_cursor_down",
            a_full_row_pushes_the_cursor_down,
        ),
        (
            "short_lines_keep_one_row_each",
            short_lines_keep_one_row_each,
        ),
    ];
    for (name, check) in checks {
        check();
        println!("test {name} ... ok");
    }
}
