//! Gallery pages. Each module exposes `pub fn build(&mut Window, &mut App) -> AnyView`
//! and is registered with one line in `registry.rs`.
pub mod alert_dialog;
pub mod button;
pub mod button_group;
pub mod colors;
pub mod dialog;
pub mod drawer;
pub mod icons;
pub mod input;
pub mod label;
pub mod layout;
pub mod motion;
pub mod popover;
pub mod sheet;
pub mod toggle;
pub mod toggle_group;
pub mod tooltip;
pub mod typography;
