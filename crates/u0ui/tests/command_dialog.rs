//! A CommandDialog opens with focus in its search field (not on the modal's
//! close button), Enter runs the first match of a typed query without ↓, and
//! closing hands focus back to where it was (design review A2, G1), unless
//! the entry that closed it moved focus on purpose (design review A3, S3).
use gpui_kit::test::{TestSupportExt as _, TestWindowExt as _};
use gpui_kit::{
    AppContext as _, Bounds, Context, FocusHandle, InteractiveElement as _, IntoElement,
    ParentElement as _, Point, Render, StatefulInteractiveElement as _, Styled as _,
    TestAppContext, Window, WindowBounds, WindowOptions, div, px, size,
};
use std::time::Duration;
use u0ui::components::{Command, CommandDialog, CommandGroup, CommandItem};

struct Page {
    open: bool,
    picked: Option<&'static str>,
    focus: FocusHandle,
    /// Where the "Navigate" entry moves focus (a new page's content).
    target: FocusHandle,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let pick = |name: &'static str| {
            let entity = entity.clone();
            move |_: &mut Window, cx: &mut gpui_kit::App| {
                entity.update(cx, |this, cx| {
                    this.picked = Some(name);
                    this.open = false;
                    cx.notify();
                })
            }
        };
        let command = Command::new("palette-command").group(
            CommandGroup::new("Pages")
                .item(CommandItem::new("Home").on_select(pick("Home")))
                .item(CommandItem::new("Profile").on_select(pick("Profile")))
                .item(CommandItem::new("Preferences").on_select(pick("Preferences")))
                .item(CommandItem::new("Navigate").on_select({
                    let entity = entity.clone();
                    let target = self.target.clone();
                    move |window: &mut Window, cx: &mut gpui_kit::App| {
                        target.focus(window, cx);
                        entity.update(cx, |this, cx| {
                            this.picked = Some("Navigate");
                            this.open = false;
                            cx.notify();
                        })
                    }
                })),
        );
        div()
            .id("page")
            .track_focus(&self.focus)
            .size_full()
            .child(
                div()
                    .id("open")
                    .test_support()
                    .w(px(120.))
                    .h(px(40.))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.open = true;
                        cx.notify();
                    }))
                    .child("Search"),
            )
            .child(div().id("main").track_focus(&self.target).h(px(40.)))
            .child(CommandDialog::new(
                "palette",
                self.open,
                {
                    let entity = entity.clone();
                    move |open, _, cx| {
                        entity.update(cx, |this, cx| {
                            this.open = open;
                            cx.notify();
                        })
                    }
                },
                command,
            ))
    }
}

fn open_page(cx: &mut TestAppContext) -> (gpui_kit::AnyWindowHandle, gpui_kit::Entity<Page>) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(1024.), px(720.)),
                })),
                ..Default::default()
            },
            cx,
            |window, cx| {
                let focus = cx.focus_handle();
                focus.focus(window, cx);
                cx.new(|cx| Page {
                    open: false,
                    picked: None,
                    focus,
                    target: cx.focus_handle(),
                })
            },
        )
        .expect("open test window")
    })
}

#[gpui_kit::test]
fn command_dialog_focuses_its_field_and_enter_runs_the_first_match(cx: &mut TestAppContext) {
    let (window, page) = open_page(cx);
    let settle = |cx: &mut TestAppContext| {
        cx.executor().advance_clock(Duration::from_secs(1));
        cx.run_until_parked();
        cx.update_window(window, |_, window, cx| window.render_frame(cx))
            .unwrap();
    };
    settle(cx);
    cx.update_window(window, |_, window, cx| window.click("open", cx))
        .unwrap();
    settle(cx);
    // Typing reaches the field at once; Enter runs the only match.
    cx.update_window(window, |_, window, cx| window.input("prof", cx))
        .unwrap();
    settle(cx);
    cx.update_window(window, |_, window, cx| window.press("enter", cx))
        .unwrap();
    settle(cx);
    cx.update_window(window, |_, window, cx| {
        let page = page.read(cx);
        assert_eq!(page.picked, Some("Profile"));
        assert!(!page.open);
        // Focus is back where it was before the palette opened.
        assert!(page.focus.is_focused(window), "focus was not restored");
    })
    .unwrap();
    // Reopened, it starts from an empty query: "pref" matches on its own.
    cx.update_window(window, |_, window, cx| window.click("open", cx))
        .unwrap();
    settle(cx);
    cx.update_window(window, |_, window, cx| window.input("pref", cx))
        .unwrap();
    settle(cx);
    cx.update_window(window, |_, window, cx| window.press("enter", cx))
        .unwrap();
    settle(cx);
    cx.update(|cx| assert_eq!(page.read(cx).picked, Some("Preferences")));
}

#[gpui_kit::test]
fn an_entry_that_moves_focus_keeps_it_there(cx: &mut TestAppContext) {
    let (window, page) = open_page(cx);
    let settle = |cx: &mut TestAppContext| {
        cx.executor().advance_clock(Duration::from_secs(1));
        cx.run_until_parked();
        cx.update_window(window, |_, window, cx| window.render_frame(cx))
            .unwrap();
    };
    settle(cx);
    cx.update_window(window, |_, window, cx| window.click("open", cx))
        .unwrap();
    settle(cx);
    cx.update_window(window, |_, window, cx| window.input("navi", cx))
        .unwrap();
    settle(cx);
    cx.update_window(window, |_, window, cx| window.press("enter", cx))
        .unwrap();
    settle(cx);
    cx.update_window(window, |_, window, cx| {
        let page = page.read(cx);
        assert_eq!(page.picked, Some("Navigate"));
        assert!(page.target.is_focused(window), "focus was pulled back");
    })
    .unwrap();
    // Closed with Escape (focus still in the field): focus returns.
    cx.update_window(window, |_, window, cx| {
        page.read(cx).focus.clone().focus(window, cx)
    })
    .unwrap();
    settle(cx);
    cx.update_window(window, |_, window, cx| window.click("open", cx))
        .unwrap();
    settle(cx);
    cx.update_window(window, |_, window, cx| window.press("escape", cx))
        .unwrap();
    settle(cx);
    cx.update_window(window, |_, window, cx| {
        let page = page.read(cx);
        assert!(!page.open);
        assert!(page.focus.is_focused(window), "focus was not restored");
    })
    .unwrap();
}
