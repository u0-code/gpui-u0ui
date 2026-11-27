//! An empty `CommandPalette` shows its no-match text inside the listbox,
//! which keeps `.t3-palette-list { padding-bottom: var(--spacing-03) }`:
//! the reference's empty list is 90px (82px of `.t3-palette-empty` and 8px),
//! the palette 175px (design review FT1B, item 3).
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AppContext as _, Bounds, Context, IntoElement, ParentElement as _, Pixels, Point, Render,
    Styled as _, TestAppContext, Window, WindowBounds, WindowOptions, canvas, div, px, size,
};
use std::{cell::Cell, rc::Rc, time::Duration};
use u0ui::t3::shell::CommandPalette;

type Slot = Rc<Cell<Option<Bounds<Pixels>>>>;

struct Page {
    card: Slot,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let slot = self.card.clone();
        div().w(px(640.)).child(
            div()
                .relative()
                .child(CommandPalette::new("palette").auto_focus(false))
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
fn the_empty_list_keeps_its_bottom_padding(cx: &mut TestAppContext) {
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
    assert!((f32::from(card.size.height) - 175.).abs() < 1., "{card:?}");
}
