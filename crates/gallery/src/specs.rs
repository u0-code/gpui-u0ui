//! Specs tabs, md3.io-style: a component's anatomy (the real component with
//! its numbered parts), its measurements in each size and density, and its
//! colors in each state, all read from the component's own look and
//! geometry functions (so the tab cannot drift from what is drawn) and shown
//! in the active theme.
//!
//! A page gets a tab with `.specs(specs::<name>)` on its registry line; a
//! spec is `fn(&mut Window, &mut App) -> AnyElement` built from the helpers
//! here ([`spec_page`], [`table`], [`swatch`]).
use gpui_kit::{
    AnyElement, App, Hsla, IntoElement, ParentElement, SharedString, Styled, Window, div,
    prelude::FluentBuilder as _, px, rems,
};
use u0ui::components::badge::{self, TagMode, TagState, TagType};
use u0ui::components::button::{ButtonState, Geometry, Look};
use u0ui::components::checkbox::{self, CheckboxLook, CheckboxLookState};
use u0ui::components::input::{FieldLook, FieldState};
use u0ui::components::switch::{SwitchGeometry, SwitchLook, SwitchSize, SwitchState};
use u0ui::components::table::{Table, TableBody, TableCell, TableHead, TableHeader, TableRow};
use u0ui::components::{
    Badge, BadgeSize, BadgeVariant, Checkbox, CheckboxState, FieldSize, Input, InputState, Switch,
};
use u0ui::prelude::*;

/// `#rrggbb` (or `#rrggbbaa` when translucent) of a color.
pub fn hex(color: Hsla) -> String {
    let c = color.to_rgb();
    let b = |v: f32| (v.clamp(0., 1.) * 255.).round() as u8;
    if c.a < 1. {
        format!("#{:02x}{:02x}{:02x}{:02x}", b(c.r), b(c.g), b(c.b), b(c.a))
    } else {
        format!("#{:02x}{:02x}{:02x}", b(c.r), b(c.g), b(c.b))
    }
}

/// A color chip and its hex value (`code-01`).
pub fn swatch(color: Hsla) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(tokens::SPACING_03)
        .child(
            div()
                .size(rems(1.))
                .flex_shrink_0()
                .bg(color)
                .border_1()
                .border_color(theme::color(ColorRole::BorderSubtle01)),
        )
        .child(
            div()
                .type_style(TypeStyle::Code01)
                .text_color(theme::color(ColorRole::TextSecondary))
                .child(hex(color)),
        )
        .into_any_element()
}

/// A swatch, or an em dash for no color.
fn maybe(color: Option<Hsla>) -> AnyElement {
    match color {
        Some(c) => swatch(c),
        None => text("—"),
    }
}

fn text(s: impl Into<SharedString>) -> AnyElement {
    div().child(s.into()).into_any_element()
}

fn px_text(v: f32) -> AnyElement {
    text(format!("{v}px"))
}

/// A compact Carbon data table of `rows` under `headers`.
pub fn table(id: &'static str, headers: &[&'static str], rows: Vec<Vec<AnyElement>>) -> AnyElement {
    Table::new(id)
        .size(u0ui::components::table::TableSize::Sm)
        .header(
            TableHeader::new()
                .child(TableRow::new().children(headers.iter().map(|h| TableHead::new(*h)))),
        )
        .body(
            TableBody::new().children(
                rows.into_iter()
                    .map(|cells| TableRow::new().children(cells.into_iter().map(TableCell::new))),
            ),
        )
        .into_any_element()
}

