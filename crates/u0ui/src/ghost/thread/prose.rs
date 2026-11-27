//! MarkdownProse and its blocks (`MarkdownProse`, `ProseCodeBlock`,
//! `MathBlock`, `MermaidBlock` in `src/ghost/workspace.jsx`; `.g-prose*`,
//! `.g-pcode`, `.g-math*`, `.g-mermaid*` in `src/ghost.css`, with the
//! `gemini-A` craft pass in `src/t3.css`).
//!
//! **Typesetting** (`.g-prose`, max 46rem): body 14/22 (gemini-A); h1 24px
//! light, h2 20, h3 16, h4 14 semibold at 1.3 line height with 1.5em / 0.5em
//! margins; paragraphs, lists, quotes, tables and figures 0.85em apart
//! (adjacent margins collapse, the first and last drop theirs); lists
//! indented 1.5em with `text-secondary` markers; GFM task lists without
//! markers; links `link-primary` underlined; quotes behind a 3px
//! `border-strong-01` rule in `text-secondary`; 13px tables with
//! `layer-accent-01` semibold heads and hairline rows; rules; footnotes in
//! 12px `text-secondary` under a hairline. Inline code follows the last
//! rule in cascade order (r2-desk): 13px mono on a borderless
//! `layer-accent-01` 60% tint, through the helper shared with the T3
//! AssistantMessage ([`crate::components::inline_code`]).
//!
//! **Input.** Build [`Block`]s directly, or hand [`MarkdownProse::markdown`]
//! a Markdown string: the built-in reader covers what the reference
//! demonstrates (ATX headings, paragraphs, nested `-`/`1.` lists, `- [ ]`
//! task lists, `>` quotes, pipe tables, `---`, fenced code with ```` ```mermaid
//! ```` and ```` ```math ```` / `$$…$$` blocks, footnotes `[^1]`, and inline
//! `` `code` ``, `**strong**`, `*em*`, `[links](url)`).
//!
//! **Mermaid and math.** The reference renders them only when `window.mermaid`
//! / `window.katex` are on the page and otherwise shows the source; there is
//! no Mermaid or KaTeX engine for GPUI, so [`MermaidBlock`] shows its source
//! under the "Mermaid" caption with a copy button and [`MathBlock`] shows the
//! TeX in mono, exactly the reference's fallback.
//!
//! Known limits of GPUI text: inline code scales with its paragraph outside
//! 14px body text and 13px tables (runs cannot change size; see
//! [`crate::components::inline_code`]), and footnote references use Unicode
//! superscript digits (runs cannot shift their baseline).
use crate::{
    components::{
        CopyButton,
        inline_code::{push_inline_code, with_code_boxes},
    },
    ghost::{
        panels::{
            code_viewer::highlight,
            head::{code_head, grow, head_icon},
        },
        primitives::{
            IconButton,
            parts::{flash, flash_on, flash_state},
        },
    },
    icons::{CarbonIcon, Icon},
    t3::craft::{mono, rem, sans},
    theme::{LayerLevel, color},
    tokens::{self, ColorRole, FontFamilyKey},
    typography::font,
};
use gpui_kit::{
    AnyElement, App, ClipboardItem, Div, ElementId, FontStyle, FontWeight, Hsla,
    InteractiveElement as _, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement as _, Styled, StyledText, TextRun, UnderlineStyle, Window, div,
    prelude::FluentBuilder as _, px,
};
use std::ops::Range;

// ── Model ───────────────────────────────────────────────────────────────

/// An inline run of prose.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Inline {
    /// Plain text.
    Text(SharedString),
    /// `` `code` ``: the outlined mono token.
    Code(SharedString),
    /// `**strong**`.
    Strong(SharedString),
    /// `*em*`.
    Em(SharedString),
    /// `[text](href)`: `link-primary`, underlined.
    Link(SharedString, SharedString),
    /// `[^n]`: a superscript footnote reference.
    FootnoteRef(SharedString),
    /// `$tex$`: inline math, shown as its TeX in mono (`.g-math-inline`).
    Math(SharedString),
}

/// One list item: its text and any nested blocks (sub-lists).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListItem {
    /// The item's text.
    pub content: Vec<Inline>,
    /// Nested lists.
    pub children: Vec<Block>,
}

