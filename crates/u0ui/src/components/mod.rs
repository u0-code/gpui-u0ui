//! Carbon components. One module per component; see `docs/ARCHITECTURE.md`
//! for the recipe ([`button`] is the reference implementation).
pub mod button;
pub mod form;
pub mod input;
pub mod label;

pub use button::{Button, ButtonKind, ButtonSize, ButtonVariant};
pub use form::{FieldSize, Validation};
pub use input::{Input, InputState};
pub use label::Label;
