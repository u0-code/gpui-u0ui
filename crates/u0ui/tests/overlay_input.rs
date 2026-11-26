//! Regression: after a modal surface closes, the window takes clicks and keys
//! again and the surface can be reopened (design review B1, blockers 1 and 2).
//!
//! A closing presence used to settle a hair above 0 (see `motion::animate`),
//! so the Base dialog host stayed mounted with an invisible scrim that
//! swallowed every press, the trigger included.
use gpui_kit::test::{TestSupportExt as _, TestWindowExt as _};
use gpui_kit::{
    AppContext as _, Bounds, Context, FocusHandle, InteractiveElement as _, IntoElement,
    MouseButton, ParentElement as _, Point, Render, Styled as _, TestAppContext, Window,
    WindowBounds, WindowOptions, div, px, size,
};
use std::time::Duration;
use u0ui::components::{Dialog, Drawer, Sheet};

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Dialog,
    Sheet,
    Drawer,
}

struct Page {
    kind: Kind,
    behind_presses: usize,
    keys: usize,
    focus: FocusHandle,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let trigger = div()
            .id("open")
            .test_support()
            .w(px(120.))
            .h(px(40.))
            .child("Open");
        let surface = match self.kind {
            Kind::Dialog => Dialog::new("dialog")
                .trigger(trigger)
                .title("Title")
                .primary_text("Save")
                .secondary_text("Cancel")
                .into_any_element(),
            Kind::Sheet => Sheet::new("sheet")
                .trigger(trigger)
                .title("Title")
                .into_any_element(),
            Kind::Drawer => Drawer::new("drawer")
                .trigger(trigger)
                .title("Title")
                .into_any_element(),
        };
        div()
            .id("page")
            .track_focus(&self.focus)
            .size_full()
            .flex()
            .flex_col()
            .gap(px(20.))
            .on_key_down(cx.listener(|this, _, _, cx| {
                this.keys += 1;
                cx.notify();
            }))
            .child(surface)
            .child(
                div()
                    .id("behind")
                    .test_support()
                    .w(px(120.))
                    .h(px(40.))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.behind_presses += 1;
                            cx.notify();
                        }),
                    )
                    .child("Behind"),
            )
    }
}

fn open_close_many_times(kind: Kind, cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let (window, page) = cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(1024.), px(720.)),
                })),
                ..Default::default()
            },
            cx,
            move |window, cx| {
                let focus = cx.focus_handle();
                focus.focus(window, cx);
                cx.new(|_| Page {
                    kind,
                    behind_presses: 0,
                    keys: 0,
                    focus,
                })
            },
        )
        .expect("open test window")
    });
    let settle = |cx: &mut TestAppContext| {
        cx.executor().advance_clock(Duration::from_secs(1));
        cx.run_until_parked();
        cx.update_window(window, |_, window, cx| window.render_frame(cx))
            .unwrap();
    };
    let presses = |cx: &mut TestAppContext| cx.update(|cx| page.read(cx).behind_presses);
    settle(cx);
    for round in 0..6 {
        cx.update_window(window, |_, window, cx| window.click("open", cx))
            .unwrap();
        settle(cx);
        let before = presses(cx);
        if round % 2 == 0 {
            // A press on the scrim is eaten and closes the surface.
            cx.update_window(window, |_, window, cx| window.click("behind", cx))
                .unwrap();
            assert_eq!(presses(cx), before, "round {round}: the scrim leaked");
        } else {
            cx.update_window(window, |_, window, cx| window.press("escape", cx))
                .unwrap();
        }
        settle(cx);
        // Closed: the page takes presses again …
        cx.update_window(window, |_, window, cx| window.click("behind", cx))
            .unwrap();
        settle(cx);
        assert_eq!(
            presses(cx),
            before + 1,
            "round {round}: the page ignores presses after the surface closed"
        );
    }
    // … and keys reach the page once focus is back on it.
    cx.update_window(window, |_, window, cx| {
        let keys = page.read(cx).keys;
        window.press("a", cx);
        assert_eq!(page.read(cx).keys, keys + 1, "keys are swallowed");
    })
    .unwrap();
}

#[gpui_kit::test]
fn dialog_releases_input_after_close(cx: &mut TestAppContext) {
    open_close_many_times(Kind::Dialog, cx);
}

#[gpui_kit::test]
fn sheet_releases_input_after_close(cx: &mut TestAppContext) {
    open_close_many_times(Kind::Sheet, cx);
}

#[gpui_kit::test]
fn drawer_releases_input_after_close(cx: &mut TestAppContext) {
    open_close_many_times(Kind::Drawer, cx);
}
