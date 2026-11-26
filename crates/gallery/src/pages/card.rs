//! Data display › Card (Carbon Tile): the reference tiles (Object storage,
//! Clickable tile, Selectable), a live tile of every kind and variant.
use crate::controls;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, SharedString,
    Styled, Window, div, rems,
};
use u0ui::components::badge::{Badge, BadgeSize, BadgeVariant};
use u0ui::components::card::{
    Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle, CardVariant,
};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| CardPage {
        kind: 0,
        variant: 0,
        selected: false,
        expanded: false,
        disabled: false,
        clicks: 0,
    })
    .into()
}

struct CardPage {
    kind: usize,
    variant: usize,
    selected: bool,
    expanded: bool,
    disabled: bool,
    clicks: usize,
}

const KINDS: [&str; 4] = ["Tile", "ClickableTile", "SelectableTile", "ExpandableTile"];

fn header(title: &'static str, description: &'static str) -> CardHeader {
    CardHeader::new()
        .child(CardTitle::new(title))
        .child(CardDescription::new(description))
}

impl Render for CardPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();

        // `components/Card/preview.html`: a `cu-row` (align-items: stretch) of
        // a 300px tile and two 220px tiles.
        let reference = div()
            .flex()
            .flex_row()
            .flex_wrap()
            .gap(tokens::SPACING_05)
            .child(
                div().flex_none().flex().child(
                    Card::new("ref-storage")
                        .w(rems(18.75))
                        .child(
                            header(
                                "Object storage",
                                "Store and retrieve any amount of unstructured data.",
                            )
                            .action(
                                Badge::new("Active")
                                    .variant(BadgeVariant::Green)
                                    .size(BadgeSize::Sm),
                            ),
                        )
                        .child(CardContent::new().child("3 buckets · 1.2 TB"))
                        .child(
                            CardFooter::new().child(
                                Button::new("ref-open", "Open")
                                    .variant(ButtonVariant::Ghost)
                                    .size(ButtonSize::Sm)
                                    .icon(CarbonIcon::ChevronRight),
                            ),
                        ),
                ),
            )
            .child(
                div().flex_none().flex().child(
                    Card::new("ref-click")
                        .w(rems(13.75))
                        .href("https://carbondesignsystem.com/components/tile/usage/")
                        .label("Clickable tile")
                        .child(header("Clickable tile", "The whole tile is the link.")),
                ),
            )
            .child(
                div().flex_none().flex().child(
                    Card::new("ref-select")
                        .w(rems(13.75))
                        .selectable(true)
                        .default_selected(true)
                        .label("Selectable")
                        .child(header("Selectable", "Pick one or more.")),
                ),
            );

        let variant = CardVariant::ALL[self.variant];
        let mut live = Card::new("live")
            .variant(variant)
            .label(KINDS[self.kind])
            .child(header(
                KINDS[self.kind],
                "A live tile; change it in the panel.",
            ))
            .child(CardContent::new().child("3 buckets · 1.2 TB"));
        live = match self.kind {
            1 => live.on_click({
                let entity = entity.clone();
                move |_, _, cx| {
                    entity.update(cx, |this, cx| {
                        this.clicks += 1;
                        cx.notify();
                    })
                }
            }),
            2 => live
                .selectable(true)
                .selected(self.selected)
                .disabled(self.disabled)
                .on_selected_change(controls::setter(&entity, |this: &mut CardPage, on| {
                    this.selected = on
                })),
            3 => live
                .expandable(
                    CardContent::new().child("Below the fold: 12 objects · eu-de · versioning on."),
                )
                .expanded(self.expanded)
                .on_expanded_change(controls::setter(&entity, |this: &mut CardPage, on| {
                    this.expanded = on
                })),
            _ => live,
        };
        let status = match self.kind {
            1 => format!("clicked {} times", self.clicks),
            2 => format!("selected = {}", self.selected),
            3 => format!("expanded = {}", self.expanded),
            _ => String::new(),
        };
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_04)
            .child(live.w(rems(20.)))
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(SharedString::from(status)),
            );

        // Every kind side by side, on background and on layer-01.
        let kinds = |level: LayerLevel, prefix: &'static str| {
            div()
                .flex()
                .flex_row()
                .flex_wrap()
                .gap(tokens::SPACING_05)
                .children(KINDS.iter().enumerate().map(move |(k, name)| {
                    let card = Card::new(SharedString::from(format!("{prefix}-{k}")))
                        .w(rems(13.75))
                        .layer(level)
                        .label(*name)
                        .child(header(name, "Hover, focus, press."));
                    let card = match k {
                        1 => card.on_click(|_, _, _| {}),
                        2 => card.selectable(true),
                        3 => card.expandable(CardContent::new().child("More detail.")),
                        _ => card,
                    };
                    div().flex_none().flex().child(card)
                }))
        };
        let matrix = div()
            .flex()
            .flex_col()
            .child(
                div()
                    .py(tokens::SPACING_05)
                    .child(kinds(LayerLevel::One, "m1")),
            )
            .child(
                div()
                    .bg(color(ColorRole::Layer01))
                    .p(tokens::SPACING_05)
                    .child(kinds(LayerLevel::Two, "m2")),
            );

        let variants = CardVariant::ALL.map(|v| v.name());
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "kind (href / selectable / expandable)",
                controls::choices(
                    "card-kind",
                    &KINDS,
                    self.kind,
                    controls::setter(&entity, |this: &mut CardPage, i| this.kind = i),
                ),
            ))
            .child(controls::field(
                "variant",
                controls::choices(
                    "card-variant",
                    &variants,
                    self.variant,
                    controls::setter(&entity, |this: &mut CardPage, i| this.variant = i),
                ),
            ))
            .child(controls::toggle(
                "card-selected",
                "selected",
                self.selected,
                controls::setter(&entity, |this: &mut CardPage, on| this.selected = on),
            ))
            .child(controls::toggle(
                "card-expanded",
                "expanded",
                self.expanded,
                controls::setter(&entity, |this: &mut CardPage, on| this.expanded = on),
            ))
            .child(controls::toggle(
                "card-disabled",
                "disabled (selectable)",
                self.disabled,
                controls::setter(&entity, |this: &mut CardPage, on| this.disabled = on),
            ));

        controls::page_body(
            div()
                .flex()
                .flex_col()
                .gap(tokens::SPACING_07)
                .child(controls::block("Reference preview", reference))
                .child(controls::block("Live", live))
                .child(controls::block("Kinds on background and layer-01", matrix)),
            Some(config.into_any_element()),
        )
    }
}
