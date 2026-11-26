//! Carbon UI for GPUI: IBM Carbon v11 visuals, shadcn/ui component API,
//! Material Symbols Sharp icons. Kit owns behavior; this crate owns appearance.
#![warn(missing_docs)]

pub mod icons;
pub mod motion;
pub mod theme;
pub mod tokens;
pub mod typography;

pub use icons::{Assets, CarbonIcon, Icon, IconSize};
pub use theme::LayerLevel;
pub use tokens::{CarbonTheme, ColorRole, TypeStyle};
pub use typography::TypeStyled;

/// Initialize u0ui after `gpui_kit::init`: registers IBM Plex, installs the
/// theme and Kit bridge, maps Kit icons to Material Symbols Sharp.
pub fn init(cx: &mut gpui_kit::App) {
    typography::register_fonts(cx);
    theme::init(cx);
    icons::init(cx);
}
