//! T3 · Files & preview, T3 · Settings, T3 · Usage & workflows and T3 ·
//! Onboarding & feedback pages (`src/t3/workspace.jsx`).
pub mod files;
pub mod settings;

use gpui_kit::{ParentElement, SharedString, Styled, div, px};
use u0ui::prelude::*;

/// `.cu-row`: a wrapping row, 16px gaps.
pub(crate) fn row() -> gpui_kit::Div {
    div()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_start()
        .gap(tokens::SPACING_05)
}

/// `.cu-stack` with a `gap` px.
pub(crate) fn stack(gap: f32) -> gpui_kit::Div {
    div().flex().flex_col().gap(px(gap))
}

/// Demo text in `text-primary`.
pub(crate) fn text(s: impl Into<SharedString>) -> gpui_kit::Div {
    div()
        .type_style(TypeStyle::BodyCompact01)
        .text_color(color(ColorRole::TextPrimary))
        .child(s.into())
}

/// A status line under a live demo ("Selected: …").
pub(crate) fn note(s: impl Into<SharedString>) -> gpui_kit::Div {
    div()
        .type_style(TypeStyle::Label01)
        .text_color(color(ColorRole::TextSecondary))
        .child(s.into())
}
