//! `Ghost · Primitives` (`src/ghost/primitives.jsx`): desktop-density building
//! blocks Carbon UI lacked. One submodule per card; everything is re-exported
//! here, so callers write `u0ui::ghost::primitives::HighlightedText`.
mod highlighted_text;

pub use highlighted_text::{HighlightedText, fuzzy_match, highlight_runs, highlight_style};
