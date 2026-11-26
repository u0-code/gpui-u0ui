//! Data display › Separator: the reference "Account / Profile | Billing |
//! Access" demo, a live separator and one per layer.
use crate::controls;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled, Window,
    div, rems,
};
use u0ui::components::separator::{Separator, SeparatorOrientation};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| SeparatorPage { orientation: 0 }).into()
}

struct SeparatorPage {
    orientation: usize,
}

fn text(s: &'static str) -> impl IntoElement {
    div()
        .type_style(TypeStyle::BodyCompact01)
        .text_color(color(ColorRole::TextPrimary))
        .child(s)
}

impl Render for SeparatorPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let orientation = SeparatorOrientation::ALL[self.orientation];

        // `components/Separator/preview.html`: max-width 360; a 12px-margin
        // rule, then a 20px-tall row with 12px gaps and vertical rules.
        let reference = div()
            .max_w(rems(22.5))
            .flex()
            .flex_col()
            .child(text("Account"))
            .child(div().my(rems(0.75)).child(Separator::new()))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(rems(0.75))
                    .h(rems(1.25))
                    .items_center()
                    .child(text("Profile"))
                    .child(Separator::vertical())
                    .child(text("Billing"))
                    .child(Separator::vertical())
                    .child(text("Access")),
            );

        let live = div()
            .h(rems(5.))
            .max_w(rems(22.5))
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            .child(match orientation {
                SeparatorOrientation::Horizontal => div().w_full().child(Separator::new()),
                SeparatorOrientation::Vertical => div()
                    .h_full()
                    .flex()
                    .child(Separator::new().orientation(orientation)),
            });

        // `border-subtle` is contextual: 00 on background, 01 on layer-01, 02 on layer-02.
        let layers = div().flex().flex_col().children(
            [
                (
                    LayerLevel::One,
                    ColorRole::Background,
                    "On background (border-subtle-00)",
                ),
                (
                    LayerLevel::Two,
                    ColorRole::Layer01,
                    "On layer-01 (border-subtle-01)",
                ),
                (
                    LayerLevel::Three,
                    ColorRole::Layer02,
                    "On layer-02 (border-subtle-02)",
                ),
            ]
            .map(|(level, bg, label)| {
                div()
                    .bg(color(bg))
                    .p(tokens::SPACING_05)
                    .flex()
                    .flex_col()
                    .gap(tokens::SPACING_04)
                    .child(
                        div()
                            .type_style(TypeStyle::Label01)
                            .text_color(color(ColorRole::TextSecondary))
                            .child(label),
                    )
                    .child(Separator::new().layer(level))
            }),
        );

        let names = SeparatorOrientation::ALL.map(|o| o.name());
        let config = controls::field(
            "orientation",
            controls::choices(
                "separator-orientation",
                &names,
                self.orientation,
                controls::setter(&entity, |this: &mut SeparatorPage, i| this.orientation = i),
            ),
        );

        controls::page_body(
            div()
                .flex()
                .flex_col()
                .gap(tokens::SPACING_07)
                .child(controls::block("Reference preview", reference))
                .child(controls::block("Live", live))
                .child(controls::block("Layers", layers)),
            Some(config.into_any_element()),
        )
    }
}
