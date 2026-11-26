//! Navigation › Accordion: the reference preview and a configurable live
//! accordion.
use crate::controls;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled, Window, div,
};
use u0ui::components::{Accordion, AccordionAlign, AccordionItem, AccordionSize};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| AccordionPage {
        align: 1,
        size: 1,
        flush: false,
        disabled: true,
    })
    .into()
}

struct AccordionPage {
    align: usize,
    size: usize,
    flush: bool,
    disabled: bool,
}

impl Render for AccordionPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let toggle = |f: fn(&mut AccordionPage, bool)| {
            let entity = entity.clone();
            move |on: bool, _: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    f(this, on);
                    cx.notify();
                })
            }
        };
        let pick = |f: fn(&mut AccordionPage, usize)| {
            let entity = entity.clone();
            move |i: usize, _: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    f(this, i);
                    cx.notify();
                })
            }
        };

        // `components/Accordion/preview.html`.
        let reference = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Reference preview"))
            .child(
                Accordion::new("ref-accordion")
                    .item(
                        AccordionItem::new("What is a resource group?")
                            .default_open(true)
                            .child("A way to organize account resources for access control and billing."),
                    )
                    .item(
                        AccordionItem::new("How is usage billed?")
                            .child("Per second, rounded to the nearest minute."),
                    )
                    .item(AccordionItem::new("Unavailable section").disabled(true)),
            );

        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Live"))
            .child(
                Accordion::new("live-accordion")
                    .align(AccordionAlign::ALL[self.align])
                    .size(AccordionSize::ALL[self.size])
                    .flush(self.flush)
                    .item(
                        AccordionItem::new("What is a resource group?")
                            .default_open(true)
                            .child("A way to organize account resources for access control and billing."),
                    )
                    .item(
                        AccordionItem::new("How is usage billed?").child(
                            "Per second, rounded to the nearest minute. Reserved capacity is billed monthly in advance, and unused hours do not carry over to the next period.",
                        ),
                    )
                    .item(
                        AccordionItem::new("Can I move resources between groups?")
                            .child("Yes. Moving keeps the resource's history and its access policies are re-evaluated."),
                    )
                    .item(AccordionItem::new("Unavailable section").disabled(self.disabled)),
            );

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "align",
                controls::choices(
                    "acc-align",
                    &AccordionAlign::ALL.map(|a| a.name()),
                    self.align,
                    pick(|this, i| this.align = i),
                ),
            ))
            .child(controls::field(
                "size",
                controls::choices(
                    "acc-size",
                    &AccordionSize::ALL.map(|s| s.name()),
                    self.size,
                    pick(|this, i| this.size = i),
                ),
            ))
            .child(controls::toggle(
                "acc-flush",
                "isFlush",
                self.flush,
                toggle(|this, on| this.flush = on),
            ))
            .child(controls::toggle(
                "acc-disabled",
                "disabled item",
                self.disabled,
                toggle(|this, on| this.disabled = on),
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
