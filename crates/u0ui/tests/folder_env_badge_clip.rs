//! A squeezed `EnvironmentBadge` in a `ProjectFolderHeader` keeps its name
//! whole and clips the connection dot (`.t3-folder-main .t3-envbadge {
//! min-width: 0; overflow: hidden }` over an un-ellipsized `<span>`): the
//! reference's "marketing-site" row shows "build-box" in an 81px badge with
//! the dot cut off (design review FT1B, item 4).
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AppContext as _, Bounds, Context, IntoElement, ParentElement as _, Point, Render, Styled as _,
    TestAppContext, Window, WindowBounds, WindowOptions, div, px, size,
};
use std::time::Duration;
use u0ui::t3::shell::{
    ConnectionStatus, EnvironmentBadge, FolderCounts, MachineKind, ProjectFolderHeader,
};

struct Page {
    width: f32,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().flex().flex_col().w(px(self.width)).child(
            ProjectFolderHeader::new("folder", "marketing-site")
                .environment(
                    EnvironmentBadge::new("build-box")
                        .id("env-dot")
                        .kind(MachineKind::Server)
                        .status(ConnectionStatus::Connected),
                )
                .counts(FolderCounts {
                    failed: 1,
                    ..Default::default()
                }),
        )
    }
}

/// Whether the dot shows in a `width` sidebar.
fn dot_visible(cx: &mut TestAppContext, width: f32) -> bool {
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
    cx.update_window(window, |_, window, _| window.find("env-dot").visible())
        .unwrap()
}

#[gpui_kit::test]
fn a_squeezed_badge_clips_its_dot_not_its_name(cx: &mut TestAppContext) {
    // The gallery's 300px reference panel.
    assert!(!dot_visible(cx, 300.));
}

#[gpui_kit::test]
fn a_roomy_badge_shows_its_dot(cx: &mut TestAppContext) {
    assert!(dot_visible(cx, 600.));
}
