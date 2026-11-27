//! T3 · Timeline (`src/t3/timeline.jsx`; `t3.css` "Messages" through
//! "Thread navigation", plus the craft-pass and r-round overrides at the end
//! of the file, which win).
mod approval;
pub(crate) mod card;

pub use approval::{
    ApprovalCard, ApprovalKind, ApprovalState, ApprovalTexts, approval_look, approval_shortcut,
};
