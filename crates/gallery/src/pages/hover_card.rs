//! Overlays › HoverCard: the reference preview (open) and a configurable live
//! card.
use crate::controls;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, FontWeight, IntoElement, ParentElement, Render, Styled,
    Window, div, rems,
};
use std::time::Duration;
use u0ui::components::{Align, HoverCard};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| HoverCardPage { align: 4, delay: 1 }).into()
}

struct HoverCardPage {
    /// Index into [`Align::ALL`].
    align: usize,
    /// Index into [`DELAYS`].
    delay: usize,
}

const DELAYS: [(&str, u64); 3] = [("0ms", 0), ("300ms", 300), ("700ms", 700)];

/// `cu-avatar cu-avatar--lg` + `cu-avatar-fallback` (blue): a gallery stand-in
/// until the u0ui Avatar lands.
fn avatar(initials: &'static str) -> impl IntoElement {
    div()
        .size(rems(3.))
        .flex_shrink_0()
        .rounded_full()
        .flex()
        .items_center()
        .justify_center()
        .bg(color(ColorRole::TagBackgroundBlue))
        .text_color(color(ColorRole::TagColorBlue))
        .type_style(TypeStyle::Label01)
        .font_weight(FontWeight::SEMIBOLD)
        .child(initials)
}

/// The reference card body.
fn profile() -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .gap(rems(0.75))
        .child(avatar("CD"))
        .child(
            div()
                .flex()
                .flex_col()
                .child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("Carbon Design System"),
                )
                .child(
                    div()
                        .mt(rems(0.25))
                        .text_color(color(ColorRole::TextSecondary))
                        .child("IBM’s open-source design system."),
                ),
        )
}

fn link(id: &'static str) -> Button {
    Button::new(id, "@carbon-design").variant(ButtonVariant::Link)
}

impl Render for HoverCardPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let reference = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Reference preview"))
            .child(
                div().flex().pb(rems(7.)).child(
                    HoverCard::new("ref-hover-card", link("ref-link"))
                        .default_open(true)
                        .child(profile()),
                ),
            );

        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Live: hover or focus the link"))
            .child(
                div()
                    .flex()
                    .justify_center()
                    .py(rems(10.))
                    .bg(color(ColorRole::Background))
                    .border_1()
                    .border_color(color(ColorRole::BorderSubtle01))
                    .child(
                        HoverCard::new("live-hover-card", link("live-link"))
                            .align(Align::ALL[self.align])
                            .open_delay(Duration::from_millis(DELAYS[self.delay].1))
                            .child(profile()),
                    ),
            );

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "align",
                controls::choices("hc-align", &Align::ALL.map(|a| a.name()), self.align, {
                    let entity = entity.clone();
                    move |i, _, cx| {
                        entity.update(cx, |this, cx| {
                            this.align = i;
                            cx.notify();
                        })
                    }
                }),
            ))
            .child(controls::field(
                "openDelay",
                controls::choices("hc-delay", &DELAYS.map(|d| d.0), self.delay, {
                    let entity = entity.clone();
                    move |i, _, cx| {
                        entity.update(cx, |this, cx| {
                            this.delay = i;
                            cx.notify();
                        })
                    }
                }),
            ));

        controls::page_body(
            div()
                .flex()
                .flex_col()
                .gap(tokens::SPACING_07)
                .child(reference)
                .child(live),
            Some(config.into_any_element()),
        )
    }
}