/// A block of prose.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Block {
    /// `#`…`####` heading (level 1–4).
    Heading(u8, Vec<Inline>),
    /// A paragraph.
    Paragraph(Vec<Inline>),
    /// A bulleted (`ordered: false`) or numbered list starting at `start`.
    List {
        /// Numbered?
        ordered: bool,
        /// The first number.
        start: usize,
        /// The items.
        items: Vec<ListItem>,
    },
    /// A GFM task list (`contains-task-list`): (checked, text) items.
    TaskList(Vec<(bool, Vec<Inline>)>),
    /// A block quote.
    Quote(Vec<Inline>),
    /// A table: head cells, then rows of cells.
    Table(Vec<Vec<Inline>>, Vec<Vec<Vec<Inline>>>),
    /// `---`.
    Rule,
    /// A fenced code block ([`ProseCodeBlock`]).
    Code {
        /// The fence's language.
        language: SharedString,
        /// The code.
        code: SharedString,
    },
    /// Display math ([`MathBlock`]).
    Math(SharedString),
    /// A Mermaid diagram ([`MermaidBlock`]).
    Mermaid(SharedString),
    /// The footnotes (`.footnotes`), numbered from 1.
    Footnotes(Vec<Vec<Inline>>),
}

// ── Markdown reader ─────────────────────────────────────────────────────

/// Split inline Markdown into runs: `` `code` ``, `**strong**`, `*em*` /
/// `_em_`, `[text](url)`, `[^n]`, `$math$`; anything unmatched stays text.
pub fn parse_inlines(s: &str) -> Vec<Inline> {
    let mut out = Vec::new();
    let mut text = String::new();
    let mut i = 0;
    let flush = |text: &mut String, out: &mut Vec<Inline>| {
        if !text.is_empty() {
            out.push(Inline::Text(std::mem::take(text).into()));
        }
    };
    while i < s.len() {
        let rest = &s[i..];
        let closed = |open: &str, close: &str| -> Option<(usize, &str)> {
            let body = rest.strip_prefix(open)?;
            let end = body.find(close)?;
            (end > 0).then(|| (open.len() + end + close.len(), &body[..end]))
        };
        let token = if let Some((n, b)) = closed("`", "`") {
            Some((n, Inline::Code(b.to_string().into())))
        } else if let Some((n, b)) = closed("**", "**") {
            Some((n, Inline::Strong(b.to_string().into())))
        } else if let Some((n, b)) = closed("[^", "]") {
            Some((n, Inline::FootnoteRef(b.to_string().into())))
        } else if let Some((n, b)) = closed("$", "$") {
            Some((n, Inline::Math(b.to_string().into())))
        } else if let Some((n, b)) = closed("*", "*").or_else(|| closed("_", "_")) {
            Some((n, Inline::Em(b.to_string().into())))
        } else if rest.starts_with('[') {
            rest.find("](").and_then(|mid| {
                let tail = &rest[mid + 2..];
                tail.find(')').map(|end| {
                    (
                        mid + 2 + end + 1,
                        Inline::Link(
                            rest[1..mid].to_string().into(),
                            tail[..end].to_string().into(),
                        ),
                    )
                })
            })
        } else {
            None
        };
        match token {
            Some((n, inline)) => {
                flush(&mut text, &mut out);
                out.push(inline);
                i += n;
            }
            None => {
                let c = rest.chars().next().unwrap_or_default();
                text.push(c);
                i += c.len_utf8();
            }
        }
    }
    flush(&mut text, &mut out);
    out
}

/// A list line: (indent, ordered, number, task state, text).
fn list_line(line: &str) -> Option<(usize, bool, usize, Option<bool>, &str)> {
    let indent = line.len() - line.trim_start().len();
    let t = line.trim_start();
    let (ordered, number, rest) = if let Some(r) = t
        .strip_prefix("- ")
        .or_else(|| t.strip_prefix("* "))
        .or_else(|| t.strip_prefix("+ "))
    {
        (false, 1, r)
    } else {
        let digits: String = t.chars().take_while(|c| c.is_ascii_digit()).collect();
        let r = t[digits.len()..].strip_prefix(". ")?;
        (true, digits.parse().ok()?, r)
    };
    let (task, rest) = if let Some(r) = rest.strip_prefix("[ ] ") {
        (Some(false), r)
    } else if let Some(r) = rest
        .strip_prefix("[x] ")
        .or_else(|| rest.strip_prefix("[X] "))
    {
        (Some(true), r)
    } else {
        (None, rest)
    };
    Some((indent, ordered, number, task, rest))
}

/// Build nested lists from consecutive list lines.
fn build_list(lines: &[(usize, bool, usize, Option<bool>, String)]) -> Block {
    if lines.iter().all(|l| l.3.is_some()) {
        return Block::TaskList(
            lines
                .iter()
                .map(|l| (l.3.unwrap_or(false), parse_inlines(&l.4)))
                .collect(),
        );
    }
    let base = lines[0].0;
    let mut items: Vec<ListItem> = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let (_, _, _, _, text) = &lines[i];
        let mut j = i + 1;
        while j < lines.len() && lines[j].0 > base {
            j += 1;
        }
        let children = if j > i + 1 {
            vec![build_list(&lines[i + 1..j])]
        } else {
            Vec::new()
        };
        items.push(ListItem {
            content: parse_inlines(text),
            children,
        });
        i = j;
    }
    Block::List {
        ordered: lines[0].1,
        start: lines[0].2,
        items,
    }
}

