//! Ghost · Primitives › SecretField: a live secret field reporting its source,
//! plus the three reference fields.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, rems,
};
use u0ui::ghost::primitives::{SecretField, SecretSource};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| SecretFieldPage {
        source: SecretSource::Plain,
    })
    .into()
}

struct SecretFieldPage {
    source: SecretSource,
}

/// `components/SecretField/preview.html`: 420px wide, 16px apart.
pub fn reference(prefix: &'static str) -> impl IntoElement {
    div()
        .w(rems(26.25))
        .max_w_full()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_05)
        .child(
            SecretField::new((prefix, 0usize))
                .label("API key")
                .value("sk-live-123456"),
        )
        .child(
            SecretField::new((prefix, 1usize))
                .label("GitHub token")
                .source(SecretSource::OnePassword),
        )
        .child(
            SecretField::new((prefix, 2usize))
                .label("Signing key")
                .source(SecretSource::Keychain),
        )
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(
            SecretField::new("thumb-secret")
                .label("API key")
                .value("sk-live-123456"),
        )
        .into_any_element()
}

impl Render for SecretFieldPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let live = div()
            .w(rems(26.25))
            .max_w_full()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(
                SecretField::new("secret-live")
                    .label("API key")
                    .value("sk-live-123456")
                    .on_source_change(controls::setter(&entity, |this: &mut Self, s| {
                        this.source = s
                    })),
            )
            .child(
                div()
                    .type_style(TypeStyle::Code01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(format!("source: \"{}\"", self.source.name())),
            );
        Body::new()
            .playground(live)
            .hint("Switch the source from the menu; the eye reveals a typed value")
            .example(controls::block(
                "Reference preview",
                reference("secret-ref"),
            ))
    }
}
