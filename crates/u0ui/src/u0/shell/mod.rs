//! `u0 · Shell` (`src/ghost/workspace.jsx`, "Shell"): the sidebar pieces
//! u0 needs on top of the T3 shell. One submodule per card (or per tightly
//! coupled pair); everything is re-exported here, so callers write
//! `u0ui::u0::shell::ServerChip`.
//!
//! Values come from `src/ghost.css`; the identity glyphs, status dots and the
//! spinner are the T3 ones ([`crate::t3::shell`], [`crate::t3::base`]).
mod filter_menu;
mod server;
mod thread_row;

pub use filter_menu::{
    FilterItem, FilterMenu, FilterMenuTexts, filter_items, option_focus, toggle_value,
    toggles_checkbox, trigger_ink,
};
pub use server::{
    ServerChip, ServerChipLook, ServerChipSize, ServerHeader, ServerState, reason_rule,
};
pub use thread_row::{RowFlags, RowServer, ThreadRow, ThreadRowTexts, ThreadStatus, row_look};