/// The cells of a `| a | b |` row.
fn table_cells(line: &str) -> Vec<Vec<Inline>> {
    line.trim()
        .trim_start_matches('|')
        .trim_end_matches('|')
        .split('|')
        .map(|c| parse_inlines(c.trim()))
        .collect()
}

/// Read Markdown into [`Block`]s (see the module docs for what it covers).
pub fn parse_markdown(src: &str) -> Vec<Block> {
    let lines: Vec<&str> = src.lines().collect();
    let mut blocks = Vec::new();
    let mut notes: Vec<Vec<Inline>> = Vec::new();
    let mut para: Vec<&str> = Vec::new();
    let flush = |para: &mut Vec<&str>, blocks: &mut Vec<Block>| {
        if !para.is_empty() {
            blocks.push(Block::Paragraph(parse_inlines(&para.join(" "))));
            para.clear();
        }
    };
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let t = line.trim();
        if t.is_empty() {
            flush(&mut para, &mut blocks);
            i += 1;
        } else if let Some(lang) = t.strip_prefix("```") {
            flush(&mut para, &mut blocks);
            let mut body = Vec::new();
            i += 1;
            while i < lines.len() && !lines[i].trim_start().starts_with("```") {
                body.push(lines[i]);
                i += 1;
            }
            i += 1;
            let code: SharedString = body.join("\n").into();
            blocks.push(match lang.trim() {
                "mermaid" => Block::Mermaid(code),
                "math" | "tex" | "latex" => Block::Math(code),
                l => Block::Code {
                    language: l.to_string().into(),
                    code,
                },
            });
        } else if let Some(rest) = t.strip_prefix("$$") {
            flush(&mut para, &mut blocks);
            if let Some(one) = rest.strip_suffix("$$") {
                blocks.push(Block::Math(one.trim().to_string().into()));
                i += 1;
            } else {
                let mut body = vec![rest];
                i += 1;
                while i < lines.len() && !lines[i].trim().ends_with("$$") {
                    body.push(lines[i].trim());
                    i += 1;
                }
                if let Some(last) = lines.get(i) {
                    body.push(last.trim().trim_end_matches("$$"));
                }
                i += 1;
                blocks.push(Block::Math(body.join(" ").trim().to_string().into()));
            }
        } else if t.starts_with('#') {
            flush(&mut para, &mut blocks);
            let level = t.chars().take_while(|c| *c == '#').count();
            blocks.push(Block::Heading(
                level.clamp(1, 4) as u8,
                parse_inlines(t[level..].trim()),
            ));
            i += 1;
        } else if t == "---" || t == "***" {
            flush(&mut para, &mut blocks);
            blocks.push(Block::Rule);
            i += 1;
        } else if t.starts_with('>') {
            flush(&mut para, &mut blocks);
            let mut body = Vec::new();
            while i < lines.len() && lines[i].trim().starts_with('>') {
                body.push(lines[i].trim().trim_start_matches('>').trim());
                i += 1;
            }
            blocks.push(Block::Quote(parse_inlines(&body.join(" "))));
        } else if t.starts_with('|') {
            flush(&mut para, &mut blocks);
            let head = table_cells(t);
            i += 1;
            if i < lines.len() && lines[i].contains("---") {
                i += 1;
            }
            let mut rows = Vec::new();
            while i < lines.len() && lines[i].trim().starts_with('|') {
                rows.push(table_cells(lines[i]));
                i += 1;
            }
            blocks.push(Block::Table(head, rows));
        } else if let Some(rest) = t.strip_prefix("[^") {
            flush(&mut para, &mut blocks);
            let text = rest.split_once("]:").map_or(rest, |(_, r)| r).trim();
            notes.push(parse_inlines(text));
            i += 1;
        } else if list_line(line).is_some() {
            flush(&mut para, &mut blocks);
            let mut items = Vec::new();
            while i < lines.len() {
                match list_line(lines[i]) {
                    Some((indent, ordered, n, task, text)) => {
                        items.push((indent, ordered, n, task, text.to_string()));
                        i += 1;
                    }
                    None => break,
                }
            }
            blocks.push(build_list(&items));
        } else {
            para.push(t);
            i += 1;
        }
    }
    flush(&mut para, &mut blocks);
    if !notes.is_empty() {
        blocks.push(Block::Footnotes(notes));
    }
    blocks
}

