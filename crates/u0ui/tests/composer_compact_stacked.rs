//! `ComposerCompact::stacked` lays the side-chat input out in two rows, as
//! u0's SplitSideChat board does in a 400px column (`.ovl .g-ccompact`):
//! at least 88px tall, the input across the top, attach and send below it
//! at the end, inside its 8px end and bottom padding.
use gpui_kit::test::TestQueryExt as _;
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AppContext as _, Bounds, Context, InteractiveElement as _, IntoElement, ParentElement as _,
    Pixels, Point, Render, Styled as _, TestAppContext, TestSupportExt as _, Window, WindowBounds,
    WindowOptions, div, px, size,
};
use std::time::Duration;
use u0ui::u0::composer::{ComposerCompact, stacked};

struct Page {
    stacked: bool,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().w(px(368.)).child(
            div().id("wrap").test_support().child(
                ComposerCompact::new("cc", "Claude")
                    .stacked(self.stacked)
                    .on_attach(|_, _| {}),
            ),
        )
    }
}

/// (the field, its send button, its attach button).
fn measure(
    cx: &mut TestAppContext,
    stacked: bool,
) -> (Bounds<Pixels>, Bounds<Pixels>, Bounds<Pixels>) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let window = cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(600.), px(300.)),
                })),
                ..Default::default()
            },
            cx,
            move |_, cx| cx.new(|_| Page { stacked }),
        )
        .expect("open test window")
        .0
    });
    cx.executor().advance_clock(Duration::from_millis(300));
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| window.render_frame(cx))
        .unwrap();
    cx.update_window(window, |_, window, _| {
        (
            window.find("wrap").bounds(),
            window.find_by_label("Send").bounds(),
            window.find_by_label("Attach").bounds(),
        )
    })
    .unwrap()
}

#[gpui_kit::test]
fn stacked_puts_the_controls_in_a_row_below_the_input(cx: &mut TestAppContext) {
    let (field, send, attach) = measure(cx, true);
    assert!(
        (f32::from(field.size.height) - stacked::MIN_HEIGHT).abs() <= 0.5,
        "{field:?}"
    );
    let [top, right, bottom, _] = stacked::PADDING;
    // Send sits in the bottom-right corner, inside the padding (and, at the
    // bottom, the 1px `border-strong-01` rule).
    assert!(
        (f32::from(field.right() - send.right()) - right).abs() <= 0.5,
        "{send:?}"
    );
    assert!(
        (f32::from(field.bottom() - send.bottom()) - (bottom + 1.)).abs() <= 0.5,
        "{send:?}"
    );
    // Below the input row: padding, the 22px input, the row gap.
    assert!(
        f32::from(send.top() - field.top()) >= top + stacked::INPUT_HEIGHT + stacked::ROW_GAP,
        "{send:?}"
    );
    // Attach just before send, 4px apart, on the same row.
    assert!(
        (f32::from(send.left() - attach.right()) - stacked::COLUMN_GAP).abs() <= 0.5,
        "{attach:?} {send:?}"
    );
    assert_eq!(attach.top(), send.top());
}

#[gpui_kit::test]
fn unstacked_stays_one_line(cx: &mut TestAppContext) {
    let (field, send, _) = measure(cx, false);
    assert!(f32::from(field.size.height) < 40., "{field:?}");
    assert!((field.center().y - send.center().y).abs() <= px(0.5));
}
