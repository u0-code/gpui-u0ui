//! Building blocks of the Carbon pattern examples (`Carbon · Patterns`
//! cards, `src/patterns.css` in the design reference). The pattern
//! compositions themselves live in the gallery; only the reusable parts are
//! here.
//!
//! - [`actionable_toast`]: [`ActionableToast`], the toast with one tertiary
//!   action (`.cds--actionable-notification--toast`).
//! - [`badge`]: count and dot badges on icon buttons (`.cds--badge-indicator`, `.cu-pdot`).
//! - [`caption`]: the examples' caption line (`.cu-pcap`).
//! - [`combo_button`]: [`ComboButton`] (`.cds--combo-button`), a primary
//!   action with a menu of related actions.
//! - [`empty_state`]: [`PatternEmptyState`] (`.cu-pempty`), the empty
//!   state in its small- and large-space layouts.
//! - [`fluid`]: [`FluidTextInput`] (text or password) and [`FluidDropdown`],
//!   the label-inside 4rem fields of `FluidForm`.
//! - [`frame`]: [`PatternFrame`], the static modal anatomy (`Frame`,
//!   `.cu-pframe`) the dialog and form examples compare side by side.
//! - [`grid`]: [`PatternGrid`] (`.cu-pgrid`), the examples' grid.
//! - [`indicator`]: icon, shape and difference status indicators
//!   (`.cds--icon-indicator`, `.cds--shape-indicator`, `.cu-pdiff`).
//! - [`menu`]: [`PatternMenu`] (`.cu-pmenu`), a menu drawn open in place.
//! - [`number_input`]: [`NumberInput`] (`.cds--number`), the full-width
//!   number field with its − / + buttons at the end.
//! - [`pictogram`]: [`Pictogram`] (`Pic`, `.cu-ppic`), an icon drawn large
//!   on an accent square, the empty-state illustration.
//! - [`popover_panel`]: [`PatternPopover`] (`.cu-ppop`) with its
//!   [`PatternRule`] (`.cu-phr`) and [`PatternLink`] rows (`.cu-plink`).
//! - [`progress_indicator`]: [`ProgressIndicator`] (`.cds--progress`), the
//!   wizard step indicator.
//! - [`row`]: [`PatternRow`] (`.cu-prow`), a list row with trailing actions.
//! - [`search`]: [`PatternSearch`] (`.cds--search`), the search field.
//! - [`table_skeleton`]: [`DataTableSkeleton`] (`.cds--data-table.cds--skeleton`).
//! - [`text_toolbar`]: the rich-text formatting toolbar (`.cu-ptbar`).
pub mod actionable_toast;
pub mod badge;
pub mod caption;
pub mod combo_button;
pub mod empty_state;
pub mod fluid;
pub mod frame;
pub mod grid;
pub mod indicator;
pub mod menu;
pub mod number_input;
pub mod pictogram;
pub mod popover_panel;
pub mod progress_indicator;
pub mod row;
pub mod search;
pub mod table_skeleton;
pub mod text_toolbar;

pub use actionable_toast::ActionableToast;
pub use badge::{BadgeIndicator, Badged};
pub use caption::Caption;
pub use combo_button::{ComboButton, ComboButtonSize};
pub use empty_state::{EmptyStateGeometry, EmptyStateSize, PatternEmptyState};
pub use fluid::{FluidDropdown, FluidGeometry, FluidLook, FluidTextInput};
pub use frame::{FrameButton, FrameGeometry, PatternFrame};
pub use grid::PatternGrid;
pub use indicator::{
    DiffIndicator, IconIndicator, IconIndicatorKind, ShapeIndicator, ShapeIndicatorKind,
};
pub use menu::{PatternMenu, PatternMenuItem};
pub use number_input::NumberInput;
pub use pictogram::Pictogram;
pub use popover_panel::{PatternLink, PatternPopover, PatternRule};
pub use progress_indicator::{ProgressIndicator, ProgressStep, StepStatus};
pub use row::PatternRow;
pub use search::PatternSearch;
pub use table_skeleton::DataTableSkeleton;
pub use text_toolbar::{TextToolbar, ToolbarAction, ToolbarItem};
