//! Carbon `Menu` focuses its first enabled item when it opens, from the
//! pointer or the keyboard, so Enter runs it and the first ↓ moves to the
//! second item (design review FB, item 5): DropdownMenu, ContextMenu and
//! Menubar. Also: NavigationMenu's Escape closes a menu opened with the
//! pointer while focus is still on its title (item 6).
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, Context, ElementId, Entity, IntoElement,
    ParentElement as _, Point, Render, Styled as _, TestAppContext, Window, WindowBounds,
    WindowOptions, div, px, size,
};
use std::sync::Arc;
use std::time::Duration;
use u0ui::components::{ContextMenu, DropdownMenu, MenuItem, Menubar, NavItem, NavigationMenu};

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
            .p(px(20.))
            .flex()
            .flex_col()
            .gap(px(200.))
            .child(
                DropdownMenu::new("dd")
                    .label("Actions")
                    .item(MenuItem::item("Disabled").disabled(true))
                    .item(item(&entity, "Restart"))
                    .item(item(&entity, "Resize"))
                    .item(MenuItem::divider())
                    .item(item(&entity, "Delete")),
            )
            .child(
                ContextMenu::new(
                    "ctx",
                    div().w(px(200.)).h(px(80.)).child("Right-click here"),
                )
                .item(item(&entity, "Copy"))
                .item(item(&entity, "Rename")),
            )
            .child(
                Menubar::new("bar")
                    .menu(
                        "File",
                        vec![item(&entity, "New tab"), item(&entity, "Open…")],
                    )
                    .menu("Help", vec![item(&entity, "Documentation")]),
            )
    }
}

fn open(cx: &mut TestAppContext) -> (AnyWindowHandle, Entity<Page>) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let window = cx.update(|cx| {
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
    });
    let (window, page) = window;
    (window, page)
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

fn last(page: &Entity<Page>, cx: &mut TestAppContext) -> Option<&'static str> {
    cx.update(|cx| page.read(cx).ran.last().copied())
}

fn dd_trigger() -> ElementId {
    ElementId::from((ElementId::from("dd"), "trigger"))
}

fn bar_trigger(index: usize) -> ElementId {
    let menu = ElementId::NamedChild(
        Arc::new(ElementId::from("bar")),
        format!("menu-{index}").into(),
    );
    ElementId::from((menu, "trigger"))
}

#[gpui_kit::test]
fn dropdown_menu_focuses_its_first_enabled_item(cx: &mut TestAppContext) {
    let (window, page) = open(cx);
    wait(window, cx);
    // Pointer open: Enter runs the first enabled item.
    act(window, cx, |w, cx| w.click(dd_trigger(), cx));
    act(window, cx, |w, cx| w.press("enter", cx));
    assert_eq!(last(&page, cx), Some("Restart"));
    // Pointer open again: the first ↓ moves to the second item.
    act(window, cx, |w, cx| w.click(dd_trigger(), cx));
    act(window, cx, |w, cx| w.press("down", cx));
    act(window, cx, |w, cx| w.press("enter", cx));
    assert_eq!(last(&page, cx), Some("Resize"));
    // Keyboard open (focus is back on the trigger): the same.
    act(window, cx, |w, cx| w.press("enter", cx));
    act(window, cx, |w, cx| w.press("down", cx));
    act(window, cx, |w, cx| w.press("enter", cx));
    assert_eq!(last(&page, cx), Some("Resize"));
}

#[gpui_kit::test]
fn context_menu_focuses_its_first_item(cx: &mut TestAppContext) {
    let (window, page) = open(cx);
    wait(window, cx);
    let area = ElementId::from((ElementId::from("ctx"), "area"));
    act(window, cx, |w, cx| w.right_click(area.clone(), cx));
    act(window, cx, |w, cx| w.press("enter", cx));
    assert_eq!(last(&page, cx), Some("Copy"));
    act(window, cx, |w, cx| w.right_click(area, cx));
    act(window, cx, |w, cx| w.press("down", cx));
    act(window, cx, |w, cx| w.press("enter", cx));
    assert_eq!(last(&page, cx), Some("Rename"));
}

#[gpui_kit::test]
fn menubar_focuses_the_first_item(cx: &mut TestAppContext) {
    let (window, page) = open(cx);
    wait(window, cx);
    act(window, cx, |w, cx| w.click(bar_trigger(0), cx));
    act(window, cx, |w, cx| w.press("enter", cx));
    assert_eq!(last(&page, cx), Some("New tab"));
    act(window, cx, |w, cx| w.click(bar_trigger(0), cx));
    act(window, cx, |w, cx| w.press("down", cx));
    act(window, cx, |w, cx| w.press("enter", cx));
    assert_eq!(last(&page, cx), Some("Open…"));
}

struct Nav;

impl Render for Nav {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().h(px(48.)).child(
            NavigationMenu::new("nav")
                .brand("[Platform]")
                .item(NavItem::link("Catalog"))
                .item(NavItem::menu(
                    "Manage",
                    [NavItem::link("Access (IAM)"), NavItem::link("Billing")],
                )),
        )
    }
}

#[gpui_kit::test]
fn navigation_menu_escape_closes_a_pointer_opened_menu(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let (window, _nav) = cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(900.), px(600.)),
                })),
                ..Default::default()
            },
            cx,
            |_, cx| cx.new(|_| Nav),
        )
        .expect("open test window")
    });
    wait(window, cx);
    let item = ElementId::NamedChild(Arc::new(ElementId::from("nav")), "item-1".into());
    let title = ElementId::from((item.clone(), "title"));
    let list = ElementId::from((item, "list"));
    let shown = |cx: &mut TestAppContext| {
        cx.update_window(window, |_, w, _| w.try_find(list.clone()).is_some())
            .unwrap()
    };
    act(window, cx, |w, cx| w.click(title.clone(), cx));
    assert!(shown(cx), "Manage did not open");
    let focused = cx
        .update_window(window, |_, w, _| w.find(title.clone()).focused())
        .unwrap();
    assert_eq!(focused, Some(true), "focus left the title");
    act(window, cx, |w, cx| w.press("escape", cx));
    assert!(!shown(cx), "Escape on the title did not close Manage");
}
