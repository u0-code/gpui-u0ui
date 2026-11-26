//! Data display › CodeBlock: the reference blocks (install.sh, the theme.ts
//! diff with line numbers), a live block with every prop, and Carbon's
//! CodeSnippet single / multi (with show more) / inline. Press a copy button
//! to see the "Copied!" feedback.
use crate::controls;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled, Window,
    div, rems,
};
use u0ui::components::code_block::{
    CodeBlock, CodeBlockSize, CodeBlockVariant, CodeLine, CodeSnippet, SnippetType,
};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| CodeBlockPage {
        header: true,
        line_numbers: false,
        wrap: false,
        size: 0,
        variant: 0,
        footer: false,
        diff: false,
    })
    .into()
}

struct CodeBlockPage {
    header: bool,
    line_numbers: bool,
    wrap: bool,
    size: usize,
    variant: usize,
    footer: bool,
    diff: bool,
}

const COPY: (&str, &str) = ("Copy to clipboard", "Copied!");

fn theme_lines() -> Vec<CodeLine> {
    vec![
        CodeLine::new("import { Theme } from 'carbon-ui';"),
        CodeLine::removed("const theme = 'white';"),
        CodeLine::added("const theme = 'g100';"),
        CodeLine::new("export default theme;"),
    ]
}

impl Render for CodeBlockPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        // `components/CodeBlock/preview.html`: `cu-stack`, max-width 560.
        let reference = div()
            .max_w(rems(35.))
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(
                CodeBlock::new(
                    "ref-install",
                    "npm install @carbon/react\nnpm install carbon-ui",
                )
                .title("install.sh")
                .language("bash")
                .copy_texts(COPY.0, COPY.1),
            )
            .child(
                CodeBlock::new("ref-theme", "")
                    .title("theme.ts")
                    .language("diff")
                    .line_numbers(true)
                    .lines(theme_lines())
                    .copy_texts(COPY.0, COPY.1),
            );

        let mut live = CodeBlock::new(
            "live",
            "const longLine = 'This line is long enough to scroll horizontally unless wrap is on, which shows how the block reflows.';\nconsole.log(longLine);",
        )
        .line_numbers(self.line_numbers)
        .wrap(self.wrap)
        .size(CodeBlockSize::ALL[self.size])
        .variant(CodeBlockVariant::ALL[self.variant])
        .copy_texts(COPY.0, COPY.1);
        if self.diff {
            live = live.lines(
                theme_lines()
                    .into_iter()
                    .chain([CodeLine::highlighted("// highlighted line")]),
            );
        }
        if self.header {
            live = live.title("example.ts").language("typescript");
        }
        if self.footer {
            live = live.footer("Exit code 0 · 12 ms");
        }

        let multi: String = (1..=20)
            .map(|i| format!("{i:02}  kubectl get pods --namespace production-{i}"))
            .collect::<Vec<_>>()
            .join("\n");
        let snippets = div()
            .max_w(rems(35.))
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(
                CodeSnippet::new(
                    "snippet-single",
                    SnippetType::Single,
                    "yarn add @carbon/react @carbon/styles carbon-ui --exact --registry https://registry.npmjs.org",
                )
                .copy_texts(COPY.0, COPY.1),
            )
            .child(
                CodeSnippet::new("snippet-multi", SnippetType::Multi, multi)
                    .copy_texts(COPY.0, COPY.1)
                    .expand_texts("Show more", "Show less"),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .items_center()
                    .type_style(TypeStyle::BodyCompact01)
                    .child("Run\u{a0}")
                    .child(
                        CodeSnippet::new("snippet-inline", SnippetType::Inline, "npm run build")
                            .copy_texts(COPY.0, COPY.1),
                    )
                    .child("\u{a0}before you deploy."),
            )
            .child(
                CodeSnippet::new("snippet-disabled", SnippetType::Single, "disabled snippet")
                    .disabled(true)
                    .copy_texts(COPY.0, COPY.1),
            );

        let sizes = CodeBlockSize::ALL.map(|s| s.name());
        let variants = CodeBlockVariant::ALL.map(|s| s.name());
        let t =
            |id: &'static str, label: &'static str, on: bool, f: fn(&mut CodeBlockPage, bool)| {
                controls::toggle(id, label, on, controls::setter(&entity, f))
            };
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "size",
                controls::choices(
                    "code-size",
                    &sizes,
                    self.size,
                    controls::setter(&entity, |this: &mut CodeBlockPage, i| this.size = i),
                ),
            ))
            .child(controls::field(
                "variant",
                controls::choices(
                    "code-variant",
                    &variants,
                    self.variant,
                    controls::setter(&entity, |this: &mut CodeBlockPage, i| this.variant = i),
                ),
            ))
            .child(t("code-header", "title + language", self.header, |p, v| {
                p.header = v
            }))
            .child(t("code-ln", "lineNumbers", self.line_numbers, |p, v| {
                p.line_numbers = v
            }))
            .child(t("code-wrap", "wrap", self.wrap, |p, v| p.wrap = v))
            .child(t("code-diff", "lines with variants", self.diff, |p, v| {
                p.diff = v
            }))
            .child(t("code-footer", "footer", self.footer, |p, v| p.footer = v));

        controls::page_body(
            div()
                .flex()
                .flex_col()
                .gap(tokens::SPACING_07)
                .child(controls::block("Reference preview", reference))
                .child(controls::block("Live", div().max_w(rems(35.)).child(live)))
                .child(controls::block("Carbon CodeSnippet", snippets)),
            Some(config.into_any_element()),
        )
    }
}
