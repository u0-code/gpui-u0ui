//! CodeViewer: a read-only code view (`CodeViewer` and `highlight` in
//! `src/ghost/panels.jsx`, `.g-code*` / `.g-syn-*` in `src/ghost.css`): an
//! optional `.g-codehead` (file glyph, mono path, language, "N lines", copy),
//! then a scrolling body on `background` with sticky line numbers,
//! highlighted lines and syntax colors from the `syntax-*` tokens.
//!
//! **Highlighting.** The reference ships a tiny lexical tokenizer
//! (`highlight(line)`, JS/TS/JSON/CSS/shell-ish) and says production should
//! swap in Shiki while keeping the `g-syn-<kind>` classes. Kit's tree-sitter
//! highlighter is not part of the `gpui_kit` build and paints its own theme
//! colors, so [`highlight`] ports the reference tokenizer one to one and
//! [`SyntaxKind::role`] maps each class to its token. A host with a real
//! grammar passes its own spans through [`CodeViewer::spans`].
//!
//! Behavior: the body is a focusable region (2px inset `focus` ring on
//! keyboard focus) that scrolls with the wheel and with ↑/↓, PageUp/PageDown,
//! Home/End. Without `wrap` long lines scroll sideways under line numbers
//! that stay put (`.g-code-ln { position: sticky; left: 0 }`).
use super::head::{code_head, dim, grow, head_icon, head_name};
use crate::{
    components::CopyButton,
    ghost::primitives::parts::{focus_ring, focus_visible, keyed_focus},
    icons::CarbonIcon,
    t3::craft::{mono, rem},
    theme::{LayerLevel, color},
    tokens::ColorRole,
};
use gpui_kit::{
    App, ElementId, FontStyle, FontWeight, HighlightStyle, InteractiveElement as _, IntoElement,
    KeyDownEvent, ParentElement, Pixels, RenderOnce, ScrollHandle, SharedString,
    StatefulInteractiveElement as _, Styled, StyledText, UnderlineStyle, Window, div, point,
    prelude::FluentBuilder as _, px,
};
use std::ops::Range;
use std::rc::Rc;

// ── highlight ───────────────────────────────────────────────────────────

/// A `g-syn-<kind>` class (`src/ghost.css` "Syntax").
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SyntaxKind {
    /// `keyword`.
    Keyword,
    /// `string`.
    String,
    /// `comment` (italic).
    Comment,
    /// `function`.
    Function,
    /// `number`.
    Number,
    /// `type`.
    Type,
    /// `variable` (the body's base ink).
    Variable,
    /// `tag`.
    Tag,
    /// `attribute`.
    Attribute,
    /// `operator`.
    Operator,
    /// `punctuation`.
    Punctuation,
    /// `constant`.
    Constant,
    /// `regexp`.
    Regexp,
    /// `property`.
    Property,
    /// `heading` (600 weight).
    Heading,
    /// `link` (underlined).
    Link,
    /// `invalid` (wavy underline).
    Invalid,
}

impl SyntaxKind {
    /// `.g-syn-<kind> { color: var(--syntax-<kind>) }`.
    pub const fn role(self) -> ColorRole {
        match self {
            Self::Keyword => ColorRole::SyntaxKeyword,
            Self::String => ColorRole::SyntaxString,
            Self::Comment => ColorRole::SyntaxComment,
            Self::Function => ColorRole::SyntaxFunction,
            Self::Number => ColorRole::SyntaxNumber,
            Self::Type => ColorRole::SyntaxType,
            Self::Variable => ColorRole::SyntaxVariable,
            Self::Tag => ColorRole::SyntaxTag,
            Self::Attribute => ColorRole::SyntaxAttribute,
            Self::Operator => ColorRole::SyntaxOperator,
            Self::Punctuation => ColorRole::SyntaxPunctuation,
            Self::Constant => ColorRole::SyntaxConstant,
            Self::Regexp => ColorRole::SyntaxRegexp,
            Self::Property => ColorRole::SyntaxProperty,
            Self::Heading => ColorRole::SyntaxHeading,
            Self::Link => ColorRole::SyntaxLink,
            Self::Invalid => ColorRole::SyntaxInvalid,
        }
    }

