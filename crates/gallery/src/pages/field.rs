//! Forms › Field: the reference Billing contact field set and a configurable
//! live field.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, Entity, IntoElement, ParentElement, Render, Styled,
    Window, div, rems,
};
use u0ui::components::field::{
    Field, FieldContent, FieldDescription, FieldError, FieldGroup, FieldLabel, FieldLegend,
    FieldOrientation, FieldSeparator, FieldSet, FieldTitle,
};
use u0ui::components::{Checkbox, InputGroup, InputGroupInput, InputState, Switch};
use u0ui::prelude::*;

/// Build the page.
pub fn build(window: &mut Window, cx: &mut App) -> AnyView {
    let name = cx.new(|cx| InputState::new(window, cx));
    let vat = cx.new(|cx| InputState::new(window, cx).default_value("DE12"));
    let live = cx.new(|cx| InputState::new(window, cx));
    cx.new(|_| FieldPage {
        name,
        vat,
        live,
        orientation: 0,
        invalid: true,
        description: true,
    })
    .into()
}

struct FieldPage {
    name: Entity<InputState>,
    vat: Entity<InputState>,
    live: Entity<InputState>,
    orientation: usize,
    invalid: bool,
    description: bool,
}

impl Render for FieldPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let toggle = |f: fn(&mut FieldPage, bool)| {
            let entity = entity.clone();
            move |on: bool, _: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    f(this, on);
                    cx.notify();
                })
            }
        };
        let vat_len = self.vat.read(cx).value().chars().count();
        let vat_invalid = !(9..=12).contains(&vat_len);

        // `components/Field/preview.html` (max-width 420).
        let reference = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Reference preview"))
            .child(
                div().max_w(rems(26.25)).child(
                    FieldSet::new()
                        .child(FieldLegend::new("Billing contact"))
                        .child(
                            FieldGroup::new()
                                .child(
                                    Field::new()
                                        .child(FieldLabel::new("Full name"))
                                        .child(natural_width(
                                            InputGroup::new("ref-fn").input(
                                                InputGroupInput::new(&self.name)
                                                    .placeholder("Jane Doe"),
                                            ),
                                        ))
                                        .child(FieldDescription::new("As shown on the invoice.")),
                                )
                                .child(
                                    Field::new()
                                        .invalid(vat_invalid)
                                        .child(FieldLabel::new("VAT number"))
                                        .child(natural_width(
                                            // `.cu-field[data-invalid] .cu-ig`: the field's
                                            // `invalid` outlines the group.
                                            InputGroup::new("ref-vat")
                                                .input(InputGroupInput::new(&self.vat)),
                                        ))
                                        .when_invalid(
                                            vat_invalid,
                                            FieldError::new("VAT numbers have 9–12 characters."),
                                        ),
                                )
                                .child(FieldSeparator::new().text("or"))
                                .child(
                                    Field::new()
                                        .orientation(FieldOrientation::Horizontal)
                                        .child(
                                            Checkbox::new("ref-same")
                                                .label("Same as account owner")
                                                .default_checked(true),
                                        ),
                                ),
                        ),
                ),
            );

        let orientation = FieldOrientation::ALL[self.orientation];
        let horizontal = orientation == FieldOrientation::Horizontal;
        let mut field = Field::new()
            .orientation(orientation)
            .invalid(self.invalid)
            .child(FieldLabel::new("Project ID").no_margin(horizontal))
            .child(
                div().flex_1().w_full().child(
                    InputGroup::new("live-input")
                        .input(InputGroupInput::new(&self.live).placeholder("proj-1234")),
                ),
            );
        if self.description && !horizontal {
            field = field.child(FieldDescription::new("Find it in the project settings."));
        }
        if !horizontal {
            field = field.child(FieldError::errors(if self.invalid {
                vec!["Project IDs start with proj-.", "Use lowercase letters."]
            } else {
                vec![]
            }));
        }
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(div().max_w(rems(26.25)).child(field))
            .child(controls::section("FieldContent + FieldTitle"))
            .child(
                div().max_w(rems(26.25)).child(
                    Field::new()
                        .orientation(FieldOrientation::Horizontal)
                        .child(
                            FieldContent::new()
                                .child(FieldTitle::new("Usage alerts"))
                                .child(FieldDescription::new(
                                    "Email the owners when spend passes 80% of the budget.",
                                )),
                        )
                        .child(Switch::new("live-alerts").default_checked(true)),
                ),
            );

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "orientation",
                controls::choices(
                    "field-orientation",
                    &FieldOrientation::ALL.map(|o| o.name()),
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
                "field-invalid",
                "invalid",
                self.invalid,
                toggle(|this, on| this.invalid = on),
            ))
            .child(controls::toggle(
                "field-description",
                "FieldDescription",
                self.description,
                toggle(|this, on| this.description = on),
            ));

        Body::new()
            .playground(live)
            .config(config)
            .example(reference)
    }
}

/// An input group at the reference's natural width. `.cds--form-item {
/// align-items: flex-start }` shrink-wraps the `.cu-ig`, so it is as wide as
/// its `<input>` (the default `size="20"`, 149px in Plex 14px) plus the
/// `.cu-ig-input` 1rem side padding: 181px (refrender `--computed '.cu-ig'`).
/// GPUI inputs have no intrinsic width, so the preview gives it explicitly.
fn natural_width(group: InputGroup) -> impl IntoElement {
    div().w(rems(181. / 16.)).child(group)
}

trait WhenInvalid {
    fn when_invalid(self, invalid: bool, error: FieldError) -> Self;
}

impl WhenInvalid for Field {
    fn when_invalid(self, invalid: bool, error: FieldError) -> Self {
        if invalid { self.child(error) } else { self }
    }
}
