//! A list's remove buttons are keyed by their item, not their position
//! (design review FG3C). With index keys, removing item `i` handed its focus
//! handle to the next item's button: the ring landed on it, Tab skipped it,
//! and a second Enter removed that item too. The reference keys rows by
//! identity (`key={x}`, `key={it.id}`), so after a removal nothing shows
//! focus and the next Tab reaches the next item's button.
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, Context, ElementId, Entity, FocusHandle,
    InteractiveElement as _, IntoElement, ParentElement as _, Point, Render, SharedString,
    Styled as _, TestAppContext, Window, WindowBounds, WindowOptions, div, px, size,
};
use std::time::Duration;
use u0ui::t3::review::IssueState;
use u0ui::u0::panels::{LinkedItem, LinkedItemsSection};
use u0ui::u0::settings::DictionaryEditor;

#[derive(Clone, Copy)]
enum Kind {
    Dictionary,
    Linked,
}

struct Page {
    kind: Kind,
    before: FocusHandle,
    items: Entity<Vec<SharedString>>,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let items = self.items.read(cx).clone();
        let store = self.items.clone();
        let body = match self.kind {
            Kind::Dictionary => DictionaryEditor::new("dict")
                .words(items)
                .on_change(move |words, _, cx| {
                    let words = words.to_vec();
                    store.update(cx, |v, _| *v = words);
                })
                .into_any_element(),
            Kind::Linked => LinkedItemsSection::new(
                "linked",
                items
                    .iter()
                    .map(|i| LinkedItem::issue(i.clone(), IssueState::Open, "Title")),
            )
            .on_unlink(move |id, _, cx| {
                store.update(cx, |v, cx| {
                    v.retain(|x| *x != id);
                    cx.notify();
                })
            })
            .into_any_element(),
        };
        div()
            .w(px(600.))
            .flex()
            .flex_col()
            .child(
                div()
                    .id("before")
                    .track_focus(&self.before.clone().tab_index(0).tab_stop(true))
                    .size(px(10.)),
            )
            .child(body)
    }
}

fn open(
    kind: Kind,
    cx: &mut TestAppContext,
) -> (AnyWindowHandle, FocusHandle, Entity<Vec<SharedString>>) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let words = ["alpha", "beta", "gamma"].map(SharedString::from).to_vec();
    let (before, items) = cx.update(|cx| (cx.focus_handle(), cx.new(|_| words)));
    let page = Page {
        kind,
        before: before.clone(),
        items: items.clone(),
    };
    let window = cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(800.), px(500.)),
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
    (window, before, items)
}

fn settle(window: AnyWindowHandle, cx: &mut TestAppContext) {
    cx.executor().advance_clock(Duration::from_millis(300));
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| window.render_frame(cx))
        .unwrap();
}

fn press(window: AnyWindowHandle, key: &str, cx: &mut TestAppContext) {
    cx.update_window(window, |_, window, cx| window.press(key, cx))
        .unwrap();
    settle(window, cx);
}

fn button(scope: &str, item: &str) -> ElementId {
    let part = if scope == "dict" { "rm" } else { "unlink" };
    ElementId::from((
        ElementId::from(SharedString::from(scope.to_string())),
        SharedString::from(format!("{part}-{item}")),
    ))
}

fn focused(window: AnyWindowHandle, id: ElementId, cx: &mut TestAppContext) -> bool {
    cx.update_window(window, |_, window, _| {
        window.find(id).focused().unwrap_or(false)
    })
    .unwrap()
}

fn remove_first_then_next(kind: Kind, scope: &str, cx: &mut TestAppContext) {
    let (window, before, items) = open(kind, cx);
    cx.update_window(window, |_, window, cx| before.focus(window, cx))
        .unwrap();
    settle(window, cx);
    // Tab forward until alpha's button holds focus.
    for _ in 0..6 {
        if focused(window, button(scope, "alpha"), cx) {
            break;
        }
        press(window, "tab", cx);
    }
    assert!(focused(window, button(scope, "alpha"), cx));

    press(window, "enter", cx);
    let left = |cx: &mut TestAppContext| items.read_with(cx, |v, _| v.clone());
    assert_eq!(left(cx), ["beta", "gamma"].map(SharedString::from).to_vec());
    assert!(
        !focused(window, button(scope, "beta"), cx),
        "the next item's button inherited focus"
    );

    // A second Enter removes nothing more.
    press(window, "enter", cx);
    assert_eq!(left(cx).len(), 2, "a second Enter removed the next item");

    // Tab goes to the next item's button.
    press(window, "tab", cx);
    assert!(
        focused(window, button(scope, "beta"), cx),
        "Tab skipped the next item's button"
    );
}

#[gpui_kit::test]
fn dictionary_word_removed(cx: &mut TestAppContext) {
    remove_first_then_next(Kind::Dictionary, "dict", cx);
}

#[gpui_kit::test]
fn linked_item_unlinked(cx: &mut TestAppContext) {
    remove_first_then_next(Kind::Linked, "linked", cx);
}
