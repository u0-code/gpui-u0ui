//! Overlays › Sheet: the reference preview (a contained sheet, open), and a
//! live window-level sheet with every side and option.
use crate::{controls, stand_in};
use gpui_kit::{
    AnyView, App, AppContext as _, Context, Entity, IntoElement, ParentElement, Render, Styled,
    Window, component::input::InputState, div, rems,
};
use u0ui::components::{Sheet, SheetSide};
use u0ui::prelude::*;

/// Build the page.
pub fn build(window: &mut Window, cx: &mut App) -> AnyView {
    let name = stand_in::input_state("vm-104", window, cx);
    let live_name = stand_in::input_state("vm-104", window, cx);
    cx.new(|_| SheetPage {
        name,
        live_name,
        reference_open: true,
        live_open: false,
        contained_open: true,
        side: 0,
        description: true,
        footer: true,
        close_button: true,
    })
    .into()
}

struct SheetPage {
    name: Entity<InputState>,
    live_name: Entity<InputState>,
    reference_open: bool,
    live_open: bool,
    contained_open: bool,
    side: usize,
    description: bool,
    footer: bool,
    close_button: bool,
}

/// The reference footer: Cancel (secondary) + Save, xl, splitting the width.
fn footer_button(id: &'static str, label: &'static str, variant: ButtonVariant) -> Button {
    Button::new(id, label)
        .variant(variant)
        .size(ButtonSize::Xl)
        .full_width(true)
}

impl Render for SheetPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let update = |f: fn(&mut SheetPage, bool)| {
            let entity = entity.clone();
            move |on: bool, _: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    f(this, on);
                    cx.notify();
                })
            }
        };
        let side = SheetSide::ALL[self.side];

        // `components/Sheet/preview.html`: a contained sheet in a 330px box.
        let reference = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Reference preview"))
            .child(
                div()
                    .relative()
                    .h(rems(330. / 16.))
                    .overflow_hidden()
                    .bg(color(ColorRole::Background))
                    .border_1()
                    .border_color(color(ColorRole::BorderSubtle01))
                    .child(
                        div().flex().p(tokens::SPACING_05).child(
                            Button::new("ref-sheet-reopen", "Open sheet")
                                .variant(ButtonVariant::Tertiary)
                                .size(ButtonSize::Sm)
                                .on_click({
                                    let entity = entity.clone();
                                    move |_, _, cx| {
                                        entity.update(cx, |this, cx| {
                                            this.reference_open = true;
                                            cx.notify();
                                        })
                                    }
                                }),
                        ),
                    )
                    .child(
                        Sheet::new("ref-sheet")
                            .contained(true)
                            .open(self.reference_open)
                            .on_open_change(update(|this, open| this.reference_open = open))
                            .title("Edit instance")
                            .description("Changes apply after restart.")
                            .footer(footer_button(
                                "ref-sheet-cancel",
                                "Cancel",
                                ButtonVariant::Secondary,
                            ))
                            .footer(footer_button(
                                "ref-sheet-save",
                                "Save",
                                ButtonVariant::Primary,
                            ))
                            .child(stand_in::text_field(
                                "ref-sheet-name",
                                "Name",
                                &self.name,
                                LayerLevel::One,
                            )),
                    ),
            );

        let close = |field: fn(&mut SheetPage, bool)| {
            let entity = entity.clone();
            move |_: &gpui_kit::ClickEvent, _: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    field(this, false);
                    cx.notify();
                })
            }
        };
        let configured = |sheet: Sheet, prefix: &'static str, name: &Entity<InputState>| {
            let mut sheet = sheet
                .side(side)
                .title("Edit instance")
                .show_close_button(self.close_button)
                .child(stand_in::text_field(prefix, "Name", name, LayerLevel::One));
            if self.description {
                sheet = sheet.description("Changes apply after restart.");
            }
            sheet
        };
        let mut live_sheet = configured(
            Sheet::new("live-sheet")
                .open(self.live_open)
                .on_open_change(update(|this, open| this.live_open = open))
                .trigger(
                    Button::new("live-sheet-open", "Open sheet").variant(ButtonVariant::Tertiary),
                ),
            "live-sheet-name",
            &self.live_name,
        );
        let mut contained_sheet = configured(
            Sheet::new("contained-sheet")
                .contained(true)
                .open(self.contained_open)
                .on_open_change(update(|this, open| this.contained_open = open)),
            "contained-sheet-name",
            &self.name,
        );
        if self.footer {
            live_sheet = live_sheet
                .footer(
                    footer_button("live-cancel", "Cancel", ButtonVariant::Secondary)
                        .on_click(close(|this, v| this.live_open = v)),
                )
                .footer(
                    footer_button("live-save", "Save", ButtonVariant::Primary)
                        .on_click(close(|this, v| this.live_open = v)),
                );
            contained_sheet = contained_sheet
                .footer(
                    footer_button("contained-cancel", "Cancel", ButtonVariant::Secondary)
                        .on_click(close(|this, v| this.contained_open = v)),
                )
                .footer(
                    footer_button("contained-save", "Save", ButtonVariant::Primary)
                        .on_click(close(|this, v| this.contained_open = v)),
                );
        }

        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Live (window)"))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .p(tokens::SPACING_05)
                    .bg(color(ColorRole::Layer01))
                    .child(live_sheet),
            )
            .child(controls::section("Live (contained)"))
            .child(
                div()
                    .relative()
                    .h(rems(22.))
                    .overflow_hidden()
                    .bg(color(ColorRole::Background))
                    .border_1()
                    .border_color(color(ColorRole::BorderSubtle01))
                    .child(
                        div().flex().p(tokens::SPACING_05).child(
                            Button::new("contained-open", "Open contained sheet")
                                .variant(ButtonVariant::Tertiary)
                                .size(ButtonSize::Sm)
                                .on_click({
                                    let entity = entity.clone();
                                    move |_, _, cx| {
                                        entity.update(cx, |this, cx| {
                                            this.contained_open = true;
                                            cx.notify();
                                        })
                                    }
                                }),
                        ),
                    )
                    .child(contained_sheet),
            );

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "side",
                controls::choices(
                    "sheet-side",
                    &SheetSide::ALL.map(|s| s.name()),
                    self.side,
                    {
                        let entity = entity.clone();
                        move |i, _, cx| {
                            entity.update(cx, |this, cx| {
                                this.side = i;
                                cx.notify();
                            })
                        }
                    },
                ),
            ))
            .child(controls::toggle(
                "sheet-description",
                "description",
                self.description,
                update(|this, on| this.description = on),
            ))
            .child(controls::toggle(
                "sheet-footer",
                "footer",
                self.footer,
                update(|this, on| this.footer = on),
            ))
            .child(controls::toggle(
                "sheet-close",
                "showCloseButton",
                self.close_button,
                update(|this, on| this.close_button = on),
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
