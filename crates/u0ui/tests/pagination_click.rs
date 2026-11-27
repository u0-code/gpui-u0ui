//! Regression: a controlled Pagination (the gallery playground: `page` +
//! `on_page_change` writing back into the view) moves on a press of Next
//! and on a pick in the page select (design review FB, blocker 1).
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AppContext as _, Bounds, Context, ElementId, IntoElement, ParentElement as _, Point, Render,
    Styled as _, TestAppContext, Window, WindowBounds, WindowOptions, div, px, size,
};
use std::sync::Arc;
use std::time::Duration;
use u0ui::components::Pagination;

struct Page {
    page: usize,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        div().p(px(20.)).w(px(1000.)).child(
            Pagination::new("pg", 103)
                .page(self.page)
                .on_page_change(move |p, _, cx| {
                    entity.update(cx, |this, cx| {
                        this.page = p;
                        cx.notify();
                    })
                }),
        )
    }
}

fn child(name: &str) -> ElementId {
    ElementId::NamedChild(Arc::new(ElementId::from("pg")), name.into())
}

/// The arrow's tooltip trigger wraps the button: `((button, "tooltip"), "trigger")`.
fn arrow(name: &str) -> ElementId {
    ElementId::from((ElementId::from((child(name), "tooltip")), "trigger"))
}

#[gpui_kit::test]
fn controlled_pagination_moves_on_next_and_page_select(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let (window, view) = cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(1100.), px(700.)),
                })),
                ..Default::default()
            },
            cx,
            |_, cx| cx.new(|_| Page { page: 1 }),
        )
        .expect("open test window")
    });
    let wait = |ms: u64, cx: &mut TestAppContext| {
        cx.executor().advance_clock(Duration::from_millis(ms));
        cx.run_until_parked();
        cx.update_window(window, |_, window, cx| window.render_frame(cx))
            .unwrap();
    };
    let page = |cx: &mut TestAppContext| cx.update(|cx| view.read(cx).page);
    wait(50, cx);
    wait(50, cx);

    cx.update_window(window, |_, window, cx| window.click(arrow("forward"), cx))
        .unwrap();
    wait(300, cx);
    assert_eq!(page(cx), 2, "Next did not move to page 2");
    cx.update_window(window, |_, window, cx| window.click(arrow("forward"), cx))
        .unwrap();
    wait(300, cx);
    assert_eq!(page(cx), 3);
    cx.update_window(window, |_, window, cx| window.click(arrow("back"), cx))
        .unwrap();
    wait(300, cx);
    assert_eq!(page(cx), 2, "Previous did not move back");

    // The page select: open it with the pointer, pick page 5 by keyboard.
    let trigger = ElementId::from((child("page"), "value"));
    cx.update_window(window, |_, window, cx| window.click(trigger, cx))
        .unwrap();
    wait(300, cx);
    for _ in 0..3 {
        cx.update_window(window, |_, window, cx| window.press("down", cx))
            .unwrap();
        wait(50, cx);
    }
    cx.update_window(window, |_, window, cx| window.press("enter", cx))
        .unwrap();
    wait(300, cx);
    assert_eq!(page(cx), 5, "the page select did not move to page 5");

    // Focus returns to the trigger after a pick (design review FB, item 2)…
    let page_trigger = || ElementId::from((child("page"), "value"));
    let focused = |cx: &mut TestAppContext, id: ElementId| {
        cx.update_window(window, |_, window, _| window.find(id).focused())
            .unwrap()
    };
    assert_eq!(
        focused(cx, page_trigger()),
        Some(true),
        "focus not back after a pick"
    );
    // …and after Escape, with the next Tab going on to Previous.
    cx.update_window(window, |_, window, cx| window.press("enter", cx))
        .unwrap();
    wait(300, cx);
    assert_eq!(
        focused(cx, page_trigger()),
        Some(false),
        "the menu took focus"
    );
    cx.update_window(window, |_, window, cx| window.press("escape", cx))
        .unwrap();
    wait(300, cx);
    assert_eq!(page(cx), 5);
    assert_eq!(
        focused(cx, page_trigger()),
        Some(true),
        "focus not back after Escape"
    );
    cx.update_window(window, |_, window, cx| window.press("tab", cx))
        .unwrap();
    wait(300, cx);
    assert_eq!(focused(cx, page_trigger()), Some(false));
    assert_eq!(
        focused(cx, child("back")),
        Some(true),
        "Tab did not go on to Previous"
    );
}
