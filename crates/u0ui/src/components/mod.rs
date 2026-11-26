//! Carbon components. One module per component; see `docs/ARCHITECTURE.md`
//! for the recipe ([`button`] is the reference implementation).
pub mod button;
pub mod dialog;
pub mod form;
pub mod input;
pub mod label;
pub mod overlay;

pub use button::{Button, ButtonKind, ButtonSize, ButtonVariant};
pub use dialog::{Dialog, DialogSize};
pub use form::{FieldSize, Validation};
pub use input::{Input, InputState};
pub use label::Label;
