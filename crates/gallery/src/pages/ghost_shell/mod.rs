//! Ghost · Shell pages (`src/ghost/workspace.jsx` "Shell" in the design
//! reference). The layout helpers are the T3 · Shell ones.
pub mod right_panel_tabs_overflow;
pub mod server;

use gpui_kit::{IntoElement, ParentElement, SharedString, Styled, div};
use u0ui::prelude::*;

/// "Last action: …" under a playground.
pub(crate) fn last_action(text: &SharedString) -> impl IntoElement {
    div()
        .type_style(TypeStyle::Label01)
        .text_color(color(ColorRole::TextSecondary))
        .child(format!("Last action: {text}"))
}

/// The machine kinds a Ghost server can be (`kind`: laptop / desktop /
/// server).
pub(crate) const KINDS: [u0ui::t3::shell::MachineKind; 3] = [
    u0ui::t3::shell::MachineKind::Laptop,
    u0ui::t3::shell::MachineKind::Desktop,
    u0ui::t3::shell::MachineKind::Server,
];

/// The [`KINDS`] names, for a choices control.
pub(crate) const KIND_NAMES: [&str; 3] = ["laptop", "desktop", "server"];
