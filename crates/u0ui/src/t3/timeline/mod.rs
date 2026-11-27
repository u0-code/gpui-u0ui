//! T3 · Timeline (`src/t3/timeline.jsx`; `t3.css` "Messages" through
//! "Thread navigation", plus the craft-pass and r-round overrides at the end
//! of the file, which win).
//!
//! | Card | Types |
//! | --- | --- |
//! | ApprovalCard | [`ApprovalCard`] |
//! | AssistantMessage | [`AssistantMessage`], [`Prose`] |
//! | CommandOutputBlock | [`CommandOutputBlock`] |
//! | ProposedPlanCard | [`ProposedPlanCard`] |
//! | RequestCard | [`RequestCard`] |
//! | SecretRequestCard | [`SecretRequestCard`] |
//! | SubagentRow | [`SubagentRow`], [`SubagentStatusDot`], [`SubagentBar`] |
//! | ThreadErrorBanner | [`ThreadErrorBanner`] |
//! | ThreadNavigation | [`ThreadFindBar`], [`TimelineMinimap`], [`ScrollToEndButton`] |
//! | TimelineDivider | [`TimelineDivider`] |
//! | UserInputCard | [`UserInputCard`] |
//! | UserMessage | [`UserMessage`] |
//! | WorkingIndicator | [`WorkingIndicator`] |
//! | WorkLog | [`WorkLog`], [`ToolActivityIcon`] |
//! | WorktreeSetupCard | [`WorktreeSetupCard`] |
mod approval;
pub(crate) mod card;
mod messages;
mod navigation;
mod requests;
mod thread;
mod work;

pub use approval::{
    ApprovalCard, ApprovalKind, ApprovalState, ApprovalTexts, approval_look, approval_shortcut,
};
pub use messages::{
    AssistantMessage, AssistantMessageTexts, FOLDED_HEIGHT, Prose, ProseSpan, UserMessage,
    UserMessageTexts, inline_code_fill,
};
pub use navigation::{
    FIND_BAR_SIZE, FindTexts, MINIMAP_SIZE, MinimapKind, MinimapMark, ScrollToEndButton,
    ThreadFindBar, TimelineMinimap, scroll_to_end_label,
};
pub use requests::{
    Answer, InputKey, InputOption, InputProgress, InputQuestion, PlanStep, PlanTexts,
    ProposedPlanCard, RequestCard, SecretField, SecretRequestCard, SecretTexts, UserInputCard,
    UserInputTexts, input_key, option_colors,
};
pub use thread::{DividerKind, ThreadBannerKind, ThreadErrorBanner, TimelineDivider};
pub use work::{
    CHEVRON_MOTION, SHIMMER_PERIOD, ToolActivityIcon, ToolKind, ToolStatus, WorkEntry, WorkLog,
    WorkLogTexts, WorkingIndicator, WorkingState, shimmer_intensity,
};
