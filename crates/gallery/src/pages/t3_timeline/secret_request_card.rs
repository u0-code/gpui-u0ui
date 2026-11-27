//! T3 · Timeline › SecretRequestCard: the STRIPE_API_KEY request
//! (`components/SecretRequestCard/preview.html`) and a live card with one or
//! two fields; Save securely reports how many characters were entered.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div,
};
use u0ui::prelude::*;
use u0ui::t3::timeline::{SecretField, SecretRequestCard};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| SecretPage {
        two: false,
        last: "".into(),
    })
    .into()
}

struct SecretPage {
    two: bool,
    last: SharedString,
}

const DESCRIPTION: &str = "Needed to run the payment integration tests. Stored in your secret provider, never in the thread.";

impl Render for SecretPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let fields = if self.two {
            vec![
                SecretField::new("Client ID"),
                SecretField::new("Client secret"),
            ]
        } else {
            vec![SecretField::new("Secret value")]
        };
        let card = SecretRequestCard::new(
            SharedString::from(format!("secret-live-{}", self.two)),
            "STRIPE_API_KEY",
        )
        .description(DESCRIPTION)
        .fields(fields)
        .on_save({
            let entity = entity.clone();
            move |values, _, cx| {
                entity.update(cx, |this, cx| {
                    let lens: Vec<String> = values.iter().map(|v| v.len().to_string()).collect();
                    this.last = format!("onSave: {} characters", lens.join(" + ")).into();
                    cx.notify();
                })
            }
        })
        .on_decline({
            let entity = entity.clone();
            move |_, cx| {
                entity.update(cx, |this, cx| {
                    this.last = "onDecline".into();
                    cx.notify();
                })
            }
        });
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .w_full()
            .max_w(gpui_kit::rems(35.))
            .child(card)
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(self.last.clone()),
            );
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::toggle(
                "secret-two",
                "Two fields",
                self.two,
                controls::setter(&entity, |this: &mut Self, on| this.two = on),
            ));
        Body::new()
            .playground(live)
            .config(config)
            .example(controls::block(
                "Reference preview",
                div().w_full().max_w(gpui_kit::rems(35.)).child(
                    SecretRequestCard::new("secret-ref", "STRIPE_API_KEY")
                        .description(DESCRIPTION)
                        .fields(vec![SecretField::new("Secret value")]),
                ),
            ))
    }
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(gpui_kit::rems(15.))
        .child(SecretRequestCard::new("secret-th", "API key"))
        .into_any_element()
}
