//! Ghost · Primitives › TagInput: a live creatable tag field (with an
//! optional maximum) plus the reference field.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, rems,
};
use u0ui::ghost::primitives::TagInput;

const SUGGESTIONS: [&str; 6] = [
    "frontend",
    "backend",
    "urgent",
    "docs",
    "design-system",
    "infra",
];

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| TagInputPage {
        value: vec!["frontend".into(), "urgent".into()],
        max: false,
    })
    .into()
}

struct TagInputPage {
    value: Vec<SharedString>,
    max: bool,
}

/// `components/TagInput/preview.html`: 360px wide.
pub fn reference(id: &'static str) -> impl IntoElement {
    div().w(rems(22.5)).max_w_full().child(
        TagInput::new(id)
            .label("Tags")
            .default_value(["frontend", "urgent"])
            .suggestions(SUGGESTIONS),
    )
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(
            TagInput::new("thumb-tags")
                .label("Tags")
                .default_value(["frontend", "urgent"]),
        )
        .into_any_element()
}

impl Render for TagInputPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let mut tags = TagInput::new("tags-live")
            .label("Tags")
            .value(self.value.clone())
            .suggestions(SUGGESTIONS)
            .on_value_change(controls::setter(&entity, |this: &mut Self, v| {
                this.value = v
            }));
        if self.max {
            tags = tags.max(3);
        }
        let config = controls::toggle(
            "tags-max",
            "max = 3",
            self.max,
            controls::setter(&entity, |this: &mut Self, on| this.max = on),
        );
        Body::new()
            .playground(div().w(rems(22.5)).max_w_full().child(tags))
            .hint("↑/↓ choose, Enter or comma adds, Backspace on empty removes the last tag")
            .config(config)
            .example(controls::block("Reference preview", reference("tags-ref")))
    }
}
