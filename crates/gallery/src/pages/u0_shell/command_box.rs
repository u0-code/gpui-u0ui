//! u0 · Shell CommandBox: the command centre in a top bar, its palette
//! opened by a press.
use super::last_action;
use crate::controls;
use crate::page::Body;
use crate::pages::t3_shell::stack;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, px,
};
use u0ui::prelude::*;
use u0ui::u0::shell::CommandBox;

const LABEL: &str = "Search threads, projects, commands and settings";

/// A 38px top bar on `background`, as the design places the box: the
/// title left and the buttons right, each `flex: 1 1 0`, the box between
/// them `flex: 0 1 400px; min-width: 32px`.
fn bar(width: f32, title: &'static str, command: CommandBox) -> gpui_kit::Div {
    let side = || div().flex().flex_1().flex_basis(px(0.)).items_center();
    div()
        .flex()
        .items_center()
        .gap(tokens::SPACING_03)
        .w(px(width))
        .max_w_full()
        .h(px(38.))
        .px(tokens::SPACING_05)
        .bg(color(ColorRole::Background))
        .border_b_1()
        .border_color(color(ColorRole::BorderSubtle01))
        .type_style(TypeStyle::BodyCompact01)
        .text_color(color(ColorRole::TextPrimary))
        .child(side().child(div().whitespace_nowrap().child(title)))
        .child(
            div()
                .flex()
                .flex_shrink(1.)
                .flex_basis(px(400.))
                .min_w(px(32.))
                .child(command),
        )
        .child(side().justify_end().child(div().w(px(64.)).h(px(28.))))
}

struct CommandBoxPage {
    open: bool,
    shortcut: bool,
    action: SharedString,
}

/// Build the CommandBox page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| CommandBoxPage {
        open: false,
        shortcut: true,
        action: "none".into(),
    })
    .into()
}

/// Overview specimen.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(px(240.))
        .child(CommandBox::new("thumb-cmdbox", "Search threads").shortcut("⌘K"))
        .into_any_element()
}

impl Render for CommandBoxPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let mut command = CommandBox::new("live-cmdbox", LABEL)
            .open(self.open)
            .on_click({
                let entity = entity.clone();
                move |_, cx| {
                    entity.update(cx, |this, cx| {
                        this.open = !this.open;
                        this.action = "Open the palette".into();
                        cx.notify();
                    })
                }
            });
        if self.shortcut {
            command = command.shortcut("⌘K");
        }
        let live = stack()
            .gap(px(16.))
            .child(bar(960., "Add dark theme tokens", command))
            .child(last_action(&self.action));
        let config = stack()
            .gap(tokens::SPACING_06)
            .child(controls::toggle(
                "cmdbox-open",
                "open (its palette shown)",
                self.open,
                controls::setter(&entity, |this: &mut CommandBoxPage, on| this.open = on),
            ))
            .child(controls::toggle(
                "cmdbox-shortcut",
                "shortcut",
                self.shortcut,
                controls::setter(&entity, |this: &mut CommandBoxPage, on| this.shortcut = on),
            ));
        Body::new()
            .playground(live)
            .hint("Click the box, or Tab to it and press Enter: it opens the palette")
            .config(config)
            .example(controls::block(
                "Less room: right of centre, the words cut",
                bar(
                    560.,
                    "Migrate auth to passkeys",
                    CommandBox::new("ref-cmdbox-cut", LABEL).shortcut("⌘K"),
                ),
            ))
            .example(controls::block(
                "Under 160px: the magnifier alone",
                bar(
                    400.,
                    "Migrate auth to passkeys",
                    CommandBox::new("ref-cmdbox-narrow", LABEL).shortcut("⌘K"),
                ),
            ))
    }
}
