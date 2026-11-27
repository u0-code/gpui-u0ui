//! `Ghost · Thread` (`src/ghost/workspace.jsx`, "Thread"; `src/ghost.css`
//! plus the later `g-*` overrides in `src/t3.css`, which win): the pieces of
//! an agent transcript. One submodule per card; everything is re-exported
//! here, so callers write `u0ui::ghost::thread::TurnFooter`.
mod items;
mod markers;
pub(crate) mod parts;
pub mod thinking_block;
pub mod tool_call_fold;
mod transcript_view_switch;
mod turn_footer;

pub use items::{
    ContinueRow, ContinueRowTexts, ContinueStatus, MessageImage, MessageImageTexts, PlanChecklist,
    PlanChecklistTexts, PlanItem, PlanStatus, accent_fill, plan_progress,
};
pub use markers::{
    CheckpointMarker, CheckpointMarkerTexts, NoteKind, NoteLine, NoteLineTexts, restore_fill,
    restore_visible,
};
pub use thinking_block::{ThinkingBlock, ThinkingBlockTexts, shimmer_mix};
pub use tool_call_fold::{ToolCallFold, ToolCallFoldTexts, ToolCallLook, ToolKind, ToolStatus};
pub use transcript_view_switch::{
    TranscriptViewSwitch, TranscriptViewSwitchTexts, default_views, is_cycle_shortcut, next_view,
    segment_look,
};
pub use turn_footer::{REST_OPACITY, TurnFooter, TurnFooterTexts, turn_footer_opacity};
