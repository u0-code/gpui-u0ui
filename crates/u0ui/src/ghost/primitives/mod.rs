//! `Ghost · Primitives` (`src/ghost/primitives.jsx`): desktop-density building
//! blocks Carbon UI lacked. One submodule per card; everything is re-exported
//! here, so callers write `u0ui::ghost::primitives::HighlightedText`.
pub mod color_picker;
pub mod copy_field;
pub mod editable_text;
mod highlighted_text;
pub mod key_recorder;
pub mod parts;
pub mod secret_field;
pub mod stepped_slider;
pub mod stepper;
pub mod time_field;

pub use color_picker::ColorPicker;
pub use copy_field::CopyField;
pub use editable_text::EditableText;
pub use highlighted_text::{HighlightedText, fuzzy_match, highlight_runs, highlight_style};
pub use key_recorder::KeyRecorder;
pub use parts::IconButton;
pub use secret_field::{SecretField, SecretSource};
pub use stepped_slider::SteppedSlider;
pub use stepper::Stepper;
pub use time_field::TimeField;
