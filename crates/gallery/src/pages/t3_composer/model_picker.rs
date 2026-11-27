//! T3 · Composer › ModelPicker: Claude (Opus default, Sonnet, Haiku), Codex
//! starting and Cursor needing sign-in, with effort and speed
//! (`components/ModelPicker/preview.html`). The picker is live: tabs switch
//! providers, the search filters, rows and effort levels choose.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div,
};
use u0ui::prelude::*;
use u0ui::t3::composer::{EffortLevel, ModelPicker, PickerModel, PickerProvider};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| PickerPage {
        value: "opus".into(),
        effort: EffortLevel::High,
        fast: false,
    })
    .into()
}

struct PickerPage {
    value: SharedString,
    effort: EffortLevel,
    fast: bool,
}

/// The preview's providers.
fn providers() -> Vec<PickerProvider> {
    vec![
        PickerProvider::new(
            "a",
            "Claude",
            vec![
                PickerModel::new("opus", "Opus", "Most capable for complex work").badge("Default"),
                PickerModel::new("sonnet", "Sonnet", "Fast and capable"),
                PickerModel::new("haiku", "Haiku", "Fastest"),
            ],
        ),
        PickerProvider::new(
            "b",
            "Codex",
            vec![PickerModel::new("gpt", "GPT", "General coding model")],
        )
        .starting(true),
        PickerProvider::new(
            "c",
            "Cursor",
            vec![PickerModel::new("auto", "Auto", "Lets Cursor pick")],
        )
        .notice("Sign in again to use Cursor models."),
    ]
}

impl Render for PickerPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let picker = ModelPicker::new("mpick-live", providers())
            .value(self.value.clone())
            .effort(self.effort)
            .fast(self.fast)
            .on_value_change(controls::setter(&entity, |this: &mut Self, v| {
                this.value = v
            }))
            .on_effort_change(controls::setter(&entity, |this: &mut Self, l| {
                this.effort = l
            }))
            .on_fast_change(controls::setter(&entity, |this: &mut Self, on| {
                this.fast = on
            }));
        let summary: SharedString = format!(
            "value: {} · effort: {} · fast: {}",
            self.value,
            self.effort.name(),
            self.fast
        )
        .into();
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .w_full()
            .child(picker)
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(summary),
            );
        let levels = EffortLevel::ALL.map(|l| l.name());
        let current = EffortLevel::ALL
            .iter()
            .position(|l| *l == self.effort)
            .unwrap_or(2);
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "effort",
                controls::choices(
                    "mpick-effort",
                    &levels,
                    current,
                    controls::setter(&entity, |this: &mut Self, i| {
                        this.effort = EffortLevel::ALL[i]
                    }),
                ),
            ))
            .child(controls::toggle(
                "mpick-fast",
                "fast",
                self.fast,
                controls::setter(&entity, |this: &mut Self, on| this.fast = on),
            ));
        Body::new()
            .playground(live)
            .hint("Tab through providers, models and effort; Enter or Space chooses a model.")
            .config(config)
    }
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(gpui_kit::rems(15.))
        .h(gpui_kit::rems(8.))
        .overflow_hidden()
        .child(ModelPicker::new("mpick-th", providers()).value("opus"))
        .into_any_element()
}
