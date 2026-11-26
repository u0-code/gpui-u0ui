//! Navigation › Collapsible: the reference preview and a controlled live one.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled, Window, div,
};
use u0ui::components::{Checkbox, Collapsible};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| CollapsiblePage { open: false }).into()
}

struct CollapsiblePage {
    open: bool,
}

impl Render for CollapsiblePage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let set = move |open: bool, _: &mut Window, cx: &mut App| {
            entity.update(cx, |this, cx| {
                this.open = open;
                cx.notify();
            })
        };
        // `components/Collapsible/preview.html`.
        let reference = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Reference preview"))
            .child(
                Collapsible::new("ref-collapsible", "Advanced options")
                    .default_open(true)
                    .child(Checkbox::new("ref-ipv6").label("Enable IPv6"))
                    .child(
                        Checkbox::new("ref-encrypt")
                            .label("Encrypt at rest")
                            .default_checked(true),
                    ),
            );
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(
                Collapsible::new("live-collapsible", "Network settings")
                    .open(self.open)
                    .on_open_change(set.clone())
                    .child(Checkbox::new("live-ipv6").label("Enable IPv6"))
                    .child(Checkbox::new("live-dns").label("Use private DNS"))
                    .child(
                        Checkbox::new("live-encrypt")
                            .label("Encrypt at rest")
                            .default_checked(true),
                    ),
            )
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(format!("open: {}", self.open)),
            );
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::toggle("collapsible-open", "open", self.open, set));
        Body::new()
            .playground(live)
            .hint("controlled: the open switch in the panel follows it")
            .config(config)
            .example(reference)
    }
}
