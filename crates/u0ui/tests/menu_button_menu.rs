//! A labeled `DropdownMenu` is Carbon's `MenuButton`: its menu is centred
//! under the trigger (`menuAlignment="bottom"`) and as wide as the trigger
//! within 12rem (with icons) and 18rem, so under the BranchToolbar's
//! "MacBook Pro" trigger (140px) it is 192px wide and starts 26px before
//! the trigger (design review FT1B, item 6).
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, Context, ElementId, Entity, InteractiveElement as _,
    IntoElement, ParentElement as _, Point, Render, Styled as _, TestAppContext,
    TestSupportExt as _, Window, WindowBounds, WindowOptions, div, px, size,
};
use std::time::Duration;
use u0ui::components::{DropdownMenu, DropdownMenuKind, DropdownMenuSize, MenuItem};
use u0ui::icons::CarbonIcon;

#[derive(Default)]
struct Page {
    ran: Vec<&'static str>,
}

fn item(entity: &Entity<Page>, label: &'static str) -> MenuItem {
    let entity = entity.clone();
    MenuItem::item(label)
        .icon(CarbonIcon::Laptop)
        .on_select(move |_, cx| {
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
            .flex()
            .items_start()
            .child(
                DropdownMenu::new("env")
                    .label("MacBook Pro")
                    .kind(DropdownMenuKind::Ghost)
                    .size(DropdownMenuSize::Sm)
                    .items([
                        item(&entity, "MacBook Pro"),
                        item(&entity, "build-box"),
                        MenuItem::divider(),
                        item(&entity, "Add to another machine"),
                    ]),
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
fn the_menu_is_centred_under_its_trigger(cx: &mut TestAppContext) {
    let (window, page) = open(cx);
    wait(window, cx);
    let trigger = ElementId::from((ElementId::from("env"), "trigger"));
    let at = bounds(window, cx, trigger.clone());
    assert!(f32::from(at.size.width) < 172., "{at:?}");
    act(window, cx, |w, cx| w.click(trigger.clone(), cx));
    // 10px before the trigger's start, in the first row: inside a menu
    // centred under it, outside one that hangs from its start edge.
    let probe = at.origin + gpui_kit::point(px(-10.), at.size.height + px(16.));
    click_page(window, cx, probe);
    assert_eq!(ran(&page, cx), Some("MacBook Pro"));
}

#[gpui_kit::test]
fn the_menu_does_not_grow_with_a_long_label(cx: &mut TestAppContext) {
    let (window, page) = open(cx);
    wait(window, cx);
    let trigger = ElementId::from((ElementId::from("env"), "trigger"));
    let at = bounds(window, cx, trigger.clone());
    act(window, cx, |w, cx| w.click(trigger.clone(), cx));
    // The menu ends 96px right of the trigger's centre: 4px past that, in
    // the "Add to another machine" row, is outside it.
    let centre = at.origin.x + at.size.width / 2.;
    let row = at.origin.y + at.size.height + px(4. + 2. * 32. + 9. + 16.);
    click_page(window, cx, Point::new(centre + px(100.), row));
    assert_eq!(ran(&page, cx), None);
}
