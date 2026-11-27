//! A `MobileSheet`'s list card sits 98px under the sheet top, so in the
//! reference 420px phone the sheet leaves 70px of backdrop (`.t3-msheet {
//! margin-top: 70px }`); a `PhoneFrame` marks its screen, so full-bleed
//! children round their bottom corners to the screen's curve (design
//! review FT4B, items 3 and 4).
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AnyElement, App, AppContext as _, Bounds, Context, IntoElement, ParentElement as _, Pixels,
    Point, Render, RenderOnce, Styled as _, TestAppContext, Window, WindowBounds, WindowOptions,
    canvas, div, px, size,
};
use std::{cell::Cell, rc::Rc, time::Duration};
use u0ui::icons::CarbonIcon;
use u0ui::t3::mobile::{ListRow, MobileListSection, MobileSheet, PhoneFrame, in_phone_screen};

type Slot = Rc<Cell<Option<Bounds<Pixels>>>>;

/// `child` with its bounds recorded into `slot`.
fn measured(slot: &Slot, child: impl IntoElement) -> gpui_kit::Div {
    let slot = slot.clone();
    div().relative().child(child).child(
        canvas(move |bounds, _, _| slot.set(Some(bounds)), |_, _, _, _| {})
            .absolute()
            .top_0()
            .left_0()
            .size_full(),
    )
}

/// Records [`in_phone_screen`] while it renders.
#[derive(IntoElement)]
struct Probe(Rc<Cell<Option<bool>>>);

impl RenderOnce for Probe {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        self.0.set(Some(in_phone_screen()));
        div()
    }
}

struct Page(Box<dyn Fn() -> AnyElement>);

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        (self.0)()
    }
}

fn render(
    cx: &mut TestAppContext,
    build: impl Fn() -> AnyElement + 'static,
) -> gpui_kit::AnyWindowHandle {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let window = cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(1000.), px(800.)),
                })),
                ..Default::default()
            },
            cx,
            move |_, cx| cx.new(|_| Page(Box::new(build))),
        )
        .expect("open test window")
        .0
    });
    for _ in 0..2 {
        cx.executor().advance_clock(Duration::from_millis(500));
        cx.run_until_parked();
        cx.update_window(window, |_, window, cx| window.render_frame(cx))
            .unwrap();
    }
    window
}

fn near(a: Pixels, b: f32) -> bool {
    (f32::from(a) - b).abs() < 0.5
}

#[gpui_kit::test]
fn the_sheet_list_card_sits_98px_under_the_sheet_top(cx: &mut TestAppContext) {
    let sheet: Slot = Rc::default();
    let list: Slot = Rc::default();
    let screen = Rc::new(Cell::new(None));
    let outside = Rc::new(Cell::new(None));
    let (s, l, p, o) = (sheet.clone(), list.clone(), screen.clone(), outside.clone());
    let window = render(cx, move || {
        div()
            .child(Probe(o.clone()))
            .child(
                PhoneFrame::new().height(420.).child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .child(Probe(p.clone()))
                        .child(
                            measured(
                                &s,
                                MobileSheet::new("sheet", "New thread")
                                    .inline(false)
                                    .primary("Start")
                                    .intro("Choose where the agent runs.")
                                    .child(measured(
                                        &l,
                                        MobileListSection::new("")
                                            .raised(true)
                                            .row(
                                                ListRow::new("run-on", "Run on")
                                                    .icon(CarbonIcon::Laptop)
                                                    .value("MacBook Pro"),
                                            )
                                            .row(
                                                ListRow::new("branch", "Branch")
                                                    .icon(CarbonIcon::Branch)
                                                    .value("main"),
                                            )
                                            .row(
                                                ListRow::new("model", "Model")
                                                    .icon(CarbonIcon::Bot)
                                                    .value("Opus"),
                                            ),
                                    )),
                            )
                            .mt_auto(),
                        ),
                ),
            )
            .into_any_element()
    });
    let _ = window;
    let sheet = sheet.get().expect("sheet");
    let list = list.get().expect("list");
    // Grabber (8 + 5), head 44, intro 21 + 12, empty heading 8.
    assert!(
        near(list.origin.y - sheet.origin.y + px(8.), 98.),
        "{sheet:?} {list:?}"
    );
    // In the 420px reference phone (three rows), the sheet leaves 70px of
    // backdrop under the 44px status inset: it is 286px tall and starts
    // 10 + 44 + 70 in.
    assert!(near(sheet.size.height, 286.), "{sheet:?}");
    assert!(near(sheet.origin.y, 124.), "{sheet:?}");
    // The frame marks its screen, and only its screen.
    assert_eq!(screen.get(), Some(true));
    assert_eq!(outside.get(), Some(false));
}
