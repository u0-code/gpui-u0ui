//! Tab and Shift+Tab in an open NavigationMenu header menu (design review
//! FB3, item 2). The reference (`HeaderMenu`) keeps document order: the
//! title, its items, then what follows the menu. Tab on the last item moves
//! to the next element after the menu (the search action) and closes it;
//! Shift+Tab on the first item goes back to the title and closes it.
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, Context, ElementId, FocusHandle,
    InteractiveElement as _, IntoElement, ParentElement as _, Point, Render, Styled as _,
    TestAppContext, Window, WindowBounds, WindowOptions, div, px, size,
};
use std::sync::Arc;
use std::time::Duration;
use u0ui::components::{NavItem, NavigationMenu};

struct Nav {
    search: FocusHandle,
}

impl Render for Nav {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().h(px(48.)).child(
            NavigationMenu::new("nav")
                .brand_prefix("IBM")
                .brand("[Platform]")
                .item(NavItem::link("Catalog").active(true))
                .item(NavItem::link("Docs"))
                .item(NavItem::menu(
                    "Manage",
                    [
                        NavItem::link("Access (IAM)"),
                        NavItem::link("Billing"),
                        NavItem::link("Support"),
                    ],
                ))
                .action(
                    div()
                        .id("search")
                        .size(px(32.))
                        .track_focus(&self.search.clone().tab_stop(true)),
                ),
        )
    }
}

fn open(cx: &mut TestAppContext) -> (AnyWindowHandle, FocusHandle) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let (window, nav) = cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(960.), px(400.)),
                })),
                ..Default::default()
            },
            cx,
            |_, cx| {
                cx.new(|cx| Nav {
                    search: cx.focus_handle(),
                })
            },
        )
        .expect("open test window")
    });
    wait(window, cx);
    let search = cx.update(|cx| nav.read(cx).search.clone());
    (window, search)
}

fn wait(window: AnyWindowHandle, cx: &mut TestAppContext) {
    for _ in 0..3 {
        cx.executor().advance_clock(Duration::from_millis(100));
        cx.run_until_parked();
        cx.update_window(window, |_, window, cx| window.render_frame(cx))
            .unwrap();
    }
}

fn press(window: AnyWindowHandle, key: &str, cx: &mut TestAppContext) {
    cx.update_window(window, |_, w, cx| w.press(key, cx))
        .unwrap();
    wait(window, cx);
}

fn manage() -> ElementId {
    ElementId::NamedChild(Arc::new(ElementId::from("nav")), "item-2".into())
}

fn title() -> ElementId {
    ElementId::from((manage(), "title"))
}

fn link(i: usize) -> ElementId {
    ElementId::NamedChild(Arc::new(manage()), format!("link-{i}").into())
}

fn shown(window: AnyWindowHandle, cx: &mut TestAppContext) -> bool {
    cx.update_window(window, |_, w, _| {
        w.try_find(ElementId::from((manage(), "list"))).is_some()
    })
    .unwrap()
}

fn focused(window: AnyWindowHandle, id: ElementId, cx: &mut TestAppContext) -> bool {
    cx.update_window(window, |_, w, _| w.find(id).focused())
        .unwrap()
        == Some(true)
}

/// Open Manage with the pointer, then ↓ to its first item.
fn open_manage(window: AnyWindowHandle, cx: &mut TestAppContext) {
    cx.update_window(window, |_, w, cx| w.click(title(), cx))
        .unwrap();
    wait(window, cx);
    assert!(shown(window, cx), "Manage did not open");
    press(window, "down", cx);
    assert!(focused(window, link(0), cx), "↓ did not focus Access");
}

#[gpui_kit::test]
fn tab_walks_the_items_then_leaves_the_menu_for_the_next_element(cx: &mut TestAppContext) {
    let (window, search) = open(cx);
    open_manage(window, cx);
    press(window, "tab", cx);
    assert!(focused(window, link(1), cx), "Tab did not move to Billing");
    assert!(shown(window, cx));
    press(window, "tab", cx);
    assert!(focused(window, link(2), cx), "Tab did not move to Support");
    press(window, "tab", cx);
    assert!(!shown(window, cx), "Tab past Support left Manage open");
    let on_search = cx
        .update_window(window, |_, w, _| search.is_focused(w))
        .unwrap();
    assert!(
        on_search,
        "Tab past Support did not reach the search action"
    );
}

#[gpui_kit::test]
fn shift_tab_on_the_first_item_returns_to_the_title(cx: &mut TestAppContext) {
    let (window, _) = open(cx);
    open_manage(window, cx);
    press(window, "tab", cx);
    press(window, "shift-tab", cx);
    assert!(
        focused(window, link(0), cx),
        "Shift+Tab did not go back to Access"
    );
    press(window, "shift-tab", cx);
    assert!(
        !shown(window, cx),
        "Shift+Tab to the title left Manage open"
    );
    assert!(
        focused(window, title(), cx),
        "Shift+Tab did not reach Manage"
    );
}

#[gpui_kit::test]
fn tab_on_an_open_title_enters_its_first_item(cx: &mut TestAppContext) {
    let (window, _) = open(cx);
    cx.update_window(window, |_, w, cx| w.click(title(), cx))
        .unwrap();
    wait(window, cx);
    assert!(focused(window, title(), cx));
    press(window, "tab", cx);
    assert!(shown(window, cx), "Tab on the open title closed Manage");
    assert!(focused(window, link(0), cx), "Tab did not enter Access");
}
