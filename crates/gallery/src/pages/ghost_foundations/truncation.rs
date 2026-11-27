//! Ghost · Foundations › Truncation: paths and IDs truncate in the middle,
//! titles at the end; live over a resizable width.
//!
//! The ⌘P row stands in for the Ghost · Palette `FileResultRow`: the file
//! name (fuzzy-highlighted) whole, its directory truncated.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, rems,
};
use u0ui::ghost::foundations::{Truncate, TruncateMode};
use u0ui::ghost::primitives::{HighlightedText, fuzzy_match};
use u0ui::prelude::*;

const PATH: &str = "packages/web/src/components/sidebar/SidebarThreadRow.tsx";
const BRANCH: &str = "feature/very-long-branch-name-for-dark-tokens";
const TITLE: &str = "Investigate the memory spike that happens after the third reconnect";

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| TruncationPage { width: 1, mode: 2 }).into()
}

struct TruncationPage {
    width: usize,
    mode: usize,
}

fn cell(content: impl IntoElement) -> impl IntoElement {
    div()
        .flex_1()
        .min_w_0()
        .bg(color(ColorRole::Layer01))
        .px(tokens::SPACING_03)
        .py(tokens::SPACING_02)
        .child(content)
}

/// The ⌘P file row: icon, highlighted name, truncated directory.
fn file_row(query: &str) -> impl IntoElement {
    let (dir, file) = PATH.rsplit_once('/').unwrap_or(("", PATH));
    let hits = fuzzy_match(PATH, query).unwrap_or_default();
    let offset = dir.chars().count() + 1;
    let file_hits: Vec<usize> = hits
        .iter()
        .filter(|&&i| i >= offset)
        .map(|i| i - offset)
        .collect();
    let dir_hits: Vec<usize> = hits.iter().copied().filter(|&i| i < offset - 1).collect();
    div()
        .flex()
        .items_center()
        .gap(tokens::SPACING_03)
        .h(u0ui::density::current().row())
        .px(tokens::SPACING_04)
        .child(
            Icon::carbon(CarbonIcon::Document)
                .size(IconSize::S16)
                .color(color(ColorRole::IconSecondary)),
        )
        .child(
            div()
                .flex_none()
                .whitespace_nowrap()
                .child(HighlightedText::new(file).indices(file_hits)),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .overflow_hidden()
                .whitespace_nowrap()
                .text_ellipsis()
                .text_size(rems(0.75))
                .text_color(color(ColorRole::TextSecondary))
                .child(HighlightedText::new(dir).indices(dir_hits)),
        )
}

/// `components/Truncation/preview.html`: 110px labels beside 260px cells.
pub fn reference(prefix: &'static str) -> impl IntoElement {
    let row = |label: &'static str, content: AnyElement| {
        div()
            .flex()
            .items_center()
            .gap(tokens::SPACING_05)
            .child(
                div()
                    .w(rems(110. / 16.))
                    .flex_none()
                    .text_color(color(ColorRole::TextSecondary))
                    .child(label),
            )
            .child(div().w(rems(260. / 16.)).flex().child(cell(content)))
    };
    div()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_04)
        .type_style(TypeStyle::BodyCompact01)
        .text_color(color(ColorRole::TextPrimary))
        .child(row(
            "Path · middle",
            Truncate::new((prefix, 0usize), PATH)
                .mode(TruncateMode::Path)
                .into_any_element(),
        ))
        .child(row(
            "Branch · middle",
            Truncate::new((prefix, 1usize), BRANCH)
                .mode(TruncateMode::Middle)
                .into_any_element(),
        ))
        .child(row(
            "Title · end",
            Truncate::new((prefix, 2usize), TITLE).into_any_element(),
        ))
        .child(row("Palette ⌘P", file_row("sbtr").into_any_element()))
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(12.))
        .type_style(TypeStyle::BodyCompact01)
        .text_color(color(ColorRole::TextPrimary))
        .child(cell(
            Truncate::new("trunc-thumb", PATH).mode(TruncateMode::Path),
        ))
        .into_any_element()
}

impl Render for TruncationPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let widths = [10., 16.25, 24.];
        let mode = TruncateMode::ALL[self.mode];
        let text = match mode {
            TruncateMode::End => TITLE,
            TruncateMode::Middle => BRANCH,
            TruncateMode::Path => PATH,
        };
        let live = div()
            .w(rems(widths[self.width]))
            .flex()
            .type_style(TypeStyle::BodyCompact01)
            .text_color(color(ColorRole::TextPrimary))
            .child(cell(Truncate::new("trunc-live", text).mode(mode)));
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::field(
                "mode",
                controls::choices(
                    "trunc-mode",
                    &["end", "middle", "path"],
                    self.mode,
                    controls::setter(&entity, |this: &mut Self, i| this.mode = i),
                ),
            ))
            .child(controls::field(
                "width",
                controls::choices(
                    "trunc-width",
                    &["160", "260", "384"],
                    self.width,
                    controls::setter(&entity, |this: &mut Self, i| this.width = i),
                ),
            ));
        Body::new()
            .playground(live)
            .hint("Hover for the full text")
            .config(config)
            .example(controls::block("Reference preview", reference("trunc-ref")))
    }
}
