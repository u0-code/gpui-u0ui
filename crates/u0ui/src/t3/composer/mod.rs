//! T3 · Composer (`src/t3/composer.jsx`; `t3.css` "Context chips" through
//! "Queue", plus the craft-pass overrides at the end of the file, which win).
mod chip;
mod controls;
mod menu;

pub use chip::{
    ChipGeometry, ChipLook, ContextChip, ContextChipKind, ContextChipShape, ContextChipState,
};
pub use controls::{
    ComposerControl, EffortGlyph, EffortLevel, ReadOnlyComposerBar, composer_control_colors,
    effort_bar_color, effort_bar_height,
};
pub use menu::{COMMAND_MENU_SIZE, CommandMenuItem, CommandMenuSection, ComposerCommandMenu};
