//! Data display › Badge (Carbon Tag): the reference rows (shadcn variants,
//! the Carbon colors, a dismissible `region: eu-de`), a live tag with every
//! prop, and every type × kind.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, SharedString,
    Styled, Window, div, prelude::FluentBuilder as _,
};
use u0ui::components::badge::{Badge, BadgeSize, BadgeVariant};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| BadgePage {
        variant: 7,
        size: 1,
        kind: 0,
        disabled: false,
        icon: false,
        selected: false,
        dismissed: false,
        clicks: 0,
    })
    .into()
}

struct BadgePage {
    variant: usize,
    size: usize,
    kind: usize,
    disabled: bool,
    icon: bool,
    selected: bool,
    dismissed: bool,
    clicks: usize,
}

const KINDS: [&str; 4] = ["Tag", "DismissibleTag", "OperationalTag", "SelectableTag"];

fn row() -> gpui_kit::Div {
    div()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap(tokens::SPACING_02)
}

impl Render for BadgePage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let variant = BadgeVariant::ALL[self.variant];
        let size = BadgeSize::ALL[self.size];

        // `components/Badge/preview.html`: two `cu-row`s with 4px gaps, 8px apart.
        let reference = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_03)
            .child(
                row()
                    .child(Badge::new("Default"))
                    .child(Badge::new("Secondary").variant(BadgeVariant::Secondary))
                    .child(Badge::new("Destructive").variant(BadgeVariant::Destructive))
                    .child(Badge::new("Outline").variant(BadgeVariant::Outline)),
            )
            .child(
                row()
                    .children(
                        [
                            BadgeVariant::Blue,
                            BadgeVariant::Cyan,
                            BadgeVariant::Teal,
                            BadgeVariant::Green,
                            BadgeVariant::Purple,
                            BadgeVariant::Magenta,
                            BadgeVariant::Red,
                            BadgeVariant::WarmGray,
                            BadgeVariant::HighContrast,
                        ]
                        .map(|v| Badge::new(v.name()).variant(v)),
                    )
                    .child(
                        Badge::new("region: eu-de")
                            .id("ref-dismiss")
                            .variant(BadgeVariant::Blue)
                            .dismiss_label("Dismiss")
                            .on_dismiss(|_, _, _| {}),
                    ),
            );

        let mut live = Badge::new("Live tag")
            .id("live")
            .variant(variant)
            .size(size)
            .disabled(self.disabled);
        if self.icon {
            live = live.icon(CarbonIcon::Tag);
        }
        live = match self.kind {
            1 => live.dismiss_label("Dismiss").on_dismiss({
                let entity = entity.clone();
                move |_, _, cx| {
                    entity.update(cx, |this, cx| {
                        this.dismissed = true;
                        cx.notify();
                    })
                }
            }),
            2 => live.on_click({
                let entity = entity.clone();
                move |_, _, cx| {
                    entity.update(cx, |this, cx| {
                        this.clicks += 1;
                        cx.notify();
                    })
                }
            }),
            3 => live.selectable(
                self.selected,
                controls::setter(&entity, |this: &mut BadgePage, on| this.selected = on),
            ),
            _ => live,
        };
        let status = match self.kind {
            1 if self.dismissed => "dismissed (toggle a control to restore)".to_string(),
            1 => "press × to dismiss".to_string(),
            2 => format!("clicked {} times", self.clicks),
            3 => format!("selected = {}", self.selected),
            _ => String::new(),
        };
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_04)
            .child(row().when(!(self.kind == 1 && self.dismissed), |r| r.child(live)))
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(SharedString::from(status)),
            );

        // Every Carbon type in each kind, at the live size.
        let matrix =
            div()
                .flex()
                .flex_col()
                .gap(tokens::SPACING_04)
                .children(KINDS.iter().enumerate().map(|(k, kind)| {
                    row()
                        .children(BadgeVariant::ALL[3..].iter().map(|v| {
                            let id = SharedString::from(format!("m-{k}-{}", v.name()));
                            let tag = Badge::new(v.name()).id(id).variant(*v).size(size);
                            match k {
                                1 => tag.dismiss_label("Dismiss").on_dismiss(|_, _, _| {}),
                                2 => tag.on_click(|_, _, _| {}),
                                3 => tag.selectable(*v == BadgeVariant::Blue, |_, _, _| {}),
                                _ => tag,
                            }
                            .into_any_element()
                        }))
                        .child(
                            div()
                                .type_style(TypeStyle::Label01)
                                .text_color(color(ColorRole::TextSecondary))
                                .child(*kind),
                        )
                }));
        let disabled = row().children(KINDS.iter().enumerate().map(|(k, kind)| {
            let tag = Badge::new(*kind)
                .id(SharedString::from(format!("d-{k}")))
                .variant(BadgeVariant::Blue)
                .size(size)
                .disabled(true);
            match k {
                1 => tag.dismiss_label("Dismiss").on_dismiss(|_, _, _| {}),
                2 => tag.on_click(|_, _, _| {}),
                3 => tag.selectable(false, |_, _, _| {}),
                _ => tag,
            }
        }));

        let variants = BadgeVariant::ALL.map(|v| v.name());
        let sizes = BadgeSize::ALL.map(|s| s.name());
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "variant",
                controls::choices(
                    "badge-variant",
                    &variants,
                    self.variant,
                    controls::setter(&entity, |this: &mut BadgePage, i| this.variant = i),
                ),
            ))
            .child(controls::field(
                "size",
                controls::choices(
                    "badge-size",
                    &sizes,
                    self.size,
                    controls::setter(&entity, |this: &mut BadgePage, i| this.size = i),
                ),
            ))
            .child(controls::field(
                "kind (onDismiss / onClick / selectable)",
                controls::choices(
                    "badge-kind",
                    &KINDS,
                    self.kind,
                    controls::setter(&entity, |this: &mut BadgePage, i| {
                        this.kind = i;
                        this.dismissed = false;
                    }),
                ),
            ))
            .child(controls::toggle(
                "badge-icon",
                "icon",
                self.icon,
                controls::setter(&entity, |this: &mut BadgePage, on| {
                    this.icon = on;
                    this.dismissed = false;
                }),
            ))
            .child(controls::toggle(
                "badge-disabled",
                "disabled",
                self.disabled,
                controls::setter(&entity, |this: &mut BadgePage, on| {
                    this.disabled = on;
                    this.dismissed = false;
                }),
            ));

        Body::new()
            .playground(live)
            .config(config)
            .example(controls::block("Reference preview", reference))
            .example(controls::block("Types × kinds", matrix))
            .example(controls::block("Disabled", disabled))
    }
}
