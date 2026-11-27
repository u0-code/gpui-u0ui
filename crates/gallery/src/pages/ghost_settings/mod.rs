//! Ghost · Settings pages (the settings half of `src/ghost/admin.jsx` in the
//! design reference).
pub mod account_row;
pub mod agent_row;
pub mod consent_sheet;
pub mod dictionary_editor;
pub mod read_aloud_control;
pub mod scope_header;
pub mod script_row;
pub mod settings_dialog;
pub mod settings_list_row;
pub mod settings_row_overridden;
pub mod telemetry_preview;
pub mod theme_editor;

use gpui_kit::{Div, Styled, div, rems};
use u0ui::prelude::*;

/// A list of rows `width_px` wide on `layer-01`
/// (`style={{ width, background: 'var(--layer-01)' }}` in the previews).
pub fn list(width_px: f32) -> Div {
    div()
        .w(rems(width_px / 16.))
        .max_w_full()
        .flex()
        .flex_col()
        .bg(color(ColorRole::Layer01))
}

/// The "handler fired" line under a playground.
pub fn note(text: impl Into<gpui_kit::SharedString>) -> Div {
    use gpui_kit::ParentElement as _;
    div()
        .type_style(TypeStyle::Label01)
        .text_color(color(ColorRole::TextSecondary))
        .child(text.into())
}
