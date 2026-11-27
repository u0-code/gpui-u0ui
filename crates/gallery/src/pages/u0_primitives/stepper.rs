//! u0 · Primitives › Stepper: a live compact number input with every
//! prop, plus the four reference fields.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, rems,
};
use u0ui::prelude::*;
use u0ui::u0::primitives::Stepper;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| StepperPage {
        value: 3.,
        unit: false,
        disabled: false,
        invalid: false,
        bounded: true,
    })
    .into()
}

struct StepperPage {
    value: f64,
    unit: bool,
    disabled: bool,
    invalid: bool,
    bounded: bool,
}

/// `components/Stepper/preview.html`: four fields in a row, 24px apart.
pub fn reference(prefix: &'static str) -> impl IntoElement {
    div()
        .flex()
        .flex_wrap()
        .items_start()
        .gap(tokens::SPACING_06)
        .child(
            Stepper::new((prefix, 0usize))
                .label("Max parallel agents")
                .default_value(3.)
                .min(1.)
                .max(8.)
                .helper_text("1–8"),
        )
        .child(
            Stepper::new((prefix, 1usize))
                .label("Timeout")
                .default_value(30.)
                .min(5.)
                .step(5.)
                .unit("min")
                .width(rems(8.)),
        )
        .child(
            Stepper::new((prefix, 2usize))
                .label("Retries")
                .default_value(12.)
                .max(10.)
                .invalid(true)
                .invalid_text("At most 10"),
        )
        .child(
            Stepper::new((prefix, 3usize))
                .label("Disabled")
                .default_value(2.)
                .disabled(true),
        )
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .child(
            Stepper::new("thumb-stepper")
                .label("Max parallel agents")
                .default_value(3.)
                .min(1.)
                .max(8.)
                .helper_text("1–8"),
        )
        .into_any_element()
}

impl Render for StepperPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let mut live = Stepper::new("stepper-live")
            .label("Max parallel agents")
            .value(self.value)
            .disabled(self.disabled)
            .invalid(self.invalid)
            .invalid_text("At most 8")
            .on_value_change(controls::setter(&entity, |this: &mut Self, v| {
                this.value = v
            }));
        if self.bounded {
            live = live.min(1.).max(8.).helper_text("1–8");
        }
        if self.unit {
            live = live.unit("min").width(rems(8.));
        }
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::toggle(
                "stepper-bounded",
                "min 1 · max 8",
                self.bounded,
                controls::setter(&entity, |this: &mut Self, on| this.bounded = on),
            ))
            .child(controls::toggle(
                "stepper-unit",
                "unit",
                self.unit,
                controls::setter(&entity, |this: &mut Self, on| this.unit = on),
            ))
            .child(controls::toggle(
                "stepper-invalid",
                "invalid",
                self.invalid,
                controls::setter(&entity, |this: &mut Self, on| this.invalid = on),
            ))
            .child(controls::toggle(
                "stepper-disabled",
                "disabled",
                self.disabled,
                controls::setter(&entity, |this: &mut Self, on| this.disabled = on),
            ));
        Body::new()
            .playground(live)
            .hint("↑/↓ step, type and press Enter or Tab to commit (clamped)")
            .config(config)
            .example(controls::block(
                "Reference preview",
                reference("stepper-ref"),
            ))
    }
}
