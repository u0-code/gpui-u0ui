//! Ghost · Foundations › AgentColors: the eight `agent-accent-*` slots, the
//! provider marks mapped onto them, and a live slot mapping.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, px, rems,
};
use u0ui::ghost::foundations::{AGENT_ACCENTS, TokenSwatches, agent_accent};
use u0ui::prelude::*;
use u0ui::t3::ProviderIcon;

const AGENTS: [(&str, u8); 5] = [
    ("Claude", 1),
    ("Codex", 2),
    ("Gemini", 3),
    ("Cursor", 4),
    ("Local", 5),
];

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| AgentColorsPage { slot: 0 }).into()
}

struct AgentColorsPage {
    slot: usize,
}

fn providers() -> impl IntoElement {
    div()
        .flex()
        .flex_wrap()
        .items_center()
        .gap(tokens::SPACING_05)
        .children(AGENTS.map(|(name, slot)| {
            div()
                .flex()
                .items_center()
                .gap(tokens::SPACING_03)
                .type_style(TypeStyle::BodyCompact01)
                .text_color(color(ColorRole::TextPrimary))
                .child(ProviderIcon::new(name).accent(Some(slot)))
                .child(name)
        }))
}

/// `components/AgentColors/preview.html` (the ContinueRow belongs to the
/// Ghost · Thread cards and joins this specimen when they land).
pub fn reference(id: &'static str) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_05)
        .child(
            TokenSwatches::new(id, AGENT_ACCENTS)
                .label("Agent accent slots")
                .columns(8),
        )
        .child(providers())
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .flex()
        .gap(px(2.))
        .children(AGENT_ACCENTS.map(|r| div().size(rems(1.5)).bg(color(r))))
        .into_any_element()
}

impl Render for AgentColorsPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let slot = self.slot + 1;
        let accent = agent_accent(slot);
        // `.t3-provider--accent { background: var(--pa); color: var(--agent-accent-text) }`.
        let live = div()
            .flex()
            .items_center()
            .gap(tokens::SPACING_04)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(rems(2.))
                    .bg(color(accent))
                    .text_color(color(ColorRole::AgentAccentText))
                    .type_style(TypeStyle::HeadingCompact01)
                    .child("C"),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .type_style(TypeStyle::BodyCompact01)
                    .text_color(color(ColorRole::TextPrimary))
                    .child("Claude")
                    .child(
                        div()
                            .type_style(TypeStyle::Code01)
                            .text_color(color(ColorRole::TextSecondary))
                            .child(format!("{{ claude: {slot} }} → {}", accent.name())),
                    ),
            );
        let config = controls::field(
            "slot",
            controls::choices(
                "agent-slot",
                &["1", "2", "3", "4", "5", "6", "7", "8"],
                self.slot,
                controls::setter(&entity, |this: &mut Self, i| this.slot = i),
            ),
        );
        Body::new()
            .playground(live)
            .hint("Map each agent to a slot; the name always shows as text")
            .config(config)
            .example(controls::block(
                "Reference preview",
                reference("agent-swatches"),
            ))
    }
}
