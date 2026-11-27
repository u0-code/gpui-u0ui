//! T3 · Pull requests & issues › PullRequestStatus: the PR vocabulary as in
//! the reference — state glyphs with their names and two mini badges;
//! checks glyphs and review decisions; label chips and a diff stat.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, rems, rgb,
};
use u0ui::prelude::*;
use u0ui::t3::review::{
    ChecksGlyph, ChecksStatus, DiffStat, LabelChip, PullRequestState, PullRequestStateGlyph,
    ReviewDecision, ReviewDecisionBadge,
};
use u0ui::t3::shell::PullRequestMiniBadge;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        state: 0,
        checks: 0,
        review: 0,
    })
    .into()
}

struct Page {
    state: usize,
    checks: usize,
    review: usize,
}

fn row() -> gpui_kit::Div {
    div()
        .flex()
        .flex_wrap()
        .items_center()
        .gap(tokens::SPACING_05)
}

/// `components/PullRequestStatus/preview.html`.
fn reference() -> gpui_kit::Div {
    div()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_05)
        .child(
            row()
                .children(PullRequestState::ALL.map(|s| {
                    div()
                        .flex()
                        .items_center()
                        .gap(rems(0.375))
                        .text_size(rems(0.75))
                        .text_color(color(ColorRole::TextPrimary))
                        .child(PullRequestStateGlyph::new(s))
                        .child(s.name())
                }))
                .child(PullRequestMiniBadge::new(482, PullRequestState::Open))
                .child(PullRequestMiniBadge::new(470, PullRequestState::Merged)),
        )
        .child(
            row()
                .children(ChecksStatus::ALL.map(ChecksGlyph::new))
                .children(ReviewDecision::ALL.map(ReviewDecisionBadge::new)),
        )
        .child(
            row()
                .child(LabelChip::new("bug", rgb(0xda1e28)))
                .child(LabelChip::new("design", rgb(0x8a3ffc)))
                .child(LabelChip::new("good first issue", rgb(0x24a148)))
                .child(DiffStat::new(312, 14).bar(true)),
        )
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    row()
        .w(rems(15.))
        .children(PullRequestState::ALL.map(PullRequestStateGlyph::new))
        .child(ReviewDecisionBadge::new(ReviewDecision::Approved))
        .child(LabelChip::new("design", rgb(0x8a3ffc)))
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let live = row()
            .child(PullRequestStateGlyph::new(PullRequestState::ALL[self.state]).size(20.))
            .child(ChecksGlyph::new(ChecksStatus::ALL[self.checks]))
            .child(ReviewDecisionBadge::new(ReviewDecision::ALL[self.review]));
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::field(
                "PullRequestStateGlyph state",
                controls::choices(
                    "prs-state",
                    &["open", "draft", "merged", "closed"],
                    self.state,
                    controls::setter(&entity, |this: &mut Page, i| this.state = i),
                ),
            ))
            .child(controls::field(
                "ChecksGlyph status",
                controls::choices(
                    "prs-checks",
                    &["passed", "failed", "running", "none"],
                    self.checks,
                    controls::setter(&entity, |this: &mut Page, i| this.checks = i),
                ),
            ))
            .child(controls::field(
                "ReviewDecisionBadge decision",
                controls::choices(
                    "prs-review",
                    &["approved", "changes", "awaiting"],
                    self.review,
                    controls::setter(&entity, |this: &mut Page, i| this.review = i),
                ),
            ));
        Body::new()
            .playground(live)
            .hint("Hover a glyph for its name")
            .config(config)
            .example(controls::block("Reference preview", reference()))
    }
}