/// The specimen on a stage beside its numbered parts.
fn anatomy(specimen: AnyElement, parts: &[&'static str]) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .flex_wrap()
        .gap(tokens::SPACING_07)
        .child(
            div()
                .flex_1()
                .min_w(rems(16.))
                .min_h(rems(12.))
                .p(tokens::SPACING_07)
                .flex()
                .items_center()
                .justify_center()
                .bg(theme::color(ColorRole::Background))
                .border_1()
                .border_color(theme::color(ColorRole::BorderSubtle01))
                .child(specimen),
        )
        .child(
            div()
                .flex_basis(rems(18.))
                .flex_grow(1.)
                .max_w(rems(24.))
                .min_w(rems(12.))
                .flex()
                .flex_col()
                .gap(tokens::SPACING_04)
                .children(parts.iter().enumerate().map(|(i, part)| {
                    div()
                        .flex()
                        .flex_row()
                        .items_start()
                        .gap(tokens::SPACING_04)
                        .child(
                            // The part number in a 24px `background-inverse` square.
                            div()
                                .size(rems(1.5))
                                .flex_shrink_0()
                                .flex()
                                .items_center()
                                .justify_center()
                                .bg(theme::color(ColorRole::BackgroundInverse))
                                .text_color(theme::color(ColorRole::TextInverse))
                                .type_style(TypeStyle::Label01)
                                .child(format!("{}", i + 1)),
                        )
                        .child(
                            // Wraps inside the legend column instead of running
                            // past it.
                            div()
                                .flex_1()
                                .min_w_0()
                                .pt(px(3.))
                                .type_style(TypeStyle::BodyCompact01)
                                .child(crate::controls::prose(
                                    part,
                                    theme::color(ColorRole::TextPrimary),
                                )),
                        )
                })),
        )
        .into_any_element()
}

/// A titled spec section.
fn section(
    title: &'static str,
    note: Option<&'static str>,
    content: AnyElement,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_05)
        .child(
            div()
                .type_style(TypeStyle::Heading03)
                .text_color(theme::color(ColorRole::TextPrimary))
                .child(title),
        )
        .when_some(note, |this, note| {
            this.child(div().max_w(rems(40.)).type_style(TypeStyle::Body01).child(
                crate::controls::prose(note, theme::color(ColorRole::TextSecondary)),
            ))
        })
        .child(content)
}

/// Anatomy, measurements and state colors, stacked.
pub fn spec_page(
    anatomy: AnyElement,
    measurements: AnyElement,
    measurements_note: &'static str,
    colors: AnyElement,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_09)
        .child(section("Anatomy", None, anatomy))
        .child(section(
            "Measurements",
            Some(measurements_note),
            measurements,
        ))
        .child(section(
            "State colors",
            Some("Resolved in the active theme; switch themes to compare."),
            colors,
        ))
        .into_any_element()
}

/// Button: kinds × states and sizes × densities.
pub fn button(_: &mut Window, _: &mut App) -> AnyElement {
    let specimen = Button::new("spec-button", "Create project")
        .icon(CarbonIcon::Add)
        .size(ButtonSize::Lg)
        .into_any_element();
    let mut sizes = Vec::new();
    for size in ButtonSize::ALL {
        let compact = Geometry::of(size, Density::Compact, ButtonKind::Primary, false);
        let comfortable = Geometry::of(size, Density::Comfortable, ButtonKind::Primary, false);
        sizes.push(vec![
            text(size.name()),
            px_text(size.carbon_height()),
            px_text(compact.height),
            px_text(comfortable.height),
            text(format!(
                "{} / {} / {}",
                comfortable.padding_top, comfortable.padding_right, comfortable.padding_left
            )),
        ]);
    }
    let states = [
        ("Enabled", ButtonState::default()),
        (
            "Hover",
            ButtonState {
                hovered: true,
                ..Default::default()
            },
        ),
        (
            "Active",
            ButtonState {
                pressed: true,
                ..Default::default()
            },
        ),
        (
            "Disabled",
            ButtonState {
                disabled: true,
                ..Default::default()
            },
        ),
    ];
    let kinds = [
        ("Primary", ButtonKind::Primary),
        ("Secondary", ButtonKind::Secondary),
        ("Tertiary", ButtonKind::Tertiary),
        ("Ghost", ButtonKind::Ghost),
        ("Danger", ButtonKind::Danger),
        ("Danger tertiary", ButtonKind::DangerTertiary),
        ("Danger ghost", ButtonKind::DangerGhost),
    ];
    let rows = kinds
        .iter()
        .map(|(name, kind)| {
            let mut cells = vec![text(*name)];
            cells.extend(
                states
                    .iter()
                    .map(|(_, state)| swatch(Look::of(*kind, *state, false).background)),
            );
            cells.push(swatch(Look::of(*kind, ButtonState::default(), false).text));
            cells
        })
        .collect();
    let focus = Look::of(
        ButtonKind::Primary,
        ButtonState {
            focused: true,
            ..Default::default()
        },
        false,
    );
    spec_page(
        anatomy(
            specimen,
            &[
                "Container: square, kind fill (`button-*`)",
                "Label: body-compact-01, text-on-color",
                "Icon: 16px Material Symbol, 16px from the end",
                "Focus: 2px focus ring, 1px focus-inset inside on filled kinds",
            ],
        ),
        div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(table(
                "spec-button-sizes",
                &[
                    "Size",
                    "Carbon",
                    "Compact",
                    "Comfortable",
                    "Padding t / r / l",
                ],
                sizes,
            ))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap(tokens::SPACING_06)
                    .type_style(TypeStyle::BodyCompact01)
                    .child("Focus ring")
                    .child(maybe(focus.focus_ring))
                    .child("Focus inset")
                    .child(maybe(focus.focus_inset)),
            )
            .into_any_element(),
        "Heights by size in Carbon's own scale and in the gallery's two densities; padding in px.",
        table(
            "spec-button-colors",
            &["Kind", "Enabled", "Hover", "Active", "Disabled", "Text"],
            rows,
        ),
    )
}