// ── Geometry ────────────────────────────────────────────────────────────

/// `.g-prose { font-size: 0.875rem; line-height: 1.375rem }` (gemini-A).
pub const BODY: (f32, f32) = (14., 22.);
/// `.g-prose { max-width: 46rem }`.
pub const MAX_WIDTH: f32 = 736.;
/// `margin: 0 0 0.85em` at 14px.
const GAP: f32 = 14. * 0.85;

/// A block's (top, bottom) margins in px, before collapsing.
pub fn block_margins(block: &Block) -> (f32, f32) {
    match block {
        // `.g-prose h1…h4 { margin: 1.5em 0 0.5em }` at each heading's size.
        Block::Heading(level, _) => {
            let s = heading_size(*level).0;
            (1.5 * s, 0.5 * s)
        }
        // `.g-prose hr { margin: 1.5em 0 }`.
        Block::Rule => (21., 21.),
        // `.g-math { padding: var(--spacing-03) 0 }`, no margin.
        Block::Math(_) => (0., 0.),
        // `.footnotes { margin-top: 1.5em }` at its 12px.
        Block::Footnotes(_) => (18., 0.),
        _ => (0., GAP),
    }
}

/// The space above each block: adjacent margins collapse to the larger, and
/// the first block's top margin drops (`> :first-child { margin-top: 0 }`).
pub fn block_gaps(blocks: &[Block]) -> Vec<f32> {
    let mut out = Vec::with_capacity(blocks.len());
    let mut prev_bottom: Option<f32> = None;
    for b in blocks {
        let (top, bottom) = block_margins(b);
        out.push(prev_bottom.map_or(0., |p| p.max(top)));
        prev_bottom = Some(bottom);
    }
    out
}

/// (font size, weight): `.g-prose h1 { font-size: 1.5rem; font-weight: 400 }`,
/// `h2 1.25rem`, `h3 1rem`, `h4 0.875rem`, the rest 600.
pub fn heading_size(level: u8) -> (f32, u16) {
    match level {
        1 => (24., 400),
        2 => (20., 600),
        3 => (16., 600),
        _ => (14., 600),
    }
}

/// `1` → `¹`: footnote references as superscript digits.
pub fn superscript(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '0' => '⁰',
            '1' => '¹',
            '2' => '²',
            '3' => '³',
            '4' => '⁴',
            '5' => '⁵',
            '6' => '⁶',
            '7' => '⁷',
            '8' => '⁸',
            '9' => '⁹',
            c => c,
        })
        .collect()
}

// ── Inline text ─────────────────────────────────────────────────────────

/// The runs of `inlines` in `ink` at the body family in a paragraph of
/// `body` px, plus the byte ranges of inline code (with its padding).
fn runs(inlines: &[Inline], ink: Hsla, body: f32) -> (String, Vec<TextRun>, Vec<Range<usize>>) {
    let mut text = String::new();
    let mut runs = Vec::new();
    let mut code = Vec::new();
    let sans = |w: u16, italic: bool| {
        let mut f = font(FontFamilyKey::Sans, w);
        if italic {
            f.style = FontStyle::Italic;
        }
        f
    };
    for inline in inlines {
        if let Inline::Code(s) = inline {
            code.push(push_inline_code(&mut text, &mut runs, s, ink, body));
            continue;
        }
        let (s, f, c, underline) = match inline {
            Inline::Text(s) => (s.to_string(), sans(400, false), ink, false),
            Inline::Strong(s) => (s.to_string(), sans(600, false), ink, false),
            Inline::Em(s) => (s.to_string(), sans(400, true), ink, false),
            Inline::Link(t, _) => (
                t.to_string(),
                sans(400, false),
                color(ColorRole::LinkPrimary),
                true,
            ),
            Inline::FootnoteRef(n) => (
                superscript(n),
                sans(400, false),
                color(ColorRole::LinkPrimary),
                false,
            ),
            Inline::Code(_) => continue,
            Inline::Math(s) => (s.to_string(), font(FontFamilyKey::Mono, 400), ink, false),
        };
        text.push_str(&s);
        runs.push(TextRun {
            len: s.len(),
            font: f,
            color: c,
            background_color: None,
            underline: underline.then_some(UnderlineStyle {
                thickness: px(1.),
                color: Some(c),
                wavy: false,
            }),
            strikethrough: None,
        });
    }
    (text, runs, code)
}

/// Inline prose in a paragraph of `body` px: a `StyledText` with the
/// inline-code tint painted behind it.
fn inline_text(inlines: &[Inline], ink: Hsla, body: f32) -> Div {
    let (text, runs, code) = runs(inlines, ink, body);
    with_code_boxes(StyledText::new(text).with_runs(runs), code, body)
}

