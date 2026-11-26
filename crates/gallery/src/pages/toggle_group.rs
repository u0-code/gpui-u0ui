//! Actions › ToggleGroup: the reference content switcher (Day/Week/Month) and
//! multiple-select B/I/U, plus a configurable live group.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, FontWeight, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, rems,
};
use u0ui::components::toggle::ToggleSize;
use u0ui::components::toggle_group::{ToggleGroup, ToggleGroupItem, ToggleGroupType};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| ToggleGroupPage {
        kind: 0,
        size: 1,
        disabled_item: false,
        value: vec!["week".into()],
    })
    .into()
}

struct ToggleGroupPage {
    kind: usize,
    size: usize,
    disabled_item: bool,
    value: Vec<SharedString>,
}

impl Render for ToggleGroupPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let set = |f: fn(&mut ToggleGroupPage, usize)| {
            let entity = entity.clone();
            move |i: usize, _: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    f(this, i);
                    cx.notify();
                })
            }
        };
        let kind = ToggleGroupType::ALL[self.kind];
        let size = ToggleSize::ALL[self.size];

        // `components/ToggleGroup/preview.html` (`cu-stack`).
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
                        div().max_w(rems(26.25)).child(
                            ToggleGroup::new("ref-range")
                                .default_value(["week"])
                                .item(ToggleGroupItem::new("day").child("Day"))
                                .item(ToggleGroupItem::new("week").child("Week"))
                                .item(ToggleGroupItem::new("month").child("Month")),
                        ),
                    )
                    .child(
                        ToggleGroup::new("ref-format")
                            .kind(ToggleGroupType::Multiple)
                            .default_value(["b"])
                            .item(
                                ToggleGroupItem::new("b")
                                    .child(div().font_weight(FontWeight::BOLD).child("B")),
                            )
                            .item(ToggleGroupItem::new("i").child(div().italic().child("I")))
                            .item(ToggleGroupItem::new("u").child(div().underline().child("U"))),
                    ),
            );

        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(
                div().max_w(rems(26.25)).child(
                    ToggleGroup::new(SharedString::from(format!("live-{}", kind.name())))
                        .kind(kind)
                        .size(size)
                        .value(self.value.clone())
                        .on_value_change({
                            let entity = entity.clone();
                            move |value, _, cx| {
                                let value = value.to_vec();
                                entity.update(cx, |this, cx| {
                                    this.value = value;
                                    cx.notify();
                                })
                            }
                        })
                        .item(ToggleGroupItem::new("day").child("Day"))
                        .item(
                            ToggleGroupItem::new("week")
                                .child(Icon::carbon(CarbonIcon::Calendar).size(IconSize::S16))
                                .child("Week"),
                        )
                        .item(
                            ToggleGroupItem::new("month")
                                .disabled(self.disabled_item)
                                .child("Month"),
                        )
                        .item(ToggleGroupItem::new("year").child("Year")),
                ),
            )
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(SharedString::from(format!("value = {:?}", self.value))),
            );

        let sizes = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Sizes"))
            .children(ToggleSize::ALL.iter().map(|s| {
                div().max_w(rems(26.25)).child(
                    ToggleGroup::new(SharedString::from(format!("size-{}", s.name())))
                        .size(*s)
                        .item(ToggleGroupItem::new("a").child(s.name()))
                        .item(ToggleGroupItem::new("b").child("Second"))
                        .item(ToggleGroupItem::new("c").disabled(true).child("Disabled")),
                )
            }));

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "type",
                controls::choices(
                    "tg-type",
                    &ToggleGroupType::ALL.map(|t| t.name()),
                    self.kind,
                    set(|this, i| this.kind = i),
                ),
            ))
            .child(controls::field(
                "size",
                controls::choices(
                    "tg-size",
                    &ToggleSize::ALL.map(|s| s.name()),
                    self.size,
                    set(|this, i| this.size = i),
                ),
            ))
            .child(controls::toggle(
                "tg-disabled",
                "Month disabled",
                self.disabled_item,
                {
                    let entity = entity.clone();
                    move |on, _, cx| {
                        entity.update(cx, |this, cx| {
                            this.disabled_item = on;
                            cx.notify();
                        })
                    }
                },
            ));

        Body::new()
            .playground(live)
            .hint("←/→ move the selection")
            .config(config)
            .example(reference)
            .example(sizes)
    }
}
