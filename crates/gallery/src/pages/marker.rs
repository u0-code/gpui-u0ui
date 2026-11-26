//! Chat & AI › Marker: the reference status / exported / "Today" markers and
//! a configurable live marker.
use crate::controls;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled, Window,
    div, prelude::FluentBuilder as _, rems,
};
use u0ui::components::marker::{Marker, MarkerContent, MarkerIcon, MarkerVariant};
use u0ui::components::spinner::{Spinner, SpinnerSize};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| MarkerPage {
        variant: 1,
        icon: 1,
    })
    .into()
}

struct MarkerPage {
    variant: usize,
    icon: usize,
}

const ICONS: [&str; 3] = ["none", "icon", "spinner"];

impl Render for MarkerPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();

        // `components/Marker/preview.html`: a 16px-gap stack, max 480.
        let reference = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .max_w(rems(30.))
            .child(
                Marker::new()
                    .status("marker-ref-status", "Searching 3 sources…")
                    .child(
                        MarkerIcon::new()
                            .child(Spinner::new("marker-ref-spin").size(SpinnerSize::Sm)),
                    )
                    .child(MarkerContent::new().child("Searching 3 sources…")),
            )
            .child(
                Marker::new()
                    .variant(MarkerVariant::Border)
                    .child(
                        MarkerIcon::new()
                            .child(Icon::carbon(CarbonIcon::CheckmarkFilled).size(IconSize::S16)),
                    )
                    .child(MarkerContent::new().child("Conversation exported")),
            )
            .child(
                Marker::new()
                    .variant(MarkerVariant::Separator)
                    .child("Today"),
            );

        let live = div().max_w(rems(30.)).child(
            Marker::new()
                .variant(MarkerVariant::ALL[self.variant])
                .when(self.icon == 1, |this| {
                    this.child(
                        MarkerIcon::new()
                            .child(Icon::carbon(CarbonIcon::CheckmarkFilled).size(IconSize::S16)),
                    )
                })
                .when(self.icon == 2, |this| {
                    this.child(
                        MarkerIcon::new()
                            .child(Spinner::new("marker-live-spin").size(SpinnerSize::Sm)),
                    )
                })
                .child(MarkerContent::new().child("Conversation exported")),
        );

        let variants = MarkerVariant::ALL.map(|v| v.name());
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "variant",
                controls::choices(
                    "marker-variant",
                    &variants,
                    self.variant,
                    controls::setter(&entity, |this: &mut MarkerPage, i| this.variant = i),
                ),
            ))
            .child(controls::field(
                "MarkerIcon",
                controls::choices(
                    "marker-icon",
                    &ICONS,
                    self.icon,
                    controls::setter(&entity, |this: &mut MarkerPage, i| this.icon = i),
                ),
            ));

        controls::page_body(
            div()
                .flex()
                .flex_col()
                .gap(tokens::SPACING_07)
                .child(controls::block("Reference preview", reference))
                .child(controls::block("Live", live)),
            Some(config.into_any_element()),
        )
    }
}
