//! T3 · Timeline › UserMessage: the folded G90 request with two context
//! chips and an image (`components/UserMessage/preview.html`); hover the
//! message for Copy and Edit, Show more unfolds it.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, rgb,
};
use u0ui::prelude::*;
use u0ui::t3::composer::{AttachmentImage, ContextChip, ContextChipKind};
use u0ui::t3::timeline::UserMessage;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| UserPage {
        folded: true,
        contexts: true,
        images: true,
        generation: 0,
        last: "".into(),
    })
    .into()
}

struct UserPage {
    folded: bool,
    contexts: bool,
    images: bool,
    generation: usize,
    last: SharedString,
}

const TEXT: &str = "Add the G90 theme to tokens.json and regenerate the cover. Keep the brand blue on the large block, and check contrast for every text token in both dark themes. Then open a PR against main with a short summary.";

fn message(id: SharedString, folded: bool, contexts: bool, images: bool) -> UserMessage {
    let mut m = UserMessage::new(id.clone(), TEXT)
        .time("10:42")
        .folded(folded);
    if contexts {
        m = m.contexts(vec![
            ContextChip::new(
                (gpui_kit::ElementId::from(id.clone()), "c0"),
                ContextChipKind::File,
                "tokens.json",
            ),
            ContextChip::new(
                (gpui_kit::ElementId::from(id.clone()), "c1"),
                ContextChipKind::PrOpen,
                "#482",
            ),
        ]);
    }
    if images {
        m = m.images(vec![AttachmentImage::new("shot.png").color(rgb(0x0f62fe))]);
    }
    m
}

impl Render for UserPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let note = |what: &'static str| {
            let entity = entity.clone();
            move |_: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    this.last = what.into();
                    cx.notify();
                })
            }
        };
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_09)
            .w_full()
            .max_w(gpui_kit::rems(45.))
            .child(
                message(
                    format!("umsg-live-{}-{}", self.generation, self.folded).into(),
                    self.folded,
                    self.contexts,
                    self.images,
                )
                .on_copy(note("onCopy"))
                .on_edit(note("onEdit")),
            )
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
                "umsg-folded",
                "folded",
                self.folded,
                controls::setter(&entity, |this: &mut Self, on| this.folded = on),
            ))
            .child(controls::toggle(
                "umsg-contexts",
                "contexts",
                self.contexts,
                controls::setter(&entity, |this: &mut Self, on| this.contexts = on),
            ))
            .child(controls::toggle(
                "umsg-images",
                "images",
                self.images,
                controls::setter(&entity, |this: &mut Self, on| this.images = on),
            ));
        Body::new()
            .playground(live)
            .hint("Hover the message for Copy and Edit and resend.")
            .config(config)
            .example(controls::block(
                "Reference preview",
                div()
                    .w_full()
                    .max_w(gpui_kit::rems(45.))
                    .pb(tokens::SPACING_07)
                    .child(message("umsg-ref".into(), true, true, true)),
            ))
    }
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(gpui_kit::rems(15.))
        .child(UserMessage::new("umsg-th", "Add the G90 theme to tokens.json.").time("10:42"))
        .into_any_element()
}
