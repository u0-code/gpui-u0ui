//! Foundations › Colors: every color role as a swatch in the active theme.
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled, Window,
    div, rems,
};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| ColorsPage).into()
}

struct ColorsPage;

/// `#rrggbb` or `rgba(r, g, b, a)` like `tokens.json`.
fn css(role: ColorRole) -> String {
    let c = theme::rgba(role);
    let b = |v: f32| (v * 255.).round() as u8;
    if c.a < 1. {
        format!(
            "rgba({}, {}, {}, {})",
            b(c.r),
            b(c.g),
            b(c.b),
            (c.a * 100.).round() / 100.
        )
    } else {
        format!("#{:02x}{:02x}{:02x}", b(c.r), b(c.g), b(c.b))
    }
}

impl Render for ColorsPage {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .flex_wrap()
            .gap(tokens::SPACING_03)
            .children(ColorRole::ALL.iter().map(|role| {
                div()
                    .w(rems(11.))
                    .flex()
                    .flex_col()
                    .bg(color(ColorRole::Layer01))
                    .child(
                        div()
                            .h(rems(3.))
                            .bg(color(*role))
                            .border_1()
                            .border_color(color(ColorRole::BorderSubtle01)),
                    )
                    .child(
                        div()
                            .p(tokens::SPACING_03)
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .type_style(TypeStyle::Label01)
                                    .text_color(color(ColorRole::TextPrimary))
                                    .truncate()
                                    .child(role.name()),
                            )
                            .child(
                                div()
                                    .type_style(TypeStyle::Code01)
                                    .text_color(color(ColorRole::TextSecondary))
                                    .truncate()
                                    .child(css(*role)),
                            ),
                    )
            }))
    }
}
