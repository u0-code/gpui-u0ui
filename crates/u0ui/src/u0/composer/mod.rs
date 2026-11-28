//! `u0 · Composer` (`src/ghost/workspace.jsx`, "Composer additions";
//! `.g-cchip*`, `.g-ccompact*`, `.g-modes`, `.g-mentions`, `.g-pop`,
//! `.g-opt*` in `src/ghost.css`): what u0 adds around the T3 composer.
//!
//! - [`ComposerChip`]: context attached to the input (a quote, terminal
//!   output, a review comment, a pull request or issue).
//! - [`ComposerCompact`]: the one-line side-chat input.
//! - [`ModeMenu`] / [`ModeList`]: the agent mode picker, as a toolbar menu
//!   button or as an expanded list with descriptions.
//! - [`MentionList`]: the `@` mention popup (files and sessions, grouped,
//!   fuzzy-highlighted).
mod chip;
mod compact;
mod lists;

pub use chip::{ChipKind, ChipState, ComposerChip, ComposerChipTexts, chip_state_dot};
pub use compact::{ComposerCompact, ComposerCompactTexts, send_colors, stacked};
pub use lists::{
    MentionGroup, MentionItem, MentionKind, MentionList, MentionListTexts, Mode, ModeList,
    ModeMenu, option_look,
};
