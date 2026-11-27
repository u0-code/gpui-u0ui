//! T3 · Thread chrome, git, diffs, pull requests and issues
//! (`src/t3/review.jsx`). Split by card group; everything is re-exported here.
pub mod diff;

pub use crate::t3::craft::{AvatarTone, actor_tone};

pub use diff::{
    DiffFileHeader, DiffFileHeaderTexts, DiffFileStatus, DiffFileTree, DiffHunk, DiffLine,
    DiffStat, DiffView, FileStatus, LineKind, Segment, StartTruncatedPath, StatBlock, TreeFile,
};
pub mod review_thread;
pub use review_thread::{ReviewComment, ReviewThreadCard, ReviewThreadTexts};
pub mod pull_request;
pub use pull_request::{
    Check, ChecksGlyph, ChecksList, ChecksStatus, GuideRail, GuideStop, Label, LabelChip,
    PullRequestRow, PullRequestStack, PullRequestState, PullRequestStateGlyph, ReviewDecision,
    ReviewDecisionBadge, StackLayer, StopState,
};
pub mod discussion;
pub use discussion::{
    EventKind, PullRequestTimeline, Reaction, ReactionBar, ReviewForm, ReviewFormTexts,
    ReviewVerdict, TimelineEvent,
};
pub mod issue;
pub use issue::{IssueRow, IssueState, IssueStateGlyph, Priority};
pub mod pr_detail;
pub use pr_detail::{
    DetailHeaderTexts, MergeControls, MergeMethod, MergeTexts, PullRequestDetailHeader,
};
pub mod thread;
pub use thread::{
    BranchToolbar, BranchToolbarTexts, ChatHeader, ContextMeter, GitAction, GitActionsControl,
    GitQuiet, GitState, HeaderEnvironment, HeaderLayout, MeterTone, ThreadDetailsSection,
    ThreadHoverCard, WorkspaceMode, git_chip_colors,
};
