//! T3 · Diff & review › ReviewThreadCard: the reference discussion (two
//! comments) and pending draft, u0's unsent draft (Not sent yet, Edit,
//! Delete), and a live card whose Resolve toggles and whose replies, sent
//! with Enter, are added to it.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, rems,
};
use u0ui::components::AvatarColor;
use u0ui::prelude::*;
use u0ui::t3::review::{ReviewComment, ReviewThreadCard};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        resolved: false,
        pending: false,
        replies: 1,
        said: Vec::new(),
    })
    .into()
}

struct Page {
    resolved: bool,
    pending: bool,
    replies: usize,
    /// The replies sent in the live card.
    said: Vec<SharedString>,
}

fn discussion() -> [ReviewComment; 2] {
    [
        ReviewComment::new(
            "jd",
            "2h",
            "Should null be allowed here? The form expects a date.",
        ),
        ReviewComment::new(
            "ak",
            "1h",
            "Yes, the field is optional now. Updated the form schema too.",
        )
        .color(AvatarColor::Teal),
    ]
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(
            ReviewThreadCard::new("rtc-thumb")
                .pending(true)
                .comments([ReviewComment::new(
                    "you",
                    "now",
                    "Nit: rename tz to timeZone.",
                )]),
        )
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        // `components/ReviewThreadCard/preview.html` (`cu-stack`, gap 12).
        let reference = div()
            .flex()
            .flex_col()
            .gap(rems(0.75))
            .child(ReviewThreadCard::new("rtc-ref-0").comments(discussion()))
            .child(ReviewThreadCard::new("rtc-ref-1").pending(true).comments([
                ReviewComment::new("you", "now", "Nit: rename tz to timeZone."),
            ]));
        let said = self
            .said
            .iter()
            .map(|s| ReviewComment::new("you", "now", s.clone()).color(AvatarColor::Gray));
        let live = ReviewThreadCard::new("rtc-live")
            .comments(discussion().into_iter().take(self.replies + 1).chain(said))
            .on_reply({
                let entity = entity.clone();
                move |text, _, cx| {
                    entity.update(cx, |this, cx| {
                        this.said.push(text);
                        cx.notify();
                    })
                }
            })
            .resolved(self.resolved)
            .pending(self.pending)
            .on_resolve({
                let entity = entity.clone();
                move |_, cx| {
                    entity.update(cx, |this, cx| {
                        this.resolved = !this.resolved;
                        cx.notify();
                    })
                }
            });
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::toggle(
                "rtc-resolved",
                "resolved",
                self.resolved,
                controls::setter(&entity, |this: &mut Page, v| this.resolved = v),
            ))
            .child(controls::toggle(
                "rtc-pending",
                "pending",
                self.pending,
                controls::setter(&entity, |this: &mut Page, v| this.pending = v),
            ))
            .child(controls::field(
                "comments",
                controls::choices(
                    "rtc-replies",
                    &["1", "2"],
                    self.replies,
                    controls::setter(&entity, |this: &mut Page, i| this.replies = i),
                ),
            ));
        Body::new()
            .playground(live)
            .hint("Resolve conversation toggles the resolved state; Enter in Reply… adds a reply")
            .config(config)
            .example(controls::block("Reference preview", reference))
            .example(controls::block(
                "Draft not sent yet (u0 Main board)",
                ReviewThreadCard::new("rtc-draft")
                    .draft(true)
                    .comments([ReviewComment::new(
                        "You",
                        "10:22",
                        "Put g90 before g100, like the Carbon docs.",
                    )
                    .note("on line 15")
                    .icon(CarbonIcon::User)
                    .color(AvatarColor::Gray)]),
            ))
    }
}