    /// The span style: the token color, plus `.g-syn-comment { font-style:
    /// italic }`, `.g-syn-heading { font-weight: 600 }`, `.g-syn-link {
    /// text-decoration: underline }`, `.g-syn-invalid { text-decoration: wavy
    /// underline }`.
    pub fn style(self) -> HighlightStyle {
        let ink = color(self.role());
        let underline = |wavy| UnderlineStyle {
            thickness: px(1.),
            color: Some(ink),
            wavy,
        };
        HighlightStyle {
            color: Some(ink),
            font_style: (self == Self::Comment).then_some(FontStyle::Italic),
            font_weight: (self == Self::Heading).then_some(FontWeight::SEMIBOLD),
            underline: match self {
                Self::Link => Some(underline(false)),
                Self::Invalid => Some(underline(true)),
                _ => None,
            },
            ..Default::default()
        }
    }
}

/// `KW`: the reference's keyword list.
const KEYWORDS: &[&str] = &[
    "const",
    "let",
    "var",
    "function",
    "return",
    "if",
    "else",
    "for",
    "while",
    "import",
    "from",
    "export",
    "default",
    "class",
    "extends",
    "new",
    "await",
    "async",
    "try",
    "catch",
    "throw",
    "type",
    "interface",
    "enum",
    "switch",
    "case",
    "break",
    "continue",
    "of",
    "in",
    "as",
    "fn",
    "pub",
    "use",
    "mod",
    "impl",
    "struct",
    "match",
    "self",
    "mut",
    "where",
];

/// `/^(true|false|null|undefined|None|Some)$/`.
const CONSTANTS: &[&str] = &["true", "false", "null", "undefined", "None", "Some"];

fn is_word(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '$'
}

/// `[{}()[\];,.]`.
fn is_punct(c: char) -> bool {
    matches!(c, '{' | '}' | '(' | ')' | '[' | ']' | ';' | ',' | '.')
}

/// `[=+\-*/<>!&|?:%^~]`.
fn is_op(c: char) -> bool {
    matches!(
        c,
        '=' | '+' | '-' | '*' | '/' | '<' | '>' | '!' | '&' | '|' | '?' | ':' | '%' | '^' | '~'
    )
}

/// The end (byte index) of a quoted run opened at `start` by `quote`, or
/// `None` when it never closes (`"(?:[^"\\]|\\.)*"`; backticks take no
/// escapes: `` `[^`]*` ``).
fn quoted_end(line: &str, start: usize, quote: char) -> Option<usize> {
    let mut chars = line[start + 1..].char_indices();
    while let Some((i, c)) = chars.next() {
        if c == quote {
            return Some(start + 1 + i + 1);
        }
        if c == '\\' && quote != '`' {
            chars.next();
        }
    }
    None
}

