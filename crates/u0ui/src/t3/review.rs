//! T3 · Thread chrome, git, diffs, pull requests and issues
//! (`src/t3/review.jsx`). Split by card group; everything is re-exported here.
pub mod diff;

pub use crate::t3::craft::{Tone, actor_tone};

pub use diff::{
    DiffFileHeader, DiffFileHeaderTexts, DiffFileStatus, DiffFileTree, DiffHunk, DiffLine,
    DiffStat, DiffView, FileStatus, LineKind, Segment, StartTruncatedPath, StatBlock, TreeFile,
};
pub mod review_thread;
pub use review_thread::{ReviewComment, ReviewThreadCard, ReviewThreadTexts};
pub mod pull_request;
pub use pull_request::{
    Check, ChecksGlyph, ChecksList, ChecksStatus, DetailHeaderTexts, GuideRail, GuideStop, Label,
    LabelChip, PullRequestDetailHeader, PullRequestRow, PullRequestStack, PullRequestState,
    PullRequestStateGlyph, ReviewDecision, ReviewDecisionBadge, StackLayer, StopState,
};
pub mod discussion;
pub use discussion::{
    EventKind, PullRequestTimeline, Reaction, ReactionBar, ReviewForm, ReviewFormTexts,
    ReviewVerdict, TimelineEvent,
};
pub mod issue;
pub use issue::{IssueRow, IssueState, IssueStateGlyph, Priority};
