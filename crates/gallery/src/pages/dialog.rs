//! Overlays › Dialog: the reference preview (open on arrival, as the
//! reference card is), and a configurable live dialog.
use crate::{controls, stand_in};
use gpui_kit::{
    AnyView, App, AppContext as _, Context, Entity, Focusable as _, IntoElement, ParentElement,
    Render, SharedString, Styled, Window, component::input::InputState, div,
};
use u0ui::components::{Dialog, DialogSize};
use u0ui::prelude::*;

/// Build the page.
pub fn build(window: &mut Window, cx: &mut App) -> AnyView {
    let name = stand_in::input_state("carbon-ui", window, cx);
    let live_name = stand_in::input_state("carbon-ui", window, cx);
    cx.new(|_| DialogPage {
        reference_open: true,
        live_open: false,
        name,
        live_name,
        size: 1,
        danger: false,
        passive: false,
        primary_disabled: false,
        with_label: true,
        with_description: true,
        close_on_backdrop: true,
        last: "—".into(),
    })
    .into()
}

struct DialogPage {
    reference_open: bool,
    live_open: bool,
    name: Entity<InputState>,
    live_name: Entity<InputState>,
    size: usize,
    danger: bool,
    passive: bool,
    primary_disabled: bool,
    with_label: bool,
    with_description: bool,
    close_on_backdrop: bool,
    last: SharedString,
}

impl Render for DialogPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let update = |f: fn(&mut DialogPage, bool)| {
            let entity = entity.clone();
            move |on: bool, _: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    f(this, on);
                    cx.notify();
                })
            }
        };
        let log = |text: &'static str| {
            let entity = entity.clone();
            move |_: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    this.last = text.into();
                    if text != "Primary" {
                        this.live_open = false;
                    }
                    cx.notify();
                })
            }
        };

        // `components/Dialog/preview.html`.
        let reference = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Reference preview"))
            .child(
                Dialog::new("ref-dialog")
                    .open(self.reference_open)
                    .on_open_change(update(|this, open| this.reference_open = open))
                    .trigger(Button::new("ref-open", "Open dialog"))
                    .label("Settings")
                    .title("Rename project")
                    .primary_text("Save")
                    .secondary_text("Cancel")
                    .on_primary({
                        let entity = entity.clone();
                        move |_, cx| {
                            entity.update(cx, |this, cx| {
                                this.reference_open = false;
                                cx.notify();
                            })
                        }
                    })
                    .initial_focus(&self.name.read(cx).focus_handle(cx))
                    .child(stand_in::text_field(
                        "Project name",
                        &self.name,
                        LayerLevel::Two,
                    )),
            );

        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Live"))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .items_center()
                    .gap(tokens::SPACING_05)
                    .p(tokens::SPACING_05)
                    .bg(color(ColorRole::Layer01))
                    .child(
                        Dialog::new("live-dialog")
                            .open(self.live_open)
                            .on_open_change(update(|this, open| this.live_open = open))
                            .trigger(
                                Button::new("live-open", "Open live dialog")
                                    .variant(ButtonVariant::Tertiary),
                            )
                            .size(DialogSize::ALL[self.size])
                            .when_label(self.with_label)
                            .title("Rename project")
                            .when_description(self.with_description)
                            .primary_text(if self.danger { "Delete" } else { "Save" })
                            .secondary_text("Cancel")
                            .danger(self.danger)
                            .passive(self.passive)
                            .primary_disabled(self.primary_disabled)
                            .close_on_backdrop(self.close_on_backdrop)
                            .on_primary(log("Primary"))
                            .on_secondary(log("Secondary"))
                            .child(stand_in::text_field(
                                "Project name",
                                &self.live_name,
                                LayerLevel::Two,
                            )),
                    )
                    .child(
                        div()
                            .type_style(TypeStyle::Label01)
                            .text_color(color(ColorRole::TextSecondary))
                            .child(format!("Last action: {}", self.last)),
                    ),
            );

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "size",
                controls::choices(
                    "dialog-size",
                    &DialogSize::ALL.map(|s| s.name()),
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
                "dialog-danger",
                "danger",
                self.danger,
                update(|this, on| this.danger = on),
            ))
            .child(controls::toggle(
                "dialog-passive",
                "passive",
                self.passive,
                update(|this, on| this.passive = on),
            ))
            .child(controls::toggle(
                "dialog-primary-disabled",
                "primaryDisabled",
                self.primary_disabled,
                update(|this, on| this.primary_disabled = on),
            ))
            .child(controls::toggle(
                "dialog-label",
                "label",
                self.with_label,
                update(|this, on| this.with_label = on),
            ))
            .child(controls::toggle(
                "dialog-description",
                "description",
                self.with_description,
                update(|this, on| this.with_description = on),
            ))
            .child(controls::toggle(
                "dialog-backdrop",
                "close on scrim press",
                self.close_on_backdrop,
                update(|this, on| this.close_on_backdrop = on),
            ));

        controls::page_body(
            div()
                .flex()
                .flex_col()
                .gap(tokens::SPACING_07)
                .child(reference)
                .child(live),
            Some(config.into_any_element()),
        )
    }
}

trait DemoParts {
    fn when_label(self, on: bool) -> Self;
    fn when_description(self, on: bool) -> Self;
}

impl DemoParts for Dialog {
    fn when_label(self, on: bool) -> Self {
        if on { self.label("Settings") } else { self }
    }
    fn when_description(self, on: bool) -> Self {
        if on {
            self.description("The new name appears in every workspace that links this project.")
        } else {
            self
        }
    }
}
