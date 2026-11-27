//! Ghost · Right column › CodeViewer: the live viewer (head, wrap, start
//! line, highlighted lines, height) and the reference file.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, prelude::FluentBuilder as _, rems,
};
use u0ui::ghost::panels::CodeViewer;
use u0ui::prelude::*;

/// `components/CodeViewer/preview.html`'s file.
pub const CODE: &str = "import { tokens } from './tokens';\n\n// Resolve a token for the active theme\nexport function resolve(name: string, theme = 'g100') {\n  const value = tokens[theme]?.[name];\n  if (value === undefined) throw new Error(`Unknown token ${name}`);\n  return value;\n}\n\nexport const DEFAULT_THEME = 'white';";

const LANGS: [&str; 3] = ["TypeScript", "Rust", "none"];
const RUST: &str = "use crate::theme::color;\n\n/// The ink for a token.\npub fn ink(name: &str) -> Option<Hsla> {\n    match name {\n        \"focus\" => Some(color(ColorRole::Focus)),\n        _ => None, // a deliberately long trailing comment that runs past the panel edge\n    }\n}\n";
const HEIGHTS: [&str; 3] = ["250", "360", "120"];
const STARTS: [&str; 3] = ["1", "12", "998"];

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        head: true,
        wrap: false,
        highlight: true,
        lang: 0,
        start: 1,
        height: 0,
    })
    .into()
}

struct Page {
    head: bool,
    wrap: bool,
    highlight: bool,
    lang: usize,
    /// Index into [`STARTS`].
    start: usize,
    height: usize,
}

/// The reference viewer.
fn reference() -> impl IntoElement {
    CodeViewer::new("cv-ref", CODE)
        .path("src/theme/resolve.ts")
        .language("TypeScript")
        .start_line(12)
        .highlight_lines([15, 16])
        .max_height(250.)
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(
            CodeViewer::new(
                "cv-thumb",
                "// Resolve a token\nexport function resolve(name) {\n  return tokens[name];\n}",
            )
            .highlight_lines([2])
            .max_height(80.),
        )
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let e = cx.entity();
        let (code, path) = if self.lang == 1 {
            (RUST, "crates/u0ui/src/ink.rs")
        } else {
            (CODE, "src/theme/resolve.ts")
        };
        let start: usize = STARTS[self.start].parse().unwrap_or(1);
        let live = div().w_full().max_w(rems(40.)).child(
            CodeViewer::new("cv-live", code)
                .when(self.head, |v| v.path(path))
                .when(self.lang < 2, |v| v.language(LANGS[self.lang]))
                .start_line(start)
                .when(self.highlight, |v| {
                    v.highlight_lines([start + 3, start + 4])
                })
                .wrap(self.wrap)
                .max_height(HEIGHTS[self.height].parse().unwrap_or(360.)),
        );
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::toggle(
                "cv-head",
                "path (head)",
                self.head,
                controls::setter(&e, |p: &mut Page, v| p.head = v),
            ))
            .child(controls::toggle(
                "cv-wrap",
                "wrap",
                self.wrap,
                controls::setter(&e, |p: &mut Page, v| p.wrap = v),
            ))
            .child(controls::toggle(
                "cv-hl",
                "highlightLines",
                self.highlight,
                controls::setter(&e, |p: &mut Page, v| p.highlight = v),
            ))
            .child(controls::field(
                "language",
                controls::choices(
                    "cv-lang",
                    &LANGS,
                    self.lang,
                    controls::setter(&e, |p: &mut Page, v| p.lang = v),
                ),
            ))
            .child(controls::field(
                "startLine",
                controls::choices(
                    "cv-start",
                    &STARTS,
                    self.start,
                    controls::setter(&e, |p: &mut Page, v| p.start = v),
                ),
            ))
            .child(controls::field(
                "maxHeight",
                controls::choices(
                    "cv-height",
                    &HEIGHTS,
                    self.height,
                    controls::setter(&e, |p: &mut Page, v| p.height = v),
                ),
            ));
        Body::new()
            .playground(live)
            .hint("Tab into the code, then ↑/↓, PageUp/PageDown, Home/End scroll it")
            .config(config)
            .example(controls::block("Reference preview", reference()))
    }
}
