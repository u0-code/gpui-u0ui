//! Actions › ButtonGroup: the reference sets (horizontal Cancel/Save, stacked
//! Continue/Back) and a configurable live set.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, SharedString,
    Styled, Window, div, rems,
};
use u0ui::components::button_group::{
    ButtonGroup, ButtonGroupOrientation, ButtonGroupSeparator, ButtonGroupText,
};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| ButtonGroupPage {
        orientation: 0,
        buttons: 1,
        disabled: false,
        parts: false,
        last: None,
    })
    .into()
}

struct ButtonGroupPage {
    orientation: usize,
    /// 0: two buttons, 1: three buttons.
    buttons: usize,
    disabled: bool,
    parts: bool,
    last: Option<&'static str>,
}

impl Render for ButtonGroupPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let toggle = |f: fn(&mut ButtonGroupPage, bool)| {
            let entity = entity.clone();
            move |on: bool, _: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    f(this, on);
                    cx.notify();
                })
            }
        };
        let click = |name: &'static str| {
            let entity = entity.clone();
            move |_: &gpui_kit::ClickEvent, _: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    this.last = Some(name);
                    cx.notify();
                })
            }
        };
        let orientation = ButtonGroupOrientation::ALL[self.orientation];

        // `components/ButtonGroup/preview.html` (`cu-stack`).
        let reference = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Reference preview"))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(tokens::SPACING_06)
                    .child(
                        ButtonGroup::new()
                            .button(
                                Button::new("ref-cancel", "Cancel")
                                    .variant(ButtonVariant::Secondary),
                            )
                            .button(Button::new("ref-save", "Save changes")),
                    )
                    .child(
                        div().max_w(rems(15.)).child(
                            ButtonGroup::new()
                                .orientation(ButtonGroupOrientation::Vertical)
                                .button(Button::new("ref-continue", "Continue"))
                                .button(
                                    Button::new("ref-back", "Back").variant(ButtonVariant::Ghost),
                                ),
                        ),
                    ),
            );

        let mut group = ButtonGroup::new().orientation(orientation);
        if self.parts {
            group = group
                .child(ButtonGroupText::new("Draft"))
                .child(ButtonGroupSeparator::new());
        }
        if self.buttons == 1 {
            group = group.button(
                Button::new("live-delete", "Delete")
                    .variant(ButtonVariant::DangerGhost)
                    .disabled(self.disabled)
                    .on_click(click("Delete")),
            );
        }
        group = group
            .button(
                Button::new("live-cancel", "Cancel")
                    .variant(ButtonVariant::Secondary)
                    .disabled(self.disabled)
                    .on_click(click("Cancel")),
            )
            .button(
                Button::new("live-save", "Save changes")
                    .disabled(self.disabled)
                    .on_click(click("Save changes")),
            );

        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(
                div()
                    .when_vertical(orientation)
                    .p(tokens::SPACING_05)
                    .bg(color(ColorRole::Layer01))
                    .child(group),
            )
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(SharedString::from(format!(
                        "Last action: {}",
                        self.last.unwrap_or("—")
                    ))),
            );

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "orientation",
                controls::choices(
                    "bg-orientation",
                    &ButtonGroupOrientation::ALL.map(|o| o.name()),
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
            .child(controls::field(
                "buttons",
                controls::choices("bg-buttons", &["2", "3"], self.buttons, {
                    let entity = entity.clone();
                    move |i, _, cx| {
                        entity.update(cx, |this, cx| {
                            this.buttons = i;
                            cx.notify();
                        })
                    }
                }),
            ))
            .child(controls::toggle(
                "bg-parts",
                "ButtonGroupText + Separator",
                self.parts,
                toggle(|this, on| this.parts = on),
            ))
            .child(controls::toggle(
                "bg-disabled",
                "disabled",
                self.disabled,
                toggle(|this, on| this.disabled = on),
            ));

        Body::new()
            .playground(live)
            .config(config)
            .example(reference)
    }
}

trait WhenVertical {
    fn when_vertical(self, orientation: ButtonGroupOrientation) -> Self;
}

impl WhenVertical for gpui_kit::Div {
    fn when_vertical(self, orientation: ButtonGroupOrientation) -> Self {
        if orientation == ButtonGroupOrientation::Vertical {
            self.max_w(rems(15.))
        } else {
            self
        }
    }
}
