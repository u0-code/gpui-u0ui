//! `Ghost · Primitives` (`src/ghost/primitives.jsx`): desktop-density building
//! blocks Carbon UI lacked. One submodule per card; everything is re-exported
//! here, so callers write `u0ui::ghost::primitives::HighlightedText`.
pub mod color_picker;
pub mod copy_field;
pub mod editable_text;
mod highlighted_text;
pub mod key_recorder;
pub mod key_value_editor;
pub mod parts;
pub mod qr_code;
pub mod secret_field;
pub mod stepped_slider;
pub mod stepper;
pub mod tag_input;
pub mod time_field;
pub mod tree;
pub mod tree_table;

pub use color_picker::ColorPicker;
pub use copy_field::CopyField;
pub use editable_text::EditableText;
pub use highlighted_text::{HighlightedText, fuzzy_match, highlight_runs, highlight_style};
pub use key_recorder::KeyRecorder;
pub use key_value_editor::{EnvVar, KeyValueEditor};
pub use parts::IconButton;
pub use qr_code::{QrCode, QrLevel};
pub use secret_field::{SecretField, SecretSource};
pub use stepped_slider::SteppedSlider;
pub use stepper::Stepper;
pub use tag_input::TagInput;
pub use time_field::TimeField;
pub use tree::{Tree, TreeItem};
pub use tree_table::{CellAlign, TreeTable, TreeTableColumn, TreeTableRow};
