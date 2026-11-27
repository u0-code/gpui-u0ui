//! PalettePage focus and keys, as the reference's `<input key={stack.length}
//! autoFocus onKeyDown=…>` behaves:
//! - every page change (push, the Back arrow, Backspace) leaves focus in the
//!   new page's search box (design review FG2B #1);
//! - the search box has focus as soon as the palette appears (FG2C #1).
use gpui_kit::test::{TestSupportExt as _, TestWindowExt as _};
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, Context, ElementId, Entity, FocusHandle,
    InteractiveElement as _, IntoElement, ParentElement as _, Point, Render, SharedString,
    StatefulInteractiveElement as _, Styled as _, TestAppContext, Window, WindowBounds,
    WindowOptions, div, px, size,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use u0ui::u0::palette::{PaletteLevel, PalettePage};

/// What the content callback last saw: (page title, query).
type Seen = Rc<RefCell<(SharedString, SharedString)>>;

struct Page {
    seen: Seen,
    focus: FocusHandle,
    /// A focusable result row inside the palette (not its input).
    item_focus: FocusHandle,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let seen = self.seen.clone();
        let item_focus = self.item_focus.clone();
        div().id("page").track_focus(&self.focus).size_full().child(
            PalettePage::new("pal").content(move |nav, _, _| {
                *seen.borrow_mut() = (nav.page.title.clone(), nav.query.clone());
                let nav = nav.clone();
                vec![
                    div()
                        .id("go")
                        .track_focus(&item_focus)
                        .test_support()
                        .h(px(32.))
                        .w(px(200.))
                        .on_click(move |_, window, cx| {
                            nav.push(
                                PaletteLevel::new(format!("Page {}", nav.depth + 1)),
                                window,
                                cx,
                            )
                        })
                        .child("Switch agent")
                        .into_any_element(),
                ]
            }),
        )
    }
}

/// A window whose page first focuses itself, as the gallery's page does.
fn open(cx: &mut TestAppContext, seen: &Seen) -> (AnyWindowHandle, Entity<Page>) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    cx.update(|cx| {
        let seen = seen.clone();
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(800.), px(600.)),
                })),
                ..Default::default()
            },
            cx,
            move |window, cx| {
                let focus = cx.focus_handle();
                focus.focus(window, cx);
                cx.new(|cx| Page {
                    seen,
                    focus,
                    item_focus: cx.focus_handle(),
                })
            },
        )
        .expect("open test window")
    })
}

fn settle(cx: &mut TestAppContext, window: AnyWindowHandle) {
    cx.executor().advance_clock(Duration::from_millis(500));
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| window.render_frame(cx))
        .unwrap();
}

fn press(cx: &mut TestAppContext, window: AnyWindowHandle, key: &str) {
    cx.update_window(window, |_, window, cx| window.press(key, cx))
        .unwrap();
    settle(cx, window);
}

fn type_text(cx: &mut TestAppContext, window: AnyWindowHandle, text: &str) {
    cx.update_window(window, |_, window, cx| window.input(text, cx))
        .unwrap();
    settle(cx, window);
}

fn click(cx: &mut TestAppContext, window: AnyWindowHandle, id: impl Into<ElementId>) {
    let id = id.into();
    cx.update_window(window, |_, window, cx| window.click(id, cx))
        .unwrap();
    settle(cx, window);
}

#[gpui_kit::test]
fn every_page_change_focuses_the_search_box(cx: &mut TestAppContext) {
    let seen: Seen = Rc::new(RefCell::new(Default::default()));
    let (window, _page) = open(cx, &seen);
    settle(cx, window);

    // Push by clicking an item: typing reaches the new page's box.
    click(cx, window, "go");
    type_text(cx, window, "co");
    assert_eq!(
        seen.borrow().clone(),
        ("Page 2".into(), "co".into()),
        "push"
    );

    // Push again, then pop with the Back arrow: the arrow goes away and the
    // box on the page below has focus.
    click(cx, window, "go");
    let back: ElementId = (ElementId::from("pal"), "back").into();
    click(cx, window, back);
    type_text(cx, window, "x");
    assert_eq!(
        seen.borrow().clone(),
        ("Page 2".into(), "x".into()),
        "Back arrow"
    );

    // Backspace empties the box, then pops; the top page's box takes typing.
    press(cx, window, "backspace");
    press(cx, window, "backspace");
    type_text(cx, window, "n");
    assert_eq!(
        seen.borrow().clone(),
        ("Commands".into(), "n".into()),
        "Backspace"
    );
}

/// `<input autoFocus>`: typing right after the palette appears reaches the
/// search box, with nothing clicked first.
#[gpui_kit::test]
fn the_search_box_has_focus_on_mount(cx: &mut TestAppContext) {
    let seen: Seen = Rc::new(RefCell::new(Default::default()));
    let (window, _page) = open(cx, &seen);
    settle(cx, window);
    type_text(cx, window, "co");
    assert_eq!(seen.borrow().clone(), ("Commands".into(), "co".into()));
}
