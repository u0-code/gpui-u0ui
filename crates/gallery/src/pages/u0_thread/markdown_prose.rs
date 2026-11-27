//! u0 · Thread › MarkdownProse: Markdown typed in the configuration is
//! read and typeset live; the example is the reference document (headings,
//! inline code, a link and a footnote, task and nested lists, a quote, a
//! table, a code block, math, a Mermaid diagram and the footnotes).
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, Entity, IntoElement, ParentElement, Render,
    Styled, Window, div, rems,
};
use u0ui::components::{Textarea, TextareaState};
use u0ui::prelude::*;
use u0ui::u0::thread::{
    Block, Inline, ListItem, MarkdownProse, MathBlock, MermaidBlock, ProseCodeBlock,
};

/// The reference document as Markdown (what the live playground starts with).
pub const DOC: &str = "## Token migration\n\nThe `g90` theme now matches [Carbon v11](#). Two things changed[^1]:\n\n- [x] Add layer-accent tokens\n- [ ] Update the cover\n\n1. Read tokens\n2. Patch components\n   - Card\n   - Tabs\n\n> Layer tokens are contextual; use the Layer component to step down.\n\n| Token | White | G100 |\n|---|---|---|\n| layer-01 | #f4f4f4 | #262626 |\n\n```ts\nexport const layer = (n: 1 | 2 | 3) => `var(--layer-0${n})`;\n```\n\n$$c = \\sqrt{a^2 + b^2}$$\n\n```mermaid\ngraph LR\n  A[Read tokens] --> B[Patch]\n  B --> C{Tests pass?}\n  C -->|yes| D[Ship]\n```\n\n[^1]: See the release notes for 11.86.";

fn t(s: &str) -> Inline {
    Inline::Text(s.to_string().into())
}

/// `components/MarkdownProse/preview.html`, built block by block.
fn reference_blocks() -> Vec<Block> {
    let item = |s: &str| ListItem {
        content: vec![t(s)],
        children: Vec::new(),
    };
    vec![
        Block::Heading(2, vec![t("Token migration")]),
        Block::Paragraph(vec![
            t("The "),
            Inline::Code("g90".into()),
            t(" theme now matches "),
            Inline::Link("Carbon v11".into(), "#".into()),
            t(". Two things changed"),
            Inline::FootnoteRef("1".into()),
            t(":"),
        ]),
        Block::TaskList(vec![
            (true, vec![t("Add layer-accent tokens")]),
            (false, vec![t("Update the cover")]),
        ]),
        Block::List {
            ordered: true,
            start: 1,
            items: vec![
                item("Read tokens"),
                ListItem {
                    content: vec![t("Patch components")],
                    children: vec![Block::List {
                        ordered: false,
                        start: 1,
                        items: vec![item("Card"), item("Tabs")],
                    }],
                },
            ],
        },
        Block::Quote(vec![t(
            "Layer tokens are contextual; use the Layer component to step down.",
        )]),
        Block::Table(
            vec![vec![t("Token")], vec![t("White")], vec![t("G100")]],
            vec![vec![
                vec![t("layer-01")],
                vec![t("#f4f4f4")],
                vec![t("#262626")],
            ]],
        ),
        Block::Code {
            language: "ts".into(),
            code: "export const layer = (n: 1 | 2 | 3) => `var(--layer-0${n})`;".into(),
        },
        Block::Math("c = \\sqrt{a^2 + b^2}".into()),
        Block::Mermaid(
            "graph LR\n  A[Read tokens] --> B[Patch]\n  B --> C{Tests pass?}\n  C -->|yes| D[Ship]"
                .into(),
        ),
        Block::Footnotes(vec![vec![t("See the release notes for 11.86.")]]),
    ]
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(MarkdownProse::markdown(
            "mp-thumb",
            "### Token migration\n\nThe `g90` theme now matches [Carbon v11](#).\n\n- [x] Add tokens",
        ))
        .into_any_element()
}

/// Build the page.
pub fn build(window: &mut Window, cx: &mut App) -> AnyView {
    let source = cx.new(|cx| TextareaState::new(window, cx).default_value(DOC));
    cx.new(|cx| {
        cx.observe(&source, |_, _, cx| cx.notify()).detach();
        Page { source }
    })
    .into()
}

struct Page {
    source: Entity<TextareaState>,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let src = self.source.read(cx).value().to_string();
        let live = MarkdownProse::markdown("mp-live", &src);
        let config = div().child(
            Textarea::new("mp-src", &self.source)
                .label("Markdown")
                .rows(14)
                .layer(LayerLevel::Two),
        );
        let parts = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .max_w(rems(46.))
            .child(ProseCodeBlock::new(
                "mp-code",
                "rust",
                "fn main() {\n    println!(\"hi\");\n}",
            ))
            .child(MathBlock::new("E = mc^2"))
            .child(MermaidBlock::new("mp-mmd", "graph TD\n  A --> B"));
        Body::new()
            .playground(live)
            .hint("Edit the Markdown: it is read and typeset as you type")
            .config(config)
            .example(controls::block(
                "Reference preview",
                MarkdownProse::new("mp-ref", reference_blocks()),
            ))
            .example(controls::block(
                "ProseCodeBlock, MathBlock, MermaidBlock",
                parts,
            ))
    }
}
