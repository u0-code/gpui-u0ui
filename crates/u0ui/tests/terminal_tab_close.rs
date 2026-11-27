//! Regression: closing a terminal tab moves the active tab the way a
//! browser tab strip does (design review FT3B). With "test" selected,
//! closing it with its × leaves "dev" active (`aria-selected`, the
//! `.t3-term-tab--on` look), not an out-of-range index. The u0
//! `TerminalTabs` strip follows the same rule when uncontrolled.
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, Context, ElementId, IntoElement, ParentElement as _,
    Point, Render, SharedString, Styled as _, TestAppContext, Window, WindowBounds, WindowOptions,
    div, px, size,
};
use std::time::Duration;
use u0ui::t3::workspace::{TerminalDrawer, TerminalTab};
use u0ui::u0::panels::{TerminalTab as UTab, TerminalTabs};

struct Drawer {
    tabs: Vec<TerminalTab>,
}

impl Render for Drawer {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        div()
            .w(px(900.))
            .h(px(240.))
            .child(
                TerminalDrawer::new("td", self.tabs.clone()).on_close(move |i, _, cx| {
                    entity.update(cx, |this, cx| {
                        this.tabs.remove(i);
                        cx.notify();
                    })
                }),
            )
    }
}

struct Strip {
    tabs: Vec<UTab>,
}

impl Render for Strip {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        div()
            .w(px(900.))
            .child(
                TerminalTabs::new("tt", self.tabs.clone()).on_close(move |i, _, cx| {
                    entity.update(cx, |this, cx| {
                        this.tabs.remove(i);
                        cx.notify();
                    })
                }),
            )
    }
}

fn part(root: &'static str, name: String) -> ElementId {
    ElementId::from((ElementId::from(root), SharedString::from(name)))
}

fn open<V: Render>(cx: &mut TestAppContext, view: impl FnOnce() -> V + 'static) -> AnyWindowHandle {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let window = cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(1000.), px(400.)),
                })),
                ..Default::default()
            },
            cx,
            |_, cx| cx.new(|_| view()),
        )
        .expect("open test window")
    });
    window.0
}

fn wait(window: AnyWindowHandle, cx: &mut TestAppContext) {
    cx.executor().advance_clock(Duration::from_millis(100));
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| window.render_frame(cx))
        .unwrap();
}

#[gpui_kit::test]
fn closing_the_active_drawer_tab_activates_its_neighbor(cx: &mut TestAppContext) {
    let window = open(cx, || Drawer {
        tabs: vec![
            TerminalTab::new("dev").running(true),
            TerminalTab::new("test"),
            TerminalTab::new("build"),
            TerminalTab::new("lint"),
        ],
    });
    let tab = |i: usize| part("td", format!("tab-{i}"));
    let close = |i: usize| part("td", format!("tab-x-{i}"));
    let click = |id: ElementId, cx: &mut TestAppContext| {
        cx.update_window(window, |_, w, cx| w.click(id, cx))
            .unwrap();
        wait(window, cx);
    };
    let active = |cx: &mut TestAppContext| {
        (0..4)
            .filter(|&i| {
                cx.update_window(window, |_, w, _| {
                    w.try_find(tab(i)).and_then(|e| e.selected())
                })
                .unwrap()
                    == Some(true)
            })
            .collect::<Vec<_>>()
    };
    wait(window, cx);
    assert_eq!(active(cx), [0]);

    // Close a tab before the active one: the index shifts down with it.
    click(tab(2), cx);
    assert_eq!(active(cx), [2]);
    click(close(0), cx);
    // [test, build, lint]: "build" is now index 1 and still active.
    assert_eq!(
        active(cx),
        [1],
        "closing an earlier tab moved the selection"
    );

    // Close the active middle tab: the next one ("lint") becomes active.
    click(close(1), cx);
    assert_eq!(active(cx), [1]);

    // Close the active last tab: the previous one becomes active.
    click(close(1), cx);
    assert_eq!(active(cx), [0], "the remaining tab is not active");
}

#[gpui_kit::test]
fn closing_the_active_terminal_tab_activates_its_neighbor(cx: &mut TestAppContext) {
    let window = open(cx, || Strip {
        tabs: ["dev", "test", "build", "lint"]
            .into_iter()
            .map(UTab::new)
            .collect(),
    });
    let tab = |i: usize| part("tt", format!("t{i}"));
    let close = |i: usize| ElementId::from((tab(i), "x"));
    let click = |id: ElementId, cx: &mut TestAppContext| {
        cx.update_window(window, |_, w, cx| w.click(id, cx))
            .unwrap();
        wait(window, cx);
    };
    let active = |cx: &mut TestAppContext| {
        (0..4)
            .filter(|&i| {
                cx.update_window(window, |_, w, _| {
                    w.try_find(tab(i)).and_then(|e| e.selected())
                })
                .unwrap()
                    == Some(true)
            })
            .collect::<Vec<_>>()
    };
    wait(window, cx);
    assert_eq!(active(cx), [0]);

    click(tab(2), cx);
    assert_eq!(active(cx), [2]);
    click(close(0), cx);
    assert_eq!(
        active(cx),
        [1],
        "closing an earlier tab moved the selection"
    );
    click(close(1), cx);
    assert_eq!(active(cx), [1]);
    click(close(1), cx);
    assert_eq!(active(cx), [0], "the remaining tab is not active");
}
