//! Typography: a prose container (shadcn Typography / Typeset, `.cu-typeset*`
//! in `custom.css`) that sets Carbon type tokens on rendered markdown-like
//! blocks.
//!
//! GPUI has no cascading HTML styles, so the blocks are typed
//! ([`TypeBlock`]): `h1` `heading-06` (42/1.199, 300), `h2` `heading-05`
//! (32/1.25), `h3` `heading-03` (20/1.4), `h4` `heading-02` (16/1.5, 600),
//! paragraphs and lists `body-02` (16/24), quotes 20/28 light with a 2px
//! `border-strong-01` rule, tables 14px with `layer-accent-01` headers.
//!
//! - **Spacing**: each block keeps the CSS margins (`h2 { margin: spacing-09
//!   0 spacing-05 }`, `p { margin: 0 0 spacing-05 }`, …); between two blocks
//!   the larger of the adjoining margins applies, as CSS margins collapse
//!   ([`gap_between`]). The first block's top margin and the last block's
//!   bottom margin collapse through the container, as they do in the
//!   reference (the container has no padding or border).
//! - **Inline runs** ([`Inline`]): text, links (`link-primary`, underlined,
//!   opened with the system handler) and code (IBM Plex Mono on `layer-01`).
//!   Deviations: GPUI text runs share one font size and have no padding, so
//!   inline code is 16px (CSS: 14px) without its 4px side padding.
//! - **Quote font**: IBM Plex Serif Light (bundled), as the reference sets.
use crate::{
    theme::color,
    tokens::{ColorRole, FontFamilyKey, TypeStyle},
    typography::{TypeStyled as _, font},
};
use gpui_kit::{
    AnyElement, App, ElementId, Font, Hsla, InteractiveText, IntoElement, ParentElement,
    RenderOnce, SharedString, Styled, StyledText, TextRun, UnderlineStyle, Window, div,
    prelude::FluentBuilder as _, px, rems,
};

/// `.cu-typeset { max-width: 40rem }` in px.
pub const MAX_WIDTH: f32 = 640.;

/// An inline run in a paragraph, list item or table cell.
#[derive(Clone, Debug, PartialEq)]
pub enum Inline {
    /// Plain text.
    Text(SharedString),
    /// `<a>`: text and target.
    Link(SharedString, SharedString),
    /// `<code>`.
    Code(SharedString),
}

impl From<&'static str> for Inline {
    fn from(text: &'static str) -> Self {
        Inline::Text(text.into())
    }
}

/// A block of the typeset.
#[derive(Clone, Debug, PartialEq)]
pub enum TypeBlock {
    /// `h1`–`h4` (level 1..=4) with its text.
    Heading(u8, SharedString),
    /// `p`.
    Paragraph(Vec<Inline>),
    /// `ul` (`ordered = false`) or `ol`, one inline list per item.
    List {
        /// `ol` when true.
        ordered: bool,
        /// The items.
        items: Vec<Vec<Inline>>,
    },
    /// `blockquote`.
    Quote(SharedString),
    /// `table`: header cells and rows.
    Table {
        /// `th` cells.
        header: Vec<SharedString>,
        /// `td` rows.
        rows: Vec<Vec<SharedString>>,
    },
}

impl TypeBlock {
    /// (margin-top, margin-bottom) in px from `.cu-typeset h1…table`.
    pub fn margins(&self) -> (f32, f32) {
        match self {
            // `h1 { margin: 0 0 var(--spacing-06) }`.
            TypeBlock::Heading(1, _) => (0., 24.),
            // `h2 { margin: var(--spacing-09) 0 var(--spacing-05) }`.
            TypeBlock::Heading(2, _) => (48., 16.),
            // `h3 { margin: var(--spacing-07) 0 var(--spacing-03) }`.
            TypeBlock::Heading(3, _) => (32., 8.),
            // `h4 { margin: var(--spacing-06) 0 var(--spacing-02) }`.
            TypeBlock::Heading(..) => (24., 4.),
            // `p, ul, ol, blockquote, table { margin: 0 0 var(--spacing-05) }`.
            _ => (0., 16.),
        }
    }

    /// The heading's type style (`h1` heading-06 … `h4` heading-02).
    pub fn heading_style(level: u8) -> TypeStyle {
        match level {
            1 => TypeStyle::Heading06,
            2 => TypeStyle::Heading05,
            3 => TypeStyle::Heading03,
            _ => TypeStyle::Heading02,
        }
    }
}

