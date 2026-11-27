//! u0 · Foundations › Density: the compact desktop row and field sizes,
//! the comfortable fallback and the 44px palette input, with a live
//! `density::zone` (the reference's `<Density value>`).
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, px, rems,
};
use u0ui::density::{self, Density};
use u0ui::prelude::*;
use u0ui::tokens::{DENSITY_ROW_LG, DENSITY_ROW_MD, DENSITY_ROW_SM, TokenLength};
use u0ui::u0::primitives::Stepper;

const ROWS: [&str; 3] = [
    "Add dark theme tokens",
    "Fix flaky test",
    "Write migration guide",
];

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| DensityPage { comfortable: false }).into()
}

struct DensityPage {
    comfortable: bool,
}

fn column(title: &'static str, height: TokenLength) -> impl IntoElement {
    // `.g-densitydemo h4 { font-size: 0.75rem; font-weight: 400; color: text-secondary }`,
    // `.g-row { gap: spacing-03; padding: 0 spacing-04; border-bottom; background: layer-01 }`.
    div()
        .flex_1()
        .min_w_0()
        .child(
            div()
                .mb(tokens::SPACING_03)
                .type_style(TypeStyle::Label01)
                .text_color(color(ColorRole::TextSecondary))
                .child(title),
        )
        .children(ROWS.map(|t| {
            div()
                .flex()
                .items_center()
                .gap(tokens::SPACING_03)
                .h(height)
                .px(tokens::SPACING_04)
                .border_b(px(1.))
                .border_color(color(ColorRole::BorderSubtle01))
                .bg(color(ColorRole::Layer01))
                .type_style(TypeStyle::BodyCompact01)
                .text_color(color(ColorRole::TextPrimary))
                .child(
                    Icon::carbon(CarbonIcon::Chat)
                        .size(IconSize::S16)
                        .color(color(ColorRole::IconSecondary)),
                )
                .child(t)
        }))
}

fn stepper_in(d: Density, id: &'static str, label: &'static str) -> impl IntoElement {
    density::zone(d, move |_, _| {
        Stepper::new(id)
            .label(label)
            .default_value(4.)
            .into_any_element()
    })
}

/// `components/Density/preview.html`.
pub fn reference() -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_05)
        .child(
            div()
                .flex()
                .gap(tokens::SPACING_05)
                .child(column("density-row-sm · 28px", DENSITY_ROW_SM))
                .child(column("density-row-md · 32px（默认）", DENSITY_ROW_MD))
                .child(column("density-row-lg · 36px", DENSITY_ROW_LG)),
        )
        .child(
            div()
                .flex()
                .items_end()
                .gap(tokens::SPACING_05)
                .child(stepper_in(
                    Density::Compact,
                    "density-ref-compact",
                    "Compact field · 32px",
                ))
                .child(stepper_in(
                    Density::Comfortable,
                    "density-ref-comfortable",
                    "Comfortable field · 40px",
                ))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .child(
                            div()
                                .mb(tokens::SPACING_03)
                                .type_style(TypeStyle::Label01)
                                .text_color(color(ColorRole::TextSecondary))
                                .child("⌘K input · 44px"),
                        )
                        .child(
                            div()
                                .h(density::palette_input())
                                .bg(color(ColorRole::Field01))
                                .flex()
                                .items_center()
                                .px(rems(1.))
                                .type_style(TypeStyle::BodyCompact01)
                                .text_color(color(ColorRole::TextPlaceholder))
                                .child("Type a command or search"),
                        ),
                ),
        )
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(column("density-row-md · 32px", DENSITY_ROW_MD))
        .into_any_element()
}

impl Render for DensityPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let d = if self.comfortable {
            Density::Comfortable
        } else {
            Density::Compact
        };
        let live = density::zone(d, move |_, _| {
            div()
                .flex()
                .flex_col()
                .gap(tokens::SPACING_05)
                .child(
                    Stepper::new("density-live-stepper")
                        .label("Max parallel agents")
                        .default_value(3.)
                        .min(1.)
                        .max(8.),
                )
                .child(
                    u0ui::components::Button::new("density-live-button", "Create thread")
                        .size(u0ui::components::ButtonSize::Lg),
                )
                .child(
                    div()
                        .type_style(TypeStyle::Code01)
                        .text_color(color(ColorRole::TextSecondary))
                        .child(format!(
                            "row {}px · field {}px",
                            d.row().px_value(),
                            d.field().px_value()
                        )),
                )
                .into_any_element()
        });
        let config = controls::toggle(
            "density-comfortable",
            "value = comfortable",
            self.comfortable,
            controls::setter(&entity, |this: &mut Self, on| this.comfortable = on),
        );
        Body::new()
            .playground(live)
            .hint("density::zone tightens or relaxes every u0ui control inside it")
            .config(config)
            .example(controls::block("Reference preview", reference()))
    }
}
