//! Forms › Switch: the reference toggles (Auto-scaling, Public endpoint, Small,
//! Locked) and a configurable live switch.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, SharedString,
    Styled, Window, div,
};
use u0ui::components::switch::{Switch, SwitchSize};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| SwitchPage {
        size: 0,
        checked: true,
        disabled: false,
        readonly: false,
        state_text: true,
    })
    .into()
}

struct SwitchPage {
    size: usize,
    checked: bool,
    disabled: bool,
    readonly: bool,
    state_text: bool,
}

/// The reference's default state texts (`labelOn = 'On', labelOff = 'Off'`).
fn on_off(switch: Switch) -> Switch {
    switch.label_on("On").label_off("Off")
}

impl Render for SwitchPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let toggle = |f: fn(&mut SwitchPage, bool)| {
            let entity = entity.clone();
            move |on: bool, _: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    f(this, on);
                    cx.notify();
                })
            }
        };
        let size = SwitchSize::ALL[self.size];

        // `components/Switch/preview.html` (`cu-row`, gap 48).
        let reference = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Reference preview"))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .items_center()
                    .gap(tokens::SPACING_09)
                    .child(on_off(
                        Switch::new("ref-autoscale")
                            .label("Auto-scaling")
                            .default_checked(true),
                    ))
                    .child(on_off(Switch::new("ref-public").label("Public endpoint")))
                    .child(on_off(
                        Switch::new("ref-small")
                            .label("Small")
                            .size(SwitchSize::Sm)
                            .default_checked(true),
                    ))
                    .child(on_off(
                        Switch::new("ref-locked").label("Locked").disabled(true),
                    )),
            );

        let mut live_switch = Switch::new("live")
            .label("Auto-scaling")
            .size(size)
            .checked(self.checked)
            .disabled(self.disabled)
            .readonly(self.readonly)
            .on_checked_change({
                let entity = entity.clone();
                move |on, _, cx| {
                    entity.update(cx, |this, cx| {
                        this.checked = on;
                        cx.notify();
                    })
                }
            });
        if self.state_text {
            live_switch = on_off(live_switch);
        }
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(live_switch)
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(SharedString::from(format!("checked = {}", self.checked))),
            );

        // Every size × state.
        let matrix = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Sizes × states"))
            .children(SwitchSize::ALL.iter().map(|s| {
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap(tokens::SPACING_07)
                    .children(
                        [
                            ("off", false, false, false),
                            ("on", true, false, false),
                            ("disabled", false, true, false),
                            ("disabled on", true, true, false),
                            ("read-only", true, false, true),
                        ]
                        .map(|(name, on, disabled, readonly)| {
                            on_off(
                                Switch::new(SharedString::from(format!("m-{}-{name}", s.name())))
                                    .label(format!("{} · {name}", s.name()))
                                    .size(*s)
                                    .default_checked(on)
                                    .disabled(disabled)
                                    .readonly(readonly),
                            )
                        }),
                    )
            }));

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "size",
                controls::choices(
                    "switch-size",
                    &SwitchSize::ALL.map(|s| s.name()),
                    self.size,
                    {
                        let entity = entity.clone();
                        move |i, _, cx| {
                            entity.update(cx, |this, cx| {
                                this.size = i;
                                cx.notify();
                            })
                        }
                    },
                ),
            ))
            .child(controls::toggle(
                "switch-checked",
                "checked",
                self.checked,
                toggle(|this, on| this.checked = on),
            ))
            .child(controls::toggle(
                "switch-text",
                "labelOn / labelOff",
                self.state_text,
                toggle(|this, on| this.state_text = on),
            ))
            .child(controls::toggle(
                "switch-disabled",
                "disabled",
                self.disabled,
                toggle(|this, on| this.disabled = on),
            ))
            .child(controls::toggle(
                "switch-readonly",
                "readOnly",
                self.readonly,
                toggle(|this, on| this.readonly = on),
            ));

        Body::new()
            .playground(live)
            .config(config)
            .example(reference)
            .example(matrix)
    }
}