/// The reference `highlight(line)`: one line split into classified byte
/// ranges (unclassified runs, such as plain identifiers and spaces, are left
/// out and keep the body's `syntax-variable` ink).
///
/// Tokens, in the reference regex's order: `//` or `#` comments to the end
/// of the line, `"…"` / `'…'` strings with escapes, `` `…` `` templates,
/// numbers, identifiers, single punctuation, operator runs, whitespace, any
/// other character. Identifiers are keywords, constants, types (capitalised),
/// or functions (followed by `(`).
pub fn highlight(line: &str) -> Vec<(Range<usize>, SyntaxKind)> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < line.len() {
        let rest = &line[i..];
        let c = rest.chars().next().unwrap_or_default();
        // `(\/\/.*$|#.*$|…)`: comments run to the end.
        if rest.starts_with("//") || c == '#' {
            out.push((i..line.len(), SyntaxKind::Comment));
            break;
        }
        let (end, kind) = if matches!(c, '"' | '\'' | '`') {
            // An unclosed quote matches `.` alone, which `/^["'`]/` still
            // classifies as a string.
            (
                quoted_end(line, i, c).unwrap_or(i + 1),
                Some(SyntaxKind::String),
            )
        } else if c.is_ascii_digit() {
            // `\b\d+(?:\.\d+)?\b`; a digit run glued to letters falls back to
            // single `.` matches that `/^\d/` still calls numbers.
            let mut j = i + rest
                .find(|ch: char| !ch.is_ascii_digit())
                .unwrap_or(rest.len());
            if line[j..].starts_with('.')
                && line[j + 1..].starts_with(|ch: char| ch.is_ascii_digit())
            {
                let frac = &line[j + 1..];
                j += 1 + frac
                    .find(|ch: char| !ch.is_ascii_digit())
                    .unwrap_or(frac.len());
            }
            (j, Some(SyntaxKind::Number))
        } else if c.is_ascii_alphabetic() || c == '_' || c == '$' {
            let end = i + rest.find(|ch: char| !is_word(ch)).unwrap_or(rest.len());
            let word = &line[i..end];
            let kind = if KEYWORDS.contains(&word) {
                Some(SyntaxKind::Keyword)
            } else if CONSTANTS.contains(&word) {
                Some(SyntaxKind::Constant)
            } else if word.starts_with(|ch: char| ch.is_ascii_uppercase()) {
                Some(SyntaxKind::Type)
            } else if line[end..].trim_start().starts_with('(') {
                Some(SyntaxKind::Function)
            } else {
                None
            };
            (end, kind)
        } else if is_punct(c) {
            (i + 1, Some(SyntaxKind::Punctuation))
        } else if is_op(c) {
            let end = i + rest.find(|ch: char| !is_op(ch)).unwrap_or(rest.len());
            (end, Some(SyntaxKind::Operator))
        } else if c.is_whitespace() {
            let end = i + rest
                .find(|ch: char| !ch.is_whitespace())
                .unwrap_or(rest.len());
            (end, None)
        } else {
            (i + c.len_utf8(), None)
        };
        if let Some(kind) = kind {
            out.push((i..end, kind));
        }
        i = end;
    }
    out
}

/// The lines of `code` (`code.replace(/\n$/, '').split('\n')`).
pub fn code_lines(code: &str) -> Vec<&str> {
    code.strip_suffix('\n')
        .unwrap_or(code)
        .split('\n')
        .collect()
}

// ── geometry ────────────────────────────────────────────────────────────

/// `.g-code-table { font-size: 0.75rem; line-height: 1.25rem }`: 20px rows.
pub const ROW: f32 = 20.;
/// IBM Plex Mono's advance (600/1000 em) at 12px.
pub const MONO_ADVANCE: f32 = 7.2;
/// `.g-code-ln { padding: 0 var(--spacing-04) 0 var(--spacing-05) }`.
pub const LN_PADDING: (f32, f32) = (16., 12.);

/// The line-number column's width for numbers up to `last`: the digits
/// plus `.g-code-ln`'s padding (`width: 1%` shrinks it to its content).
pub fn gutter_width(last: usize) -> f32 {
    let digits = last.max(1).to_string().len() as f32;
    LN_PADDING.0 + digits * MONO_ADVANCE + LN_PADDING.1
}

/// Where a key scrolls the body, from offset `y` (≤ 0) with `viewport` and
/// `max` scroll in px: ↑/↓ one row, PageUp/PageDown a viewport, Home/End
/// the ends. `None` for other keys.
pub fn scroll_target(key: &str, y: f32, viewport: f32, max: f32) -> Option<f32> {
    let to = match key {
        "up" => y + ROW * 2.,
        "down" => y - ROW * 2.,
        "pageup" => y + viewport,
        "pagedown" => y - viewport,
        "home" => 0.,
        "end" => -max,
        _ => return None,
    };
    Some(to.clamp(-max, 0.))
}

// ── CodeViewer ──────────────────────────────────────────────────────────

/// The viewer's user-facing strings (the reference's English defaults).
#[derive(Clone)]
pub struct CodeViewerTexts {
    /// `"{n} lines"`.
    pub lines: Rc<dyn Fn(usize) -> SharedString>,
    /// The copy button's `iconDescription`: `"Copy file"`.
    pub copy: SharedString,
    /// The copy feedback (Carbon `CopyButton` default `"Copied!"`).
    pub copied: SharedString,
    /// The body's name when there is no path: `"Code"`.
    pub region: SharedString,
}

