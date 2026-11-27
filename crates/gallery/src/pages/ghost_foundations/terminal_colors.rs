//! Ghost · Foundations › TerminalColors: the 16 ANSI colors on the
//! always-dark terminal (the Ghost panels' `TerminalView`), with the
//! selection and cursor tokens.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, prelude::FluentBuilder as _, px, rems,
};
use u0ui::ghost::foundations::{ANSI, TokenSwatches};
use u0ui::ghost::panels::TerminalView;
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

/// `h(C.TerminalView, {rows: 6, lines})`; the live page can select the
/// second line to show `terminal-selection`.
fn terminal(id: &'static str, selection: bool) -> impl IntoElement {
    TerminalView::new(id)
        .rows(6)
        .lines(LINES)
        .when(selection, |t| t.selection(1, 1))
}

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
        .child(terminal("ansi-ref-term", false))
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
            .playground(
                div()
                    .w_full()
                    .child(terminal("ansi-live-term", self.selection)),
            )
            .hint("terminal-background stays dark in every theme")
            .config(config)
            .example(controls::block(
                "Reference preview",
                reference("ansi-swatches"),
            ))
    }
}
