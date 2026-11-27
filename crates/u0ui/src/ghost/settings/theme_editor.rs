//! Ghost · Settings › [`ThemeEditor`] (`.g-themeed`): a theme's tokens as a
//! grid of swatches, each opening a color picker, a live preview under them,
//! and Import VS Code theme / Export in the header.
//!
//! The pickers are Kit's [`ColorPicker`] (popover, palette, HSLA sliders and
//! hex field), drawn at the reference's 28px chip size; the editor keeps one
//! [`ColorPickerState`] per token and reads the current values from them, so
//! the preview follows every pick. [`import_vscode`] maps a VS Code theme's
//! `colors` onto the tokens and [`contrast_ratio`] is the check to run before
//! saving (`text-primary` on `background` ≥ [`MIN_CONTRAST`]).
use super::row::{Handler, ghost_button};
use crate::{
    icons::CarbonIcon,
    t3::craft::{focus_ring_offset, mono, rem, sans},
    t3::press::{hover, inset_frame, track_hover},
    theme::color,
    tokens::{self, ColorRole},
};
use gpui_kit::component::{
    Sizable as _, Size,
    color_picker::{ColorPicker, ColorPickerState},
};
use gpui_kit::{
    App, AppContext as _, ElementId, Entity, Hsla, InteractiveElement as _, IntoElement,
    ParentElement, RenderOnce, Rgba, SharedString, StatefulInteractiveElement as _, Styled, Window,
    canvas, div, prelude::FluentBuilder as _, px,
};
use std::rc::Rc;

/// The contrast the editor asks of `text-primary` on `background` before a
/// theme is saved (WCAG AA for body text, README "4.5:1").
pub const MIN_CONTRAST: f32 = 4.5;

/// `.g-themeed-grid { grid-template-columns: repeat(auto-fill, minmax(11rem, 1fr)) }`.
const TILE_MIN: f32 = 176.;
/// `.g-themeed-grid { gap: var(--spacing-03) }`.
const GRID_GAP: f32 = 8.;
/// `.g-sw-chip { width: 1.75rem; height: 1.75rem }`.
const CHIP: f32 = 28.;

/// One token of the edited theme (`tokens: {name, value}[]`).
#[derive(Clone, Debug, PartialEq)]
pub struct ThemeToken {
    /// The token name, e.g. `text-primary`.
    pub name: SharedString,
    /// Its color.
    pub value: Hsla,
}

impl ThemeToken {
    /// A token.
    pub fn new(name: impl Into<SharedString>, value: impl Into<Hsla>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }

    /// A token from a `#rrggbb` / `#rgb` string; `None` if it does not parse.
    pub fn hex(name: impl Into<SharedString>, hex: &str) -> Option<Self> {
        parse_hex(hex).map(|value| Self::new(name, value))
    }
}

/// Parse `#rrggbb`, `#rgb`, `#rrggbbaa` (the `<input type=color>` and VS
/// Code formats).
pub fn parse_hex(hex: &str) -> Option<Hsla> {
    Rgba::try_from(hex.trim()).ok().map(Hsla::from)
}

/// `#rrggbb`, lower case, as `<input type=color>` reports a value.
pub fn to_hex(value: Hsla) -> String {
    let c = value.to_rgb();
    let b = |v: f32| (v.clamp(0., 1.) * 255.).round() as u8;
    format!("#{:02x}{:02x}{:02x}", b(c.r), b(c.g), b(c.b))
}

