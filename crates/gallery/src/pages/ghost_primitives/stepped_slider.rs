//! Ghost · Primitives › SteppedSlider: the telemetry levels, live.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, rems,
};
use u0ui::ghost::primitives::SteppedSlider;
use u0ui::prelude::*;

const STEPS: [&str; 4] = ["Off", "Crashes", "Usage", "Full"];
const DESCRIPTIONS: [&str; 4] = [
    "Nothing leaves this Mac.",
    "Crash reports only.",
    "Crash reports and anonymous usage counts.",
    "Everything above plus performance traces.",
];

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| SteppedSliderPage {
        value: 1,
        describe: true,
    })
    .into()
}

struct SteppedSliderPage {
    value: usize,
    describe: bool,
}

/// `components/SteppedSlider/preview.html`: 420px wide, telemetry 0–3.
pub fn reference(id: &'static str) -> impl IntoElement {
    div().w(rems(26.25)).max_w_full().child(
        SteppedSlider::new(id, STEPS)
            .label("Telemetry")
            .default_value(1)
            .describe(|i| DESCRIPTIONS[i].into()),
    )
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(
            SteppedSlider::new("thumb-sslider", STEPS)
                .label("Telemetry")
                .default_value(1),
        )
        .into_any_element()
}

impl Render for SteppedSliderPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let mut slider = SteppedSlider::new("sslider-live", STEPS)
            .label("Telemetry")
            .value(self.value)
            .on_value_change(controls::setter(&entity, |this: &mut Self, v| {
                this.value = v
            }));
        if self.describe {
            slider = slider.describe(|i| DESCRIPTIONS[i].into());
        }
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::field(
                "value",
                controls::choices(
                    "sslider-value",
                    &STEPS,
                    self.value,
                    controls::setter(&entity, |this: &mut Self, i| this.value = i),
                ),
            ))
            .child(controls::toggle(
                "sslider-describe",
                "describe",
                self.describe,
                controls::setter(&entity, |this: &mut Self, on| this.describe = on),
            ));
        Body::new()
            .playground(div().w(rems(26.25)).max_w_full().child(slider))
            .hint("←/→ step, Home/End jump; press or drag on the track; tick names are clickable")
            .config(config)
            .example(controls::block(
                "Reference preview",
                reference("sslider-ref"),
            ))
    }
}
