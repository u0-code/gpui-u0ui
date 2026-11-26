//! Forms › Slider: the reference Memory and Price range sliders and a
//! configurable live slider.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, Entity, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, rems,
};
use u0ui::components::slider::{Slider, SliderState};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    let memory = cx.new(|_| {
        SliderState::new()
            .min(0.)
            .max(64.)
            .step(1.)
            .default_value(24.)
    });
    let price = cx.new(|_| {
        SliderState::new()
            .min(0.)
            .max(1000.)
            .step(1.)
            .default_value((200., 750.))
    });
    let live = cx.new(|_| {
        SliderState::new()
            .min(0.)
            .max(100.)
            .step(5.)
            .default_value(40.)
    });
    cx.new(|_| SliderPage {
        memory,
        price,
        live,
        disabled: false,
        hide_input: false,
    })
    .into()
}

struct SliderPage {
    memory: Entity<SliderState>,
    price: Entity<SliderState>,
    live: Entity<SliderState>,
    disabled: bool,
    hide_input: bool,
}

impl Render for SliderPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let toggle = |f: fn(&mut SliderPage, bool)| {
            let entity = entity.clone();
            move |on: bool, _: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    f(this, on);
                    cx.notify();
                })
            }
        };

        // `components/Slider/preview.html` (`cu-stack`, max-width 520).
        let reference = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Reference preview"))
            .child(
                div()
                    .max_w(rems(32.5))
                    .flex()
                    .flex_col()
                    .gap(tokens::SPACING_06)
                    .child(Slider::new("ref-memory", &self.memory).label("Memory (GB)"))
                    .child(Slider::new("ref-price", &self.price).label("Price range")),
            );

        let live_value = self.live.read(cx).value();
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(
                div().max_w(rems(32.5)).child(
                    Slider::new("live", &self.live)
                        .label("CPU threshold (%)")
                        .disabled(self.disabled)
                        .hide_input(self.hide_input),
                ),
            )
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(SharedString::from(format!("value = {live_value}"))),
            );

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::toggle(
                "slider-disabled",
                "disabled",
                self.disabled,
                toggle(|this, on| this.disabled = on),
            ))
            .child(controls::toggle(
                "slider-hide-input",
                "hideTextInput",
                self.hide_input,
                toggle(|this, on| this.hide_input = on),
            ));

        Body::new()
            .playground(live)
            .hint("drag, click the track, ←/→, Shift ×4, type a value")
            .config(config)
            .example(reference)
    }
}
