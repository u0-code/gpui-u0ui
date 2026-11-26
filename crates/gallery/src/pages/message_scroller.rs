//! Chat & AI › MessageScroller: the reference 260px transcript of eight
//! questions and replies, and a live one that grows ("Add message") so the
//! follow / stop-following / jump behavior can be tried.
use crate::controls;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled, Window,
    div, px,
};
use u0ui::components::bubble::{Bubble, BubbleAlign, BubbleVariant};
use u0ui::components::message_scroller::{MessageScroller, MessageScrollerItem};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| ScrollerPage {
        count: 8,
        auto_scroll: true,
        height: 1,
    })
    .into()
}

struct ScrollerPage {
    count: usize,
    auto_scroll: bool,
    height: usize,
}

const HEIGHTS: [f32; 3] = [200., 260., 320.];
const HEIGHT_NAMES: [&str; 3] = ["200", "260", "320 (default)"];

/// `preview.html`: odd items are agent replies at the start, even ones user
/// questions at the end.
fn item(prefix: &'static str, i: usize) -> MessageScrollerItem {
    let reply = i % 2 == 1;
    MessageScrollerItem::new((prefix, i)).child(
        Bubble::new()
            .align(if reply {
                BubbleAlign::Start
            } else {
                BubbleAlign::End
            })
            .variant(if reply {
                BubbleVariant::Secondary
            } else {
                BubbleVariant::Default
            })
            .child(if reply {
                format!("Reply {i}")
            } else {
                format!("Question {i}")
            }),
    )
}

impl Render for ScrollerPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();

        let reference = MessageScroller::new("scroller-ref")
            .height(px(260.))
            .jump_label("Jump to latest")
            .children((0..8).map(|i| item("scroller-ref-item", i)));

        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_04)
            .child(
                MessageScroller::new("scroller-live")
                    .height(px(HEIGHTS[self.height]))
                    .auto_scroll(self.auto_scroll)
                    .jump_label("Jump to latest")
                    .children((0..self.count).map(|i| item("scroller-live-item", i))),
            )
            .child(
                div().flex().child(
                    Button::new("scroller-add", "Add message")
                        .variant(ButtonVariant::Tertiary)
                        .size(ButtonSize::Sm)
                        .icon(CarbonIcon::Add)
                        .on_click({
                            let entity = entity.clone();
                            move |_, _, cx| {
                                entity.update(cx, |this, cx| {
                                    this.count += 1;
                                    cx.notify();
                                })
                            }
                        }),
                ),
            );

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::toggle(
                "scroller-auto",
                "autoScroll",
                self.auto_scroll,
                controls::setter(&entity, |this: &mut ScrollerPage, on| this.auto_scroll = on),
            ))
            .child(controls::field(
                "height",
                controls::choices(
                    "scroller-height",
                    &HEIGHT_NAMES,
                    self.height,
                    controls::setter(&entity, |this: &mut ScrollerPage, i| this.height = i),
                ),
            ));

        let note = div()
            .type_style(TypeStyle::Label01)
            .text_color(color(ColorRole::TextSecondary))
            .child(
                "At the end, new messages scroll into view. Scroll up to stop following; \
                 \"Jump to latest\" returns.",
            );

        controls::page_body(
            div()
                .flex()
                .flex_col()
                .gap(tokens::SPACING_07)
                .child(controls::block("Reference preview", reference))
                .child(controls::block("Live", live.child(note))),
            Some(config.into_any_element()),
        )
    }
}
