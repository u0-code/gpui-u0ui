//! `TerminalDrawer` output wraps like `.t3-term-view { overflow: auto;
//! white-space: pre-wrap }` (t3.css): a line longer than the column breaks
//! at the edge and the view never grows sideways (design review FINAL-T3,
//! the AppShell demo at 940px).
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AppContext as _, Bounds, Context, IntoElement, ParentElement as _, Pixels, Point, Render,
    Styled as _, TestAppContext, Window, WindowBounds, WindowOptions, div, px, size,
};
use std::time::Duration;
use u0ui::t3::workspace::{TerminalDrawer, TerminalTab};

struct Page {
    width: f32,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().flex().flex_col().w(px(self.width)).h(px(400.)).child(
            TerminalDrawer::new("term", [TerminalTab::new("dev").running(true)]).lines([
                "$ bun run dev",
                "",
                "➜  Local: http://localhost:5173/",
            ]),
        )
    }
}

/// The output screen's bounds in a `width` column.
fn screen(cx: &mut TestAppContext, width: f32) -> Bounds<Pixels> {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
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
            move |_, cx| cx.new(|_| Page { width }),
        )
        .expect("open test window")
        .0
    });
    cx.executor().advance_clock(Duration::from_secs(1));
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| window.render_frame(cx))
        .unwrap();
    cx.update_window(window, |_, window, _| {
        window
            .find((gpui_kit::ElementId::from("term"), "screen"))
            .bounds()
    })
    .unwrap()
}

/// `.t3-term-view { padding: spacing-03 … }` top + bottom.
const PAD_Y: f32 = 16.;
/// `line-height: 1.33333` at 0.75rem.
const LINE: f32 = 16.;

fn rows(b: Bounds<Pixels>) -> f32 {
    ((f32::from(b.size.height) - PAD_Y) / LINE).round()
}

#[gpui_kit::test]
fn a_long_line_wraps_in_a_narrow_column(cx: &mut TestAppContext) {
    // The AppShell main column at a 940px window (234px).
    let b = screen(cx, 234.);
    assert!(rows(b) > 3., "wrapped rows: {}", rows(b));
    assert!(f32::from(b.size.width) <= 234., "no sideways growth: {b:?}");
}

#[gpui_kit::test]
fn short_lines_keep_one_row_each(cx: &mut TestAppContext) {
    let b = screen(cx, 600.);
    assert_eq!(rows(b), 3.);
}
