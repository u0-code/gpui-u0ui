//! Data display › Empty: the reference "No buckets yet" state (a 5rem icon
//! box, title, description, two actions) and a live state with each media variant.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled, Window, div,
};
use u0ui::components::empty::{
    Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyMediaVariant, EmptyTitle,
};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| EmptyPage {
        media: 1,
        actions: true,
    })
    .into()
}

struct EmptyPage {
    media: usize,
    actions: bool,
}

fn state(id: &'static str, variant: EmptyMediaVariant, actions: bool) -> Empty {
    let media = EmptyMedia::new().variant(variant);
    let icon = Icon::carbon(CarbonIcon::Folder)
        .size(IconSize::S32)
        .color(media.icon_color());
    let mut empty = Empty::new().child(
        EmptyHeader::new()
            .child(media.child(icon))
            .child(EmptyTitle::new("No buckets yet"))
            .child(EmptyDescription::new(
                "Buckets hold your objects. Create one to start uploading data.",
            )),
    );
    if actions {
        empty = empty.child(
            EmptyContent::new()
                .child(Button::new((id, 0usize), "Create bucket").icon(CarbonIcon::Add))
                .child(Button::new((id, 1usize), "Learn more").variant(ButtonVariant::Ghost)),
        );
    }
    empty
}

impl Render for EmptyPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let variants = EmptyMediaVariant::ALL.map(|v| v.name());
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "EmptyMedia variant",
                controls::choices(
                    "empty-media",
                    &variants,
                    self.media,
                    controls::setter(&entity, |this: &mut EmptyPage, i| this.media = i),
                ),
            ))
            .child(controls::toggle(
                "empty-actions",
                "EmptyContent",
                self.actions,
                controls::setter(&entity, |this: &mut EmptyPage, on| this.actions = on),
            ));
        Body::new()
            .playground(state(
                "live",
                EmptyMediaVariant::ALL[self.media],
                self.actions,
            ))
            .config(config)
            .example(controls::block(
                "Reference preview",
                state("ref", EmptyMediaVariant::Icon, true),
            ))
    }
}
