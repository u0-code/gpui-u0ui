//! Carbon UI for GPUI: IBM Carbon v11 visuals, shadcn/ui component API,
//! Material Symbols Sharp icons. Kit owns behavior; this crate owns appearance.
#![warn(missing_docs)]

pub mod motion;
pub mod theme;
pub mod tokens;

pub use theme::LayerLevel;
pub use tokens::{CarbonTheme, ColorRole, TypeStyle};

/// Initialize u0ui after `gpui_kit::init`: installs the theme and Kit bridge.
pub fn init(cx: &mut gpui_kit::App) {
    theme::init(cx);
}
