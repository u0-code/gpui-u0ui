//! `ThreadHoverCard`'s last message is a bare `<p>` (`.t3-hovercard-last`
//! sets no size), so Carbon's `p` rule makes it 16px/24px: the reference
//! card is 181.42px tall, three 24px lines (design review FT1B, item 1).
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AppContext as _, Bounds, Context, IntoElement, ParentElement as _, Pixels, Point, Render,
    Styled as _, TestAppContext, Window, WindowBounds, WindowOptions, canvas, div, px, size,
};
use std::{cell::Cell, rc::Rc, time::Duration};
use u0ui::t3::Status;
use u0ui::t3::review::ThreadHoverCard;

type Slot = Rc<Cell<Option<Bounds<Pixels>>>>;

struct Page {
    card: Slot,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let slot = self.card.clone();
        div().flex().items_start().child(
            div()
                .relative()
                .child(
                    ThreadHoverCard::new("Fix flaky date picker test", "carbon-ui")
                        .status(Status::NeedsYou)
                        .provider("Codex \u{b7} GPT")
                        .time("3m ago")
                        .last_message(
                            "I need approval to run the full e2e suite against the staging \
                             database. It takes about 6 minutes.",
                        ),
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

#[gpui_kit::test]
fn the_last_message_is_a_16_on_24_paragraph(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let card: Slot = Rc::default();
    let window = cx.update(|cx| {
        let card = card.clone();
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(960.), px(400.)),
                })),
                ..Default::default()
            },
            cx,
            move |_, cx| cx.new(|_| Page { card }),
        )
        .expect("open test window")
        .0
    });
    cx.executor().advance_clock(Duration::from_secs(1));
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| window.render_frame(cx))
        .unwrap();
    let card = card.get().expect("measured");
    assert!((f32::from(card.size.width) - 320.).abs() < 0.5, "{card:?}");
    // 16 + top row + 8 + 18 title + 8 + 3 × 24 + 8 + 16 meta + 16.
    assert!(
        (f32::from(card.size.height) - 181.42).abs() < 2.,
        "{card:?}"
    );
}