/// WCAG relative luminance of a color.
fn luminance(value: Hsla) -> f32 {
    let c = value.to_rgb();
    let lin = |v: f32| {
        if v <= 0.040_45 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * lin(c.r) + 0.7152 * lin(c.g) + 0.0722 * lin(c.b)
}

/// The WCAG contrast ratio of two colors (1–21).
pub fn contrast_ratio(a: Hsla, b: Hsla) -> f32 {
    let (la, lb) = (luminance(a), luminance(b));
    let (hi, lo) = if la >= lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

/// The token a VS Code theme color key maps to (README: `editor.background`,
/// `foreground`, `focusBorder`, `terminal.ansi*` …); `None` for keys the
/// system has no token for.
pub fn vscode_token(key: &str) -> Option<&'static str> {
    Some(match key {
        "editor.background" => "background",
        "sideBar.background" | "editorWidget.background" => "layer-01",
        "editor.foreground" | "foreground" => "text-primary",
        "descriptionForeground" => "text-secondary",
        "button.background" | "textLink.foreground" => "interactive",
        "focusBorder" => "focus",
        "terminal.ansiRed" | "errorForeground" => "support-error",
        "terminal.ansiGreen" => "support-success",
        "terminal.ansiYellow" => "support-warning",
        "terminal.ansiBlue" => "support-info",
        _ => return None,
    })
}

/// Import a VS Code theme's `colors` over `base`: each mapped key replaces
/// its token's value; tokens nothing maps to keep the base value. When two
/// keys map to one token (`editor.foreground`, `foreground`) the first wins.
pub fn import_vscode<'a>(
    base: &[ThemeToken],
    colors: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> Vec<ThemeToken> {
    let mut out = base.to_vec();
    let mut set = vec![false; out.len()];
    for (key, hex) in colors {
        let (Some(token), Some(value)) = (vscode_token(key), parse_hex(hex)) else {
            continue;
        };
        if let Some(i) = out.iter().position(|t| t.name == token)
            && !set[i]
        {
            out[i].value = value;
            set[i] = true;
        }
    }
    out
}

/// How many columns `repeat(auto-fill, minmax(11rem, 1fr))` makes in `width`
/// px with an 8px gap (at least one).
pub fn grid_columns(width: f32) -> usize {
    (((width + GRID_GAP) / (TILE_MIN + GRID_GAP)).floor() as usize).max(1)
}

/// The editor's user-facing strings.
#[derive(Clone, Debug)]
pub struct ThemeEditorTexts {
    /// The import button ("Import VS Code theme").
    pub import: SharedString,
    /// The export button ("Export").
    pub export: SharedString,
    /// The preview's title ("Preview").
    pub preview_title: SharedString,
    /// The preview's secondary line ("Secondary text on layer.").
    pub preview_body: SharedString,
    /// The preview's primary button ("Primary").
    pub preview_primary: SharedString,
}

impl Default for ThemeEditorTexts {
    /// The reference strings.
    fn default() -> Self {
        Self {
            import: "Import VS Code theme".into(),
            export: "Export".into(),
            preview_title: "Preview".into(),
            preview_body: "Secondary text on layer.".into(),
            preview_primary: "Primary".into(),
        }
    }
}

/// The theme editor (`ThemeEditor`). See the module docs.
#[derive(IntoElement)]
pub struct ThemeEditor {
    id: ElementId,
    name: SharedString,
    tokens: Vec<ThemeToken>,
    texts: ThemeEditorTexts,
    on_import: Option<Handler>,
    on_export: Option<Handler>,
}

impl ThemeEditor {
    /// An editor over `tokens` (`name` defaults to "Custom theme").
    pub fn new(id: impl Into<ElementId>, tokens: impl IntoIterator<Item = ThemeToken>) -> Self {
        Self {
            id: id.into(),
            name: "Custom theme".into(),
            tokens: tokens.into_iter().collect(),
            texts: ThemeEditorTexts::default(),
            on_import: None,
            on_export: None,
        }
    }

    /// The theme's name (`name`).
    pub fn name(mut self, name: impl Into<SharedString>) -> Self {
        self.name = name.into();
        self
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: ThemeEditorTexts) -> Self {
        self.texts = texts;
        self
    }

    /// Import VS Code theme (`onImport`).
    pub fn on_import(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_import = Some(Rc::new(f));
        self
    }

    /// Export.
    pub fn on_export(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_export = Some(Rc::new(f));
        self
    }

    /// The current value of each token, as edited, under `id`. Use it to
    /// save or export (and to run the [`contrast_ratio`] check).
    pub fn values(id: impl Into<ElementId>, window: &mut Window, cx: &mut App) -> Vec<Hsla> {
        let id = id.into();
        let pickers = window.use_keyed_state((id, "pickers"), cx, |_, _| Vec::new());
        pickers
            .read(cx)
            .iter()
            .map(|p: &Entity<ColorPickerState>| p.read(cx).value().unwrap_or_default())
            .collect()
    }
}

/// The value of token `name` in `values` (`vals[name]`), or `fallback`.
fn lookup(tokens: &[ThemeToken], values: &[Hsla], name: &str, fallback: Hsla) -> Hsla {
    tokens
        .iter()
        .position(|t| t.name == name)
        .and_then(|i| values.get(i).copied())
        .unwrap_or(fallback)
}

impl RenderOnce for ThemeEditor {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        // One Kit picker state per token, made once (`useState(() => …)`).
        let tokens = self.tokens.clone();
        let pickers = window.use_keyed_state((id.clone(), "pickers"), cx, |_, _| {
            Vec::<Entity<ColorPickerState>>::new()
        });
        if pickers.read(cx).len() != tokens.len() {
            let made: Vec<_> = tokens
                .iter()
                .map(|t| {
                    let v = t.value;
                    cx.new(|cx| ColorPickerState::new(window, cx).default_value(v))
                })
                .collect();
            pickers.update(cx, |p, _| *p = made);
        }
        let pickers: Vec<_> = pickers.read(cx).clone();
        let values: Vec<Hsla> = pickers
            .iter()
            .zip(&tokens)
            .map(|(p, t)| p.read(cx).value().unwrap_or(t.value))
            .collect();

        // `repeat(auto-fill, minmax(11rem, 1fr))`: measure the grid's width.
        let width = window.use_keyed_state((id.clone(), "width"), cx, |_, _| 0f32);
        let cols = grid_columns(*width.read(cx));
        let measure = {
            let width = width.clone();
            canvas(
                move |bounds, window, cx| {
                    // In CSS px at Carbon's 16px rem, so zoom keeps the track count.
                    let w =
                        bounds.size.width.as_f32() * tokens::REM_PX / window.rem_size().as_f32();
                    width.update(cx, |cur, cx| {
                        if (*cur - w).abs() > 0.5 {
                            *cur = w;
                            cx.notify();
                        }
                    });
                },
                |_, _, _, _| {},
            )
            .absolute()
            .top_0()
            .left_0()
            .size_full()
        };

        let chip = rem(CHIP).to_pixels(window.rem_size());
        let mut tiles: Vec<_> = Vec::new();
        for (i, (t, picker)) in tokens.iter().zip(&pickers).enumerate() {
            let tile_id: ElementId = (id.clone(), SharedString::from(format!("sw-{i}"))).into();
            let (over, store) = hover((tile_id.clone(), "hover"), window, cx);
            let focus = gpui_kit::Focusable::focus_handle(picker.read(cx), cx);
            let ring = focus.is_focused(window) && window.last_input_was_keyboard();
            let open = picker.clone();
            tiles.push(
                div()
                    .id(tile_id.clone())
                    .flex()
                    .flex_1()
                    .min_w_0()
                    .items_center()
                    .gap(tokens::SPACING_03)
                    .p(tokens::SPACING_02)
                    .cursor_pointer()
                    // `.g-sw { background: var(--layer-01) }`, `:hover` `layer-hover-01`.
                    .bg(color(if over {
                        ColorRole::LayerHover01
                    } else {
                        ColorRole::Layer01
                    }))
                    .on_hover(track_hover(store))
                    .child(
                        div()
                            .relative()
                            .flex_none()
                            .size(rem(CHIP))
                            .child(
                                ColorPicker::new(picker)
                                    .with_size(Size::Size(chip))
                                    .accessibility_label(t.name.clone()),
                            )
                            // `.g-sw-chip { box-shadow: inset 0 0 0 1px var(--border-subtle-01) }`.
                            .child(
                                div()
                                    .absolute()
                                    .top_0()
                                    .left_0()
                                    .size_full()
                                    .shadow(inset_frame(color(ColorRole::BorderSubtle01))),
                            )
                            // `.g-sw-chip:focus-within { outline: 2px solid var(--focus); outline-offset: 1px }`.
                            .when(ring, |this| this.child(focus_ring_offset(3.))),
                    )
                    .child(
                        // `.g-sw-text span { font-size: 0.6875rem }`, both mono.
                        div()
                            .id((tile_id, "text"))
                            .flex()
                            .flex_col()
                            .min_w_0()
                            .on_click(move |_, _, cx| {
                                open.update(cx, |s, cx| s.set_open(true, cx));
                            })
                            .child(
                                mono(div(), 11., 400)
                                    .overflow_hidden()
                                    .whitespace_nowrap()
                                    .text_ellipsis()
                                    .child(t.name.clone()),
                            )
                            .child(
                                mono(div(), 11., 400)
                                    .text_color(color(ColorRole::TextSecondary))
                                    .overflow_hidden()
                                    .whitespace_nowrap()
                                    .text_ellipsis()
                                    .child(to_hex(values[i])),
                            ),
                    )
                    .into_any_element(),
            );
        }
        let n = tiles.len();
        let mut rows = Vec::new();
        let mut it = tiles.into_iter();
        for _ in 0..n.div_ceil(cols) {
            let mut row = div().flex().gap(px(GRID_GAP)).w_full();
            for _ in 0..cols {
                row = match it.next() {
                    Some(tile) => row.child(tile),
                    // `auto-fill` keeps the empty tracks of the last row.
                    None => row.child(div().flex_1().min_w_0()),
                };
            }
            rows.push(row);
        }

        let fg = color(ColorRole::TextPrimary);
        let bg_val = lookup(&tokens, &values, "background", color(ColorRole::Background));
        let fg_val = lookup(&tokens, &values, "text-primary", fg);
        let layer_val = lookup(&tokens, &values, "layer-01", color(ColorRole::Layer01));
        let dim_val = lookup(
            &tokens,
            &values,
            "text-secondary",
            color(ColorRole::TextSecondary),
        );
        let accent_val = lookup(
            &tokens,
            &values,
            "interactive",
            color(ColorRole::Interactive),
        );

        sans(div(), 14., 400)
            .id(id.clone())
            .flex()
            .flex_col()
            // `.g-themeed { gap: var(--spacing-04) }`.
            .gap(tokens::SPACING_04)
            .w_full()
            .text_color(fg)
            .child(
                // `.g-themeed-head { gap: var(--spacing-02); font-size: 0.875rem }`, `b { font-weight: 600 }`.
                div()
                    .flex()
                    .items_center()
                    .gap(tokens::SPACING_02)
                    .child(
                        sans(div(), 14., 600)
                            .min_w_0()
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .text_ellipsis()
                            .child(self.name.clone()),
                    )
                    .child(div().flex_1())
                    .child(ghost_button(
                        (id.clone(), "import"),
                        self.texts.import.clone(),
                        Some(CarbonIcon::DocumentImport),
                        self.on_import.clone(),
                    ))
                    .child(ghost_button(
                        (id.clone(), "export"),
                        self.texts.export.clone(),
                        Some(CarbonIcon::Download),
                        self.on_export.clone(),
                    )),
            )
            .child(
                div()
                    .relative()
                    .flex()
                    .flex_col()
                    .gap(px(GRID_GAP))
                    .w_full()
                    .child(measure)
                    .children(rows),
            )
            .child(
                // `.g-themeed-preview { padding: var(--spacing-04); font-size: 0.875rem;
                // box-shadow: inset 0 0 0 1px var(--border-subtle-01) }`, painted in the
                // edited `background` / `text-primary`.
                div()
                    .p(tokens::SPACING_04)
                    .bg(bg_val)
                    .text_color(fg_val)
                    .shadow(inset_frame(color(ColorRole::BorderSubtle01)))
                    .child(
                        // `style={{ background: vals['layer-01'], padding: 12 }}`.
                        div()
                            .flex()
                            .flex_col()
                            .items_start()
                            .p(rem(12.))
                            .bg(layer_val)
                            // A bare `<b>` (Carbon's `b { font: inherit }`): 400.
                            .child(sans(div(), 14., 400).child(self.texts.preview_title.clone()))
                            .child(
                                // `p { margin: 4px 0 8px; color: vals['text-secondary'] }`.
                                div()
                                    .mt(rem(4.))
                                    .mb(rem(8.))
                                    .text_color(dim_val)
                                    .child(self.texts.preview_body.clone()),
                            )
                            .child(
                                // `background: vals.interactive; color: #fff; padding: 4px 12px`
                                // (`text-on-color`, white in every theme).
                                div()
                                    .px(rem(12.))
                                    .py(rem(4.))
                                    .bg(accent_val)
                                    .text_color(color(ColorRole::TextOnColor))
                                    .child(self.texts.preview_primary.clone()),
                            ),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_round_trips() {
        for hex in [
            "#0b0d12", "#161a22", "#e6e9ef", "#5b8cff", "#ffffff", "#000000",
        ] {
            assert_eq!(to_hex(parse_hex(hex).unwrap()), hex);
        }
        assert_eq!(to_hex(parse_hex("#fff").unwrap()), "#ffffff");
        assert!(parse_hex("blue").is_none());
    }

    #[test]
    fn contrast_matches_wcag() {
        let black = parse_hex("#000000").unwrap();
        let white = parse_hex("#ffffff").unwrap();
        assert!((contrast_ratio(black, white) - 21.).abs() < 0.01);
        assert!((contrast_ratio(white, white) - 1.).abs() < 0.001);
        // The reference "Midnight" pair passes; mid grey on white does not.
        let bg = parse_hex("#0b0d12").unwrap();
        let fg = parse_hex("#e6e9ef").unwrap();
        assert!(contrast_ratio(fg, bg) >= MIN_CONTRAST);
        assert!(contrast_ratio(parse_hex("#999999").unwrap(), white) < MIN_CONTRAST);
    }

    #[test]
    fn vscode_import_maps_known_keys_and_keeps_the_rest() {
        let base = vec![
            ThemeToken::hex("background", "#ffffff").unwrap(),
            ThemeToken::hex("text-primary", "#161616").unwrap(),
            ThemeToken::hex("layer-01", "#f4f4f4").unwrap(),
            ThemeToken::hex("focus", "#0f62fe").unwrap(),
        ];
        let out = import_vscode(
            &base,
            [
                ("editor.background", "#1e1e1e"),
                ("editor.foreground", "#d4d4d4"),
                ("foreground", "#cccccc"),
                ("focusBorder", "#007fd4"),
                ("activityBar.background", "#333333"),
            ],
        );
        assert_eq!(to_hex(out[0].value), "#1e1e1e");
        assert_eq!(to_hex(out[1].value), "#d4d4d4");
        assert_eq!(out[2], base[2]);
        assert_eq!(to_hex(out[3].value), "#007fd4");
        assert_eq!(vscode_token("terminal.ansiRed"), Some("support-error"));
        assert_eq!(vscode_token("activityBar.background"), None);
    }

    #[test]
    fn grid_fills_11rem_tracks() {
        assert_eq!(grid_columns(0.), 1);
        assert_eq!(grid_columns(175.), 1);
        assert_eq!(grid_columns(176. * 2. + 8.), 2);
        assert_eq!(grid_columns(760.), 4);
    }
}
