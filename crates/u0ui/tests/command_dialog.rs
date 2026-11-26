//! A CommandDialog opens with focus in its search field (not on the modal's
//! close button), Enter runs the first match of a typed query without ↓, and
//! closing hands focus back to where it was (design review A2, G1).
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
                .item(CommandItem::new("Preferences").on_select(pick("Preferences"))),
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

#[gpui_kit::test]
fn command_dialog_focuses_its_field_and_enter_runs_the_first_match(cx: &mut TestAppContext) {
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
            |window, cx| {
                let focus = cx.focus_handle();
                focus.focus(window, cx);
                cx.new(|_| Page {
                    open: false,
                    picked: None,
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
