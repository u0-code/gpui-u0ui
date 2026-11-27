//! u0 · Primitives › EditableText: live inline rename with optional
//! validation, plus the two reference rows (14px and 20px).
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, rems,
};
use u0ui::prelude::*;
use u0ui::u0::primitives::EditableText;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| EditableTextPage {
        value: "Add dark theme tokens".into(),
        required: true,
    })
    .into()
}

struct EditableTextPage {
    value: SharedString,
    required: bool,
}

fn required(t: &str) -> Option<SharedString> {
    t.is_empty().then(|| "Name is required".into())
}

/// `components/EditableText/preview.html`: 14px, then 20px, 12px apart.
pub fn reference(prefix: &'static str) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_04)
        .text_color(color(ColorRole::TextPrimary))
        .child(
            div()
                .text_size(rems(0.875))
                .child(EditableText::new((prefix, 0usize)).default_value("Add dark theme tokens")),
        )
        .child(
            div().text_size(rems(1.25)).child(
                EditableText::new((prefix, 1usize))
                    .default_value("carbon-ui")
                    .validate(required),
            ),
        )
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .text_size(rems(1.25))
        .text_color(color(ColorRole::TextPrimary))
        .child(EditableText::new("thumb-edit").default_value("carbon-ui"))
        .into_any_element()
}

impl Render for EditableTextPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let mut text = EditableText::new("edit-live")
            .value(self.value.clone())
            .on_value_change(controls::setter(&entity, |this: &mut Self, v| {
                this.value = v
            }));
        if self.required {
            text = text.validate(required);
        }
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_04)
            .text_size(rems(0.875))
            .text_color(color(ColorRole::TextPrimary))
            .child(text)
            .child(
                div()
                    .type_style(TypeStyle::Code01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(format!("value: \"{}\"", self.value)),
            );
        let config = controls::toggle(
            "edit-required",
            "validate (required)",
            self.required,
            controls::setter(&entity, |this: &mut Self, on| this.required = on),
        );
        Body::new()
            .playground(live)
            .hint("Hover for the pencil; Enter or blur commits, Esc cancels")
            .config(config)
            .example(controls::block("Reference preview", reference("edit-ref")))
    }
}
