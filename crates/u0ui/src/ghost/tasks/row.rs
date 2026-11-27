//! The list-row frame the task and workflow rows share (`src/ghost.css`
//! "admin rows": `.g-task, .g-wfrow, …`, `.g-task-main`, `.g-task-meta`).
use crate::{
    density,
    icons::{CarbonIcon, Icon},
    t3::craft::{rem, sans},
    theme::color,
    tokens::{self, ColorRole},
};
use gpui_kit::{AnyElement, Div, IntoElement, ParentElement, SharedString, Styled, div, px};

/// `.g-task, .g-wfrow { display: flex; align-items: center; gap:
/// var(--spacing-04); min-height: var(--density-row-lg); padding:
/// var(--spacing-02) var(--spacing-03) var(--spacing-02) var(--spacing-05);
/// background: var(--layer-01); border-bottom: 1px solid
/// var(--border-subtle-01); font-size: 0.875rem; color: var(--text-primary) }`.
pub(crate) fn frame() -> Div {
    sans(div(), 14., 400)
        .relative()
        .flex()
        .items_center()
        .w_full()
        .gap(tokens::SPACING_04)
        .min_h(density::current().row_lg())
        .pt(tokens::SPACING_02)
        .pb(tokens::SPACING_02)
        .pr(tokens::SPACING_03)
        .pl(tokens::SPACING_05)
        .bg(color(ColorRole::Layer01))
        .border_b_1()
        .border_color(color(ColorRole::BorderSubtle01))
        .text_color(color(ColorRole::TextPrimary))
}

/// The leading 16px glyph (`.g-dim-icon { fill: var(--icon-secondary);
/// flex: none }`).
pub(super) fn lead_icon(name: CarbonIcon) -> Div {
    div().flex().flex_none().child(
        Icon::carbon(name)
            .size_length(rem(16.))
            .color(color(ColorRole::IconSecondary)),
    )
}

/// The name and meta column: `.g-task-main { display: flex; flex: 1;
/// min-width: 0; flex-direction: column; gap: 2px }`; its `b` is 400 weight,
/// one line with an ellipsis, in `name_ink`.
pub(super) fn main(
    name: SharedString,
    name_ink: gpui_kit::Hsla,
    meta: impl IntoIterator<Item = AnyElement>,
) -> Div {
    div()
        .flex()
        .flex_col()
        .flex_1()
        .min_w_0()
        .gap(px(2.))
        .child(
            div()
                .min_w_0()
                .overflow_hidden()
                .whitespace_nowrap()
                .text_ellipsis()
                .text_color(name_ink)
                .child(name),
        )
        .child(
            // `.g-task-meta { display: flex; flex-wrap: wrap; align-items:
            // center; gap: var(--spacing-02) var(--spacing-03); font-size:
            // 0.75rem; color: var(--text-secondary) }`.
            sans(div(), 12., 400)
                .flex()
                .flex_wrap()
                .items_center()
                .gap_x(tokens::SPACING_03)
                .gap_y(tokens::SPACING_02)
                .text_color(color(ColorRole::TextSecondary))
                .children(meta),
        )
}

/// `.t3-metadot { color: var(--text-helper); padding: 0 2px }`.
pub(super) fn metadot() -> AnyElement {
    div()
        .px(px(2.))
        .text_color(color(ColorRole::TextHelper))
        .child("·")
        .into_any_element()
}

/// A plain meta run.
pub(super) fn meta_text(text: impl Into<SharedString>) -> AnyElement {
    div()
        .whitespace_nowrap()
        .child(text.into())
        .into_any_element()
}

/// Runs joined by `.t3-metadot` separators.
pub(super) fn with_dots(runs: impl IntoIterator<Item = AnyElement>) -> Vec<AnyElement> {
    let mut out = Vec::new();
    for (i, run) in runs.into_iter().enumerate() {
        if i > 0 {
            out.push(metadot());
        }
        out.push(run);
    }
    out
}
