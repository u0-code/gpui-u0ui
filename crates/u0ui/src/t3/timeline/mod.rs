//! T3 · Timeline (`src/t3/timeline.jsx`; `t3.css` "Messages" through
//! "Thread navigation", plus the craft-pass and r-round overrides at the end
//! of the file, which win).
//!
//! | Card | Types |
//! | --- | --- |
//! | ApprovalCard | [`ApprovalCard`] |
//! | ProposedPlanCard | [`ProposedPlanCard`] |
//! | RequestCard | [`RequestCard`] |
//! | SecretRequestCard | [`SecretRequestCard`] |
//! | ThreadErrorBanner | [`ThreadErrorBanner`] |
//! | ThreadNavigation | [`ThreadFindBar`], [`TimelineMinimap`], [`ScrollToEndButton`] |
//! | TimelineDivider | [`TimelineDivider`] |
//! | UserInputCard | [`UserInputCard`] |
mod approval;
pub(crate) mod card;
mod navigation;
mod requests;
mod thread;

pub use approval::{
    ApprovalCard, ApprovalKind, ApprovalState, ApprovalTexts, approval_look, approval_shortcut,
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
