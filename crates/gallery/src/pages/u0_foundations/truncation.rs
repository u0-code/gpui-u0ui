//! u0 · Foundations › Truncation: paths and IDs truncate in the middle,
//! titles at the end; live over a resizable width. The ⌘P row is the
//! u0 · Palette `FileResultRow`.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, rems,
};
use u0ui::prelude::*;
use u0ui::u0::foundations::{Truncate, TruncateMode};
use u0ui::u0::palette::FileResultRow;

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

/// `components/Truncation/preview.html`: 110px labels beside 260px cells.
pub fn reference(prefix: &'static str) -> impl IntoElement {
    let row = |label: &'static str, content: AnyElement, padded: bool| {
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
            .child(div().w(rems(260. / 16.)).flex().child(if padded {
                cell(content).into_any_element()
            } else {
                // `h('span', {style: {background: 'var(--layer-01)'}}, FileResultRow)`.
                div()
                    .flex_1()
                    .min_w_0()
                    .bg(color(ColorRole::Layer01))
                    .child(content)
                    .into_any_element()
            }))
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
            true,
        ))
        .child(row(
            "Branch · middle",
            Truncate::new((prefix, 1usize), BRANCH)
                .mode(TruncateMode::Middle)
                .into_any_element(),
            true,
        ))
        .child(row(
            "Title · end",
            Truncate::new((prefix, 2usize), TITLE).into_any_element(),
            true,
        ))
        .child(row(
            "Palette ⌘P",
            FileResultRow::new((prefix, 3usize), PATH, "sbtr").into_any_element(),
            false,
        ))
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