// ── Blocks ──────────────────────────────────────────────────────────────

/// The prose's user-facing strings (the reference's defaults).
#[derive(Clone, Debug)]
pub struct ProseTexts {
    /// The code block's copy name: `"Copy code"`.
    pub copy_code: SharedString,
    /// After copying: `"Copied"`.
    pub copied: SharedString,
    /// The diagram caption: `"Mermaid"`.
    pub mermaid: SharedString,
    /// The diagram's copy name: `"Copy source"`.
    pub copy_source: SharedString,
}

impl Default for ProseTexts {
    fn default() -> Self {
        Self {
            copy_code: "Copy code".into(),
            copied: "Copied".into(),
            mermaid: "Mermaid".into(),
            copy_source: "Copy source".into(),
        }
    }
}

/// A fenced code block in prose (`ProseCodeBlock`, `.g-pcode`): the
/// language in a `.g-codehead`, a copy button that turns into a checkmark
/// for 1.5s, then the code in 12px mono at 1.5 line height in syntax colors,
/// scrolling sideways.
#[derive(IntoElement)]
pub struct ProseCodeBlock {
    id: ElementId,
    language: SharedString,
    code: SharedString,
    texts: ProseTexts,
}

impl ProseCodeBlock {
    /// A block of `code` in `language`.
    pub fn new(
        id: impl Into<ElementId>,
        language: impl Into<SharedString>,
        code: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            language: language.into(),
            code: code.into(),
            texts: ProseTexts::default(),
        }
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: ProseTexts) -> Self {
        self.texts = texts;
        self
    }
}

/// `useFlash()`: 1500ms.
const FLASH_MS: u64 = 1500;

impl RenderOnce for ProseCodeBlock {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let state = flash_state((id.clone(), "flash"), window, cx);
        let copied = flash_on(&state, FLASH_MS, cx);
        let code = self.code.clone();
        let lines: Vec<AnyElement> = self
            .code
            .split('\n')
            .map(|l| {
                div()
                    .min_h(rem(18.))
                    .whitespace_nowrap()
                    .child(
                        StyledText::new(SharedString::from(l.to_string()))
                            .with_highlights(highlight(l).into_iter().map(|(r, k)| (r, k.style()))),
                    )
                    .into_any_element()
            })
            .collect();
        // `.g-pcode { margin: 0 0 0.85em; background: var(--layer-01) }`.
        div()
            .id(id.clone())
            .flex()
            .flex_col()
            .w_full()
            .bg(color(ColorRole::Layer01))
            .child(
                code_head()
                    .child(
                        mono(div(), 12., 400)
                            .text_color(color(ColorRole::TextSecondary))
                            .child(self.language.clone()),
                    )
                    .child(grow())
                    .child(
                        IconButton::new(
                            (id.clone(), "copy"),
                            if copied {
                                CarbonIcon::Checkmark
                            } else {
                                CarbonIcon::Copy
                            },
                            if copied {
                                self.texts.copied.clone()
                            } else {
                                self.texts.copy_code.clone()
                            },
                        )
                        .icon_px(14.)
                        .on_click(move |_, _, cx| {
                            cx.write_to_clipboard(ClipboardItem::new_string(code.to_string()));
                            flash(&state, FLASH_MS, cx);
                        }),
                    ),
            )
            .child(
                // `.g-pcode pre { padding: var(--spacing-04); overflow-x: auto;
                // font-size: 0.75rem; line-height: 1.5 }`.
                mono(div(), 12., 400)
                    .id((id, "pre"))
                    .overflow_x_scroll()
                    .p(tokens::SPACING_04)
                    .line_height(rem(18.))
                    .text_color(color(ColorRole::SyntaxVariable))
                    .child(div().flex().flex_col().children(lines)),
            )
    }
}

/// Display math (`MathBlock`, `.g-math`): the TeX source centred in 13px
/// mono with 8px above and below (the reference's no-KaTeX fallback).
#[derive(IntoElement)]
pub struct MathBlock {
    tex: SharedString,
}

impl MathBlock {
    /// A formula in TeX (`tex`).
    pub fn new(tex: impl Into<SharedString>) -> Self {
        Self { tex: tex.into() }
    }
}

impl RenderOnce for MathBlock {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        // `.g-math { padding: var(--spacing-03) 0; text-align: center;
        // font-family: var(--font-mono); font-size: 0.8125rem }`.
        mono(div(), 13., 400)
            .flex()
            .justify_center()
            .py(tokens::SPACING_03)
            .text_color(color(ColorRole::TextPrimary))
            .child(self.tex)
    }
}

