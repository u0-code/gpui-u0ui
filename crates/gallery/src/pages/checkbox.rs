//! Forms › Checkbox: the reference list (checked, unchecked, indeterminate,
//! disabled) and a configurable live checkbox.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, SharedString,
    Styled, Window, div, rems,
};
use u0ui::components::Checkbox;
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| CheckboxPage {
        checked: true,
        indeterminate: false,
        disabled: false,
        readonly: false,
        invalid: false,
        warn: false,
        helper: false,
    })
    .into()
}

struct CheckboxPage {
    checked: bool,
    indeterminate: bool,
    disabled: bool,
    readonly: bool,
    invalid: bool,
    warn: bool,
    helper: bool,
}

/// `.cds--form-item.cds--checkbox-wrapper { margin-block-end: 0.375rem }`.
fn list() -> gpui_kit::Div {
    div().flex().flex_col().gap(rems(0.375))
}

impl Render for CheckboxPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let toggle = |f: fn(&mut CheckboxPage, bool)| {
            let entity = entity.clone();
            move |on: bool, _: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    f(this, on);
                    cx.notify();
                })
            }
        };

        // `components/Checkbox/preview.html`.
        let reference = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Reference preview"))
            .child(
                list()
                    .child(
                        Checkbox::new("ref-notes")
                            .label("Send me release notes")
                            .default_checked(true),
                    )
                    .child(Checkbox::new("ref-audit").label("Enable audit logs"))
                    .child(
                        Checkbox::new("ref-all")
                            .label("Select all")
                            .indeterminate(true),
                    )
                    .child(
                        Checkbox::new("ref-policy")
                            .label("Managed by policy")
                            .disabled(true),
                    ),
            );

        let mut live_box = Checkbox::new("live")
            .label("Enable audit logs")
            .checked(self.checked)
            .indeterminate(self.indeterminate)
            .disabled(self.disabled)
            .readonly(self.readonly)
            .invalid(self.invalid)
            .invalid_text("Audit logs are required for this plan.")
            .warn(self.warn)
            .warn_text("Logs are kept for 30 days.")
            .on_checked_change({
                let entity = entity.clone();
                move |on, _, cx| {
                    entity.update(cx, |this, cx| {
                        this.checked = on;
                        this.indeterminate = false;
                        cx.notify();
                    })
                }
            });
        if self.helper {
            live_box = live_box.helper_text("Stored in your region.");
        }
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(live_box)
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(SharedString::from(format!(
                        "checked = {}, indeterminate = {}",
                        self.checked, self.indeterminate
                    ))),
            );

        let states = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("States"))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap(tokens::SPACING_07)
                    .child(
                        list()
                            .child(Checkbox::new("s-off").label("Unchecked"))
                            .child(Checkbox::new("s-on").label("Checked").default_checked(true))
                            .child(
                                Checkbox::new("s-mixed")
                                    .label("Indeterminate")
                                    .indeterminate(true),
                            ),
                    )
                    .child(
                        list()
                            .child(Checkbox::new("s-d-off").label("Disabled").disabled(true))
                            .child(
                                Checkbox::new("s-d-on")
                                    .label("Disabled checked")
                                    .default_checked(true)
                                    .disabled(true),
                            )
                            .child(
                                Checkbox::new("s-ro")
                                    .label("Read-only checked")
                                    .default_checked(true)
                                    .readonly(true),
                            ),
                    )
                    .child(
                        list()
                            .child(
                                Checkbox::new("s-inv")
                                    .label("Invalid")
                                    .invalid(true)
                                    .invalid_text("Required."),
                            )
                            .child(
                                Checkbox::new("s-warn")
                                    .label("Warning")
                                    .default_checked(true)
                                    .warn(true)
                                    .warn_text("Double-check this."),
                            ),
                    ),
            );

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::toggle(
                "cb-checked",
                "checked",
                self.checked,
                toggle(|this, on| this.checked = on),
            ))
            .child(controls::toggle(
                "cb-indeterminate",
                "indeterminate",
                self.indeterminate,
                toggle(|this, on| this.indeterminate = on),
            ))
            .child(controls::toggle(
                "cb-helper",
                "helperText",
                self.helper,
                toggle(|this, on| this.helper = on),
            ))
            .child(controls::toggle(
                "cb-invalid",
                "invalid",
                self.invalid,
                toggle(|this, on| this.invalid = on),
            ))
            .child(controls::toggle(
                "cb-warn",
                "warn",
                self.warn,
                toggle(|this, on| this.warn = on),
            ))
            .child(controls::toggle(
                "cb-disabled",
                "disabled",
                self.disabled,
                toggle(|this, on| this.disabled = on),
            ))
            .child(controls::toggle(
                "cb-readonly",
                "readOnly",
                self.readonly,
                toggle(|this, on| this.readonly = on),
            ));

        Body::new()
            .playground(live)
            .config(config)
            .example(reference)
            .example(states)
    }
}
