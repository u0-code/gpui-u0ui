//! Chat & AI › Bubble: the reference thread (user, agent with reactions,
//! muted suggestion, destructive error), a configurable live bubble and every
//! variant.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled, Window,
    div, rems,
};
use u0ui::components::bubble::{
    Bubble, BubbleAlign, BubbleContent, BubbleGroup, BubbleReactions, BubbleVariant, ReactionsSide,
};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| BubblePage {
        variant: 1,
        align: 0,
        reactions: true,
        side: 1,
        reactions_align: 1,
        long: false,
    })
    .into()
}

struct BubblePage {
    variant: usize,
    align: usize,
    reactions: bool,
    side: usize,
    reactions_align: usize,
    long: bool,
}

impl Render for BubblePage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();

        // `components/Bubble/preview.html`.
        let reference = BubbleGroup::new()
            .max_w(rems(32.5))
            .child(
                Bubble::new()
                    .align(BubbleAlign::End)
                    .child("Can you summarize last week’s incidents?"),
            )
            .child(
                Bubble::new()
                    .variant(BubbleVariant::Secondary)
                    .child(BubbleContent::new().child(
                        "Two incidents: a 14-minute API latency spike on Tuesday and a failed \
                         backup on Friday. Both are resolved.",
                    ))
                    .child(BubbleReactions::new().child("👍 2")),
            )
            .child(
                Bubble::new()
                    .variant(BubbleVariant::Muted)
                    .child("Suggestion: Show the timeline"),
            )
            .child(
                Bubble::new()
                    .variant(BubbleVariant::Destructive)
                    .child("Couldn’t load Friday’s logs."),
            );

        let text = if self.long {
            "A longer answer wraps inside the bubble: it never grows wider than 36rem or 85% \
             of the column, whichever is smaller, and the square corner stays on the sender’s side."
        } else {
            "eu-de has 14 VMs under 5% CPU for 7 days."
        };
        let live = BubbleGroup::new().max_w(rems(32.5)).pb(rems(1.)).child(
            Bubble::new()
                .variant(BubbleVariant::ALL[self.variant])
                .align(BubbleAlign::ALL[self.align])
                .child(text)
                .when(self.reactions, |this| {
                    this.child(
                        BubbleReactions::new()
                            .side(ReactionsSide::ALL[self.side])
                            .align(BubbleAlign::ALL[self.reactions_align])
                            .child("👍 2")
                            .child("🎉 1"),
                    )
                }),
        );

        let all = BubbleGroup::new()
            .max_w(rems(32.5))
            .children(BubbleVariant::ALL.map(|v| {
                Bubble::new()
                    .variant(v)
                    .align(if v == BubbleVariant::Default {
                        BubbleAlign::End
                    } else {
                        BubbleAlign::Start
                    })
                    .child(v.name())
            }));

        let variants = BubbleVariant::ALL.map(|v| v.name());
        let aligns = BubbleAlign::ALL.map(|a| a.name());
        let sides = ReactionsSide::ALL.map(|s| s.name());
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "variant",
                controls::choices(
                    "bubble-variant",
                    &variants,
                    self.variant,
                    controls::setter(&entity, |this: &mut BubblePage, i| this.variant = i),
                ),
            ))
            .child(controls::field(
                "align",
                controls::choices(
                    "bubble-align",
                    &aligns,
                    self.align,
                    controls::setter(&entity, |this: &mut BubblePage, i| this.align = i),
                ),
            ))
            .child(controls::toggle(
                "bubble-reactions",
                "BubbleReactions",
                self.reactions,
                controls::setter(&entity, |this: &mut BubblePage, on| this.reactions = on),
            ))
            .child(controls::field(
                "reactions side",
                controls::choices(
                    "bubble-side",
                    &sides,
                    self.side,
                    controls::setter(&entity, |this: &mut BubblePage, i| this.side = i),
                ),
            ))
            .child(controls::field(
                "reactions align",
                controls::choices(
                    "bubble-reactions-align",
                    &aligns,
                    self.reactions_align,
                    controls::setter(&entity, |this: &mut BubblePage, i| this.reactions_align = i),
                ),
            ))
            .child(controls::toggle(
                "bubble-long",
                "long text",
                self.long,
                controls::setter(&entity, |this: &mut BubblePage, on| this.long = on),
            ));

        Body::new()
            .playground(div().pt(rems(1.)).child(live))
            .config(config)
            .example(controls::block("Reference preview", reference))
            .example(controls::block("Variants", all))
    }
}

use gpui_kit::prelude::FluentBuilder as _;
