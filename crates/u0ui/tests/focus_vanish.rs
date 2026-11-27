//! Chromium keeps the sequential focus navigation starting point where a
//! focused element was removed, so the next Tab continues after it and
//! Shift+Tab before it (design review FG3B): a TerminalTabs tab closed with
//! Delete, a TagInput chip removed with its focused × (an anchor in the
//! list), and a dismissible tag or a dialog button whose caller removes it
//! (`parts::vacate_focus`). Without it Tab restarted at the window's first
//! control.
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, Context, Entity, FocusHandle,
    InteractiveElement as _, IntoElement, ParentElement as _, Point, Render, SharedString,
    Styled as _, TestAppContext, Window, WindowBounds, WindowOptions, div, px, size,
};
use std::time::Duration;
use u0ui::components::{Badge, Button};
use u0ui::u0::panels::{TerminalTab, TerminalTabs};
use u0ui::u0::primitives::TagInput;

#[derive(Clone, Copy)]
enum Kind {
    Tabs,
    Tags,
    Dismiss,
    Button,
}

struct Page {
    kind: Kind,
    stops: [FocusHandle; 2],
    items: Entity<Vec<SharedString>>,
}

fn stop(handle: &FocusHandle, id: &'static str) -> impl IntoElement {
    div()
        .id(id)
        .track_focus(&handle.clone().tab_index(0).tab_stop(true))
        .size(px(10.))
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let items = self.items.read(cx).clone();
        let store = self.items.clone();
        let remove = move |i: usize, cx: &mut gpui_kit::App| {
            store.update(cx, |v, cx| {
                v.remove(i);
                cx.notify();
            })
        };
        let body = match self.kind {
            Kind::Tabs => {
                TerminalTabs::new("tt", items.iter().map(|t| TerminalTab::new(t.clone())))
                    .on_close(move |i, _, cx| remove(i, cx))
                    .into_any_element()
            }
            Kind::Tags => {
                let store = self.items.clone();
                TagInput::new("ti")
                    .value(items.clone())
                    .on_value_change(move |v, _, cx| {
                        store.update(cx, |s, cx| {
                            *s = v;
                            cx.notify();
                        })
                    })
                    .into_any_element()
            }
            Kind::Dismiss => div()
                .flex()
                .children(items.iter().enumerate().map(|(i, t)| {
                    let remove = remove.clone();
                    Badge::new(t.clone())
                        .id(SharedString::from(format!("tag-{t}")))
                        .dismiss_label("Remove")
                        .on_dismiss(move |_, _, cx| remove(i, cx))
                }))
                .into_any_element(),
            Kind::Button => div()
                .flex()
                .children(items.iter().enumerate().map(|(i, t)| {
                    let remove = remove.clone();
                    Button::new(SharedString::from(format!("b-{t}")), t.clone())
                        .on_click(move |_, _, cx| remove(i, cx))
                }))
                .into_any_element(),
        };
        div()
            .w(px(700.))
            .flex()
            .flex_col()
            .gap(px(10.))
            .child(stop(&self.stops[0], "before"))
            .child(body)
            .child(stop(&self.stops[1], "after"))
    }
}

struct Harness {
    window: AnyWindowHandle,
    stops: [FocusHandle; 2],
    items: Entity<Vec<SharedString>>,
}

fn open(kind: Kind, items: &[&str], cx: &mut TestAppContext) -> Harness {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let items: Vec<SharedString> = items.iter().map(|s| SharedString::from(*s)).collect();
    let (stops, items) =
        cx.update(|cx| ([cx.focus_handle(), cx.focus_handle()], cx.new(|_| items)));
    let page = Page {
        kind,
        stops: stops.clone(),
        items: items.clone(),
    };
    let window = cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(800.), px(400.)),
                })),
                ..Default::default()
            },
            cx,
            move |_, cx| cx.new(|_| page),
        )
        .expect("open test window")
        .0
    });
    settle(window, cx);
    Harness {
        window,
        stops,
        items,
    }
}

fn settle(window: AnyWindowHandle, cx: &mut TestAppContext) {
    cx.executor().advance_clock(Duration::from_millis(300));
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| window.render_frame(cx))
        .unwrap();
}

fn press(key: &str, h: &Harness, cx: &mut TestAppContext) {
    cx.update_window(h.window, |_, window, cx| window.press(key, cx))
        .unwrap();
    settle(h.window, cx);
}

fn focused(h: &Harness, cx: &mut TestAppContext) -> Option<FocusHandle> {
    cx.update_window(h.window, |_, window, cx| window.focused(cx))
        .unwrap()
}

