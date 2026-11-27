//! The UserMessage bubble is `max-width: min(36rem, 90%)` (t3.css:275
//! `.t3-umsg-bubble`): in a narrow column it stops at 90% of the column,
//! in a wide one at 36rem (design review CLOSE1).
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AppContext as _, Bounds, Context, ElementId, IntoElement, ParentElement as _, Point, Render,
    Styled as _, TestAppContext, Window, WindowBounds, WindowOptions, div, px, size,
};
use std::time::Duration;
use u0ui::t3::timeline::UserMessage;

const LONG: &str = "Refactor the settings dialog so every section loads lazily, keep the \
    search box focused while filtering, and make sure the server picker remembers its choice \
    between launches without touching the global preferences file.";

struct Page {
    width: f32,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().w(px(self.width)).child(UserMessage::new("msg", LONG))
    }
}

fn bubble_width(width: f32, cx: &mut TestAppContext) -> f32 {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let window = cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(1400.), px(800.)),
                })),
                ..Default::default()
            },
            cx,
            move |_, cx| cx.new(|_| Page { width }),
        )
        .expect("open test window")
        .0
    });
    cx.executor().advance_clock(Duration::from_millis(300));
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| {
        window.render_frame(cx);
        let b = window.find((ElementId::from("msg"), "bubble")).bounds();
        f32::from(b.size.width)
    })
    .unwrap()
}

#[gpui_kit::test]
fn narrow_column_caps_at_ninety_percent(cx: &mut TestAppContext) {
    let w = bubble_width(245., cx);
    assert!(w <= 245. * 0.9 + 0.5, "bubble {w}px in a 245px column");
    assert!(w >= 245. * 0.9 - 2., "a long message fills its 90%: {w}px");
}

#[gpui_kit::test]
fn wide_column_caps_at_36rem(cx: &mut TestAppContext) {
    let w = bubble_width(1200., cx);
    assert!((w - 576.).abs() <= 0.5, "bubble {w}px, expected 576");
}
