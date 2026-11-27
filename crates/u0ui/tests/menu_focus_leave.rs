//! Leaving an open menu (design review FB2, items 1 and 2).
//!
//! - Carbon `Menu` closes when focus leaves it: Tab and Shift+Tab close it
//!   and return focus to the element that had it at open (Carbon prevents the
//!   Tab, focuses that element, then calls `onClose`); focus moving anywhere
//!   else closes it too. DropdownMenu, ContextMenu and Menubar share it.
//! - Menubar: after ←/→ moved to another menu, picking an item or Escape
//!   returns focus to the trigger of the menu that was open.
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, Context, ElementId, Entity, FocusHandle,
    InteractiveElement as _, IntoElement, ParentElement as _, Point, Render, Styled as _,
    TestAppContext, Window, WindowBounds, WindowOptions, div, px, size,
};
use std::sync::Arc;
use std::time::Duration;
use u0ui::components::{ContextMenu, DropdownMenu, MenuItem, Menubar};

struct Page {
    ran: Vec<&'static str>,
    elsewhere: FocusHandle,
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
                    .item(item(&entity, "Restart"))
                    .item(
                        MenuItem::item("Move to")
                            .submenu([item(&entity, "Production"), item(&entity, "Staging")]),
                    )
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
                    .menu(
                        "View",
                        vec![item(&entity, "Zoom in"), item(&entity, "Zoom out")],
                    )
                    .menu("Help", vec![item(&entity, "Documentation")]),
            )
            .child(
                div()
                    .w(px(40.))
                    .h(px(20.))
                    .track_focus(&self.elsewhere.clone().tab_stop(true)),
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
            |_, cx| {
                cx.new(|cx| Page {
                    ran: Vec::new(),
                    elsewhere: cx.focus_handle(),
                })
            },
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

fn last(page: &Entity<Page>, cx: &mut TestAppContext) -> Option<&'static str> {
    cx.update(|cx| page.read(cx).ran.last().copied())
}

fn shown(window: AnyWindowHandle, id: &ElementId, cx: &mut TestAppContext) -> bool {
    cx.update_window(window, |_, w, _| w.try_find(id.clone()).is_some())
        .unwrap()
}

fn focused(window: AnyWindowHandle, id: &ElementId, cx: &mut TestAppContext) -> Option<bool> {
    cx.update_window(window, |_, w, _| w.find(id.clone()).focused())
        .unwrap()
}

fn menu_of(surface: ElementId) -> ElementId {
    ElementId::from((surface, "menu"))
}

fn bar_menu(index: usize) -> ElementId {
    ElementId::NamedChild(
        Arc::new(ElementId::from("bar")),
        format!("menu-{index}").into(),
    )
}

fn bar_trigger(index: usize) -> ElementId {
    ElementId::from((bar_menu(index), "trigger"))
}

#[gpui_kit::test]
fn tab_and_shift_tab_close_a_dropdown_menu_and_return_to_its_trigger(cx: &mut TestAppContext) {
    let (window, page) = open(cx);
    wait(window, cx);
    let trigger = ElementId::from((ElementId::from("dd"), "trigger"));
    let menu = menu_of(ElementId::from("dd"));
    for key in ["tab", "shift-tab"] {
        act(window, cx, |w, cx| w.click(trigger.clone(), cx));
        assert!(shown(window, &menu, cx), "Actions did not open");
        act(window, cx, |w, cx| w.press(key, cx));
        assert!(!shown(window, &menu, cx), "{key} left the menu open");
        // Focus is back on the trigger: Enter opens the menu again and the
        // next Enter runs its first item.
        act(window, cx, |w, cx| w.press("enter", cx));
        assert!(
            shown(window, &menu, cx),
            "{key} did not return to the trigger"
        );
        act(window, cx, |w, cx| w.press("enter", cx));
        assert_eq!(last(&page, cx), Some("Restart"));
    }
}

#[gpui_kit::test]
fn a_submenu_keeps_the_menu_open_and_tab_closes_both(cx: &mut TestAppContext) {
    let (window, page) = open(cx);
    wait(window, cx);
    let trigger = ElementId::from((ElementId::from("dd"), "trigger"));
    let menu = menu_of(ElementId::from("dd"));
    act(window, cx, |w, cx| w.click(trigger.clone(), cx));
    act(window, cx, |w, cx| w.press("down", cx));
    act(window, cx, |w, cx| w.press("right", cx));
    assert!(
        shown(window, &menu, cx),
        "entering the submenu closed the menu"
    );
    act(window, cx, |w, cx| w.press("down", cx));
    act(window, cx, |w, cx| w.press("enter", cx));
    assert_eq!(last(&page, cx), Some("Staging"));
    act(window, cx, |w, cx| w.click(trigger.clone(), cx));
    act(window, cx, |w, cx| w.press("down", cx));
    act(window, cx, |w, cx| w.press("right", cx));
    act(window, cx, |w, cx| w.press("tab", cx));
    assert!(
        !shown(window, &menu, cx),
        "Tab in a submenu left the menu open"
    );
}

