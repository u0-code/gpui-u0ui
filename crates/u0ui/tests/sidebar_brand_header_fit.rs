//! In a narrow sidebar, `SidebarBrandHeader` keeps the brand name whole and
//! lets its four actions run on into the 8px end padding, as the reference
//! does (`.t3-sbhead-brand { flex: 1 }` keeps `min-width: auto`; design
//! review FT1B, item 2).
use gpui_kit::test::TestQueryExt as _;
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AppContext as _, Bounds, Context, IntoElement, ParentElement as _, Pixels, Point, Render,
    Styled as _, TestAppContext, Window, WindowBounds, WindowOptions, div, px, size,
};
use std::time::Duration;
use u0ui::t3::shell::SidebarBrandHeader;

struct Page {
    width: f32,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .w(px(self.width))
            .child(SidebarBrandHeader::new("brand", "Workbench"))
    }
}

/// The Back and New thread buttons' bounds in a `width` sidebar.
fn measure(cx: &mut TestAppContext, width: f32) -> (Bounds<Pixels>, Bounds<Pixels>) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let window = cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(800.), px(200.)),
                })),
                ..Default::default()
            },
            cx,
            move |_, cx| cx.new(|_| Page { width }),
        )
        .expect("open test window")
        .0
    });
    cx.executor().advance_clock(Duration::from_secs(1));
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| window.render_frame(cx))
        .unwrap();
    cx.update_window(window, |_, window, _| {
        (
            window.find_by_label("Back").bounds(),
            window.find_by_label("New thread (⌘N)").bounds(),
        )
    })
    .unwrap()
}

#[gpui_kit::test]
fn the_brand_keeps_its_width_and_the_actions_overflow(cx: &mut TestAppContext) {
    let (back, new) = measure(cx, 255.);
    // Squeezed, the brand would end where four 32px actions and the 8px
    // padding leave off: 255 - 8 - 4 × 32 - 8 gap = 111, Back at 119. Whole,
    // it holds the 20px mark, an 8px gap and all of "Workbench" (~75px).
    assert!(
        f32::from(back.origin.x) > 16. + 20. + 8. + 70. + 8.,
        "{back:?}"
    );
    // So the last action runs on into the 8px end padding.
    assert!(f32::from(new.right()) > 255. - 8. + 0.5, "{new:?}");
    assert!((f32::from(new.right() - back.origin.x) - 4. * 32.).abs() < 0.5);
}
