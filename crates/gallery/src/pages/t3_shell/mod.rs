//! T3 · Shell and T3 · Identity & status pages (`src/t3/shell.jsx`).
pub mod identity;

use gpui_kit::{ElementId, IntoElement, ParentElement, SharedString, Styled, div};
use u0ui::prelude::*;

/// `.cu-row`: a wrapping row, 16px gaps, centred.
pub(crate) fn row() -> gpui_kit::Div {
    div()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap(tokens::SPACING_05)
}

/// `.cu-stack`: a column.
pub(crate) fn stack() -> gpui_kit::Div {
    div().flex().flex_col()
}

/// Demo text in `text-primary`.
pub(crate) fn label_text(text: impl Into<SharedString>, style: TypeStyle) -> impl IntoElement {
    div()
        .type_style(style)
        .text_color(color(ColorRole::TextPrimary))
        .child(text.into())
}

/// An element id `"<prefix>-<name>"`.
pub(crate) fn eid(prefix: &str, name: &str) -> ElementId {
    ElementId::Name(format!("{prefix}-{name}").into())
}
