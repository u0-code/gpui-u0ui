//! `u0 · Foundations` (`src/ghost/foundations.jsx`): the desktop rules
//! (density, CJK, window chrome, three selection states, scrims, syntax and
//! ANSI colors, agent slots, truncation, layout breakpoints, reduced motion)
//! as reusable layout pieces and token specimens.
//!
//! The tokens themselves live in [`crate::tokens`]; the rules that extend
//! the foundation modules are [`crate::density::zone`] (`<Density>`),
//! [`crate::density::palette_input`] and [`crate::theme::Selection`] (the
//! state-current / state-selected / state-focus looks).
pub mod motion_specimen;
pub mod rem_scale;
pub mod scrim;
pub mod shell_layout;
pub mod state_row;
pub mod swatches;
pub mod truncate;
pub mod window_chrome;

pub use motion_specimen::MotionSpecimen;
pub use rem_scale::{RemScale, rem_scale};
pub use scrim::{Scrim, ScrimAlign, ScrimVariant};
pub use shell_layout::{ShellLayout, ShellMode};
pub use state_row::StateRow;
pub use swatches::TokenSwatches;
pub use truncate::{Truncate, TruncateMode};
pub use window_chrome::WindowChrome;

use crate::tokens::ColorRole;

/// The eight agent accent slots (`agent-accent-1…8`). The app maps agents
/// to slots; no vendor color is shipped.
pub const AGENT_ACCENTS: [ColorRole; 8] = [
    ColorRole::AgentAccent1,
    ColorRole::AgentAccent2,
    ColorRole::AgentAccent3,
    ColorRole::AgentAccent4,
    ColorRole::AgentAccent5,
    ColorRole::AgentAccent6,
    ColorRole::AgentAccent7,
    ColorRole::AgentAccent8,
];

/// The accent for slot `n` (1–8, wrapping), for mapping agents to slots.
pub fn agent_accent(n: usize) -> ColorRole {
    AGENT_ACCENTS[(n.max(1) - 1) % AGENT_ACCENTS.len()]
}

/// The 17 syntax tokens, in the reference order (map Shiki / highlight.js /
/// Tree-sitter scopes onto these; comments are also italic, `invalid` also
/// wavy-underlined).
pub const SYNTAX: [ColorRole; 17] = [
    ColorRole::SyntaxKeyword,
    ColorRole::SyntaxString,
    ColorRole::SyntaxComment,
    ColorRole::SyntaxFunction,
    ColorRole::SyntaxNumber,
    ColorRole::SyntaxType,
    ColorRole::SyntaxVariable,
    ColorRole::SyntaxTag,
    ColorRole::SyntaxAttribute,
    ColorRole::SyntaxOperator,
    ColorRole::SyntaxPunctuation,
    ColorRole::SyntaxConstant,
    ColorRole::SyntaxRegexp,
    ColorRole::SyntaxProperty,
    ColorRole::SyntaxHeading,
    ColorRole::SyntaxLink,
    ColorRole::SyntaxInvalid,
];

/// The 16 ANSI colors: SGR 30–37 then 90–97.
pub const ANSI: [ColorRole; 16] = [
    ColorRole::AnsiBlack,
    ColorRole::AnsiRed,
    ColorRole::AnsiGreen,
    ColorRole::AnsiYellow,
    ColorRole::AnsiBlue,
    ColorRole::AnsiMagenta,
    ColorRole::AnsiCyan,
    ColorRole::AnsiWhite,
    ColorRole::AnsiBrightBlack,
    ColorRole::AnsiBrightRed,
    ColorRole::AnsiBrightGreen,
    ColorRole::AnsiBrightYellow,
    ColorRole::AnsiBrightBlue,
    ColorRole::AnsiBrightMagenta,
    ColorRole::AnsiBrightCyan,
    ColorRole::AnsiBrightWhite,
];

/// The ANSI color for an SGR foreground code (30–37, 90–97).
pub fn ansi_sgr(code: u8) -> Option<ColorRole> {
    match code {
        30..=37 => Some(ANSI[(code - 30) as usize]),
        90..=97 => Some(ANSI[(code - 90 + 8) as usize]),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slots_and_sgr_codes() {
        assert_eq!(agent_accent(1), ColorRole::AgentAccent1);
        assert_eq!(agent_accent(9), ColorRole::AgentAccent1);
        assert_eq!(ansi_sgr(31), Some(ColorRole::AnsiRed));
        assert_eq!(ansi_sgr(90), Some(ColorRole::AnsiBrightBlack));
        assert_eq!(ansi_sgr(38), None);
        assert_eq!(SYNTAX[16].name(), "syntax-invalid");
    }
}
