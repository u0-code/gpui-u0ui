//! Carbon components. One module per component; see `docs/ARCHITECTURE.md`
//! for the recipe ([`button`] is the reference implementation).
pub mod button;

pub use button::{Button, ButtonKind, ButtonSize, ButtonVariant};
