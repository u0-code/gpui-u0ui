//! FilterMenu keyboard: every option is a tab stop after the search field,
//! toggled by Space, like the reference's `<label><input type="checkbox">`
//! rows (design review FG2 #5). Before, Tab went from the search straight to
//! "Select all".
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AppContext as _, Bounds, Context, IntoElement, ParentElement as _, Point, Render, SharedString,
    Styled as _, TestAppContext, Window, WindowBounds, WindowOptions, div, px, size,
};
use std::time::Duration;
use u0ui::u0::shell::{FilterItem, FilterMenu};

struct Page {
    changes: Vec<Vec<SharedString>>,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let page = cx.entity().downgrade();
        div().size_full().child(
            FilterMenu::new(
                "fm",
                "Projects",
                [
                    FilterItem::new("carbon-ui", "carbon-ui").count(6),
                    FilterItem::new("marketing-site", "marketing-site").count(2),
                    FilterItem::new("api-gateway", "api-gateway").count(9),
                ],
            )
            .default_value(["carbon-ui"])
            .on_value_change(move |v, _, cx| {
                page.update(cx, |p, cx| {
                    p.changes.push(v);
                    cx.notify();
                })
                .ok();
            }),
        )
    }
}

#[gpui_kit::test]
fn options_are_tab_stops_toggled_by_space(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let (window, page) = cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(800.), px(600.)),
                })),
                ..Default::default()
            },
            cx,
            |_, cx| cx.new(|_| Page { changes: vec![] }),
        )
        .expect("open test window")
    });
    let settle = |cx: &mut TestAppContext| {
        cx.executor().advance_clock(Duration::from_millis(500));
        cx.run_until_parked();
        cx.update_window(window, |_, window, cx| window.render_frame(cx))
            .unwrap();
    };
    let press = |key: &str, cx: &mut TestAppContext| {
        cx.update_window(window, |_, window, cx| window.press(key, cx))
            .unwrap();
        settle(cx);
    };
    settle(cx);
    // Tab to the trigger and open it: the search field takes focus.
    press("tab", cx);
    press("enter", cx);
    // Tab leaves the search for the first option; Space unticks it.
    press("tab", cx);
    press("space", cx);
    // The next Tab reaches the second option; Enter leaves it (a checkbox
    // ignores Enter) and Space ticks it, once.
    press("tab", cx);
    press("enter", cx);
    press("space", cx);
    let changes = cx.update(|cx| page.read(cx).changes.clone());
    assert_eq!(
        changes,
        vec![
            Vec::<SharedString>::new(),
            vec![SharedString::from("marketing-site")]
        ],
        "Tab from the search must reach each option in turn, and Space toggle it: {changes:?}"
    );
}
