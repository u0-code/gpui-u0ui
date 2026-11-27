//! Ghost · Primitives › TimeField: a live segmented time field (24/12-hour,
//! minute step, disabled) plus the reference fields.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div,
};
use u0ui::ghost::primitives::TimeField;
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| TimeFieldPage {
        value: "09:00".into(),
        hour12: false,
        disabled: false,
        step: 0,
    })
    .into()
}

struct TimeFieldPage {
    value: SharedString,
    hour12: bool,
    disabled: bool,
    step: usize,
}

const STEPS: [u32; 3] = [5, 1, 15];

/// `components/TimeField/preview.html`.
pub fn reference(prefix: &'static str) -> impl IntoElement {
    div()
        .flex()
        .flex_wrap()
        .gap(tokens::SPACING_06)
        .child(
            TimeField::new((prefix, 0usize))
                .label("Run daily at")
                .default_value("09:00"),
        )
        .child(
            TimeField::new((prefix, 1usize))
                .label("12-hour")
                .default_value("18:30")
                .hour12(true),
        )
        .child(
            TimeField::new((prefix, 2usize))
                .label("Disabled")
                .default_value("07:15")
                .disabled(true),
        )
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    TimeField::new("thumb-time")
        .label("12-hour")
        .default_value("18:30")
        .hour12(true)
        .into_any_element()
}

impl Render for TimeFieldPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(
                TimeField::new("time-live")
                    .label("Run daily at")
                    .value(self.value.clone())
                    .hour12(self.hour12)
                    .disabled(self.disabled)
                    .step(STEPS[self.step])
                    .on_value_change(controls::setter(&entity, |this: &mut Self, v| {
                        this.value = v
                    })),
            )
            .child(
                div()
                    .type_style(TypeStyle::Code01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(format!("value: \"{}\"", self.value)),
            );
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::toggle(
                "time-hour12",
                "hour12",
                self.hour12,
                controls::setter(&entity, |this: &mut Self, on| this.hour12 = on),
            ))
            .child(controls::toggle(
                "time-disabled",
                "disabled",
                self.disabled,
                controls::setter(&entity, |this: &mut Self, on| this.disabled = on),
            ))
            .child(controls::field(
                "step (minutes)",
                controls::choices(
                    "time-step",
                    &["5", "1", "15"],
                    self.step,
                    controls::setter(&entity, |this: &mut Self, i| this.step = i),
                ),
            ));
        Body::new()
            .playground(live)
            .hint("Tab between segments, ↑/↓ to change, type digits")
            .config(config)
            .example(controls::block("Reference preview", reference("time-ref")))
    }
}
