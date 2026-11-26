//! A bubble in a `MessageScrollerItem` renders as in the reference: the item
//! is a plain block (`.cu-scroller-item` has no CSS rule), so the bubble
//! fills the width up to `min(36rem, 85%)` at the start edge whatever its
//! `align`; in a flex column it still shrinks to its content and aligns
//! (design review C2, item 11).
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AppContext as _, Bounds, Context, IntoElement, ParentElement as _, Pixels, Point, Render,
    Styled as _, TestAppContext, Window, WindowBounds, WindowOptions, canvas, div, px, size,
};
use std::{cell::Cell, rc::Rc, time::Duration};
use u0ui::components::message_scroller::MessageScrollerItem;
use u0ui::components::{Bubble, BubbleAlign};

type Slot = Rc<Cell<Option<Bounds<Pixels>>>>;

struct Page {
    in_item: Slot,
    in_column: Slot,
}

fn probe(slot: &Slot) -> impl IntoElement {
    let slot = slot.clone();
    canvas(move |bounds, _, _| slot.set(Some(bounds)), |_, _, _, _| {})
        .absolute()
        .top_0()
        .left_0()
        .size_full()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .w(px(400.))
            .flex()
            .flex_col()
            .child(
                MessageScrollerItem::new("m1").child(
                    Bubble::new()
                        .align(BubbleAlign::End)
                        .child("Hi")
                        .child(probe(&self.in_item)),
                ),
            )
            .child(
                Bubble::new()
                    .align(BubbleAlign::End)
                    .child("Hi")
                    .child(probe(&self.in_column)),
            )
    }
}

#[gpui_kit::test]
fn scroller_items_are_blocks(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let in_item: Slot = Rc::default();
    let in_column: Slot = Rc::default();
    let (window, _page) = cx.update(|cx| {
        let (a, b) = (in_item.clone(), in_column.clone());
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(800.), px(600.)),
                })),
                ..Default::default()
            },
            cx,
            move |_, cx| {
                cx.new(|_| Page {
                    in_item: a,
                    in_column: b,
                })
            },
        )
        .expect("open test window")
    });
    cx.executor().advance_clock(Duration::from_secs(1));
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| window.render_frame(cx))
        .unwrap();

    let item = in_item.get().expect("measured");
    // Left-aligned, 85% of the 400px item (under the 36rem cap).
    assert_eq!(item.origin.x, px(0.));
    assert!((f32::from(item.size.width) - 340.).abs() < 0.5, "{item:?}");

    let column = in_column.get().expect("measured");
    // In a flex column the end bubble shrinks to "Hi" and sits at the end.
    assert!(f32::from(column.size.width) < 100., "{column:?}");
    assert!((f32::from(column.origin.x + column.size.width) - 400.).abs() < 0.5);
}
