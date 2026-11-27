//! `Ghost · Tasks & workflows` (`src/ghost/admin.jsx`, "Scheduled tasks &
//! workflows"): scheduled-task and workflow rows and their editors. One
//! submodule per card; everything is re-exported here, so callers write
//! `u0ui::ghost::tasks::WorkflowRow`.
mod row;
mod workflow_row;

pub use workflow_row::{WorkflowRow, WorkflowRowTexts};
