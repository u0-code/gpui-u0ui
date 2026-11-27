//! Ghost · Primitives › KeyRecorder: a live shortcut recorder (with an
//! optional conflict) plus the three reference recorders.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div,
};
use u0ui::ghost::primitives::KeyRecorder;
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| KeyRecorderPage {
        value: "⌘+⇧+K".into(),
        conflict: false,
    })
    .into()
}

struct KeyRecorderPage {
    value: SharedString,
    conflict: bool,
}

/// `components/KeyRecorder/preview.html`: three recorders, 24px apart.
pub fn reference(prefix: &'static str) -> impl IntoElement {
    div()
        .flex()
        .flex_wrap()
        .gap(tokens::SPACING_06)
        .child(
            KeyRecorder::new((prefix, 0usize))
                .label("Toggle side chat")
                .default_value("⌘+⇧+K"),
        )
        .child(
            KeyRecorder::new((prefix, 1usize))
                .label("New thread")
                .default_value("⌘+N")
                .conflict("New window"),
        )
        .child(KeyRecorder::new((prefix, 2usize)).label("Read aloud"))
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    KeyRecorder::new("thumb-keyrec")
        .label("Toggle side chat")
        .default_value("⌘+⇧+K")
        .into_any_element()
}

impl Render for KeyRecorderPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let mut rec = KeyRecorder::new("keyrec-live")
            .label("Toggle side chat")
            .value(self.value.clone())
            .on_value_change(controls::setter(&entity, |this: &mut Self, v| {
                this.value = v
            }));
        if self.conflict {
            rec = rec.conflict("New window");
        }
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(rec)
            .child(
                div()
                    .type_style(TypeStyle::Code01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(format!("value: \"{}\"", self.value)),
            );
        let config = controls::toggle(
            "keyrec-conflict",
            "conflict",
            self.conflict,
            controls::setter(&entity, |this: &mut Self, on| this.conflict = on),
        );
        Body::new()
            .playground(live)
            .hint("Click, then press a shortcut; Esc cancels")
            .config(config)
            .example(controls::block(
                "Reference preview",
                reference("keyrec-ref"),
            ))
    }
}
