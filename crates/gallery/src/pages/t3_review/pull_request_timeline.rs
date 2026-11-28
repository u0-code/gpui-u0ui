//! T3 · Pull requests & issues › PullRequestTimeline: the reference events
//! (push, change-request review with reactions, label, comment, merge) with
//! working reaction pills, a card whose body the host draws, and
//! ReactionBar on its own.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, rems,
};
use u0ui::t3::review::{
    EventKind, PullRequestTimeline, Reaction, ReactionBar, ReviewDecision, TimelineEvent,
};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        reactions: vec![("👍".into(), 2, true), ("👀".into(), 1, false)],
    })
    .into()
}

struct Page {
    reactions: Vec<(SharedString, usize, bool)>,
}

impl Page {
    fn toggle(&mut self, emoji: &SharedString) {
        if let Some(r) = self.reactions.iter_mut().find(|r| &r.0 == emoji) {
            if r.2 {
                r.1 -= 1;
            } else {
                r.1 += 1;
            }
            r.2 = !r.2;
        }
        self.reactions.retain(|r| r.1 > 0);
    }

    fn pills(&self) -> Vec<Reaction> {
        self.reactions
            .iter()
            .map(|(e, n, mine)| Reaction::new(e.clone(), *n).mine(*mine))
            .collect()
    }
}

/// `components/PullRequestTimeline/preview.html`.
fn events(reactions: Vec<Reaction>) -> Vec<TimelineEvent> {
    vec![
        TimelineEvent::new(EventKind::Commit, "jd", "2h").text("pushed 3 commits"),
        TimelineEvent::new(EventKind::Review, "ak", "1h")
            .review(ReviewDecision::Changes)
            .body("The g90 helper text misses 4.5:1 on layer-02. Can you check?")
            .reactions(reactions),
        TimelineEvent::new(EventKind::Label, "jd", "50m").text("added the design label"),
        TimelineEvent::new(EventKind::Comment, "jd", "30m")
            .body("Fixed in 9f2c1e0. All text tokens pass now."),
        TimelineEvent::new(EventKind::Merged, "mr", "5m").text("merged into main"),
    ]
}

fn reference_reactions() -> Vec<Reaction> {
    vec![Reaction::new("👍", 2).mine(true), Reaction::new("👀", 1)]
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(
            PullRequestTimeline::new("tl-thumb").events(
                events(Vec::new())
                    .into_iter()
                    .filter(|e| !matches!(e.kind(), EventKind::Review | EventKind::Comment)),
            ),
        )
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let live = div().w_full().max_w(rems(42.5)).child(
            PullRequestTimeline::new("tl-live")
                .events(events(self.pills()))
                .on_react({
                    let entity = entity.clone();
                    move |_, emoji, _, cx| {
                        entity.update(cx, |this, cx| {
                            this.toggle(&emoji);
                            cx.notify();
                        })
                    }
                }),
        );
        let bar = ReactionBar::new("tl-bar")
            .reactions(self.pills())
            .on_react(controls::setter(
                &entity,
                |this: &mut Page, e: SharedString| this.toggle(&e),
            ));
        Body::new()
            .playground(live)
            .hint("Press a reaction to add or remove yours")
            .example(controls::block(
                "Reference preview",
                div().w_full().max_w(rems(42.5)).child(
                    PullRequestTimeline::new("tl-ref").events(events(reference_reactions())),
                ),
            ))
            .example(controls::block(
                "A card whose body the host draws (u0: a comment's Markdown)",
                div().w_full().max_w(rems(42.5)).child(
                    PullRequestTimeline::new("tl-content").events([TimelineEvent::new(
                        EventKind::Comment,
                        "ana",
                        "20m",
                    )
                    .content(
                        div()
                            .flex()
                            .flex_col()
                            .gap(rems(0.5))
                            .child(
                                div()
                                    .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                                    .child("Ran"),
                            )
                            .child("cargo test -p gpui-base: 457 passed"),
                    )]),
                ),
            ))
            .example(controls::block("ReactionBar", bar))
    }
}
