//! `Ghost · Thread` (`src/ghost/workspace.jsx`, "Thread"; `src/ghost.css`
//! plus the later `g-*` overrides in `src/t3.css`, which win): the pieces of
//! an agent transcript. One submodule per card; everything is re-exported
//! here, so callers write `u0ui::ghost::thread::TurnFooter`.
mod transcript_view_switch;
mod turn_footer;

pub use transcript_view_switch::{
    TranscriptViewSwitch, TranscriptViewSwitchTexts, default_views, is_cycle_shortcut, next_view,
    segment_look,
};
pub use turn_footer::{REST_OPACITY, TurnFooter, TurnFooterTexts, turn_footer_opacity};
