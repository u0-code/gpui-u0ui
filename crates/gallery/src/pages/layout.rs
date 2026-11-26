//! Foundations › Spacing, size & radius.
use crate::controls;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled, Window,
    div, rems,
};
use u0ui::prelude::*;
use u0ui::tokens::{RADIUS_TOKENS, SIZE_TOKENS, SPACING_TOKENS, TokenLength};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| LayoutPage).into()
}

struct LayoutPage;

fn describe(value: TokenLength) -> String {
    match value {
        TokenLength::Rem(v) => format!("{v}rem · {}px", value.px_value()),
        TokenLength::Px(v) => format!("{v}px"),
    }
}

fn row(name: &str, value: TokenLength, visual: impl IntoElement) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(tokens::SPACING_05)
        .min_h(rems(2.))
        .child(
            div()
                .w(rems(11.))
                .flex_shrink_0()
                .type_style(TypeStyle::Code01)
                .child(name.to_string()),
        )
        .child(
            div()
                .w(rems(9.))
                .flex_shrink_0()
                .type_style(TypeStyle::Code01)
                .text_color(color(ColorRole::TextSecondary))
                .child(describe(value)),
        )
        .child(div().flex_1().min_w_0().overflow_hidden().child(visual))
}

impl Render for LayoutPage {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_07)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(tokens::SPACING_03)
                    .child(controls::section("Spacing"))
                    .children(SPACING_TOKENS.iter().map(|(name, value)| {
                        row(
                            name,
                            *value,
                            div()
                                .w(*value)
                                .h(rems(1.))
                                .bg(color(ColorRole::Interactive)),
                        )
                    })),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(tokens::SPACING_03)
                    .child(controls::section("Size"))
                    .children(SIZE_TOKENS.iter().map(|(name, value)| {
                        // Heights are drawn as heights, capped so window sizes stay readable.
                        let bar = if value.px_value() <= 96. {
                            div()
                                .h(*value)
                                .w(rems(4.))
                                .bg(color(ColorRole::Layer02))
                                .border_1()
                                .border_color(color(ColorRole::BorderStrong01))
                        } else {
                            div()
                                .h(rems(0.5))
                                .w(gpui_kit::relative((value.px_value() / 1280.).min(1.)))
                                .bg(color(ColorRole::Layer02))
                        };
                        row(name, *value, bar)
                    })),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(tokens::SPACING_03)
                    .child(controls::section("Radius"))
                    .children(RADIUS_TOKENS.iter().map(|(name, value)| {
                        row(
                            name,
                            *value,
                            div()
                                .size(rems(3.))
                                .rounded(*value)
                                .bg(color(ColorRole::Layer02))
                                .border_1()
                                .border_color(color(ColorRole::BorderStrong01)),
                        )
                    })),
            )
    }
}
