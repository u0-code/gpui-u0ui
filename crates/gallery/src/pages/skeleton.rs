//! Feedback › Skeleton (Carbon SkeletonText / SkeletonPlaceholder): the
//! reference stack (a 40% heading, a 3-line paragraph, a 3rem block) and a
//! live skeleton with every prop. Turn on "Reduce" motion in the header to
//! see the shimmer stop.
use crate::controls;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled, Window,
    div, rems,
};
use u0ui::components::skeleton::{Skeleton, SkeletonLength, SkeletonVariant};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| SkeletonPage {
        variant: 1,
        lines: 2,
        heading: false,
        width: 0,
    })
    .into()
}

struct SkeletonPage {
    variant: usize,
    lines: usize,
    heading: bool,
    width: usize,
}

const LINES: [&str; 5] = ["1", "2", "3", "4", "5"];
const WIDTHS: [&str; 4] = ["100%", "60%", "40%", "240px"];

impl Render for SkeletonPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        // `components/Skeleton/preview.html`: `cu-stack`, max-width 360, gap 12.
        let reference = div()
            .max_w(rems(22.5))
            .flex()
            .flex_col()
            .gap(rems(0.75))
            .child(
                Skeleton::text()
                    .heading(true)
                    .lines(1)
                    .width(SkeletonLength::Percent(40.)),
            )
            .child(Skeleton::text().lines(3))
            .child(Skeleton::block().height(SkeletonLength::Rem(3.)));

        let width = match self.width {
            1 => SkeletonLength::Percent(60.),
            2 => SkeletonLength::Percent(40.),
            3 => SkeletonLength::Px(240.),
            _ => SkeletonLength::Percent(100.),
        };
        let live = div().max_w(rems(22.5)).child(
            Skeleton::text()
                .variant(SkeletonVariant::ALL[self.variant])
                .lines(self.lines + 1)
                .heading(self.heading)
                .width(width),
        );

        let variants = SkeletonVariant::ALL.map(|v| v.name());
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "variant",
                controls::choices(
                    "skeleton-variant",
                    &variants,
                    self.variant,
                    controls::setter(&entity, |this: &mut SkeletonPage, i| this.variant = i),
                ),
            ))
            .child(controls::field(
                "lines (text)",
                controls::choices(
                    "skeleton-lines",
                    &LINES,
                    self.lines,
                    controls::setter(&entity, |this: &mut SkeletonPage, i| this.lines = i),
                ),
            ))
            .child(controls::field(
                "width",
                controls::choices(
                    "skeleton-width",
                    &WIDTHS,
                    self.width,
                    controls::setter(&entity, |this: &mut SkeletonPage, i| this.width = i),
                ),
            ))
            .child(controls::toggle(
                "skeleton-heading",
                "heading",
                self.heading,
                controls::setter(&entity, |this: &mut SkeletonPage, on| this.heading = on),
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