fn start(h: &Harness, cx: &mut TestAppContext) {
    let first = h.stops[0].clone();
    cx.update_window(h.window, |_, window, cx| first.focus(window, cx))
        .unwrap();
    settle(h.window, cx);
}

fn count(h: &Harness, cx: &mut TestAppContext) -> usize {
    h.items.read_with(cx, |v, _| v.len())
}

/// After the removal, Tab neither restarts at the first stop nor skips
/// past the list, and Shift+Tab returns to the stop before.
fn assert_kept(h: &Harness, cx: &mut TestAppContext, next_is_after: bool) {
    let shown = focused(h, cx);
    assert!(
        shown.as_ref().is_none_or(|f| !h.stops.contains(f)),
        "focus jumped to a stop"
    );
    press("tab", h, cx);
    let next = focused(h, cx);
    assert_ne!(next, Some(h.stops[0].clone()), "Tab restarted at the top");
    if next_is_after {
        assert_eq!(next, Some(h.stops[1].clone()));
    } else {
        assert_ne!(
            next,
            Some(h.stops[1].clone()),
            "Tab skipped the rest of the list"
        );
    }
}

fn assert_back(h: &Harness, cx: &mut TestAppContext) {
    press("shift-tab", h, cx);
    assert_eq!(focused(h, cx), Some(h.stops[0].clone()));
}

#[gpui_kit::test]
fn terminal_tab_closed_with_delete(cx: &mut TestAppContext) {
    let h = open(Kind::Tabs, &["dev", "test", "build"], cx);
    start(&h, cx);
    press("tab", &h, cx); // the active tab, "dev"
    press("delete", &h, cx);
    assert_eq!(count(&h, cx), 2);
    // "test" is now active and the only stop in the list, after "dev".
    assert_kept(&h, cx, false);
}

#[gpui_kit::test]
fn terminal_tab_closed_with_delete_then_shift_tab(cx: &mut TestAppContext) {
    let h = open(Kind::Tabs, &["dev", "test", "build"], cx);
    start(&h, cx);
    press("tab", &h, cx);
    press("delete", &h, cx);
    assert_back(&h, cx);
}

#[gpui_kit::test]
fn tag_chip_removed_with_its_focused_remove_button(cx: &mut TestAppContext) {
    let h = open(Kind::Tags, &["alpha", "beta"], cx);
    start(&h, cx);
    press("tab", &h, cx); // alpha's ×
    press("space", &h, cx);
    assert_eq!(count(&h, cx), 1);
    // Next: beta's ×, not the window's first stop.
    assert_kept(&h, cx, false);
    press("shift-tab", &h, cx);
    assert_eq!(focused(&h, cx), Some(h.stops[0].clone()));
}

#[gpui_kit::test]
fn last_dismissible_tag_removed(cx: &mut TestAppContext) {
    let h = open(Kind::Dismiss, &["alpha"], cx);
    start(&h, cx);
    press("tab", &h, cx);
    press("space", &h, cx);
    assert_eq!(count(&h, cx), 0);
    assert_kept(&h, cx, true);
}

#[gpui_kit::test]
fn dismissible_tag_removed_then_shift_tab(cx: &mut TestAppContext) {
    let h = open(Kind::Dismiss, &["alpha", "beta"], cx);
    start(&h, cx);
    press("tab", &h, cx);
    press("space", &h, cx);
    assert_eq!(count(&h, cx), 1);
    assert_back(&h, cx);
}

#[gpui_kit::test]
fn button_that_removes_itself(cx: &mut TestAppContext) {
    let h = open(Kind::Button, &["Cancel", "Rename"], cx);
    start(&h, cx);
    press("tab", &h, cx);
    press("enter", &h, cx);
    assert_eq!(count(&h, cx), 1);
    // "Rename" followed "Cancel".
    assert_kept(&h, cx, false);
}

#[gpui_kit::test]
fn button_that_stays_keeps_normal_tab(cx: &mut TestAppContext) {
    // A press that removes nothing: Tab moves on from the button as usual.
    let h = open(Kind::Button, &["One"], cx);
    start(&h, cx);
    press("tab", &h, cx);
    let button = focused(&h, cx);
    assert!(button.as_ref().is_some_and(|b| !h.stops.contains(b)));
    cx.update_window(h.window, |_, window, cx| {
        let f = window.focused(cx).unwrap();
        u0ui::u0::primitives::parts::vacate_focus(&f, window, cx);
    })
    .unwrap();
    settle(h.window, cx);
    assert_eq!(focused(&h, cx), button);
    press("tab", &h, cx);
    assert_eq!(focused(&h, cx), Some(h.stops[1].clone()));
}
