//! Foundations › Motion: every curve and duration, as boxes you can run.
use crate::controls;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, ElementId, IntoElement, ParentElement,
    Render, SharedString, Styled, Window, div, relative, rems,
};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| MotionPage {
        speed: 3,
        at_end: [false; 6],
        speed_at_end: [false; 6],
    })
    .into()
}

struct MotionPage {
    /// Index into [`Speed::ALL`] used by the curve demos.
    speed: usize,
    /// Each curve demo's target side.
    at_end: [bool; 6],
    /// Each duration demo's target side.
    speed_at_end: [bool; 6],
}

/// A labeled track with a box animated toward its target side.
fn track(
    id: impl Into<ElementId>,
    label: String,
    detail: String,
    at_end: bool,
    motion: Motion,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id: ElementId = id.into();
    let position = motion::animate(id, if at_end { 1f32 } else { 0. }, motion, window, cx);
    div()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap(tokens::SPACING_05)
        .child(
            div()
                .w(rems(14.))
                .flex_shrink_0()
                .flex()
                .flex_col()
                .child(div().type_style(TypeStyle::HeadingCompact01).child(label))
                .child(
                    div()
                        .type_style(TypeStyle::Code01)
                        .text_color(color(ColorRole::TextSecondary))
                        .child(detail),
                ),
        )
        .child(
            div()
                .flex_1()
                .min_w(rems(12.))
                .h(rems(2.))
                .relative()
                .bg(color(ColorRole::Layer01))
                .child(
                    div()
                        .absolute()
                        .top_0()
                        .left(relative(position * 0.9))
                        .w(relative(0.1))
                        .h_full()
                        .bg(color(ColorRole::Interactive)),
                ),
        )
        .into_any_element()
}

impl Render for MotionPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let speed = Speed::ALL[self.speed];
        let entity = cx.entity();
        let curves: Vec<AnyElement> = Curve::ALL
            .iter()
            .enumerate()
            .map(|(i, curve)| {
                let (x1, y1, x2, y2) = curve.points();
                track(
                    ("motion-curve", i),
                    curve.label().into(),
                    format!("cubic-bezier({x1}, {y1}, {x2}, {y2})"),
                    self.at_end[i],
                    Motion::new(*curve, speed).spatial(true),
                    window,
                    cx,
                )
            })
            .collect();
        let speeds: Vec<AnyElement> = Speed::ALL
            .iter()
            .enumerate()
            .map(|(i, speed)| {
                track(
                    ("motion-speed", i),
                    speed.name().into(),
                    format!("{}ms, productive standard", speed.millis()),
                    self.speed_at_end[i],
                    Motion::new(Curve::ProductiveStandard, *speed).spatial(true),
                    window,
                    cx,
                )
            })
            .collect();
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "Duration for the curve demos",
                controls::choices(
                    "motion-speed-choice",
                    &[
                        "fast-01",
                        "fast-02",
                        "moderate-01",
                        "moderate-02",
                        "slow-01",
                        "slow-02",
                    ],
                    self.speed,
                    {
                        let entity = entity.clone();
                        move |i, _, cx| {
                            entity.update(cx, |this, cx| {
                                this.speed = i;
                                cx.notify();
                            })
                        }
                    },
                ),
            ))
            .child(
                Button::new("motion-play", "Play")
                    .size(ButtonSize::Md)
                    .icon(CarbonIcon::Play)
                    .on_click({
                        let entity = entity.clone();
                        move |_, _, cx| {
                            entity.update(cx, |this, cx| {
                                for v in this.at_end.iter_mut().chain(this.speed_at_end.iter_mut())
                                {
                                    *v = !*v;
                                }
                                cx.notify();
                            })
                        }
                    }),
            )
            .child(
                div()
                    .type_style(TypeStyle::HelperText01)
                    .text_color(color(ColorRole::TextHelper))
                    .child(SharedString::from(if motion::reduced_motion(cx) {
                        "Reduced motion is on: movement cuts to its end."
                    } else {
                        "Reduced motion is off."
                    })),
            );
        crate::controls::page_body(
            div()
                .flex()
                .flex_col()
                .gap(tokens::SPACING_05)
                .child(controls::section("Curves"))
                .children(curves)
                .child(controls::section("Durations"))
                .children(speeds),
            Some(config.into_any_element()),
        )
    }
}
