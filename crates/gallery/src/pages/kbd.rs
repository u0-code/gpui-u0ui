//! Data display › Kbd: the reference shortcuts (⌘ K, "Press Esc to close",
//! Ctrl + Shift + P) and a live key built from typed text.
use crate::page::Body;
use crate::{controls, stand_in};
use gpui_kit::{
    AnyView, App, AppContext as _, Context, Entity, IntoElement, ParentElement, Render, Styled,
    Window, component::input::InputState, div, rems,
};
use u0ui::components::kbd::{Kbd, KbdGroup};
use u0ui::prelude::*;

/// Build the page.
pub fn build(window: &mut Window, cx: &mut App) -> AnyView {
    let keys = stand_in::input_state("Ctrl Alt Delete", window, cx);
    cx.new(|cx| {
        cx.observe(&keys, |_, _, cx| cx.notify()).detach();
        KbdPage { keys }
    })
    .into()
}

struct KbdPage {
    keys: Entity<InputState>,
}

fn text(s: &'static str) -> impl IntoElement {
    div()
        .type_style(TypeStyle::BodyCompact01)
        .text_color(color(ColorRole::TextPrimary))
        .child(s)
}

impl Render for KbdPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // `components/Kbd/preview.html` (`cu-row`: wrap, 16px gap, centered).
        let reference = div()
            .flex()
            .flex_row()
            .flex_wrap()
            .items_center()
            .gap(tokens::SPACING_05)
            .child(KbdGroup::new().child(Kbd::new("⌘")).child(Kbd::new("K")))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .child(text("Press\u{a0}"))
                    .child(Kbd::new("Esc"))
                    .child(text("\u{a0}to close")),
            )
            .child(
                KbdGroup::new()
                    .child(Kbd::new("Ctrl"))
                    .child(text("+"))
                    .child(Kbd::new("Shift"))
                    .child(text("+"))
                    .child(Kbd::new("P")),
            );

        let typed = self.keys.read(cx).value().to_string();
        let live = KbdGroup::new().children(
            typed
                .split_whitespace()
                .map(|key| Kbd::new(key.to_string()).into_any_element()),
        );

        let config = div().max_w(rems(16.)).child(stand_in::text_field(
            "kbd-keys",
            "Keys (space-separated)",
            &self.keys,
            LayerLevel::Two,
        ));

        Body::new()
            .playground(live)
            .config(config)
            .example(controls::block("Reference preview", reference))
    }
}
