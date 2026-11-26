//! Chat & AI › Message: the reference exchange (user at the end, assistant
//! with Copy / Regenerate) and a configurable live message.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled, Window,
    div, prelude::FluentBuilder as _, rems,
};
use u0ui::components::bubble::{Bubble, BubbleAlign, BubbleVariant};
use u0ui::components::message::{
    AvatarKind, Message, MessageAvatar, MessageContent, MessageFooter, MessageGroup, MessageHeader,
};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| MessagePage {
        align: 0,
        kind: 1,
        header: true,
        footer: true,
    })
    .into()
}

struct MessagePage {
    align: usize,
    kind: usize,
    header: bool,
    footer: bool,
}

fn footer(prefix: &'static str) -> MessageFooter {
    MessageFooter::new()
        .child(
            Button::new((prefix, 0usize), "Copy")
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm),
        )
        .child(
            Button::new((prefix, 1usize), "Regenerate")
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm),
        )
}

impl Render for MessagePage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();

        // `components/Message/preview.html`.
        let reference = MessageGroup::new()
            .max_w(rems(35.))
            .child(
                Message::new()
                    .align(BubbleAlign::End)
                    .avatar(MessageAvatar::user())
                    .content(
                        MessageContent::new()
                            .child(MessageHeader::new().name("You").child("10:42"))
                            .child(
                                Bubble::new()
                                    .align(BubbleAlign::End)
                                    .fill_message(true)
                                    .child("Which region has the most idle VMs?"),
                            ),
                    ),
            )
            .child(
                Message::new().avatar(MessageAvatar::agent("AI")).content(
                    MessageContent::new()
                        .child(MessageHeader::new().name("Assistant").child("10:42"))
                        .child(
                            Bubble::new()
                                .variant(BubbleVariant::Secondary)
                                .fill_message(true)
                                .child("eu-de has 14 VMs under 5% CPU for 7 days."),
                        )
                        .child(footer("msg-ref")),
                ),
            );

        let align = BubbleAlign::ALL[self.align];
        let kind = AvatarKind::ALL[self.kind];
        let avatar = match kind {
            AvatarKind::User => MessageAvatar::user(),
            AvatarKind::Agent => MessageAvatar::agent("AI"),
        };
        let live = MessageGroup::new().max_w(rems(35.)).child(
            Message::new().align(align).avatar(avatar).content(
                MessageContent::new()
                    .when(self.header, |this| {
                        this.child(MessageHeader::new().name("Assistant").child("10:42"))
                    })
                    .child(
                        Bubble::new()
                            .align(align)
                            .variant(if kind == AvatarKind::User {
                                BubbleVariant::Default
                            } else {
                                BubbleVariant::Secondary
                            })
                            .fill_message(true)
                            .child("eu-de has 14 VMs under 5% CPU for 7 days."),
                    )
                    .when(self.footer, |this| this.child(footer("msg-live"))),
            ),
        );

        let aligns = BubbleAlign::ALL.map(|a| a.name());
        let kinds = AvatarKind::ALL.map(|k| k.name());
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "align",
                controls::choices(
                    "msg-align",
                    &aligns,
                    self.align,
                    controls::setter(&entity, |this: &mut MessagePage, i| this.align = i),
                ),
            ))
            .child(controls::field(
                "MessageAvatar kind",
                controls::choices(
                    "msg-kind",
                    &kinds,
                    self.kind,
                    controls::setter(&entity, |this: &mut MessagePage, i| this.kind = i),
                ),
            ))
            .child(controls::toggle(
                "msg-header",
                "MessageHeader",
                self.header,
                controls::setter(&entity, |this: &mut MessagePage, on| this.header = on),
            ))
            .child(controls::toggle(
                "msg-footer",
                "MessageFooter",
                self.footer,
                controls::setter(&entity, |this: &mut MessagePage, on| this.footer = on),
            ));

        Body::new()
            .playground(live)
            .config(config)
            .example(controls::block("Reference preview", reference))
    }
}
