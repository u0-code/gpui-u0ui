//! Building blocks of the Carbon pattern examples (`Carbon · Patterns`
//! cards, `src/patterns.css` in the design reference). The pattern
//! compositions themselves live in the gallery; only the reusable parts are
//! here.
//!
//! - [`caption`]: the examples' caption line (`.cu-pcap`).
//! - [`frame`]: [`PatternFrame`], the static modal anatomy (`Frame`,
//!   `.cu-pframe`) the dialog and form examples compare side by side.
//! - [`grid`]: [`PatternGrid`] (`.cu-pgrid`), the examples' grid.
//! - [`pictogram`]: [`Pictogram`] (`Pic`, `.cu-ppic`), an icon drawn large
//!   on an accent square, the empty-state illustration.
//! - [`progress_indicator`]: [`ProgressIndicator`] (`.cds--progress`), the
//!   wizard step indicator.
//! - [`row`]: [`PatternRow`] (`.cu-prow`), a list row with trailing actions.
//! - [`search`]: [`PatternSearch`] (`.cds--search`), the search field.
//! - [`text_toolbar`]: the rich-text formatting toolbar (`.cu-ptbar`).
pub mod caption;
pub mod frame;
pub mod grid;
pub mod pictogram;
pub mod progress_indicator;
pub mod row;
pub mod search;
pub mod text_toolbar;

pub use caption::Caption;
pub use frame::{FrameButton, FrameGeometry, PatternFrame};
pub use grid::PatternGrid;
pub use pictogram::Pictogram;
pub use progress_indicator::{ProgressIndicator, ProgressStep, StepStatus};
pub use row::PatternRow;
pub use search::PatternSearch;
pub use text_toolbar::{TextToolbar, ToolbarAction, ToolbarItem};
