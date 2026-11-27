//! SplitButton's menu (T3 `IconMenu`) hangs from the chevron's start edge,
//! as Carbon's `Menu` places itself given `x: [left, right]`, and focuses
//! its first item when it opens (design review FT4, item 8). Near the
//! window's end it lines up with the chevron's end edge instead.
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, Context, ElementId, Entity, InteractiveElement as _,
    IntoElement, ParentElement as _, Point, Render, Styled as _, TestAppContext,
    TestSupportExt as _, Window, WindowBounds, WindowOptions, div, px, size,
};
use std::time::Duration;
use u0ui::components::MenuItem;
use u0ui::t3::mobile::SplitButton;

#[derive(Default)]
struct Page {
    ran: Vec<&'static str>,
}

fn item(entity: &Entity<Page>, label: &'static str) -> MenuItem {
    let entity = entity.clone();
    MenuItem::item(label).on_select(move |_, cx| {
        entity.update(cx, |this, cx| {
            this.ran.push(label);
            cx.notify();
        })
    })
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let menu = |id: &'static str| {
            SplitButton::new(id, "Commit & push").menu([
                item(&entity, "Commit only"),
                item(&entity, "Commit on new branch"),
                item(&entity, "Create PR"),
            ])
        };
        div()
            .id("page")
            .test_support()
            .size_full()
            .p(px(20.))
            .flex()
            .flex_col()
            .gap(px(300.))
            .child(div().flex().child(menu("start")))
            .child(div().flex().justify_end().child(menu("end")))
    }
}

fn open(cx: &mut TestAppContext) -> (AnyWindowHandle, Entity<Page>) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(900.), px(900.)),
                })),
                ..Default::default()
            },
            cx,
            |_, cx| cx.new(|_| Page::default()),
        )
        .expect("open test window")
    })
}

fn wait(window: AnyWindowHandle, cx: &mut TestAppContext) {
    for _ in 0..3 {
        cx.executor().advance_clock(Duration::from_millis(100));
        cx.run_until_parked();
        cx.update_window(window, |_, window, cx| window.render_frame(cx))
            .unwrap();
    }
}

fn act(
    window: AnyWindowHandle,
    cx: &mut TestAppContext,
    f: impl FnOnce(&mut Window, &mut gpui_kit::App),
) {
    cx.update_window(window, |_, window, cx| f(window, cx))
        .unwrap();
    wait(window, cx);
}

fn menu(id: &'static str) -> ElementId {
    ElementId::from((ElementId::from(id), "menu"))
}

fn bounds(
    window: AnyWindowHandle,
    cx: &mut TestAppContext,
    id: ElementId,
) -> Bounds<gpui_kit::Pixels> {
    cx.update_window(window, |_, w, _| w.find(id).bounds())
        .unwrap()
}

/// Click the page at `at` (window coordinates).
fn click_page(window: AnyWindowHandle, cx: &mut TestAppContext, at: Point<gpui_kit::Pixels>) {
    let origin = bounds(window, cx, ElementId::from("page")).origin;
    act(window, cx, |w, cx| w.click_at("page", at - origin, cx));
}

fn ran(page: &Entity<Page>, cx: &mut TestAppContext) -> Option<&'static str> {
    cx.update(|cx| page.read(cx).ran.last().copied())
}

#[gpui_kit::test]
fn the_menu_focuses_its_first_item_on_open(cx: &mut TestAppContext) {
    let (window, page) = open(cx);
    wait(window, cx);
    let trigger = ElementId::from((menu("start"), "trigger"));
    act(window, cx, |w, cx| w.click(trigger, cx));
    act(window, cx, |w, cx| w.press("enter", cx));
    assert_eq!(ran(&page, cx), Some("Commit only"));
}

#[gpui_kit::test]
fn the_menu_hangs_from_the_chevron_start(cx: &mut TestAppContext) {
    let (window, page) = open(cx);
    wait(window, cx);
    let trigger = ElementId::from((menu("start"), "trigger"));
    act(window, cx, |w, cx| w.click(trigger.clone(), cx));
    let chevron = bounds(window, cx, trigger.clone());
    // 150px right of the chevron's start, in the first row: inside a menu
    // that starts at the chevron's left edge (it is ~188px wide), outside
    // one that ends at its right edge.
    let probe = chevron.origin + gpui_kit::point(px(150.), chevron.size.height + px(16.));
    click_page(window, cx, probe);
    assert_eq!(ran(&page, cx), Some("Commit only"));
}

#[gpui_kit::test]
fn near_the_window_end_the_menu_lines_up_with_the_chevron_end(cx: &mut TestAppContext) {
    let (window, page) = open(cx);
    wait(window, cx);
    let trigger = ElementId::from((menu("end"), "trigger"));
    act(window, cx, |w, cx| w.click(trigger.clone(), cx));
    let chevron = bounds(window, cx, trigger.clone());
    // 150px left of the chevron's end, in the first row.
    let probe = chevron.origin
        + gpui_kit::point(chevron.size.width - px(150.), chevron.size.height + px(16.));
    click_page(window, cx, probe);
    assert_eq!(ran(&page, cx), Some("Commit only"));
}
