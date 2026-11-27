//! Ghost · Foundations › TerminalColors: the 16 ANSI colors on the
//! always-dark terminal, with the selection and cursor tokens.
//!
//! The terminal lines stand in for the Ghost panels' `TerminalView`: the
//! reference's SGR lines mapped through `ansi_sgr` on `terminal-background`.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, FontWeight, HighlightStyle, IntoElement,
    ParentElement, Render, SharedString, Styled, StyledText, Window, div, px, rems,
};
use u0ui::ghost::foundations::{ANSI, TokenSwatches, ansi_sgr};
use u0ui::prelude::*;

const LINES: [&str; 5] = [
    "\x1b[32m✓\x1b[0m 128 tests passed \x1b[90m(4.1s)\x1b[0m",
    "\x1b[33mwarn\x1b[0m  deprecated: \x1b[1mtheme.zone\x1b[0m",
    "\x1b[31m✗\x1b[0m src/date-picker.test.ts \x1b[90m› keyboard\x1b[0m",
    "\x1b[34minfo\x1b[0m  watching for changes…",
    "$ ",
];

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| TerminalPage { selection: false }).into()
}

struct TerminalPage {
    selection: bool,
}

/// Parse SGR escapes (`ESC[…m`: 0 reset, 1 bold, 30–37 / 90–97 colors) into
/// plain text and highlight runs.
pub fn sgr(line: &str) -> (String, Vec<(std::ops::Range<usize>, HighlightStyle)>) {
    let mut text = String::new();
    let mut runs = Vec::new();
    let mut style = HighlightStyle::default();
    let mut rest = line;
    while let Some(i) = rest.find("\x1b[") {
        let (head, tail) = rest.split_at(i);
        if !head.is_empty() {
            let start = text.len();
            text.push_str(head);
            runs.push((start..text.len(), style));
        }
        let Some(end) = tail.find('m') else { break };
        for code in tail[2..end].split(';').filter_map(|c| c.parse::<u8>().ok()) {
            match code {
                0 => style = HighlightStyle::default(),
                1 => style.font_weight = Some(FontWeight::SEMIBOLD),
                c => {
                    if let Some(role) = ansi_sgr(c) {
                        style.color = Some(color(role));
                    }
                }
            }
        }
        rest = &tail[end + 1..];
    }
    if !rest.is_empty() {
        let start = text.len();
        text.push_str(rest);
        runs.push((start..text.len(), style));
    }
    (text, runs)
}

fn terminal(selection: bool) -> impl IntoElement {
    let rows = LINES.iter().enumerate().map(|(i, l)| {
        let (text, runs) = sgr(l);
        let last = i == LINES.len() - 1;
        div()
            .flex()
            .h(rems(1.125))
            .items_center()
            .when_sel(selection && i == 1)
            .child(StyledText::new(SharedString::from(text)).with_highlights(runs))
            .children(last.then(|| {
                // `terminal-cursor`: a block cursor after the prompt.
                div()
                    .w(rems(0.45))
                    .h(rems(1.))
                    .bg(color(ColorRole::TerminalCursor))
            }))
    });
    // Six rows (`rows={6}`) of 12px mono on the dark terminal ground.
    div()
        .flex()
        .flex_col()
        .h(rems(6. * 1.125 + 1.5))
        .p(rems(0.75))
        .bg(color(ColorRole::TerminalBackground))
        .text_color(color(ColorRole::TerminalText))
        .type_style(TypeStyle::Code01)
        .children(rows)
}

trait Selected: Sized + Styled {
    fn when_sel(self, on: bool) -> Self {
        if on {
            self.bg(color(ColorRole::TerminalSelection))
        } else {
            self
        }
    }
}
impl<T: Styled> Selected for T {}

/// `components/TerminalColors/preview.html`.
pub fn reference(id: &'static str) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_05)
        .child(
            TokenSwatches::new(id, ANSI)
                .label("ANSI colours")
                .columns(8),
        )
        .child(terminal(false))
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .flex()
        .gap(px(2.))
        .children(
            ANSI[..8]
                .iter()
                .map(|r| div().size(rems(1.5)).bg(color(*r))),
        )
        .into_any_element()
}

impl Render for TerminalPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let config = controls::toggle(
            "terminal-selection",
            "show terminal-selection",
            self.selection,
            controls::setter(&entity, |this: &mut Self, on| this.selection = on),
        );
        Body::new()
            .playground(div().w_full().child(terminal(self.selection)))
            .hint("terminal-background stays dark in every theme")
            .config(config)
            .example(controls::block(
                "Reference preview",
                reference("ansi-swatches"),
            ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sgr_strips_escapes() {
        let (text, runs) = sgr("\x1b[32m✓\x1b[0m ok \x1b[1mbold\x1b[0m");
        assert_eq!(text, "✓ ok bold");
        assert_eq!(runs.len(), 3);
        assert_eq!(runs[2].1.font_weight, Some(FontWeight::SEMIBOLD));
    }
}
