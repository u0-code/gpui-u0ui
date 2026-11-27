//! Carbon `ComboButton` floats its menu as `MenuButton` does: centred under
//! the combo (`menuAlignment="bottom"`) and as wide as it within the menu's
//! 10rem to 18rem. Under the 144px "Commit" combo (lg) the menu is 160px,
//! from 8px before the combo to 8px past it, even when the combo sits in a
//! column that stretches its children (design review FP2, item 2).
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, Context, ElementId, Entity, InteractiveElement as _,
    IntoElement, ParentElement as _, Point, Render, Styled as _, TestAppContext,
    TestSupportExt as _, Window, WindowBounds, WindowOptions, div, px, size,
};
use std::time::Duration;
use u0ui::components::MenuItem;
use u0ui::patterns::{ComboButton, ComboButtonSize};

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
        div()
            .id("page")
            .test_support()
            .size_full()
            .p(px(200.))
            // A column stretches its children: the combo must still hug
            // its two buttons.
            .flex()
            .flex_col()
            .child(
                ComboButton::new("combo", "Commit")
                    .size(ComboButtonSize::Lg)
                    .item(item(&entity, "Commit & push"))
                    .item(item(&entity, "Commit & create PR"))
                    .item(item(&entity, "Amend last commit")),
            )
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
                    size: size(px(900.), px(700.)),
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

fn bounds(
    window: AnyWindowHandle,
    cx: &mut TestAppContext,
    id: ElementId,
) -> Bounds<gpui_kit::Pixels> {
    cx.update_window(window, |_, w, _| w.find(id).bounds())
        .unwrap()
}

fn click_page(window: AnyWindowHandle, cx: &mut TestAppContext, at: Point<gpui_kit::Pixels>) {
    let origin = bounds(window, cx, ElementId::from("page")).origin;
    cx.update_window(window, |_, w, cx| w.click_at("page", at - origin, cx))
        .unwrap();
    wait(window, cx);
}

fn ran(page: &Entity<Page>, cx: &mut TestAppContext) -> Option<&'static str> {
    cx.update(|cx| page.read(cx).ran.last().copied())
}

/// Open the menu; returns the combo's end edge and the first row's centre y.
fn open_menu(window: AnyWindowHandle, cx: &mut TestAppContext) -> (f32, f32) {
    let trigger = ElementId::from((ElementId::from("combo"), "trigger-wrap"));
    let at = bounds(window, cx, trigger.clone());
    cx.update_window(window, |_, w, cx| w.click(trigger.clone(), cx))
        .unwrap();
    wait(window, cx);
    let end = f32::from(at.origin.x + at.size.width);
    // The menu's 4px block padding, then the first 32px row.
    let row = f32::from(at.origin.y + at.size.height) + 4. + 16.;
    (end, row)
}

#[gpui_kit::test]
fn the_menu_starts_8px_before_the_combo(cx: &mut TestAppContext) {
    let (window, page) = open(cx);
    wait(window, cx);
    let (end, row) = open_menu(window, cx);
    // The combo is 144px (111 + 1 + 32): it starts at end − 144, the menu
    // 8px before that. 6px before the combo is inside the menu.
    click_page(window, cx, Point::new(px(end - 144. - 6.), px(row)));
    assert_eq!(ran(&page, cx), Some("Commit & push"));
}

#[gpui_kit::test]
fn the_menu_is_160px_not_the_row_width(cx: &mut TestAppContext) {
    let (window, page) = open(cx);
    wait(window, cx);
    let (end, row) = open_menu(window, cx);
    // The menu ends 8px past the combo: 12px past it is outside.
    click_page(window, cx, Point::new(px(end + 12.), px(row)));
    assert_eq!(ran(&page, cx), None);
}