#[gpui_kit::test]
fn focus_moving_elsewhere_closes_the_menu(cx: &mut TestAppContext) {
    let (window, page) = open(cx);
    wait(window, cx);
    let trigger = ElementId::from((ElementId::from("dd"), "trigger"));
    let menu = menu_of(ElementId::from("dd"));
    act(window, cx, |w, cx| w.click(trigger, cx));
    assert!(shown(window, &menu, cx));
    let elsewhere = cx.update(|cx| page.read(cx).elsewhere.clone());
    act(window, cx, |w, cx| elsewhere.focus(w, cx));
    assert!(
        !shown(window, &menu, cx),
        "focus left but the menu stayed open"
    );
    // Focus stays where it went.
    let still = cx
        .update_window(window, |_, w, _| elsewhere.is_focused(w))
        .unwrap();
    assert!(still, "closing pulled focus back to the trigger");
}

#[gpui_kit::test]
fn tab_closes_a_keyboard_opened_context_menu(cx: &mut TestAppContext) {
    let (window, page) = open(cx);
    wait(window, cx);
    let area = ElementId::from((ElementId::from("ctx"), "area"));
    let menu = menu_of(ElementId::from("ctx"));
    act(window, cx, |w, cx| w.click(area.clone(), cx));
    act(window, cx, |w, cx| w.press("shift-f10", cx));
    assert!(shown(window, &menu, cx), "Shift+F10 did not open the menu");
    act(window, cx, |w, cx| w.press("tab", cx));
    assert!(!shown(window, &menu, cx), "Tab left the context menu open");
    // Focus is back on the area: Shift+F10 opens it again, Enter runs Copy.
    act(window, cx, |w, cx| w.press("shift-f10", cx));
    assert!(
        shown(window, &menu, cx),
        "Tab did not return focus to the area"
    );
    act(window, cx, |w, cx| w.press("enter", cx));
    assert_eq!(last(&page, cx), Some("Copy"));
}

#[gpui_kit::test]
fn menubar_returns_focus_to_the_trigger_of_the_menu_that_was_open(cx: &mut TestAppContext) {
    let (window, page) = open(cx);
    wait(window, cx);
    // File → View, then Escape: focus on View.
    act(window, cx, |w, cx| w.click(bar_trigger(0), cx));
    act(window, cx, |w, cx| w.press("right", cx));
    assert!(
        shown(window, &menu_of(bar_menu(1)), cx),
        "→ did not open View"
    );
    assert!(!shown(window, &menu_of(bar_menu(0)), cx));
    act(window, cx, |w, cx| w.press("escape", cx));
    assert!(!shown(window, &menu_of(bar_menu(1)), cx));
    assert_eq!(
        focused(window, &bar_trigger(1), cx),
        Some(true),
        "Escape lost focus"
    );
    // The trigger works from the keyboard: ↓ opens View again.
    act(window, cx, |w, cx| w.press("down", cx));
    assert!(shown(window, &menu_of(bar_menu(1)), cx));
    act(window, cx, |w, cx| w.press("escape", cx));

    // File → View, then pick: focus on View.
    act(window, cx, |w, cx| w.click(bar_trigger(0), cx));
    act(window, cx, |w, cx| w.press("right", cx));
    act(window, cx, |w, cx| w.press("enter", cx));
    assert_eq!(last(&page, cx), Some("Zoom in"));
    assert_eq!(
        focused(window, &bar_trigger(1), cx),
        Some(true),
        "picking lost focus"
    );

    // View → File (the menu before), then Escape: focus on File.
    act(window, cx, |w, cx| w.press("enter", cx));
    act(window, cx, |w, cx| w.press("left", cx));
    assert!(
        shown(window, &menu_of(bar_menu(0)), cx),
        "← did not open File"
    );
    act(window, cx, |w, cx| w.press("down", cx));
    act(window, cx, |w, cx| w.press("enter", cx));
    assert_eq!(last(&page, cx), Some("Open…"));
    assert_eq!(focused(window, &bar_trigger(0), cx), Some(true));

    // Tab in a menubar menu closes it and returns to its trigger.
    act(window, cx, |w, cx| w.press("enter", cx));
    assert!(shown(window, &menu_of(bar_menu(0)), cx));
    act(window, cx, |w, cx| w.press("tab", cx));
    assert!(
        !shown(window, &menu_of(bar_menu(0)), cx),
        "Tab left File open"
    );
    assert_eq!(focused(window, &bar_trigger(0), cx), Some(true));
}

struct DefaultOpen {
    ran: Vec<&'static str>,
}

impl Render for DefaultOpen {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let pick = |label: &'static str| {
            let entity = entity.clone();
            MenuItem::item(label).on_select(move |_, cx| {
                entity.update(cx, |this: &mut DefaultOpen, cx| {
                    this.ran.push(label);
                    cx.notify();
                })
            })
        };
        div().p(px(20.)).child(
            DropdownMenu::new("dd")
                .default_open(true)
                .item(pick("Show archived"))
                .item(pick("Name")),
        )
    }
}

/// A menu open on arrival focuses its first item too (FB2 item 3).
#[gpui_kit::test]
fn a_default_open_menu_focuses_its_first_item(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let (window, page) = cx.update(|cx| {
        gpui_kit::open_window(WindowOptions::default(), cx, |_, cx| {
            cx.new(|_| DefaultOpen { ran: Vec::new() })
        })
        .expect("open test window")
    });
    wait(window, cx);
    assert!(shown(window, &menu_of(ElementId::from("dd")), cx));
    act(window, cx, |w, cx| w.press("enter", cx));
    assert_eq!(
        cx.update(|cx| page.read(cx).ran.clone()),
        vec!["Show archived"]
    );
}