/// The space between two adjacent blocks: CSS collapses the previous
/// block's bottom margin with the next one's top margin to the larger.
pub fn gap_between(prev: &TypeBlock, next: &TypeBlock) -> f32 {
    prev.margins().1.max(next.margins().0)
}

/// Text, its runs, and the link ranges with their targets.
type Runs = (
    String,
    Vec<TextRun>,
    Vec<(std::ops::Range<usize>, SharedString)>,
);

fn body_font() -> Font {
    font(FontFamilyKey::Sans, 400)
}

/// Build styled runs for `inlines`; returns the text, its runs and the
/// clickable link ranges with their targets.
fn runs(inlines: &[Inline], base: Hsla) -> Runs {
    let mut text = String::new();
    let mut runs = Vec::new();
    let mut links = Vec::new();
    for inline in inlines {
        let (s, run_font, run_color, background, underline) = match inline {
            Inline::Text(s) => (s.clone(), body_font(), base, None, None),
            // `.cu-typeset a { color: var(--link-primary) }` (underlined, as a browser link).
            Inline::Link(s, _) => {
                let link = color(ColorRole::LinkPrimary);
                (
                    s.clone(),
                    body_font(),
                    link,
                    None,
                    Some(UnderlineStyle {
                        thickness: px(1.),
                        color: Some(link),
                        wavy: false,
                    }),
                )
            }
            // `.cu-typeset code { font-family: var(--font-mono); background: var(--layer-01) }`.
            Inline::Code(s) => (
                s.clone(),
                font(FontFamilyKey::Mono, 400),
                base,
                Some(color(ColorRole::Layer01)),
                None,
            ),
        };
        let start = text.len();
        text.push_str(&s);
        if let Inline::Link(_, href) = inline {
            links.push((start..text.len(), href.clone()));
        }
        runs.push(TextRun {
            len: s.len(),
            font: run_font,
            color: run_color,
            background_color: background,
            underline,
            strikethrough: None,
        });
    }
    (text, runs, links)
}

fn rich(id: ElementId, inlines: &[Inline], base: Hsla) -> AnyElement {
    let (text, text_runs, links) = runs(inlines, base);
    let styled = StyledText::new(text).with_runs(text_runs);
    if links.is_empty() {
        return styled.into_any_element();
    }
    let (ranges, targets): (Vec<_>, Vec<_>) = links.into_iter().unzip();
    InteractiveText::new(id, styled)
        .on_click(ranges, move |ix, _, cx| cx.open_url(&targets[ix]))
        .into_any_element()
}

/// A Carbon typeset. See the module docs.
#[derive(IntoElement)]
pub struct Typography {
    id: ElementId,
    blocks: Vec<TypeBlock>,
}

impl Typography {
    /// An empty typeset with a stable `id` (for link hit-testing).
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            blocks: Vec::new(),
        }
    }

    /// Append a block.
    pub fn block(mut self, block: TypeBlock) -> Self {
        self.blocks.push(block);
        self
    }

    /// Append blocks.
    pub fn blocks(mut self, blocks: impl IntoIterator<Item = TypeBlock>) -> Self {
        self.blocks.extend(blocks);
        self
    }
}

