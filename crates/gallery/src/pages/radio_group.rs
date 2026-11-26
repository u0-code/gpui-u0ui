//! Forms › RadioGroup: the reference Plan group and a configurable live group.
use crate::controls;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, SharedString,
    Styled, Window, div,
};
use u0ui::components::radio_group::{RadioGroup, RadioGroupItem, RadioOrientation};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| RadioPage {
        orientation: 1,
        value: "lon".into(),
        disabled: false,
        readonly: false,
        invalid: false,
        warn: false,
        helper: true,
    })
    .into()
}

struct RadioPage {
    orientation: usize,
    value: SharedString,
    disabled: bool,
    readonly: bool,
    invalid: bool,
    warn: bool,
    helper: bool,
}

fn plan(id: &'static str) -> RadioGroup {
    RadioGroup::new(id)
        .legend("Plan")
        .item(RadioGroupItem::new("lite").label("Lite"))
        .item(RadioGroupItem::new("std").label("Standard"))
        .item(
            RadioGroupItem::new("ent")
                .label("Enterprise")
                .disabled(true),
        )
}

impl Render for RadioPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let toggle = |f: fn(&mut RadioPage, bool)| {
            let entity = entity.clone();
            move |on: bool, _: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    f(this, on);
                    cx.notify();
                })
            }
        };
        let orientation = RadioOrientation::ALL[self.orientation];

        // `components/RadioGroup/preview.html`.
        let reference = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Reference preview"))
            .child(plan("ref-plan").default_value("std"));

        let mut group = RadioGroup::new("live")
            .legend("Region")
            .orientation(orientation)
            .value(self.value.clone())
            .disabled(self.disabled)
            .readonly(self.readonly)
            .invalid(self.invalid)
            .invalid_text("Pick a region with capacity.")
            .warn(self.warn)
            .warn_text("Frankfurt is near capacity.")
            .on_value_change({
                let entity = entity.clone();
                move |value, _, cx| {
                    let value = value.clone();
                    entity.update(cx, |this, cx| {
                        this.value = value;
                        cx.notify();
                    })
                }
            })
            .item(RadioGroupItem::new("fra").label("Frankfurt"))
            .item(RadioGroupItem::new("lon").label("London"))
            .item(RadioGroupItem::new("dal").label("Dallas"))
            .item(RadioGroupItem::new("syd").label("Sydney").disabled(true))
            .item(RadioGroupItem::new("tok").label("Tokyo"));
        if self.helper {
            group = group.helper_text("Arrow keys move the selection.");
        }
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Live"))
            .child(group)
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(SharedString::from(format!("value = {}", self.value))),
            );

        let states = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::section("Orientations & states"))
            .child(
                plan("st-vertical")
                    .orientation(RadioOrientation::Vertical)
                    .default_value("lite"),
            )
            .child(plan("st-readonly").default_value("std").readonly(true))
            .child(
                plan("st-invalid")
                    .default_value("lite")
                    .invalid(true)
                    .invalid_text("Choose a paid plan."),
            );

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "orientation",
                controls::choices(
                    "radio-orientation",
                    &RadioOrientation::ALL.map(|o| o.name()),
                    self.orientation,
                    {
                        let entity = entity.clone();
                        move |i, _, cx| {
                            entity.update(cx, |this, cx| {
                                this.orientation = i;
                                cx.notify();
                            })
                        }
                    },
                ),
            ))
            .child(controls::toggle(
                "radio-helper",
                "helperText",
                self.helper,
                toggle(|this, on| this.helper = on),
            ))
            .child(controls::toggle(
                "radio-invalid",
                "invalid",
                self.invalid,
                toggle(|this, on| this.invalid = on),
            ))
            .child(controls::toggle(
                "radio-warn",
                "warn",
                self.warn,
                toggle(|this, on| this.warn = on),
            ))
            .child(controls::toggle(
                "radio-disabled",
                "disabled",
                self.disabled,
                toggle(|this, on| this.disabled = on),
            ))
            .child(controls::toggle(
                "radio-readonly",
                "readOnly",
                self.readonly,
                toggle(|this, on| this.readonly = on),
            ));

        controls::page_body(
            div()
                .flex()
                .flex_col()
                .gap(tokens::SPACING_07)
                .child(reference)
                .child(live)
                .child(states),
            Some(config.into_any_element()),
        )
    }
}
