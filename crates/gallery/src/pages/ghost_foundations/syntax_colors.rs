//! Ghost · Foundations › SyntaxColors: the 17 `syntax-*` tokens and a
//! highlighted snippet. Comments are also italic and `invalid` also
//! wavy-underlined, so color is never the only cue.
//!
//! The snippet stands in for the Ghost · Right column `CodeViewer`: the
//! reference's six lines, tokenized by hand, on `background` with line 4
//! on `highlight` behind a 2px `border-interactive` start bar.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, FontStyle, HighlightStyle, IntoElement,
    ParentElement, Render, SharedString, Styled, StyledText, UnderlineStyle, Window, div, px, rems,
};
use u0ui::ghost::foundations::{SYNTAX, TokenSwatches};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| SyntaxPage { highlight: true }).into()
}

struct SyntaxPage {
    highlight: bool,
}

use ColorRole::*;

/// The reference snippet (`src/theme.ts`), as (text, token) runs per line.
const CODE: &[&[(&str, Option<ColorRole>)]] = &[
    &[(
        "// Resolve a token for the active theme",
        Some(SyntaxComment),
    )],
    &[
        ("import", Some(SyntaxKeyword)),
        (" { ", Some(SyntaxPunctuation)),
        ("tokens", Some(SyntaxVariable)),
        (" } ", Some(SyntaxPunctuation)),
        ("from", Some(SyntaxKeyword)),
        (" ", None),
        ("'./tokens'", Some(SyntaxString)),
        (";", Some(SyntaxPunctuation)),
    ],
    &[
        ("export function", Some(SyntaxKeyword)),
        (" ", None),
        ("resolve", Some(SyntaxFunction)),
        ("(", Some(SyntaxPunctuation)),
        ("name", Some(SyntaxVariable)),
        (": ", Some(SyntaxPunctuation)),
        ("string", Some(SyntaxType)),
        (", ", Some(SyntaxPunctuation)),
        ("theme", Some(SyntaxVariable)),
        (" = ", Some(SyntaxOperator)),
        ("'g100'", Some(SyntaxString)),
        ("): ", Some(SyntaxPunctuation)),
        ("Color", Some(SyntaxType)),
        (" {", Some(SyntaxPunctuation)),
    ],
    &[
        ("  ", None),
        ("const", Some(SyntaxKeyword)),
        (" ", None),
        ("value", Some(SyntaxVariable)),
        (" = ", Some(SyntaxOperator)),
        ("tokens", Some(SyntaxVariable)),
        ("[", Some(SyntaxPunctuation)),
        ("theme", Some(SyntaxVariable)),
        ("][", Some(SyntaxPunctuation)),
        ("name", Some(SyntaxVariable)),
        ("] ", Some(SyntaxPunctuation)),
        ("??", Some(SyntaxOperator)),
        (" ", None),
        ("null", Some(SyntaxConstant)),
        (";", Some(SyntaxPunctuation)),
    ],
    &[
        ("  ", None),
        ("return", Some(SyntaxKeyword)),
        (" ", None),
        ("value", Some(SyntaxVariable)),
        (" === ", Some(SyntaxOperator)),
        ("null", Some(SyntaxConstant)),
        (" ? ", Some(SyntaxOperator)),
        ("fallback", Some(SyntaxFunction)),
        ("(", Some(SyntaxPunctuation)),
        ("name", Some(SyntaxVariable)),
        (", ", Some(SyntaxPunctuation)),
        ("0.5", Some(SyntaxNumber)),
        (") : ", Some(SyntaxPunctuation)),
        ("value", Some(SyntaxVariable)),
        (";", Some(SyntaxPunctuation)),
    ],
    &[("}", Some(SyntaxPunctuation))],
];

/// `.g-syn-*`: the token color; comments italic, `invalid` wavy-underlined,
/// headings 600, links underlined.
pub fn syntax_style(role: ColorRole) -> HighlightStyle {
    let mut s = HighlightStyle {
        color: Some(color(role)),
        ..Default::default()
    };
    match role {
        SyntaxComment => s.font_style = Some(FontStyle::Italic),
        SyntaxHeading => s.font_weight = Some(gpui_kit::FontWeight::SEMIBOLD),
        SyntaxLink | SyntaxInvalid => {
            s.underline = Some(UnderlineStyle {
                thickness: px(1.),
                color: Some(color(role)),
                wavy: role == SyntaxInvalid,
            })
        }
        _ => {}
    }
    s
}

fn code(highlight_line: Option<usize>) -> impl IntoElement {
    let lines = CODE.iter().enumerate().map(|(i, runs)| {
        let text: String = runs.iter().map(|(t, _)| *t).collect();
        let mut at = 0;
        let highlights: Vec<_> = runs
            .iter()
            .filter_map(|(t, role)| {
                let range = at..at + t.len();
                at += t.len();
                role.map(|r| (range, syntax_style(r)))
            })
            .collect();
        let on = highlight_line == Some(i + 1);
        div()
            .flex()
            .when_on(on)
            .child(
                div()
                    .w(rems(2.5))
                    .flex_none()
                    .pr(rems(0.75))
                    .text_right()
                    .text_color(color(TextHelper))
                    .child(format!("{}", i + 1)),
            )
            .child(
                div()
                    .text_color(color(SyntaxVariable))
                    .whitespace_nowrap()
                    .child(StyledText::new(SharedString::from(text)).with_highlights(highlights)),
            )
    });
    div()
        .flex()
        .flex_col()
        .py(tokens::SPACING_03)
        .bg(color(Background))
        .border_1()
        .border_color(color(BorderSubtle01))
        .type_style(TypeStyle::Code01)
        .line_height(rems(1.25))
        .overflow_hidden()
        .children(lines)
}

trait WhenOn: Sized + Styled {
    /// The highlighted line: `highlight` + 2px `border-interactive` start bar.
    fn when_on(self, on: bool) -> Self {
        if on {
            self.bg(color(Highlight))
                .border_l(px(2.))
                .border_color(color(BorderInteractive))
        } else {
            self.border_l(px(2.))
                .border_color(gpui_kit::Hsla::transparent_black())
        }
    }
}
impl<T: Styled> WhenOn for T {}

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
        .child(code(Some(4)))
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div().w(rems(15.)).child(code(Some(4))).into_any_element()
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
            .playground(div().w_full().child(code(self.highlight.then_some(4))))
            .config(config)
            .example(controls::block(
                "Reference preview",
                reference("syntax-swatches"),
            ))
    }
}