fn render_block(id: &ElementId, i: usize, block: &TypeBlock) -> AnyElement {
    let primary = color(ColorRole::TextPrimary);
    let block_id = |n: usize| {
        ElementId::NamedChild(std::sync::Arc::new(id.clone()), format!("{i}-{n}").into())
    };
    match block {
        TypeBlock::Heading(level, text) => div()
            .type_style(TypeBlock::heading_style(*level))
            .text_color(primary)
            .child(text.clone())
            .into_any_element(),
        TypeBlock::Paragraph(inlines) => div()
            .type_style(TypeStyle::Body02)
            .child(rich(block_id(0), inlines, primary))
            .into_any_element(),
        TypeBlock::List { ordered, items } => div()
            .type_style(TypeStyle::Body02)
            .flex()
            .flex_col()
            // `ul { padding-left: var(--spacing-05) }`, `ol { padding-left: var(--spacing-06) }`.
            .pl(rems(if *ordered { 1.5 } else { 1. }))
            .children(items.iter().enumerate().map(|(n, item)| {
                let marker: SharedString = if *ordered {
                    format!("{}.", n + 1).into()
                } else {
                    // `ul > li::before { content: "–"; margin-left: -spacing-05; width: spacing-05 }`.
                    "–".into()
                };
                div()
                    .relative()
                    .child(
                        div()
                            .absolute()
                            .top_0()
                            .left(rems(if *ordered { -1.5 } else { -1. }))
                            .w(rems(if *ordered { 1.25 } else { 1. }))
                            .flex()
                            .when(*ordered, |this| this.justify_end())
                            .child(marker),
                    )
                    .child(rich(block_id(n), item, primary))
            }))
            .into_any_element(),
        // `blockquote { padding-left: spacing-05; border-left: 2px solid border-strong-01;
        // font-family: var(--font-serif); font-size: 1.25rem; line-height: 1.4; font-weight: 300 }`.
        TypeBlock::Quote(text) => div()
            .pl(rems(1.))
            .border_l_2()
            .border_color(color(ColorRole::BorderStrong01))
            .font(font(FontFamilyKey::Serif, 300))
            .text_size(rems(1.25))
            .line_height(rems(1.75))
            .text_color(primary)
            .child(text.clone())
            .into_any_element(),
        TypeBlock::Table { header, rows } => {
            let cell = || div().flex_1().min_w_0().px(rems(1.)).py(rems(0.75));
            div()
                .flex()
                .flex_col()
                .w_full()
                .text_size(rems(0.875))
                .line_height(rems(1.25))
                .font(body_font())
                .child(
                    // `th { background: var(--layer-accent-01); font-weight: 600; padding: spacing-04 spacing-05 }`.
                    div()
                        .flex()
                        .flex_row()
                        .bg(color(ColorRole::LayerAccent01))
                        .font(font(FontFamilyKey::Sans, 600))
                        .text_color(primary)
                        .children(header.iter().map(|h| cell().child(h.clone()))),
                )
                .children(rows.iter().map(|row| {
                    // `td { border-bottom: 1px solid var(--border-subtle-01); color: var(--text-secondary) }`.
                    div()
                        .flex()
                        .flex_row()
                        .border_b_1()
                        .border_color(color(ColorRole::BorderSubtle01))
                        .text_color(color(ColorRole::TextSecondary))
                        .children(row.iter().map(|c| cell().child(c.clone())))
                }))
                .into_any_element()
        }
    }
}

impl RenderOnce for Typography {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let mut column = div()
            .flex()
            .flex_col()
            .w_full()
            .max_w(rems(MAX_WIDTH / 16.))
            .text_color(color(ColorRole::TextPrimary));
        for (i, block) in self.blocks.iter().enumerate() {
            let top = if i == 0 {
                0.
            } else {
                gap_between(&self.blocks[i - 1], block)
            };
            column = column.child(
                div()
                    .mt(rems(top / 16.))
                    .child(render_block(&self.id, i, block)),
            );
        }
        column
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn margins_collapse_like_css() {
        let h2 = TypeBlock::Heading(2, "Getting started".into());
        let p = TypeBlock::Paragraph(vec!["Text".into()]);
        let h3 = TypeBlock::Heading(3, "Next steps".into());
        let quote = TypeBlock::Quote("Q".into());
        assert_eq!(gap_between(&h2, &p), 16.);
        // p's 16px bottom vs h3's 32px top: 32.
        assert_eq!(gap_between(&quote, &h3), 32.);
        assert_eq!(TypeBlock::Heading(1, "".into()).margins(), (0., 24.));
        assert_eq!(TypeBlock::Heading(4, "".into()).margins(), (24., 4.));
        assert_eq!(TypeBlock::heading_style(1), TypeStyle::Heading06);
        assert_eq!(TypeBlock::heading_style(2), TypeStyle::Heading05);
    }

    #[test]
    fn inline_runs_cover_the_text() {
        crate::theme::tests_set(crate::tokens::CarbonTheme::White);
        let inlines = vec![
            Inline::from("Carbon UI pairs "),
            Inline::Link("IBM Carbon".into(), "https://carbondesignsystem.com".into()),
            Inline::Code("body-01".into()),
        ];
        let (text, runs, links) = runs(&inlines, color(ColorRole::TextPrimary));
        assert_eq!(runs.iter().map(|r| r.len).sum::<usize>(), text.len());
        assert_eq!(links.len(), 1);
        assert_eq!(&text[links[0].0.clone()], "IBM Carbon");
        assert_eq!(runs[1].color, color(ColorRole::LinkPrimary));
        assert_eq!(runs[2].background_color, Some(color(ColorRole::Layer01)));
    }
}
