//! Building blocks of the Carbon pattern examples (`Carbon · Patterns`
//! cards, `src/patterns.css` in the design reference). The pattern
//! compositions themselves live in the gallery; only the reusable parts are
//! here.
//!
//! - [`caption`]: the examples' caption line (`.cu-pcap`).
//! - [`text_toolbar`]: the rich-text formatting toolbar (`.cu-ptbar`).
pub mod caption;
pub mod text_toolbar;

pub use caption::Caption;
pub use text_toolbar::{TextToolbar, ToolbarAction, ToolbarItem};
