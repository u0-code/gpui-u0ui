//! u0 · Primitives › HighlightedText: fuzzy-match highlighting, live over a
//! typed text and query, plus the reference rows.
use crate::page::Body;
use crate::{controls, stand_in};
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, Entity, IntoElement, ParentElement, Render,
    Styled, Window, component::input::InputState, div, rems,
};
use u0ui::prelude::*;
use u0ui::u0::primitives::{HighlightedText, fuzzy_match};

/// Build the page.
pub fn build(window: &mut Window, cx: &mut App) -> AnyView {
    let text = stand_in::input_state(
        "packages/web/src/components/sidebar/SidebarThreadRow.tsx",
        window,
        cx,
    );
    let query = stand_in::input_state("sbtr", window, cx);
    cx.new(|cx| {
        cx.observe(&text, |_, _, cx| cx.notify()).detach();
        cx.observe(&query, |_, _, cx| cx.notify()).detach();
        HighlightedTextPage { text, query }
    })
    .into()
}

struct HighlightedTextPage {
    text: Entity<InputState>,
    query: Entity<InputState>,
}

/// `{ fontSize: 14 }` rows in `text-primary`.
fn row(text: &'static str, query: &'static str) -> AnyElement {
    div()
        .type_style(TypeStyle::BodyCompact01)
        .text_color(color(ColorRole::TextPrimary))
        .child(HighlightedText::new(text).query(query))
        .into_any_element()
}

/// `components/HighlightedText/preview.html`: a grid with an 8px gap.
pub fn reference() -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_03)
        .child(row("SidebarThreadRow.tsx", "stro"))
        .child(row("Open settings", "opset"))
        .child(row("packages/web/src/components", "pwsc"))
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    reference().into_any_element()
}

impl Render for HighlightedTextPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let text = self.text.read(cx).value().to_string();
        let query = self.query.read(cx).value().to_string();
        let result = match fuzzy_match(&text, &query) {
            Some(hits) => format!("fuzzyMatch → {hits:?}"),
            None => "fuzzyMatch → null (no match)".to_string(),
        };
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_03)
            .min_w_0()
            .child(
                div()
                    .type_style(TypeStyle::BodyCompact01)
                    .text_color(color(ColorRole::TextPrimary))
                    .child(HighlightedText::new(text).query(query)),
            )
            .child(
                div()
                    .type_style(TypeStyle::Code01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(result),
            );

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .max_w(rems(18.))
            .child(stand_in::text_field(
                "hl-text",
                "text",
                &self.text,
                LayerLevel::Two,
            ))
            .child(stand_in::text_field(
                "hl-query",
                "query",
                &self.query,
                LayerLevel::Two,
            ));

        Body::new()
            .playground(live)
            .hint("Type a query: characters match in order, case-insensitively")
            .config(config)
            .example(controls::block("Reference preview", reference()))
    }
}
