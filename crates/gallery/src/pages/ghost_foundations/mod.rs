//! Ghost · Foundations pages (`src/ghost/foundations.jsx` and the Ghost
//! desktop rules in the design reference README).
pub mod agent_colors;
pub mod cjk_typography;
pub mod density;
pub mod focus_states;
pub mod icon;
pub mod layout_rules;
pub mod reduced_motion;
pub mod scrim;
pub mod syntax_colors;
pub mod terminal_colors;
pub mod truncation;
pub mod window_chrome;

use gpui_kit::{IntoElement, ParentElement, SharedString, Styled, div};
use u0ui::prelude::*;

/// A `cds--label` caption above a specimen.
pub fn caption(text: impl Into<SharedString>) -> impl IntoElement {
    div()
        .type_style(TypeStyle::Label01)
        .text_color(color(ColorRole::TextSecondary))
        .mb(tokens::SPACING_03)
        .child(text.into())
}
