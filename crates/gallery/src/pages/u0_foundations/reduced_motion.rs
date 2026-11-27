//! u0 · Foundations › ReducedMotion: full motion beside its reduced
//! fallback, plus a specimen that follows the app's reduced-motion setting.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, rems,
};
use u0ui::prelude::*;
use u0ui::u0::foundations::MotionSpecimen;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| ReducedMotionPage).into()
}

struct ReducedMotionPage;

fn labeled(label: &'static str, specimen: MotionSpecimen) -> impl IntoElement {
    div()
        .flex_1()
        .min_w(rems(18.))
        .child(super::caption(label))
        .child(specimen)
}

/// `components/ReducedMotion/preview.html`: two columns, 24px apart.
pub fn reference() -> impl IntoElement {
    div()
        .flex()
        .flex_wrap()
        .gap(tokens::SPACING_06)
        .child(labeled(
            "Full motion",
            MotionSpecimen::new("motion-full").reduced(false),
        ))
        .child(labeled(
            "Reduced motion",
            MotionSpecimen::new("motion-reduced").reduced(true),
        ))
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(MotionSpecimen::new("motion-thumb"))
        .into_any_element()
}

impl Render for ReducedMotionPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let reduced = motion::reduced_motion(cx);
        let live = div()
            .w(rems(32.))
            .max_w_full()
            .child(super::caption(if reduced {
                "Reduced motion is on (header › Display › Motion)"
            } else {
                "Full motion (header › Display › Motion)"
            }))
            .child(MotionSpecimen::new("motion-live"));
        Body::new()
            .playground(live)
            .hint("Switch reduced motion in the header's Display menu: movement fades, rotation pulses, shimmer stops")
            .example(controls::block("Reference preview", reference()))
    }
}
