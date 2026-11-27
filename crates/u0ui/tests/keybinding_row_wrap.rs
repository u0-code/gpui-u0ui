//! A narrow `KeybindingRow` wraps its command instead of running it under
//! the key caps (`.t3-kbrow { grid-template-columns: 1fr 9rem 12rem 1.5rem
//! 2rem }`: the `1fr` column is at least its longest word, so the command
//! wraps onto more lines; design review FT4, item 1).
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AppContext as _, Bounds, Context, IntoElement, ParentElement as _, Pixels, Point, Render,
    Styled as _, TestAppContext, Window, WindowBounds, WindowOptions, canvas, div, px, size,
};
use std::{cell::Cell, rc::Rc, time::Duration};
use u0ui::t3::workspace::KeybindingRow;

type Slot = Rc<Cell<Option<Bounds<Pixels>>>>;

struct Page {
    width: f32,
    row: Slot,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let slot = self.row.clone();
        div().w(px(self.width)).child(
            div()
                .relative()
                .child(
                    KeybindingRow::new("kb", "Approve pending request", "⌘+⇧+Y")
                        .when_clause("pendingApproval")
                        .custom(true),
                )
                .child(
                    canvas(move |bounds, _, _| slot.set(Some(bounds)), |_, _, _, _| {})
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full(),
                ),
        )
    }
}

fn row_bounds(cx: &mut TestAppContext, width: f32) -> Bounds<Pixels> {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let row: Slot = Rc::default();
    let window = cx.update(|cx| {
        let row = row.clone();
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(1000.), px(400.)),
                })),
                ..Default::default()
            },
            cx,
            move |_, cx| cx.new(|_| Page { width, row }),
        )
        .expect("open test window")
        .0
    });
    cx.executor().advance_clock(Duration::from_secs(1));
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| window.render_frame(cx))
        .unwrap();
    row.get().expect("measured")
}

#[gpui_kit::test]
fn a_wide_row_keeps_the_command_on_one_line(cx: &mut TestAppContext) {
    let row = row_bounds(cx, 960.);
    // `min-height: size-md`: one 40px line.
    assert!((f32::from(row.size.height) - 40.).abs() < 0.5, "{row:?}");
}

#[gpui_kit::test]
fn a_narrow_row_wraps_the_command(cx: &mut TestAppContext) {
    let row = row_bounds(cx, 574.);
    // The command wraps (taller than one line) and the row keeps its width.
    assert!(f32::from(row.size.height) > 40.5, "{row:?}");
    assert!((f32::from(row.size.width) - 574.).abs() < 0.5, "{row:?}");
}
