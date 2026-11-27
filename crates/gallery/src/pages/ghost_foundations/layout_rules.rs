//! Ghost · Foundations › LayoutRules: the three-column shell at the
//! reference widths (scaled to 60% as in the preview) and a live shell whose
//! width, zoom and open columns you set.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, px, rems,
};
use u0ui::ghost::foundations::{ShellLayout, rem_scale};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| LayoutRulesPage {
        width: 1,
        zoom: 1,
        sidebar: true,
        right: true,
    })
    .into()
}

struct LayoutRulesPage {
    width: usize,
    zoom: usize,
    sidebar: bool,
    right: bool,
}

const WIDTHS: [f32; 4] = [1120., 960., 800., 600.];
const ZOOMS: [f32; 4] = [0.8, 1., 1.333, 2.];

/// `.g-shellmock`: a bold title over three placeholder bars.
fn mock(title: &'static str) -> impl IntoElement {
    let bar = |w: f32| {
        div()
            .h(rems(0.5))
            .w(gpui_kit::relative(w))
            .mb(tokens::SPACING_03)
            .bg(color(ColorRole::LayerAccent01))
    };
    div()
        .p(tokens::SPACING_04)
        .child(
            div()
                .mb(tokens::SPACING_03)
                .text_size(rems(0.875))
                .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                .text_color(color(ColorRole::TextPrimary))
                .child(title),
        )
        // `.g-shellmock i:nth-child(odd) { width: 70% }`: the title is child 1,
        // so only the middle bar is short.
        .child(bar(1.))
        .child(bar(0.7))
        .child(bar(1.))
}

fn shell(
    id: SharedString,
    width_px: f32,
    height_px: f32,
    sidebar: bool,
    right: bool,
) -> ShellLayout {
    ShellLayout::new(id)
        .width(rems(width_px / 16.))
        .height(rems(height_px / 16.))
        .sidebar(mock("Sidebar"))
        .thread(mock("Thread · min 480px"))
        .right(mock("Right column"))
        .sidebar_open(sidebar)
        .right_open(right)
}

/// `components/LayoutRules/preview.html`: four widths at `scale(0.6)`.
pub fn reference() -> impl IntoElement {
    let rows = [
        (1120., "1120px：三栏并排"),
        (960., "960px：右栏浮在线程上"),
        (800., "800px（最小宽度）：侧栏 + 线程，右栏浮起"),
        (
            600.,
            "800px 窗口放大到 133%（≈600 CSS px）：侧栏和右栏都浮起",
        ),
    ];
    div()
        .flex()
        .flex_col()
        .gap(px(10.))
        .children(rows.map(|(w, label)| {
            div()
                .child(
                    div()
                        .mb(tokens::SPACING_02)
                        .text_size(rems(0.75))
                        .text_color(color(ColorRole::TextSecondary))
                        .child(label),
                )
                .child(
                    div()
                        .w(rems(w * 0.6 / 16.))
                        .h(rems(110. * 0.6 / 16.))
                        .overflow_hidden()
                        .child(rem_scale(0.6, move |_, _| {
                            shell(format!("layout-ref-{w}").into(), w, 110., true, true)
                                .into_any_element()
                        })),
                )
        }))
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .h(rems(4.))
        .overflow_hidden()
        .child(rem_scale(0.25, |_, _| {
            shell("layout-thumb".into(), 960., 256., true, true).into_any_element()
        }))
        .into_any_element()
}

impl Render for LayoutRulesPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let width = WIDTHS[self.width];
        let zoom = ZOOMS[self.zoom];
        let (sidebar, right) = (self.sidebar, self.right);
        // The live shell is drawn at half size; the zoom scales its rems
        // the way ⌘+ / ⌘− would, so the same breakpoints apply.
        let scale = 0.5 * zoom;
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_03)
            .child(
                div()
                    .w(rems(width * 0.5 / 16.))
                    .max_w_full()
                    .h(rems(160. / 16.))
                    .overflow_hidden()
                    .child(rem_scale(scale, move |_, _| {
                        shell(
                            "layout-live".into(),
                            width / zoom,
                            320. / zoom,
                            sidebar,
                            right,
                        )
                        .into_any_element()
                    })),
            )
            .child(
                div()
                    .type_style(TypeStyle::Code01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(format!(
                        "window {width}px at {}% = {:.0} CSS px",
                        (zoom * 100.).round(),
                        width / zoom
                    )),
            );
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::field(
                "window width",
                controls::choices(
                    "layout-width",
                    &["1120", "960", "800", "600"],
                    self.width,
                    controls::setter(&entity, |this: &mut Self, i| this.width = i),
                ),
            ))
            .child(controls::field(
                "zoom",
                controls::choices(
                    "layout-zoom",
                    &["80%", "100%", "133%", "200%"],
                    self.zoom,
                    controls::setter(&entity, |this: &mut Self, i| this.zoom = i),
                ),
            ))
            .child(controls::toggle(
                "layout-sidebar",
                "sidebarOpen",
                self.sidebar,
                controls::setter(&entity, |this: &mut Self, on| this.sidebar = on),
            ))
            .child(controls::toggle(
                "layout-right",
                "rightOpen",
                self.right,
                controls::setter(&entity, |this: &mut Self, on| this.right = on),
            ));
        Body::new()
            .playground(live)
            .hint("Below 1080 CSS px the right column floats; below 720 the sidebar floats too")
            .config(config)
            .example(controls::block("Reference preview", reference()))
    }
}
