//! Data display › AspectRatio: the reference 16 : 9 (280px) and 1 : 1 (140px)
//! boxes and a live box for every Carbon ratio.
use crate::controls;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, SharedString,
    Styled, Window, div, rems,
};
use u0ui::components::aspect_ratio::{AspectRatio, CARBON_RATIOS};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| AspectRatioPage { ratio: 0 }).into()
}

struct AspectRatioPage {
    ratio: usize,
}

/// The preview's centered `text-secondary` label filling the box.
fn label(text: impl Into<SharedString>) -> impl IntoElement {
    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .type_style(TypeStyle::BodyCompact01)
        .text_color(color(ColorRole::TextSecondary))
        .child(text.into())
}

const NAMES: [&str; 9] = [
    "16 : 9", "9 : 16", "2 : 1", "1 : 2", "4 : 3", "3 : 4", "3 : 2", "2 : 3", "1 : 1",
];

impl Render for AspectRatioPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        // `components/AspectRatio/preview.html`: `cu-row` aligned to the top.
        let reference = div()
            .flex()
            .flex_row()
            .flex_wrap()
            .items_start()
            .gap(tokens::SPACING_05)
            .child(
                div()
                    .w(rems(17.5))
                    .child(AspectRatio::new(16. / 9.).child(label("16 : 9"))),
            )
            .child(
                div()
                    .w(rems(8.75))
                    .child(AspectRatio::new(1.).child(label("1 : 1"))),
            );

        let (w, h) = CARBON_RATIOS[self.ratio];
        let live = div()
            .w(rems(15.))
            .child(AspectRatio::new(w as f32 / h as f32).child(label(NAMES[self.ratio])));

        let config = controls::field(
            "ratio",
            controls::choices(
                "aspect-ratio",
                &NAMES,
                self.ratio,
                controls::setter(&entity, |this: &mut AspectRatioPage, i| this.ratio = i),
            ),
        );

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
