//! GPUI turns Enter/Space on a focused clickable element into a click on key
//! up. A component whose own key handler also acts on Enter/Space used to run
//! its action twice (design review FG3, Space audit): the Tree selected twice
//! and toggled its parent row open, the FluidDropdown opened and closed again.
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, Context, FocusHandle, InteractiveElement as _,
    IntoElement, ParentElement as _, Point, Render, SharedString, Styled as _, TestAppContext,
    Window, WindowBounds, WindowOptions, div, px, size,
};
use std::{cell::RefCell, rc::Rc, time::Duration};
use u0ui::patterns::FluidDropdown;
use u0ui::u0::primitives::{Tree, TreeItem};

struct Page {
    before: FocusHandle,
    selected: Rc<RefCell<Vec<SharedString>>>,
    picked: Rc<RefCell<Vec<usize>>>,
    dropdown: bool,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let selected = self.selected.clone();
        let picked = self.picked.clone();
        let body = if self.dropdown {
            FluidDropdown::new("fd", "Region")
                .items(["US South", "EU Central"])
                .on_change(move |i, _, _| picked.borrow_mut().push(i))
                .into_any_element()
        } else {
            Tree::new(
                "tree",
                [TreeItem::new("src", "src").children([TreeItem::new("main", "main.rs")])],
            )
            .on_select(move |id, _, _| selected.borrow_mut().push(id))
            .into_any_element()
        };
        div()
            .w(px(400.))
            .flex()
            .flex_col()
            .child(
                div()
                    .id("before")
                    .track_focus(&self.before.clone().tab_index(0).tab_stop(true))
                    .size(px(4.)),
            )
            .child(body)
    }
}

type Opened = (
    AnyWindowHandle,
    Rc<RefCell<Vec<SharedString>>>,
    Rc<RefCell<Vec<usize>>>,
);

fn open(cx: &mut TestAppContext, dropdown: bool) -> Opened {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let before = cx.update(|cx| cx.focus_handle());
    let selected = Rc::new(RefCell::new(Vec::new()));
    let picked = Rc::new(RefCell::new(Vec::new()));
    let page = Page {
        before: before.clone(),
        selected: selected.clone(),
        picked: picked.clone(),
        dropdown,
    };
    let window = cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(600.), px(500.)),
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
    cx.update_window(window, |_, window, cx| before.focus(window, cx))
        .unwrap();
    settle(window, cx);
    press("tab", window, cx);
    (window, selected, picked)
}

fn settle(window: AnyWindowHandle, cx: &mut TestAppContext) {
    cx.executor().advance_clock(Duration::from_millis(400));
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| window.render_frame(cx))
        .unwrap();
}

/// A full key press: down, then up (the up is where GPUI's keyboard click fires).
fn press(key: &str, window: AnyWindowHandle, cx: &mut TestAppContext) {
    cx.update_window(window, |_, window, cx| window.press(key, cx))
        .unwrap();
    settle(window, cx);
}

#[gpui_kit::test]
fn tree_enter_and_space_select_once_without_toggling(cx: &mut TestAppContext) {
    let (window, selected, _) = open(cx, false);
    press("enter", window, cx);
    assert_eq!(selected.borrow().as_slice(), &[SharedString::from("src")]);
    press("space", window, cx);
    assert_eq!(selected.borrow().len(), 2, "Space selects once");
    // Neither key toggled the parent open: ↓ stays on it (no child row).
    press("down", window, cx);
    press("enter", window, cx);
    assert_eq!(
        selected.borrow().last().map(|s| s.as_ref()),
        Some("src"),
        "the parent row stayed closed"
    );
}

#[gpui_kit::test]
fn fluid_dropdown_space_opens_it(cx: &mut TestAppContext) {
    let (window, _, picked) = open(cx, true);
    press("space", window, cx);
    // Open, Enter picks the active option; closed, Enter would only open it.
    press("enter", window, cx);
    assert_eq!(picked.borrow().len(), 1, "Space opened the list once");
}