/// A Mermaid diagram (`MermaidBlock`, `.g-mermaid`): a "Mermaid" caption
/// head with a copy button, then the source centred in `text-secondary`
/// mono (the reference's no-Mermaid fallback).
#[derive(IntoElement)]
pub struct MermaidBlock {
    id: ElementId,
    code: SharedString,
    texts: ProseTexts,
}

impl MermaidBlock {
    /// A diagram from its Mermaid `code`.
    pub fn new(id: impl Into<ElementId>, code: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            code: code.into(),
            texts: ProseTexts::default(),
        }
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: ProseTexts) -> Self {
        self.texts = texts;
        self
    }
}

impl RenderOnce for MermaidBlock {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        div()
            .id(id.clone())
            .flex()
            .flex_col()
            .w_full()
            .bg(color(ColorRole::Layer01))
            .child(
                code_head()
                    .child(head_icon(CarbonIcon::Flow))
                    .child(self.texts.mermaid.clone())
                    .child(grow())
                    .child(
                        CopyButton::new((id.clone(), "copy"), self.code.clone())
                            .label(self.texts.copy_source.clone())
                            .size(32.)
                            .quiet(true)
                            .layer(LayerLevel::One),
                    ),
            )
            .child(
                // `.g-mermaid-canvas { display: flex; justify-content: center;
                // padding: var(--spacing-04); overflow-x: auto } pre { color:
                // var(--text-secondary) }`.
                div()
                    .id((id, "canvas"))
                    .flex()
                    .justify_center()
                    .p(tokens::SPACING_04)
                    .overflow_x_scroll()
                    .child(
                        mono(div(), 12., 400)
                            .flex()
                            .flex_col()
                            .flex_none()
                            .text_color(color(ColorRole::TextSecondary))
                            .children(self.code.split('\n').map(|l| {
                                div()
                                    .whitespace_nowrap()
                                    .min_h(rem(16.))
                                    .child(SharedString::from(l.to_string()))
                            })),
                    ),
            )
    }
}

// ── MarkdownProse ───────────────────────────────────────────────────────

/// Typeset prose. See the module docs.
#[derive(IntoElement)]
pub struct MarkdownProse {
    id: ElementId,
    blocks: Vec<Block>,
    texts: ProseTexts,
}

impl MarkdownProse {
    /// Prose made of `blocks`.
    pub fn new(id: impl Into<ElementId>, blocks: impl IntoIterator<Item = Block>) -> Self {
        Self {
            id: id.into(),
            blocks: blocks.into_iter().collect(),
            texts: ProseTexts::default(),
        }
    }

    /// Prose read from Markdown with [`parse_markdown`].
    pub fn markdown(id: impl Into<ElementId>, src: &str) -> Self {
        Self::new(id, parse_markdown(src))
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: ProseTexts) -> Self {
        self.texts = texts;
        self
    }
}

/// A list's rows: a 1.5em marker column (`text-secondary` disc or number,
/// right-aligned), then the item and its nested lists.
fn list(ordered: bool, start: usize, items: &[ListItem], id: &ElementId, body: f32) -> Div {
    let ink = color(ColorRole::TextPrimary);
    div()
        .flex()
        .flex_col()
        .children(items.iter().enumerate().map(|(i, it)| {
            let marker = if ordered {
                format!("{}.", start + i)
            } else {
                "•".to_string()
            };
            // `.g-prose li { margin: 0.2em 0 }`.
            div()
                .flex()
                .when(i > 0, |d| d.mt(px(2.8)))
                .child(
                    // `.g-prose ul, ol { padding-left: 1.5em }`, `li::marker {
                    // color: var(--text-secondary) }`.
                    div()
                        .flex()
                        .flex_none()
                        .justify_end()
                        .w(rem(21.))
                        .pr(px(6.))
                        .text_color(color(ColorRole::TextSecondary))
                        .child(marker),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .min_w_0()
                        .child(inline_text(&it.content, ink, body))
                        .children(it.children.iter().map(|b| {
                            match b {
                                Block::List {
                                    ordered,
                                    start,
                                    items,
                                } => div()
                                    .mt(px(2.8))
                                    .child(list(*ordered, *start, items, id, body))
                                    .into_any_element(),
                                other => render_block(other, id, 0, &ProseTexts::default()),
                            }
                        })),
                )
        }))
}

/// A task-list checkbox (`accent-color: var(--interactive)`): 13px, filled
/// `interactive` with a white check when done, outlined otherwise.
fn task_box(checked: bool) -> Div {
    div()
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .size(rem(13.))
        .mt(px(5.))
        .rounded(px(2.))
        .when(checked, |d| {
            d.bg(color(ColorRole::Interactive)).child(
                Icon::carbon(CarbonIcon::Checkmark)
                    .size_length(rem(11.))
                    .color(color(ColorRole::TextOnColor)),
            )
        })
        .when(!checked, |d| {
            d.border_1().border_color(color(ColorRole::BorderStrong01))
        })
}

