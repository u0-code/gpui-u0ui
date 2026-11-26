//! Gallery pages. Each module exposes `pub fn build(&mut Window, &mut App) -> AnyView`
//! and is registered with one line in `registry.rs`.
pub mod alert_dialog;
pub mod button;
pub mod button_group;
pub mod checkbox;
pub mod colors;
pub mod context_menu;
pub mod dialog;
pub mod drawer;
pub mod dropdown_menu;
pub mod hover_card;
pub mod icons;
pub mod input;
pub mod label;
pub mod layout;
pub mod menubar;
pub mod motion;
pub mod popover;
pub mod radio_group;
pub mod sheet;
pub mod switch;
pub mod toggle;
pub mod toggle_group;
pub mod tooltip;
pub mod typography;
