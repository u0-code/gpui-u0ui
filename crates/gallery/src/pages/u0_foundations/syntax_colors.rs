//! u0 · Foundations › SyntaxColors: the 17 `syntax-*` tokens and a
//! highlighted snippet in the u0 · Right column `CodeViewer`. Comments
//! are also italic and `invalid` also wavy-underlined, so color is never the
//! only cue.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, rems,
};
use u0ui::prelude::*;
use u0ui::u0::foundations::{SYNTAX, TokenSwatches};
use u0ui::u0::panels::CodeViewer;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| SyntaxPage { highlight: true }).into()
}

struct SyntaxPage {
    highlight: bool,
}

/// The reference snippet (`src/theme.ts`).
const CODE: &str = "// Resolve a token for the active theme\n\
import { tokens } from './tokens';\n\
export function resolve(name: string, theme = 'g100'): Color {\n  \
const value = tokens[theme][name] ?? null;\n  \
return value === null ? fallback(name, 0.5) : value;\n\
}";

/// `h(C.CodeViewer, {path: 'src/theme.ts', language: 'TypeScript',
/// highlightLines: [4], maxHeight: 150, code})`.
fn code(id: &'static str, highlight_line: Option<usize>) -> CodeViewer {
    CodeViewer::new(id, CODE)
        .path("src/theme.ts")
        .language("TypeScript")
        .highlight_lines(highlight_line)
        .max_height(150.)
}

/// `components/SyntaxColors/preview.html`.
pub fn reference(id: &'static str) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_05)
        .child(
            TokenSwatches::new(id, SYNTAX)
                .label("Syntax tokens")
                .columns(6)
                .sample("Aa"),
        )
        .child(code("syntax-ref-code", Some(4)))
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(code("syntax-thumb-code", Some(4)).max_height(80.))
        .into_any_element()
}

impl Render for SyntaxPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let config = controls::toggle(
            "syntax-highlight",
            "highlightLines [4]",
            self.highlight,
            controls::setter(&entity, |this: &mut Self, on| this.highlight = on),
        );
        Body::new()
            .playground(
                div()
                    .w_full()
                    .child(code("syntax-live-code", self.highlight.then_some(4))),
            )
            .config(config)
            .example(controls::block(
                "Reference preview",
                reference("syntax-swatches"),
            ))
    }
}