fn render_block(block: &Block, id: &ElementId, i: usize, texts: &ProseTexts) -> AnyElement {
    let ink = color(ColorRole::TextPrimary);
    let secondary = color(ColorRole::TextSecondary);
    let sub = |name: &str| -> ElementId {
        (id.clone(), SharedString::from(format!("{name}-{i}"))).into()
    };
    match block {
        Block::Heading(level, inlines) => {
            let (s, w) = heading_size(*level);
            div()
                .text_size(rem(s))
                .line_height(gpui_kit::relative(1.3))
                .font_weight(FontWeight(w as f32))
                .child(inline_text(inlines, ink, s))
                .into_any_element()
        }
        Block::Paragraph(inlines) => inline_text(inlines, ink, BODY.0).into_any_element(),
        Block::List {
            ordered,
            start,
            items,
        } => list(*ordered, *start, items, id, BODY.0).into_any_element(),
        // `.g-prose ul.contains-task-list { list-style: none; padding-left:
        // 0.25em } .task-list-item { display: flex; gap: var(--spacing-03) }`.
        Block::TaskList(items) => div()
            .flex()
            .flex_col()
            .pl(px(3.5))
            .children(items.iter().enumerate().map(|(k, (done, inlines))| {
                div()
                    .flex()
                    .items_start()
                    .gap(tokens::SPACING_03)
                    .when(k > 0, |d| d.mt(px(2.8)))
                    .child(task_box(*done))
                    .child(inline_text(inlines, ink, BODY.0))
            }))
            .into_any_element(),
        // `.g-prose blockquote { padding: 0 var(--spacing-05); border-left: 3px
        // solid var(--border-strong-01); color: var(--text-secondary) }`.
        Block::Quote(inlines) => div()
            .px(tokens::SPACING_05)
            .border_l(px(3.))
            .border_color(color(ColorRole::BorderStrong01))
            .child(inline_text(inlines, secondary, BODY.0))
            .into_any_element(),
        Block::Table(head, rows) => {
            // `.g-prose table { font-size: 0.8125rem; display: block; overflow-x:
            // auto } th, td { padding: var(--spacing-02) var(--spacing-04);
            // border-bottom: 1px solid var(--border-subtle-01) } th { background:
            // var(--layer-accent-01); font-weight: 600 }`. Columns size to their
            // content, so the table is laid out column by column.
            let cols = head.len().max(rows.iter().map(Vec::len).max().unwrap_or(0));
            let cell = |inlines: Option<&Vec<Inline>>, th: bool| {
                sans(div(), 13., if th { 600 } else { 400 })
                    .line_height(rem(20.))
                    .py(tokens::SPACING_02)
                    .px(tokens::SPACING_04)
                    .border_b_1()
                    .border_color(color(ColorRole::BorderSubtle01))
                    .when(th, |d| d.bg(color(ColorRole::LayerAccent01)))
                    .whitespace_nowrap()
                    .child(inline_text(inlines.map_or(&[][..], |v| v), ink, 13.))
            };
            div()
                .id(sub("table"))
                .flex()
                .overflow_x_scroll()
                .children((0..cols).map(|c| {
                    div()
                        .flex()
                        .flex_col()
                        .flex_none()
                        .child(cell(head.get(c), true))
                        .children(rows.iter().map(|r| cell(r.get(c), false)))
                }))
                .into_any_element()
        }
        // `.g-prose hr { border-top: 1px solid var(--border-subtle-01) }`.
        Block::Rule => div()
            .h(px(1.))
            .bg(color(ColorRole::BorderSubtle01))
            .into_any_element(),
        Block::Code { language, code } => {
            ProseCodeBlock::new(sub("code"), language.clone(), code.clone())
                .texts(texts.clone())
                .into_any_element()
        }
        Block::Math(tex) => MathBlock::new(tex.clone()).into_any_element(),
        Block::Mermaid(code) => MermaidBlock::new(sub("mermaid"), code.clone())
            .texts(texts.clone())
            .into_any_element(),
        // `.g-prose .footnotes { padding-top: var(--spacing-04); border-top: 1px
        // solid var(--border-subtle-01); font-size: 0.75rem; color:
        // var(--text-secondary) }`, an ordered list.
        Block::Footnotes(notes) => sans(div(), 12., 400)
            .pt(tokens::SPACING_04)
            .border_t_1()
            .border_color(color(ColorRole::BorderSubtle01))
            .text_color(secondary)
            .line_height(rem(16.))
            .child(list(
                true,
                1,
                &notes
                    .iter()
                    .map(|n| ListItem {
                        content: n.clone(),
                        children: Vec::new(),
                    })
                    .collect::<Vec<_>>(),
                id,
                12.,
            ))
            .into_any_element(),
    }
}

