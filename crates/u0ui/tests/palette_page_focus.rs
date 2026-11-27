//! PalettePage: every page change (push, the Back arrow, Backspace) leaves
//! focus in the new page's search box, as the reference's `<input
//! key={stack.length} autoFocus>` does (design review FG2B #1). Before, a
//! push left focus where it was and the Back arrow left it on a button that
//! had just gone away.
use gpui_kit::test::{TestSupportExt as _, TestWindowExt as _};
use gpui_kit::{
    AppContext as _, Bounds, Context, ElementId, FocusHandle, InteractiveElement as _, IntoElement,
    ParentElement as _, Point, Render, SharedString, StatefulInteractiveElement as _, Styled as _,
    TestAppContext, Window, WindowBounds, WindowOptions, div, px, size,
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
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let seen = self.seen.clone();
        div().id("page").track_focus(&self.focus).size_full().child(
            PalettePage::new("pal").content(move |nav, _, _| {
                *seen.borrow_mut() = (nav.page.title.clone(), nav.query.clone());
                let nav = nav.clone();
                vec![
                    div()
                        .id("go")
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

#[gpui_kit::test]
fn every_page_change_focuses_the_search_box(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let seen: Seen = Rc::new(RefCell::new(Default::default()));
    let (window, _page) = cx.update(|cx| {
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
                cx.new(|_| Page { seen, focus })
            },
        )
        .expect("open test window")
    });
    let settle = |cx: &mut TestAppContext| {
        cx.executor().advance_clock(Duration::from_millis(500));
        cx.run_until_parked();
        cx.update_window(window, |_, window, cx| window.render_frame(cx))
            .unwrap();
    };
    let seen_now = |cx: &mut TestAppContext| {
        settle(cx);
        seen.borrow().clone()
    };
    settle(cx);

    // Push by clicking an item: typing reaches the new page's box.
    cx.update_window(window, |_, window, cx| window.click("go", cx))
        .unwrap();
    settle(cx);
    cx.update_window(window, |_, window, cx| window.input("co", cx))
        .unwrap();
    assert_eq!(seen_now(cx), ("Page 2".into(), "co".into()), "push");

    // Push again, then pop with the Back arrow: the arrow goes away and the
    // box on the page below has focus.
    cx.update_window(window, |_, window, cx| window.click("go", cx))
        .unwrap();
    settle(cx);
    let back: ElementId = (ElementId::from("pal"), "back").into();
    cx.update_window(window, |_, window, cx| window.click(back.clone(), cx))
        .unwrap();
    settle(cx);
    cx.update_window(window, |_, window, cx| window.input("x", cx))
        .unwrap();
    assert_eq!(seen_now(cx), ("Page 2".into(), "x".into()), "Back arrow");

    // Backspace empties the box, then pops; the top page's box takes typing.
    cx.update_window(window, |_, window, cx| window.press("backspace", cx))
        .unwrap();
    settle(cx);
    cx.update_window(window, |_, window, cx| window.press("backspace", cx))
        .unwrap();
    settle(cx);
    cx.update_window(window, |_, window, cx| window.input("n", cx))
        .unwrap();
    assert_eq!(seen_now(cx), ("Commands".into(), "n".into()), "Backspace");
}
