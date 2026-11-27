//! T3 · Pull requests & issues › PullRequestRow: the reference list (open
//! unread selected, draft with failing checks, merged, closed with
//! conflicts) where a click selects a row, and a configurable live row.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, prelude::FluentBuilder as _, rems, rgb,
};
use u0ui::prelude::*;
use u0ui::t3::review::{ChecksStatus, Label, PullRequestRow, PullRequestState, ReviewDecision};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        selected: 0,
        state: 0,
        checks: 2,
        review: 2,
        unread: true,
        conflict: false,
        labels: true,
    })
    .into()
}

struct Page {
    selected: usize,
    state: usize,
    checks: usize,
    review: usize,
    unread: bool,
    conflict: bool,
    labels: bool,
}

/// `components/PullRequestRow/preview.html`.
pub(crate) fn reference_rows(selected: Option<usize>) -> [PullRequestRow; 4] {
    [
        PullRequestRow::new("prr-0", 482, "Dark theme tokens")
            .repo("acme/carbon-ui")
            .author("jd")
            .branches("feat/dark-tokens", "main")
            .labels([Label::new("design", rgb(0x8a3ffc))])
            .checks(ChecksStatus::Running)
            .review(ReviewDecision::Awaiting)
            .stat(312, 14)
            .comments(3)
            .time("2h")
            .unread(true)
            .selected(selected == Some(0)),
        PullRequestRow::new("prr-1", 479, "Fix date picker null handling")
            .state(PullRequestState::Draft)
            .author("ak")
            .branches("fix/date", "main")
            .checks(ChecksStatus::Failed)
            .stat(24, 9)
            .time("5h")
            .selected(selected == Some(1)),
        PullRequestRow::new("prr-2", 470, "Upgrade Carbon to 1.121")
            .state(PullRequestState::Merged)
            .author("mr")
            .branches("deps/carbon", "main")
            .checks(ChecksStatus::Passed)
            .review(ReviewDecision::Approved)
            .stat(88, 91)
            .time("1d")
            .selected(selected == Some(2)),
        PullRequestRow::new("prr-3", 455, "Experimental grid")
            .state(PullRequestState::Closed)
            .author("ls")
            .branches("exp/grid", "main")
            .review(ReviewDecision::Changes)
            .conflict(true)
            .time("1w")
            .selected(selected == Some(3)),
    ]
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    let [row, ..] = reference_rows(Some(0));
    div().w(rems(15.)).child(row).into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let reference = div().w_full().max_w(rems(56.25)).children(
            reference_rows(Some(self.selected))
                .into_iter()
                .enumerate()
                .map(|(i, row)| {
                    let entity = entity.clone();
                    row.on_click(move |_, cx| {
                        entity.update(cx, |this, cx| {
                            this.selected = i;
                            cx.notify();
                        })
                    })
                }),
        );
        let live = div().w_full().max_w(rems(56.25)).child(
            PullRequestRow::new("prr-live", 482, "Dark theme tokens")
                .state(PullRequestState::ALL[self.state])
                .repo("acme/carbon-ui")
                .author("jd")
                .branches("feat/dark-tokens", "main")
                .when(self.labels, |r| {
                    r.labels([
                        Label::new("design", rgb(0x8a3ffc)),
                        Label::new("bug", rgb(0xda1e28)),
                    ])
                })
                .checks(ChecksStatus::ALL[self.checks])
                .review(ReviewDecision::ALL[self.review])
                .stat(312, 14)
                .comments(3)
                .time("2h")
                .unread(self.unread)
                .conflict(self.conflict),
        );
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::field(
                "state",
                controls::choices(
                    "prr-state",
                    &["open", "draft", "merged", "closed"],
                    self.state,
                    controls::setter(&entity, |this: &mut Page, i| this.state = i),
                ),
            ))
            .child(controls::field(
                "checks",
                controls::choices(
                    "prr-checks",
                    &["passed", "failed", "running", "none"],
                    self.checks,
                    controls::setter(&entity, |this: &mut Page, i| this.checks = i),
                ),
            ))
            .child(controls::field(
                "review",
                controls::choices(
                    "prr-review",
                    &["approved", "changes", "awaiting"],
                    self.review,
                    controls::setter(&entity, |this: &mut Page, i| this.review = i),
                ),
            ))
            .child(controls::toggle(
                "prr-unread",
                "unread",
                self.unread,
                controls::setter(&entity, |this: &mut Page, v| this.unread = v),
            ))
            .child(controls::toggle(
                "prr-conflict",
                "conflict",
                self.conflict,
                controls::setter(&entity, |this: &mut Page, v| this.conflict = v),
            ))
            .child(controls::toggle(
                "prr-labels",
                "labels",
                self.labels,
                controls::setter(&entity, |this: &mut Page, v| this.labels = v),
            ));
        Body::new()
            .playground(live)
            .config(config)
            .example(controls::block("Reference preview", reference))
    }
}