impl Default for CodeViewerTexts {
    fn default() -> Self {
        Self {
            lines: Rc::new(|n| format!("{n} lines").into()),
            copy: "Copy file".into(),
            copied: "Copied!".into(),
            region: "Code".into(),
        }
    }
}

type Spans = Rc<dyn Fn(&str) -> Vec<(Range<usize>, SyntaxKind)>>;

/// A read-only code view. See the module docs.
///
/// ```ignore
/// CodeViewer::new("cv", "export const x = 1;\n")
///     .path("src/x.ts")
///     .language("TypeScript")
///     .start_line(12)
///     .highlight_lines([15, 16])
/// ```
#[derive(IntoElement)]
pub struct CodeViewer {
    id: ElementId,
    code: SharedString,
    path: Option<SharedString>,
    language: Option<SharedString>,
    start_line: usize,
    highlight_lines: Vec<usize>,
    wrap: bool,
    max_height: f32,
    spans: Option<Spans>,
    texts: CodeViewerTexts,
}

impl CodeViewer {
    /// A viewer for `code`.
    pub fn new(id: impl Into<ElementId>, code: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            code: code.into(),
            path: None,
            language: None,
            start_line: 1,
            highlight_lines: Vec::new(),
            wrap: false,
            max_height: 360.,
            spans: None,
            texts: CodeViewerTexts::default(),
        }
    }

    /// The file path (`path`); with it the viewer shows its head.
    pub fn path(mut self, path: impl Into<SharedString>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// The language label in the head (`language`).
    pub fn language(mut self, language: impl Into<SharedString>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// The first line's number (`startLine`, default 1).
    pub fn start_line(mut self, start: usize) -> Self {
        self.start_line = start;
        self
    }

    /// Line numbers to highlight (`highlightLines`).
    pub fn highlight_lines(mut self, lines: impl IntoIterator<Item = usize>) -> Self {
        self.highlight_lines = lines.into_iter().collect();
        self
    }

    /// Wrap long lines (`wrap`) instead of scrolling sideways.
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    /// The body's maximum height in px (`maxHeight`, default 360).
    pub fn max_height(mut self, max_height: f32) -> Self {
        self.max_height = max_height;
        self
    }

    /// Replace the built-in [`highlight`] (e.g. with a real grammar).
    pub fn spans(mut self, f: impl Fn(&str) -> Vec<(Range<usize>, SyntaxKind)> + 'static) -> Self {
        self.spans = Some(Rc::new(f));
        self
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: CodeViewerTexts) -> Self {
        self.texts = texts;
        self
    }
}

/// One line's source text in syntax colors.
fn source(line: &str, spans: &Option<Spans>) -> StyledText {
    let runs = match spans {
        Some(f) => f(line),
        None => highlight(line),
    };
    StyledText::new(SharedString::from(line.to_string()))
        .with_highlights(runs.into_iter().map(|(r, k)| (r, k.style())))
}

/// `.g-code-hl .g-code-ln { box-shadow: inset 2px 0 0 var(--border-interactive) }`.
fn hl_bar() -> gpui_kit::Div {
    div()
        .absolute()
        .top_0()
        .bottom_0()
        .left_0()
        .w(px(2.))
        .bg(color(ColorRole::BorderInteractive))
}

/// A line-number cell: `text-helper`, right-aligned; `text-primary` with
/// the bar on a highlighted line.
fn number_cell(n: usize, hl: bool, gutter: f32) -> gpui_kit::Div {
    div()
        .relative()
        .flex_none()
        .w(rem(gutter))
        .min_h(rem(ROW))
        .pl(rem(LN_PADDING.0))
        .pr(rem(LN_PADDING.1))
        .flex()
        .justify_end()
        .bg(color(if hl {
            ColorRole::Highlight
        } else {
            ColorRole::Background
        }))
        .text_color(color(if hl {
            ColorRole::TextPrimary
        } else {
            ColorRole::TextHelper
        }))
        .child(n.to_string())
        .when(hl, |this| this.child(hl_bar()))
}

impl RenderOnce for CodeViewer {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let code = self.code.to_string();
        let lines = code_lines(&code);
        let last = self.start_line + lines.len().saturating_sub(1);
        let gutter = gutter_width(last);
        let hl = |n: usize| self.highlight_lines.contains(&n);
        let focus = keyed_focus((id.clone(), "focus"), window, cx);
        let ring = focus_visible(&focus, window);
        let scroll = window
            .use_keyed_state((id.clone(), "scroll"), cx, |_, _| ScrollHandle::new())
            .read(cx)
            .clone();

        let head = self.path.clone().map(|path| {
            code_head()
                .child(head_icon(CarbonIcon::Document))
                .child(head_name(path, ColorRole::TextPrimary))
                .when_some(self.language.clone(), |this, l| this.child(dim(l, 12.)))
                .child(grow())
                .child(dim((self.texts.lines)(lines.len()), 12.))
                .child(
                    CopyButton::new((id.clone(), "copy"), self.code.clone())
                        .label(self.texts.copy.clone())
                        .feedback(self.texts.copied.clone())
                        .size(32.)
                        .quiet(true)
                        .layer(LayerLevel::One),
                )
        });

        let numbers = lines
            .iter()
            .enumerate()
            .map(|(i, _)| self.start_line + i)
            .collect::<Vec<_>>();
        let table = if self.wrap {
            // `.g-code-body--wrap .g-code-src { white-space: pre-wrap; word-break: break-all }`:
            // each row grows with its wrapped source.
            div()
                .flex()
                .flex_col()
                .w_full()
                .children(lines.iter().zip(&numbers).map(|(line, &n)| {
                    let on = hl(n);
                    div()
                        .flex()
                        .w_full()
                        .child(number_cell(n, on, gutter))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .min_h(rem(ROW))
                                .pr(rem(16.))
                                .when(on, |d| d.bg(color(ColorRole::Highlight)))
                                .child(source(line, &self.spans)),
                        )
                }))
        } else {
            // `.g-code-src { white-space: pre }` scrolls sideways under the
            // sticky number column.
            div()
                .flex()
                .w_full()
                .child(
                    div().flex().flex_col().flex_none().children(
                        numbers
                            .iter()
                            .map(|&n| number_cell(n, hl(n), gutter).h(rem(ROW))),
                    ),
                )
                .child(
                    div()
                        .id((id.clone(), "x"))
                        .flex_1()
                        .min_w_0()
                        .overflow_x_scroll()
                        .child(div().flex().flex_col().min_w_full().children(
                            lines.iter().zip(&numbers).map(|(line, &n)| {
                                div()
                                    .h(rem(ROW))
                                    .pr(rem(16.))
                                    .whitespace_nowrap()
                                    .when(hl(n), |d| d.bg(color(ColorRole::Highlight)))
                                    .child(source(line, &self.spans))
                            }),
                        )),
                )
        };

        let key_scroll = scroll.clone();
        let body = mono(div(), 12., 400)
            .id((id.clone(), "body"))
            .track_focus(&focus)
            .tab_stop(true)
            .relative()
            .max_h(rem(self.max_height))
            .overflow_y_scroll()
            .track_scroll(&scroll)
            .bg(color(ColorRole::Background))
            .text_color(color(ColorRole::SyntaxVariable))
            .line_height(rem(ROW))
            .on_key_down(move |e: &KeyDownEvent, window, cx| {
                let s = &key_scroll;
                let viewport = f32::from(s.bounds().size.height);
                let max: Pixels = s.max_offset().y;
                if let Some(y) = scroll_target(
                    e.keystroke.key.as_str(),
                    f32::from(s.offset().y),
                    viewport,
                    f32::from(max),
                ) {
                    s.set_offset(point(s.offset().x, px(y)));
                    window.refresh();
                    cx.stop_propagation();
                }
            })
            .child(table);

        div()
            .id(id)
            .flex()
            .flex_col()
            .min_w_0()
            .bg(color(ColorRole::Layer01))
            .when_some(head, |this, h| this.child(h))
            .child(
                div()
                    .relative()
                    .child(body)
                    .when(ring, |this| this.child(focus_ring())),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(line: &str) -> Vec<(&str, SyntaxKind)> {
        highlight(line)
            .into_iter()
            .map(|(r, k)| (&line[r], k))
            .collect()
    }

    #[test]
    fn highlight_ports_the_reference_tokenizer() {
        use SyntaxKind::*;
        assert_eq!(
            kinds("import { tokens } from './tokens';"),
            vec![
                ("import", Keyword),
                ("{", Punctuation),
                ("}", Punctuation),
                ("from", Keyword),
                ("'./tokens'", String),
                (";", Punctuation),
            ]
        );
        assert_eq!(
            kinds("export function resolve(name: string, theme = 'g100') {"),
            vec![
                ("export", Keyword),
                ("function", Keyword),
                ("resolve", Function),
                ("(", Punctuation),
                (":", Operator),
                (",", Punctuation),
                ("=", Operator),
                ("'g100'", String),
                (")", Punctuation),
                ("{", Punctuation),
            ]
        );
        // Comments run to the end of the line, `#` included.
        assert_eq!(
            kinds("  // Resolve a token"),
            vec![("// Resolve a token", Comment)]
        );
        assert_eq!(kinds("x # note"), vec![("# note", Comment)]);
        // Types, constants, numbers, templates and operator runs.
        assert_eq!(
            kinds("if (v === undefined) throw new Error(`Unknown ${n}`);")
                .into_iter()
                .filter(|(_, k)| *k != Punctuation)
                .collect::<Vec<_>>(),
            vec![
                ("if", Keyword),
                ("===", Operator),
                ("undefined", Constant),
                ("throw", Keyword),
                ("new", Keyword),
                ("Error", Type),
                ("`Unknown ${n}`", String),
            ]
        );
        assert_eq!(
            kinds("1.5 + 12px"),
            vec![("1.5", Number), ("+", Operator), ("12", Number)]
        );
        // An unclosed quote is a one-character string.
        assert_eq!(kinds("\"ab"), vec![("\"", String)]);
        // Escapes stay inside the string.
        assert_eq!(kinds(r#""a\"b" x"#), vec![(r#""a\"b""#, String)]);
    }

    #[test]
    fn syntax_kinds_use_their_tokens() {
        crate::theme::tests_set(crate::CarbonTheme::G100);
        // `.g-syn-keyword { color: var(--syntax-keyword) }`.
        assert_eq!(
            SyntaxKind::Keyword.style().color,
            Some(color(ColorRole::SyntaxKeyword))
        );
        // `.g-syn-comment { font-style: italic }`.
        assert_eq!(
            SyntaxKind::Comment.style().font_style,
            Some(FontStyle::Italic)
        );
        assert_eq!(
            SyntaxKind::Heading.style().font_weight,
            Some(FontWeight::SEMIBOLD)
        );
        assert!(
            SyntaxKind::Invalid
                .style()
                .underline
                .is_some_and(|u| u.wavy)
        );
    }

    #[test]
    fn lines_and_gutter() {
        // `code.replace(/\n$/, '').split('\n')`.
        assert_eq!(code_lines("a\nb\n"), vec!["a", "b"]);
        assert_eq!(code_lines(""), vec![""]);
        // `.g-code-ln { padding: 0 12px 0 16px }` around the digits.
        assert_eq!(gutter_width(21), 16. + 2. * 7.2 + 12.);
        assert_eq!(gutter_width(9), 16. + 7.2 + 12.);
    }

    #[test]
    fn keys_scroll_within_bounds() {
        assert_eq!(scroll_target("down", 0., 200., 300.), Some(-40.));
        assert_eq!(scroll_target("up", 0., 200., 300.), Some(0.));
        assert_eq!(scroll_target("pagedown", -200., 200., 300.), Some(-300.));
        assert_eq!(scroll_target("end", 0., 200., 300.), Some(-300.));
        assert_eq!(scroll_target("home", -120., 200., 300.), Some(0.));
        assert_eq!(scroll_target("a", 0., 200., 300.), None);
    }
}
