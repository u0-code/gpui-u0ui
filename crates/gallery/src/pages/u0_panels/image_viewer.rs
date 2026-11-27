//! u0 · Right column › ImageViewer: the live viewer (zoom, fit, pan,
//! background) over a placeholder of a chosen size and paint, and the
//! reference image.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Background, Context, IntoElement, ParentElement,
    Render, Styled, Window, div, linear_color_stop, linear_gradient, rems,
};
use u0ui::prelude::*;
use u0ui::u0::panels::ImageViewer;

const SIZES: [&str; 3] = ["1440×900", "512×512", "64×64"];
const PAINTS: [&str; 3] = ["gradient", "charts-3", "support-info"];

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page { size: 0, paint: 0 }).into()
}

struct Page {
    size: usize,
    paint: usize,
}

/// `linear-gradient(135deg, var(--charts-1), var(--charts-2))`.
fn gradient() -> Background {
    linear_gradient(
        135.,
        linear_color_stop(color(ColorRole::Charts1), 0.),
        linear_color_stop(color(ColorRole::Charts2), 1.),
    )
}

/// `components/ImageViewer/preview.html`.
fn reference() -> impl IntoElement {
    ImageViewer::new("iv-ref", "hero@2x.png")
        .dimensions(1440, 900)
        .alt("Hero illustration")
        .color(gradient())
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(
            ImageViewer::new("iv-thumb", "hero.png")
                .dimensions(1440, 900)
                .color(gradient()),
        )
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let e = cx.entity();
        let (w, h, name) = match self.size {
            0 => (1440, 900, "hero@2x.png"),
            1 => (512, 512, "avatar.png"),
            _ => (64, 64, "favicon.png"),
        };
        let paint: Background = match self.paint {
            0 => gradient(),
            1 => color(ColorRole::Charts3).into(),
            _ => color(ColorRole::SupportInfo).into(),
        };
        let live = div().w_full().max_w(rems(40.)).child(
            ImageViewer::new(("iv-live", self.size), name)
                .dimensions(w, h)
                .alt("Illustration")
                .color(paint),
        );
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::field(
                "width × height",
                controls::choices(
                    "iv-size",
                    &SIZES,
                    self.size,
                    controls::setter(&e, |p: &mut Page, v| p.size = v),
                ),
            ))
            .child(controls::field(
                "color (placeholder)",
                controls::choices(
                    "iv-paint",
                    &PAINTS,
                    self.paint,
                    controls::setter(&e, |p: &mut Page, v| p.paint = v),
                ),
            ));
        Body::new()
            .playground(live)
            .hint("Zoom in past the stage, then drag or scroll to pan; the palette button cycles the backdrop")
            .config(config)
            .example(controls::block("Reference preview", reference()))
    }
}
