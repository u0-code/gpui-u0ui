//! u0 · Foundations › AgentColors: the eight `agent-accent-*` slots, the
//! provider marks mapped onto them, and a live slot mapping.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, px, rems,
};
use u0ui::prelude::*;
use u0ui::t3::{IdentitySize, ProviderIcon};
use u0ui::u0::foundations::{AGENT_ACCENTS, TokenSwatches, agent_accent};
use u0ui::u0::thread::ContinueRow;

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

/// `components/AgentColors/preview.html`: the slots, the provider marks, and
/// a 420px ContinueRow carrying slot 1.
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
        .child(
            div().w(px(420.)).max_w_full().child(
                ContinueRow::new(format!("{id}-continue"), "Add dark theme tokens")
                    .project("carbon-ui")
                    .agent("Claude")
                    .accent(1)
                    .time("2m ago"),
            ),
        )
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
        // The real mark and the real accent carrier: `ProviderIcon` draws the
        // provider's own mark (the accent only grounds an image), and the
        // ContinueRow paints the slot it is mapped to.
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(tokens::SPACING_04)
                    .child(
                        ProviderIcon::new("Claude")
                            .id("agent-live-provider")
                            .size(IdentitySize::Lg)
                            .accent(Some(slot as u8)),
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
                    ),
            )
            .child(
                div().w(px(420.)).max_w_full().child(
                    ContinueRow::new("agent-live-continue", "Add dark theme tokens")
                        .project("carbon-ui")
                        .agent("Claude")
                        .accent(slot)
                        .time("2m ago"),
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
