//! FilterMenu closes when focus leaves the panel and its trigger, like the
//! reference's `onBlur` on `.g-fmenu` (design review FG2B #2): Tab from
//! "Clear" closes it and moves on to the next control on the page (the
//! panel sits right after its trigger in the reference's DOM), and focus
//! moved anywhere else closes it too. Shift+Tab from the search field goes
//! back to the trigger and keeps it open; Tab from there re-enters it.
//! Before, Base's window-level Tab moved focus and left the panel open, and
//! Escape no longer reached it.
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, Context, ElementId, FocusHandle,
    InteractiveElement as _, IntoElement, ParentElement as _, Point, Render, Styled as _,
    TestAppContext, Window, WindowBounds, WindowOptions, div, px, size,
};
use std::time::Duration;
use u0ui::u0::shell::{FilterItem, FilterMenu};

struct Page {
    before: FocusHandle,
    after: FocusHandle,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(
                div()
                    .w(px(40.))
                    .h(px(20.))
                    .track_focus(&self.before.clone().tab_stop(true)),
            )
            .child(FilterMenu::new(
                "fm",
                "Projects",
                [
                    FilterItem::new("carbon-ui", "carbon-ui").count(6),
                    FilterItem::new("marketing-site", "marketing-site").count(2),
                    FilterItem::new("api-gateway", "api-gateway").count(9),
                ],
            ))
            .child(
                div()
                    .w(px(40.))
                    .h(px(20.))
                    .track_focus(&self.after.clone().tab_stop(true)),
            )
    }
}

struct Harness {
    window: AnyWindowHandle,
    page: gpui_kit::Entity<Page>,
}

impl Harness {
    fn new(cx: &mut TestAppContext) -> Self {
        cx.update(|cx| {
            gpui_kit::init(cx);
            u0ui::init(cx);
        });
        let (window, page) = cx.update(|cx| {
            gpui_kit::open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds {
                        origin: Point::default(),
                        size: size(px(800.), px(600.)),
                    })),
                    ..Default::default()
                },
                cx,
                |window, cx| {
                    let before = cx.focus_handle();
                    before.focus(window, cx);
                    cx.new(|cx| Page {
                        before,
                        after: cx.focus_handle(),
                    })
                },
            )
            .expect("open test window")
        });
        let h = Self { window, page };
        h.settle(cx);
        h
    }

    fn settle(&self, cx: &mut TestAppContext) {
        cx.executor().advance_clock(Duration::from_millis(500));
        cx.run_until_parked();
        cx.update_window(self.window, |_, window, cx| window.render_frame(cx))
            .unwrap();
    }

    fn press(&self, key: &str, cx: &mut TestAppContext) {
        cx.update_window(self.window, |_, window, cx| window.press(key, cx))
            .unwrap();
        self.settle(cx);
    }

    fn open(&self, cx: &mut TestAppContext) -> bool {
        let surface: ElementId = (ElementId::from("fm"), "surface").into();
        cx.update_window(self.window, |_, window, _| {
            window.try_find(surface.clone()).is_some()
        })
        .unwrap()
    }

    fn focused(&self, cx: &mut TestAppContext, which: fn(&Page) -> &FocusHandle) -> bool {
        let page = self.page.clone();
        cx.update_window(self.window, |_, window, cx| {
            which(page.read(cx)).is_focused(window)
        })
        .unwrap()
    }

    /// From the page's first stop: Tab to the trigger and open the panel.
    fn open_from_keyboard(&self, cx: &mut TestAppContext) {
        self.press("tab", cx);
        self.press("enter", cx);
        assert!(self.open(cx), "Enter on the trigger opens the panel");
    }
}

#[gpui_kit::test]
fn tab_from_clear_closes_and_moves_on(cx: &mut TestAppContext) {
    let h = Harness::new(cx);
    h.open_from_keyboard(cx);
    // Search field, three options, "Select all", "Clear".
    for _ in 0..5 {
        h.press("tab", cx);
        assert!(h.open(cx), "Tab inside the panel keeps it open");
    }
    h.press("tab", cx);
    assert!(!h.open(cx), "Tab from Clear closes the panel");
    assert!(
        h.focused(cx, |p| &p.after),
        "Tab from Clear moves on to the control after the menu"
    );
}

#[gpui_kit::test]
fn shift_tab_from_search_keeps_it_open_on_the_trigger(cx: &mut TestAppContext) {
    let h = Harness::new(cx);
    h.open_from_keyboard(cx);
    h.press("shift-tab", cx);
    assert!(h.open(cx), "Shift+Tab to the trigger keeps the panel open");
    // Tab from the open trigger re-enters the panel, then Escape closes it
    // and returns to the trigger.
    h.press("tab", cx);
    h.press("escape", cx);
    assert!(!h.open(cx), "Escape closes the panel");
    // Shift+Tab from the (closed) trigger leaves for the page.
    h.press("shift-tab", cx);
    assert!(h.focused(cx, |p| &p.before));
    // Opened again, Shift+Tab twice leaves trigger and panel: it closes.
    h.press("tab", cx);
    h.press("enter", cx);
    h.press("shift-tab", cx);
    h.press("shift-tab", cx);
    assert!(!h.open(cx), "focus left the menu: the panel closes");
    assert!(h.focused(cx, |p| &p.before));
}

#[gpui_kit::test]
fn focus_moved_elsewhere_closes_it(cx: &mut TestAppContext) {
    let h = Harness::new(cx);
    h.open_from_keyboard(cx);
    let page = h.page.clone();
    cx.update_window(h.window, |_, window, cx| {
        page.read(cx).after.clone().focus(window, cx)
    })
    .unwrap();
    h.settle(cx);
    assert!(!h.open(cx), "focus moved away: the panel closes");
    assert!(
        h.focused(cx, |p| &p.after),
        "closing does not pull focus back to the trigger"
    );
}