impl RenderOnce for MarkdownProse {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let gaps = block_gaps(&self.blocks);
        // `.g-prose { font-size: 0.875rem; line-height: 1.375rem; color:
        // var(--text-primary); max-width: 46rem }`.
        sans(div(), BODY.0, 400)
            .id(self.id.clone())
            .flex()
            .flex_col()
            .w_full()
            .max_w(rem(MAX_WIDTH))
            .line_height(rem(BODY.1))
            .text_color(color(ColorRole::TextPrimary))
            .children(self.blocks.iter().enumerate().map(|(i, b)| {
                div()
                    .mt(px(gaps[i]))
                    .child(render_block(b, &self.id, i, &self.texts))
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(s: &str) -> Inline {
        Inline::Text(s.to_string().into())
    }

    #[test]
    fn inlines() {
        assert_eq!(
            parse_inlines("The `g90` theme now matches [Carbon v11](#). Two[^1]:"),
            vec![
                t("The "),
                Inline::Code("g90".into()),
                t(" theme now matches "),
                Inline::Link("Carbon v11".into(), "#".into()),
                t(". Two"),
                Inline::FootnoteRef("1".into()),
                t(":"),
            ]
        );
        assert_eq!(
            parse_inlines("**a** *b* $c$ ` x"),
            vec![
                Inline::Strong("a".into()),
                t(" "),
                Inline::Em("b".into()),
                t(" "),
                Inline::Math("c".into()),
                t(" ` x"),
            ]
        );
    }

    #[test]
    fn blocks_from_the_reference_document() {
        let md = "## Token migration\n\nThe `g90` theme.\n\n- [x] Add tokens\n- [ ] Update the cover\n\n1. Read tokens\n2. Patch components\n   - Card\n   - Tabs\n\n> Layer tokens are contextual.\n\n| Token | White | G100 |\n|---|---|---|\n| layer-01 | #f4f4f4 | #262626 |\n\n```ts\nexport const x = 1;\n```\n\n$$c = \\sqrt{a^2 + b^2}$$\n\n```mermaid\ngraph LR\n  A --> B\n```\n\n[^1]: See the release notes.";
        let b = parse_markdown(md);
        assert!(matches!(b[0], Block::Heading(2, _)));
        assert!(matches!(b[1], Block::Paragraph(_)));
        assert_eq!(
            b[2],
            Block::TaskList(vec![
                (true, vec![t("Add tokens")]),
                (false, vec![t("Update the cover")])
            ])
        );
        match &b[3] {
            Block::List {
                ordered,
                start,
                items,
            } => {
                assert!(*ordered);
                assert_eq!(*start, 1);
                assert_eq!(items.len(), 2);
                assert!(
                    matches!(&items[1].children[0], Block::List { ordered: false, items, .. } if items.len() == 2)
                );
            }
            other => panic!("{other:?}"),
        }
        assert!(matches!(b[4], Block::Quote(_)));
        match &b[5] {
            Block::Table(head, rows) => {
                assert_eq!(head.len(), 3);
                assert_eq!(rows.len(), 1);
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(
            b[6],
            Block::Code {
                language: "ts".into(),
                code: "export const x = 1;".into()
            }
        );
        assert_eq!(b[7], Block::Math("c = \\sqrt{a^2 + b^2}".into()));
        assert_eq!(b[8], Block::Mermaid("graph LR\n  A --> B".into()));
        assert_eq!(
            b[9],
            Block::Footnotes(vec![vec![t("See the release notes.")]])
        );
    }

    #[test]
    fn margins_collapse() {
        let blocks = [
            Block::Heading(2, vec![]),
            Block::Paragraph(vec![]),
            Block::Heading(3, vec![]),
            Block::Rule,
            Block::Footnotes(vec![]),
        ];
        // First block: 0; h2's 10px bottom vs p's 0 top; p's 11.9 vs h3's 24;
        // h3's 8 vs hr's 21; hr's 21 vs footnotes' 18.
        assert_eq!(block_gaps(&blocks), vec![0., 10., 24., 21., 21.]);
        assert_eq!(heading_size(1), (24., 400));
        assert_eq!(superscript("12"), "¹²");
    }

    #[test]
    fn inline_code_is_the_shared_tint() {
        let (text, runs, code) = runs(
            &[t("a "), Inline::Code("x".into())],
            color(ColorRole::TextPrimary),
            BODY.0,
        );
        assert_eq!(&text[code[0].clone()], "\u{202F}x\u{202F}");
        assert!(runs.iter().all(|r| r.background_color.is_none()));
    }
}
