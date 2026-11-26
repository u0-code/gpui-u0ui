//! Layout › Carousel: the reference Compute / Storage / Networking tiles and a
//! configurable live carousel (orientation, slide count, controlled index).
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, SharedString,
    Styled, Window, div, rems,
};
use u0ui::components::card::{Card, CardDescription, CardHeader, CardTitle};
use u0ui::components::carousel::{
    Carousel, CarouselContent, CarouselItem, CarouselNext, CarouselOrientation, CarouselPrevious,
};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| CarouselPage {
        orientation: 0,
        slides: 0,
        index: 0,
    })
    .into()
}

struct CarouselPage {
    orientation: usize,
    slides: usize,
    index: usize,
}

const TITLES: [&str; 5] = ["Compute", "Storage", "Networking", "Databases", "Security"];
const COUNTS: [usize; 2] = [3, 5];
const COUNT_NAMES: [&str; 2] = ["3", "5"];

/// `preview.html`: `Card { height: 180, margin: 0 }` with a title and
/// "Slide N of M".
fn slides(prefix: &'static str, count: usize) -> CarouselContent {
    CarouselContent::new()
        .items((0..count).map(|i| {
            CarouselItem::new().child(
                Card::new(SharedString::from(format!("{prefix}-{i}")))
                    .h(rems(11.25))
                    .child(
                        CardHeader::new()
                            .child(CardTitle::new(TITLES[i]))
                            .child(CardDescription::new(format!("Slide {} of {count}", i + 1))),
                    ),
            )
        }))
        .dot_labels(|i| format!("Slide {}", i + 1).into())
}

impl Render for CarouselPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();

        let reference = div().max_w(rems(30.)).child(
            Carousel::new("carousel-ref")
                .label("Carousel")
                .content(slides("carousel-ref-card", 3))
                .previous(CarouselPrevious::new("Previous slide"))
                .next(CarouselNext::new("Next slide")),
        );

        let count = COUNTS[self.slides];
        let index = self.index.min(count - 1);
        let live = div().max_w(rems(30.)).child(
            Carousel::new(("carousel-live", self.orientation * 10 + count))
                .orientation(CarouselOrientation::ALL[self.orientation])
                .label("Services")
                .index(index)
                .on_index_change(controls::setter(&entity, |this: &mut CarouselPage, i| {
                    this.index = i
                }))
                .content(slides("carousel-live-card", count))
                .previous(CarouselPrevious::new("Previous slide"))
                .next(CarouselNext::new("Next slide")),
        );

        let orientation_names = CarouselOrientation::ALL.map(|o| o.name());
        let index_names: [&'static str; 5] = ["1", "2", "3", "4", "5"];
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "orientation",
                controls::choices(
                    "carousel-orientation",
                    &orientation_names,
                    self.orientation,
                    controls::setter(&entity, |this: &mut CarouselPage, i| this.orientation = i),
                ),
            ))
            .child(controls::field(
                "slides",
                controls::choices(
                    "carousel-slides",
                    &COUNT_NAMES,
                    self.slides,
                    controls::setter(&entity, |this: &mut CarouselPage, i| this.slides = i),
                ),
            ))
            .child(controls::field(
                "current slide (controlled)",
                controls::choices(
                    "carousel-index",
                    &index_names[..count],
                    index,
                    controls::setter(&entity, |this: &mut CarouselPage, i| this.index = i),
                ),
            ));

        let note = div()
            .type_style(TypeStyle::Label01)
            .text_color(color(ColorRole::TextSecondary))
            .child("Click a bar or a button, or focus one and press Left / Right.");

        Body::new()
            .playground(
                div()
                    .flex()
                    .flex_col()
                    .gap(tokens::SPACING_04)
                    .child(live)
                    .child(note),
            )
            .config(config)
            .example(controls::block("Reference preview", reference))
    }
}
