//! The DataTable toolbar search clears on Escape, as Carbon `Search` does
//! (design review C2, item 7).
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AppContext as _, Bounds, Context, Entity, Focusable as _, IntoElement, ParentElement as _,
    Point, Render, Styled as _, TestAppContext, Window, WindowBounds, WindowOptions,
    component::input::Input as KitInput, div, px, size,
};
use std::time::Duration;
use u0ui::components::data_table::{InputState, search_state};

struct Page {
    state: Entity<InputState>,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(KitInput::new(&self.state))
    }
}

#[gpui_kit::test]
fn escape_clears_the_toolbar_search(cx: &mut TestAppContext) {
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
                let state = cx.new(|cx| search_state(window, cx));
                state.read(cx).focus_handle(cx).focus(window, cx);
                cx.new(|_| Page { state })
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
    cx.update_window(window, |_, window, cx| window.input("pods", cx))
        .unwrap();
    settle(cx);
    let value = |cx: &mut TestAppContext| cx.update(|cx| page.read(cx).state.read(cx).value());
    assert_eq!(value(cx).as_ref(), "pods");
    cx.update_window(window, |_, window, cx| window.press("escape", cx))
        .unwrap();
    settle(cx);
    assert_eq!(value(cx).as_ref(), "");
}
