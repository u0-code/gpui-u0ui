//! Ghost · Primitives › QRCode: a live QR code over a typed value, size and
//! error-correction level, plus the reference pairing code and link.
use crate::page::Body;
use crate::{controls, stand_in};
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, Entity, IntoElement, ParentElement, Render,
    Styled, Window, component::input::InputState, div, rems,
};
use u0ui::ghost::primitives::{QrCode, QrLevel};
use u0ui::prelude::*;

/// Build the page.
pub fn build(window: &mut Window, cx: &mut App) -> AnyView {
    let value = stand_in::input_state("ghost://pair/7f3a-9c2e-41b0", window, cx);
    cx.new(|cx| {
        cx.observe(&value, |_, _, cx| cx.notify()).detach();
        QrCodePage {
            value,
            level: 1,
            size: 1,
        }
    })
    .into()
}

struct QrCodePage {
    value: Entity<InputState>,
    level: usize,
    size: usize,
}

const SIZES: [f32; 3] = [96., 168., 240.];

/// `components/QRCode/preview.html`: 168px and 96px, 24px apart, centered.
pub fn reference(prefix: &'static str) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap(tokens::SPACING_06)
        .child(
            QrCode::new((prefix, 0usize), "ghost://pair/7f3a-9c2e-41b0")
                .size(168.)
                .label("Pairing code"),
        )
        .child(QrCode::new((prefix, 1usize), "https://example.com").size(96.))
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    QrCode::new("thumb-qr", "ghost://pair/7f3a-9c2e-41b0")
        .size(96.)
        .into_any_element()
}

impl Render for QrCodePage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let value = self.value.read(cx).value().to_string();
        let live = QrCode::new("qr-live", value)
            .size(SIZES[self.size])
            .level(QrLevel::ALL[self.level]);
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .max_w(rems(18.))
            .child(stand_in::text_field(
                "qr-value",
                "value",
                &self.value,
                LayerLevel::Two,
            ))
            .child(controls::field(
                "level",
                controls::choices(
                    "qr-level",
                    &["L", "M", "Q", "H"],
                    self.level,
                    controls::setter(&entity, |this: &mut Self, i| this.level = i),
                ),
            ))
            .child(controls::field(
                "size",
                controls::choices(
                    "qr-size",
                    &["96", "168", "240"],
                    self.size,
                    controls::setter(&entity, |this: &mut Self, i| this.size = i),
                ),
            ));
        Body::new()
            .playground(live)
            .hint("Always dark on white with a 4-module quiet zone, in every theme")
            .config(config)
            .example(controls::block("Reference preview", reference("qr-ref")))
    }
}
