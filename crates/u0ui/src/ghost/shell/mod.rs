//! `Ghost · Shell` (`src/ghost/workspace.jsx`, "Shell"): the sidebar pieces
//! Ghost needs on top of the T3 shell. One submodule per card (or per tightly
//! coupled pair); everything is re-exported here, so callers write
//! `u0ui::ghost::shell::ServerChip`.
//!
//! Values come from `src/ghost.css`; the identity glyphs, status dots and the
//! spinner are the T3 ones ([`crate::t3::shell`], [`crate::t3::base`]).
mod server;

pub use server::{
    ServerChip, ServerChipLook, ServerChipSize, ServerHeader, ServerState, reason_rule,
};
