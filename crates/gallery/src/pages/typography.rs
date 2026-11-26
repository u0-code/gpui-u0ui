//! Foundations › Typography: every Carbon type style.
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled, Window, div,
};
use u0ui::prelude::*;
use u0ui::tokens::LineHeight;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| TypographyPage).into()
}

struct TypographyPage;

fn sample(style: TypeStyle) -> &'static str {
    match style.spec().group {
        "CJK" => "创建项目，并邀请你的团队成员。",
        "Code" => "fn main() { println!(\"u0ui\"); }",
        _ => "Create a project and invite your team",
    }
}

impl Render for TypographyPage {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .children(TypeStyle::ALL.iter().map(|style| {
                let spec = style.spec();
                let line_height = match spec.line_height {
                    LineHeight::Multiplier(m) => format!("{m}"),
                    LineHeight::Px(v) => format!("{v}px"),
                    LineHeight::Rem(v) => format!("{v}rem"),
                };
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap(tokens::SPACING_05)
                    .py(tokens::SPACING_05)
                    .border_b_1()
                    .border_color(color(ColorRole::BorderSubtle01))
                    .child(
                        div()
                            .w(gpui_kit::rems(14.))
                            .flex_shrink_0()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .type_style(TypeStyle::HeadingCompact01)
                                    .child(spec.name),
                            )
                            .child(
                                div()
                                    .type_style(TypeStyle::Code01)
                                    .text_color(color(ColorRole::TextSecondary))
                                    .child(format!(
                                        "{}px / {} / {} / {}px",
                                        spec.font_size_px(),
                                        line_height,
                                        spec.weight,
                                        spec.letter_spacing_px
                                    )),
                            ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w(gpui_kit::rems(16.))
                            .type_style(*style)
                            .child(sample(*style)),
                    )
            }))
    }
}
