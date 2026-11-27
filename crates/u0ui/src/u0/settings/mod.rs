//! `u0 · Settings` (the settings half of `src/ghost/admin.jsx`): the
//! settings window and the rows and editors inside it. One submodule per
//! card; everything is re-exported here, so callers write
//! `u0ui::u0::settings::ThemeEditor`.
mod account_row;
mod agent_row;
mod consent_sheet;
mod dialog;
mod dictionary_editor;
mod read_aloud;
pub(crate) mod row;
mod scope_header;
mod script_row;
mod settings_list_row;
mod telemetry_preview;
mod theme_editor;

pub use account_row::{AccountRow, AccountRowTexts, TokenState};
pub use agent_row::{
    AgentButton, AgentRow, AgentRowTexts, AgentState, progress_bar, status_min_width,
};
pub use consent_sheet::{ConsentActions, ConsentSheet, ConsentTexts, sample_payload};
pub use dialog::{
    SettingsDialog, SettingsDialogTexts, SettingsNavItem, SettingsSection, filter_sections,
    settings_heading,
};
pub use dictionary_editor::{
    DictionaryEditor, DictionaryTexts, WordsState, add_word, locale_cmp, remove_word,
};
pub use read_aloud::{ReadAloudControl, ReadAloudTexts, Wave, next_rate, rate_text, wave_scale};
pub use scope_header::{ScopeHeader, ScopeHeaderTexts};
pub use script_row::{RunSlot, ScriptRow, ScriptRowTexts, kind_label, shortcut_keys};
pub use settings_list_row::{ActionKind, SettingsAction, SettingsListRow};
pub use telemetry_preview::{
    Payload, Syn, TelemetryLevel, TelemetryPreview, TelemetryPreviewTexts, line_runs,
};
pub use theme_editor::{
    MIN_CONTRAST, ThemeEditor, ThemeEditorLayout, ThemeEditorTexts, ThemeToken, contrast_checks,
    contrast_ratio, grid_columns, import_vscode, on_color, parse_hex, ratio_text, to_hex,
    vscode_token,
};