/// Input: field sizes and the field look on each layer.
pub fn input(window: &mut Window, cx: &mut App) -> AnyElement {
    let state = window.use_keyed_state("spec-input-state", cx, |window, cx| {
        InputState::new(window, cx).default_value("web-frontend")
    });
    let specimen = div()
        .w(rems(18.))
        .child(
            Input::new("spec-input", &state)
                .label("Project name")
                .helper_text("Lowercase letters, numbers and dashes."),
        )
        .into_any_element();
    let sizes = FieldSize::ALL
        .iter()
        .map(|size| {
            vec![
                text(size.name()),
                px_text(size.carbon_height()),
                px_text(size.height(Density::Compact)),
                px_text(size.height(Density::Comfortable)),
            ]
        })
        .collect();
    let states = [
        ("Enabled", FieldState::default()),
        (
            "Focused",
            FieldState {
                focused: true,
                ..Default::default()
            },
        ),
        (
            "Disabled",
            FieldState {
                disabled: true,
                ..Default::default()
            },
        ),
        (
            "Read-only",
            FieldState {
                readonly: true,
                ..Default::default()
            },
        ),
    ];
    let mut rows = Vec::new();
    for (level_name, level) in [
        ("layer 1", LayerLevel::One),
        ("layer 2", LayerLevel::Two),
        ("layer 3", LayerLevel::Three),
    ] {
        for (name, state) in states {
            let look = FieldLook::of(level, state);
            rows.push(vec![
                text(format!("{name}, {level_name}")),
                swatch(look.background),
                swatch(look.border_bottom),
                swatch(look.text),
                swatch(look.placeholder),
            ]);
        }
    }
    spec_page(
        anatomy(
            specimen,
            &[
                "Label: label-01, text-secondary, 8px above",
                "Field: field-0x fill, no side borders",
                "Bottom rule: 1px border-strong-0x",
                "Helper text: helper-text-01, text-helper",
                "Focus: 2px focus outline; invalid: 2px support-error + icon",
            ],
        ),
        table(
            "spec-input-sizes",
            &["Size", "Carbon", "Compact", "Comfortable"],
            sizes,
        ),
        "Field heights by size. Text sits 16px from the start; 40px is reserved at the end for a status icon.",
        table(
            "spec-input-colors",
            &["State", "Fill", "Bottom rule", "Text", "Placeholder"],
            rows,
        ),
    )
}

/// Checkbox: box geometry and the look of each value and state.
pub fn checkbox(_: &mut Window, _: &mut App) -> AnyElement {
    let specimen = Checkbox::new("spec-checkbox")
        .label("Enable audit logs")
        .default_checked(true)
        .into_any_element();
    let values = [
        ("Unchecked", CheckboxState::Unchecked),
        ("Checked", CheckboxState::Checked),
        ("Indeterminate", CheckboxState::Indeterminate),
    ];
    let mut rows = Vec::new();
    for (name, value) in values {
        for (state, flags) in [
            ("", (false, false, false)),
            (", disabled", (true, false, false)),
            (", invalid", (false, true, false)),
            (", read-only", (false, false, true)),
        ] {
            let look = CheckboxLook::of(CheckboxLookState {
                value,
                disabled: flags.0,
                invalid: flags.1,
                readonly: flags.2,
            });
            rows.push(vec![
                text(format!("{name}{state}")),
                swatch(look.fill),
                maybe(look.border),
                swatch(look.mark),
                swatch(look.label),
            ]);
        }
    }
    spec_page(
        anatomy(
            specimen,
            &[
                "Box: 16px, 2px corner radius, 1px icon-primary border",
                "Checkmark: icon-inverse on icon-primary fill",
                "Label: body-compact-01, 30px from the box's start",
                "Focus: 2px focus outline around the box",
            ],
        ),
        table(
            "spec-checkbox-sizes",
            &["Part", "Value"],
            vec![
                vec![text("Box"), px_text(checkbox::BOX)],
                vec![
                    text("Box inset from the row top"),
                    px_text(checkbox::BOX_INSET),
                ],
                vec![
                    text("Box inset from the row start"),
                    px_text(checkbox::BOX_INSET),
                ],
                vec![text("Label start"), px_text(checkbox::LABEL_START)],
                vec![
                    text("Focus outline (width / offset)"),
                    text(format!(
                        "2px / {}px",
                        -checkbox::focus_ring_inset(false) - 2.
                    )),
                ],
            ],
        ),
        "From `.cds--checkbox-label::before` (`margin-inline-start: 0.1875rem`), the label's \
         padding and `:focus + .cds--checkbox-label::before { outline-offset: 1px }`.",
        table(
            "spec-checkbox-colors",
            &["State", "Fill", "Border", "Mark", "Label"],
            rows,
        ),
    )
}

