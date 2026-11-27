//! `Ghost · Settings` (the settings half of `src/ghost/admin.jsx`): the
//! settings window and the rows and editors inside it. One submodule per
//! card; everything is re-exported here, so callers write
//! `u0ui::ghost::settings::ThemeEditor`.
pub(crate) mod row;
mod script_row;
mod settings_list_row;
mod theme_editor;

pub use script_row::{RunSlot, ScriptRow, ScriptRowTexts, kind_label, shortcut_keys};
pub use settings_list_row::{ActionKind, SettingsAction, SettingsListRow};
pub use theme_editor::{
    MIN_CONTRAST, ThemeEditor, ThemeEditorTexts, ThemeToken, contrast_ratio, grid_columns,
    import_vscode, parse_hex, to_hex, vscode_token,
};
