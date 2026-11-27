//! u0 · Settings › ThemeEditor: the reference "Midnight (custom)" theme,
//! live (pick colors, import a VS Code theme), with the 4.5:1 contrast check
//! the editor asks for before saving.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, rems,
};
use u0ui::prelude::*;
use u0ui::u0::settings::{
    MIN_CONTRAST, ThemeEditor, ThemeToken, contrast_ratio, import_vscode, to_hex,
};

/// `components/ThemeEditor/preview.html`: the "Midnight (custom)" tokens.
pub const MIDNIGHT: [(&str, &str); 8] = [
    ("background", "#0b0d12"),
    ("layer-01", "#161a22"),
    ("text-primary", "#e6e9ef"),
    ("text-secondary", "#9aa3b2"),
    ("interactive", "#5b8cff"),
    ("support-error", "#ff6b6b"),
    ("support-success", "#3ecf8e"),
    ("focus", "#ffffff"),
];

/// A VS Code theme's `colors` (a Dark+ excerpt) for the live import.
const DARK_PLUS: [(&str, &str); 7] = [
    ("editor.background", "#1e1e1e"),
    ("editor.foreground", "#d4d4d4"),
    ("sideBar.background", "#252526"),
    ("button.background", "#0e639c"),
    ("focusBorder", "#007fd4"),
    ("terminal.ansiRed", "#cd3131"),
    ("activityBar.background", "#333333"),
];

/// The token roles the "current Carbon theme" preset reads.
const CARBON: [(&str, ColorRole); 8] = [
    ("background", ColorRole::Background),
    ("layer-01", ColorRole::Layer01),
    ("text-primary", ColorRole::TextPrimary),
    ("text-secondary", ColorRole::TextSecondary),
    ("interactive", ColorRole::Interactive),
    ("support-error", ColorRole::SupportError),
    ("support-success", ColorRole::SupportSuccess),
    ("focus", ColorRole::Focus),
];

const PRESETS: [&str; 2] = ["Midnight (custom)", "Current Carbon theme"];

/// The reference tokens.
pub fn midnight() -> Vec<ThemeToken> {
    MIDNIGHT
        .iter()
        .filter_map(|(n, v)| ThemeToken::hex(*n, v))
        .collect()
}

fn carbon() -> Vec<ThemeToken> {
    CARBON
        .iter()
        .map(|(n, role)| ThemeToken::new(*n, color(*role)))
        .collect()
}

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| ThemeEditorPage {
        preset: 0,
        tokens: midnight(),
        generation: 0,
        imported: false,
        exported: 0,
    })
    .into()
}

struct ThemeEditorPage {
    preset: usize,
    tokens: Vec<ThemeToken>,
    /// Bumped when the tokens are replaced, so the editor makes fresh pickers.
    generation: usize,
    imported: bool,
    exported: usize,
}

/// `components/ThemeEditor/preview.html`.
pub fn reference(id: &'static str) -> impl IntoElement {
    ThemeEditor::new(id, midnight()).name("Midnight (custom)")
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(
            ThemeEditor::new("thumb-theme-editor", midnight().into_iter().take(2)).name("Midnight"),
        )
        .into_any_element()
}

impl Render for ThemeEditorPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let id = ("theme-live", self.generation);
        let name = if self.imported {
            "Dark+ (imported)"
        } else {
            PRESETS[self.preset]
        };
        let import = entity.clone();
        let export = entity.clone();
        let editor = ThemeEditor::new(id, self.tokens.clone())
            .name(name)
            .on_import(move |_, cx| {
                import.update(cx, |this, cx| {
                    this.tokens = import_vscode(&this.tokens, DARK_PLUS);
                    this.imported = true;
                    this.generation += 1;
                    cx.notify();
                })
            })
            .on_export(move |_, cx| {
                export.update(cx, |this, cx| {
                    this.exported += 1;
                    cx.notify();
                })
            });

        // The save check: `text-primary` on `background`, as edited.
        let values = ThemeEditor::values(id, window, cx);
        let pick = |name: &str| {
            self.tokens
                .iter()
                .position(|t| t.name == name)
                .and_then(|i| values.get(i).copied().or(Some(self.tokens[i].value)))
        };
        let status = match (pick("text-primary"), pick("background")) {
            (Some(fg), Some(bg)) => {
                let ratio = contrast_ratio(fg, bg);
                format!(
                    "{} on {}: {ratio:.2}:1, {}",
                    to_hex(fg),
                    to_hex(bg),
                    if ratio >= MIN_CONTRAST {
                        "can be saved"
                    } else {
                        "below 4.5:1, cannot be saved"
                    }
                )
            }
            _ => "No text-primary / background pair".to_string(),
        };
        let live = div()
            .w(rems(40.))
            .max_w_full()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(editor)
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(status)
                    .child(format!(" · Export pressed {} times", self.exported)),
            );

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::field(
                "tokens",
                controls::choices(
                    "theme-preset",
                    &PRESETS,
                    self.preset,
                    controls::setter(&entity, |this: &mut Self, i| {
                        this.preset = i;
                        this.tokens = if i == 0 { midnight() } else { carbon() };
                        this.imported = false;
                        this.generation += 1;
                    }),
                ),
            ));
        Body::new()
            .playground(live)
            .hint("Click a swatch to pick a color; Import VS Code theme maps a Dark+ excerpt")
            .config(config)
            .example(controls::block(
                "Reference preview",
                reference("theme-editor-ref"),
            ))
    }
}