/// Switch: track and knob geometry and colors.
pub fn switch(_: &mut Window, _: &mut App) -> AnyElement {
    let specimen = Switch::new("spec-switch")
        .label("Auto-scaling")
        .label_on("On")
        .label_off("Off")
        .default_checked(true)
        .into_any_element();
    let sizes = SwitchSize::ALL
        .iter()
        .map(|size| {
            let g = SwitchGeometry::of(*size, false);
            vec![
                text(size.name()),
                text(format!("{} × {}px", g.width, g.height)),
                px_text(g.knob),
                px_text(g.inset),
                px_text(SwitchGeometry::of(*size, true).inset),
                px_text(g.travel),
            ]
        })
        .collect();
    let mut rows = Vec::new();
    for checked in [false, true] {
        for (name, disabled, readonly) in [
            ("", false, false),
            (", disabled", true, false),
            (", read-only", false, true),
        ] {
            let look = SwitchLook::of(SwitchState {
                checked,
                disabled,
                readonly,
            });
            rows.push(vec![
                text(format!("{}{name}", if checked { "On" } else { "Off" })),
                swatch(look.track),
                maybe(look.track_border),
                swatch(look.knob),
                swatch(look.check),
            ]);
        }
    }
    spec_page(
        anatomy(
            specimen,
            &[
                "Label: label-01, text-secondary",
                "Track: pill (border-radius-16)",
                "Knob: full radius, slides by the travel",
                "State text: body-compact-01 (On / Off)",
            ],
        ),
        table(
            "spec-switch-sizes",
            &[
                "Size",
                "Track",
                "Knob",
                "Inset",
                "Read-only inset",
                "Travel",
            ],
            sizes,
        ),
        "From `.cds--toggle__switch` and its `::before` knob; read-only moves the knob to a \
         2px inset (`.cds--toggle--readonly .cds--toggle__switch::before`).",
        table(
            "spec-switch-colors",
            &["State", "Track", "Track border", "Knob", "Check (sm)"],
            rows,
        ),
    )
}

/// Badge (Carbon Tag): sizes and the ten tag colors.
pub fn badge(_: &mut Window, _: &mut App) -> AnyElement {
    let specimen = div()
        .flex()
        .flex_row()
        .gap(tokens::SPACING_03)
        .child(Badge::new("Running").variant(BadgeVariant::Green))
        .child(
            Badge::new("Beta")
                .variant(BadgeVariant::Purple)
                .size(BadgeSize::Lg),
        )
        .into_any_element();
    let sizes = BadgeSize::ALL
        .iter()
        .map(|size| {
            let g = badge::Geometry::of(*size, false, false);
            vec![
                text(size.name()),
                px_text(g.height),
                px_text(g.padding_left),
                px_text(g.padding_right),
            ]
        })
        .collect();
    let rows = TagType::COLORS
        .iter()
        .map(|ty| {
            let look = badge::Look::of(*ty, TagMode::Static, TagState::default(), LayerLevel::One);
            let roles = ty.roles();
            vec![
                text(
                    roles
                        .map(|r| r.0.name().trim_start_matches("tag-background-").to_string())
                        .unwrap_or_default(),
                ),
                swatch(look.background),
                swatch(look.text),
                maybe(roles.map(|r| color(r.2))),
            ]
        })
        .collect();
    spec_page(
        anatomy(
            specimen,
            &[
                "Container: pill (border-radius-16), tag-background-*",
                "Label: label-01, tag-color-*",
                "Optional 16px icon or close button",
            ],
        ),
        table(
            "spec-badge-sizes",
            &["Size", "Height", "Padding start", "Padding end"],
            sizes,
        ),
        "From `.cds--tag` and its size modifiers.",
        table(
            "spec-badge-colors",
            &["Type", "Background", "Text", "Hover"],
            rows,
        ),
    )
}
