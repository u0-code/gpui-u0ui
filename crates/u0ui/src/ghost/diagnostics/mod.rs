//! `Ghost · Diagnostics` (`src/ghost/admin.jsx`, "Diagnostics"): the
//! process tree, log problems and the storage breakdown. Everything is
//! re-exported here, so callers write `u0ui::ghost::diagnostics::ProcessTree`.
mod log_row;
mod process_tree;
mod storage;

pub use log_row::{LogLevel, LogTroubleRow, LogTroubleRowTexts, count_badge};
pub use process_tree::{
    COLUMNS, Process, ProcessTree, ProcessTreeTexts, ToggleText, bar_look, branch_pids, format_mem,
    visible_rows,
};
pub use storage::{StorageBreakdown, StorageItem, StorageTexts, chart_color, format_bytes, shares};
