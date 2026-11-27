//! u0 · Settings › ThemeEditor: the reference "Midnight (custom)" theme,
//! live (pick colors, import a VS Code theme), with the 4.5:1 contrast check
//! the editor asks for before saving. The Customize layout is the body of
//! u0's "Customize Midnight" drawer (board `SetAppearance`), shown live and
//! inside a contained 600px `Sheet` with the board's footer.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, rems,
};
use u0ui::components::Sheet;
use u0ui::prelude::*;
use u0ui::u0::settings::{
    MIN_CONTRAST, ThemeEditor, ThemeEditorLayout, ThemeToken, contrast_ratio, import_vscode, to_hex,
};

/// The u0 board `SetAppearance` drawer's Midnight (`allTokens`).
const BOARD: [(&str, &str); 12] = [
    ("background", "#0b1220"),
    ("layer-01", "#121a2b"),
    ("layer-02", "#1a2438"),
    ("border-subtle-01", "#26324a"),
    ("text-primary", "#e6edf3"),
    ("text-secondary", "#9aa7b8"),
    ("link-primary", "#8ab8ff"),
    ("interactive", "#6ea8fe"),
    ("support-error", "#ff8389"),
    ("support-success", "#42be65"),
    ("support-warning", "#f1c21b"),
    ("support-info", "#78a9ff"),
];

/// The board's primary tokens: background, layer-01, text-primary,
/// text-secondary, interactive, support-error.
const BOARD_PRIMARY: [usize; 6] = [0, 1, 4, 5, 7, 8];

const LAYOUTS: [&str; 2] = ["grid", "customize"];

fn board_tokens() -> Vec<ThemeToken> {
    BOARD
        .iter()
        .filter_map(|(n, v)| ThemeToken::hex(*n, v))
        .collect()
}

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
        layout: 0,
        board: board_tokens(),
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
    layout: usize,
    /// The Customize layout's tokens, as edited.
    board: Vec<ThemeToken>,
}

impl ThemeEditorPage {
    /// The Customize editor over the board tokens, writing edits back.
    fn customize(&self, id: &'static str, entity: &gpui_kit::Entity<Self>) -> ThemeEditor {
        let entity = entity.clone();
        ThemeEditor::new(id, self.board.clone())
            .layout(ThemeEditorLayout::Customize)
            .primary(BOARD_PRIMARY)
            .on_change(move |i, value, _, cx| {
                entity.update(cx, |this, cx| {
                    this.board[i].value = value;
                    cx.notify();
                })
            })
    }
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
        if self.layout == 1 {
            let n = self.board.len();
            let live = div()
                .w(rems(34.5))
                .max_w_full()
                .flex()
                .flex_col()
                .gap(tokens::SPACING_05)
                .child(self.customize("theme-customize", &entity))
                .child(
                    div()
                        .type_style(TypeStyle::Label01)
                        .text_color(color(ColorRole::TextSecondary))
                        .child(format!(
                            "{n} tokens; pick in the plane or type a hex, Return commits"
                        )),
                );
            return self.body(live, &entity);
        }
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

        self.body(live, &entity)
    }
}

impl ThemeEditorPage {
    fn body(&self, live: impl IntoElement, entity: &gpui_kit::Entity<Self>) -> Body {
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::field(
                "layout",
                controls::choices(
                    "theme-layout",
                    &LAYOUTS,
                    self.layout,
                    controls::setter(entity, |this: &mut Self, i| this.layout = i),
                ),
            ))
            .child(controls::field(
                "tokens",
                controls::choices(
                    "theme-preset",
                    &PRESETS,
                    self.preset,
                    controls::setter(entity, |this: &mut Self, i| {
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
            .example(controls::block(
                "Customize drawer (u0 SetAppearance): a contained 600px Sheet",
                drawer(self.customize("theme-drawer", entity)),
            ))
    }
}

/// The board's drawer frame: a contained 600px `Sheet` over a sized parent,
/// its footer a row of Duplicate and the overflow, then Remove theme at the end.
fn drawer(editor: ThemeEditor) -> impl IntoElement {
    div()
        .relative()
        .w_full()
        .h(rems(48.))
        .overflow_hidden()
        .bg(color(ColorRole::Background))
        .child(
            Sheet::new("theme-drawer-sheet")
                .contained(true)
                .default_open(true)
                .width(600.)
                .footer_row(true)
                .title("Customize Midnight")
                .description(
                    "Your dark theme. Edits save as you go and show the next time the system is dark.",
                )
                .footer(
                    Button::new("theme-dup", "Duplicate")
                        .variant(ButtonVariant::Tertiary)
                        .size(ButtonSize::Sm),
                )
                .footer(
                    Button::new("theme-more", "Export as a theme file, or reset")
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .icon(CarbonIcon::OverflowMenuHorizontal)
                        .icon_only(true),
                )
                .footer(div().flex_1())
                .footer(
                    Button::new("theme-remove", "Remove theme")
                        .variant(ButtonVariant::DangerGhost)
                        .size(ButtonSize::Sm),
                )
                .child(div().pt(rems(1.)).child(editor)),
        )
}
