//! Ghost · Settings › SettingsRowOverridden: T3's `SettingsRow` in its
//! overridden state (blue start rule, "Overridden" tag, the global value and
//! Reset), live, and the three reference rows.
use super::{list, note};
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div,
};
use u0ui::components::{Badge, BadgeVariant, Switch, SwitchSize};
use u0ui::ghost::primitives::Stepper;
use u0ui::prelude::*;
use u0ui::t3::workspace::SettingsRow;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        overridden: true,
        value: 6.,
        resets: 0,
    })
    .into()
}

struct Page {
    overridden: bool,
    value: f64,
    resets: usize,
}

/// `components/SettingsRowOverridden/preview.html`: three rows, 560px on
/// `layer-01`.
fn reference() -> impl IntoElement {
    list(560.)
        .child(
            SettingsRow::new("sro-0", "Default agent")
                .description("Used for new threads")
                .control(Badge::new("Claude").variant(BadgeVariant::Gray)),
        )
        .child(
            SettingsRow::new("sro-1", "Max parallel agents")
                .overridden(true)
                .inherited_value("3")
                .on_reset(|_, _| {})
                .control(
                    Stepper::new("sro-1-stepper")
                        .default_value(6.)
                        .min(1.)
                        .max(8.),
                ),
        )
        .child(
            SettingsRow::new("sro-2", "Auto-accept edits")
                .overridden(true)
                .inherited_value("Off")
                .on_reset(|_, _| {})
                .control(
                    Switch::new("sro-2-switch")
                        .size(SwitchSize::Sm)
                        .label("Auto-accept edits")
                        .hide_label(true)
                        .default_checked(true),
                ),
        )
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    list(240.)
        .child(
            SettingsRow::new("sro-thumb", "Max parallel agents")
                .overridden(true)
                .inherited_value("3")
                .on_reset(|_, _| {}),
        )
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let (reset, set) = (entity.clone(), entity.clone());
        let live = div()
            .w_full()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_03)
            .child(
                list(560.).child(
                    SettingsRow::new("sro-live", "Max parallel agents")
                        .overridden(self.overridden)
                        .inherited_value("3")
                        .on_reset(move |_, cx| {
                            reset.update(cx, |p, cx| {
                                p.overridden = false;
                                p.value = 3.;
                                p.resets += 1;
                                cx.notify();
                            })
                        })
                        .control(
                            Stepper::new("sro-live-stepper")
                                .value(self.value)
                                .min(1.)
                                .max(8.)
                                .on_value_change(move |v, _, cx| {
                                    set.update(cx, |p, cx| {
                                        p.value = v;
                                        p.overridden = v != 3.;
                                        cx.notify();
                                    })
                                }),
                        ),
                ),
            )
            .child(note(format!("onReset × {}", self.resets)));
        let config = controls::toggle(
            "sro-overridden",
            "overridden",
            self.overridden,
            controls::setter(&entity, |p: &mut Page, v| p.overridden = v),
        );
        Body::new()
            .playground(live)
            .hint("Change the value to override the global 3; Reset goes back to it")
            .config(config)
            .example(controls::block("Reference preview", reference()))
    }
}
