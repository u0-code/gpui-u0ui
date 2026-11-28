//! `RightPanelTabs::center_controls` draws the row as u0's Main board does
//! (`.rp-tabs { display: flex; align-items: center; padding-right: 4px }`):
//! "+" a ghost `sm` icon button (32px) centred in the 40px row and 4px from
//! its end. Without it, "+" keeps `.t3-rptabs`' look: `md`, at the top.
use gpui_kit::test::TestQueryExt as _;
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AppContext as _, Bounds, Context, IntoElement, ParentElement as _, Pixels, Point, Render,
    Styled as _, TestAppContext, Window, WindowBounds, WindowOptions, div, px, size,
};
use std::time::Duration;
use u0ui::t3::shell::{RightPanelTabs, SurfaceKind, SurfaceTab};

const ADD: &str = "Open another tab (⌘T)";

struct Page {
    centred: bool,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().w(px(400.)).child(
            RightPanelTabs::new(
                "rp",
                [
                    SurfaceTab::new("diff", "Diff", SurfaceKind::Diff),
                    SurfaceTab::new("info", "Details", SurfaceKind::Details),
                ],
            )
            .center_controls(self.centred)
            .labels(ADD, "Close {}", "{} more tabs")
            .on_add(|_, _| {}),
        )
    }
}

fn add_button(cx: &mut TestAppContext, centred: bool) -> Bounds<Pixels> {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let window = cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(600.), px(200.)),
                })),
                ..Default::default()
            },
            cx,
            move |_, cx| cx.new(|_| Page { centred }),
        )
        .expect("open test window")
        .0
    });
    cx.executor().advance_clock(Duration::from_millis(300));
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| window.render_frame(cx))
        .unwrap();
    cx.update_window(window, |_, window, _| window.find_by_label(ADD).bounds())
        .unwrap()
}

#[gpui_kit::test]
fn centred_add_is_a_small_button_in_the_middle_of_the_row(cx: &mut TestAppContext) {
    let b = add_button(cx, true);
    assert_eq!(b.size.height, px(32.), "{b:?}");
    // The 40px row less its 1px bottom rule centres it.
    assert!((f32::from(b.center().y) - 19.5).abs() <= 0.5, "{b:?}");
    assert!((f32::from(b.right()) - (400. - 4.)).abs() <= 0.5, "{b:?}");
}

#[gpui_kit::test]
fn by_default_add_keeps_the_strip_look(cx: &mut TestAppContext) {
    let b = add_button(cx, false);
    assert_eq!(b.top(), px(0.), "{b:?}");
    assert!((f32::from(b.right()) - 400.).abs() <= 0.5, "{b:?}");
}
