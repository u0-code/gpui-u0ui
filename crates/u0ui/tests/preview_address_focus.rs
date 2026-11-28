//! A `PreviewChrome`'s address field kept by its caller (`PreviewAddress`)
//! takes focus from a menu item of its own More menu ("Open an address…",
//! the TerminalBrowser board), and keeps it as the menu closes, rather than
//! focus going back to More.
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, Context, ElementId, Entity, IntoElement,
    ParentElement as _, Point, Render, Styled as _, TestAppContext, Window, WindowBounds,
    WindowOptions, div, px, size,
};
use std::time::Duration;
use u0ui::components::MenuItem;
use u0ui::t3::workspace::{PreviewAddress, PreviewChrome};

struct Page {
    address: PreviewAddress,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let address = self.address.clone();
        div().w(px(400.)).child(
            PreviewChrome::new("chrome", "http://localhost:5173/cover")
                .narrow(true)
                .address(&self.address)
                .more_menu([
                    MenuItem::item("Open an address…")
                        .detail("A URL, or a local HTML file")
                        .on_select(move |window, cx| address.focus(window, cx)),
                    MenuItem::item("Reload"),
                ]),
        )
    }
}

fn open(cx: &mut TestAppContext) -> (AnyWindowHandle, Entity<Page>) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(600.), px(400.)),
                })),
                ..Default::default()
            },
            cx,
            |window, cx| {
                let address = PreviewAddress::new(window, cx);
                cx.new(|_| Page { address })
            },
        )
        .expect("open test window")
    })
}

fn wait(window: AnyWindowHandle, cx: &mut TestAppContext) {
    for _ in 0..3 {
        cx.executor().advance_clock(Duration::from_millis(100));
        cx.run_until_parked();
        cx.update_window(window, |_, window, cx| window.render_frame(cx))
            .unwrap();
    }
}

fn act(
    window: AnyWindowHandle,
    cx: &mut TestAppContext,
    f: impl FnOnce(&mut Window, &mut gpui_kit::App),
) {
    cx.update_window(window, |_, window, cx| f(window, cx))
        .unwrap();
    wait(window, cx);
}

#[gpui_kit::test]
fn open_an_address_focuses_the_field_the_caller_keeps(cx: &mut TestAppContext) {
    let (window, page) = open(cx);
    wait(window, cx);
    let address = cx.update(|cx| page.read(cx).address.clone());
    let focused = |cx: &mut TestAppContext| {
        cx.update_window(window, |_, w, cx| address.focus_handle(cx).is_focused(w))
            .unwrap()
    };
    // The chrome fills in the field the caller made.
    assert_eq!(
        cx.update(|cx| address.text(cx)),
        "http://localhost:5173/cover"
    );
    assert!(!focused(cx));
    let more = ElementId::from((
        ElementId::from((ElementId::from("chrome"), "more")),
        "button",
    ));
    act(window, cx, |w, cx| w.click(more, cx));
    act(window, cx, |w, cx| w.press("enter", cx));
    assert!(focused(cx), "the address field has focus");
}
