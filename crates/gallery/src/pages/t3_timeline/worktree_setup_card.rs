//! T3 · Timeline › WorktreeSetupCard: a running setup with three stages and
//! output, and a ready worktree whose setup script failed
//! (`components/WorktreeSetupCard/preview.html`); the head folds the body.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div,
};
use u0ui::prelude::*;
use u0ui::t3::timeline::{StageStatus, WorktreeSetupCard, WorktreeStage, WorktreeState};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| WorktreePage { state: 0 }).into()
}

struct WorktreePage {
    state: usize,
}

fn running(id: &'static str) -> WorktreeSetupCard {
    WorktreeSetupCard::new(
        id,
        WorktreeState::Running,
        "feat/dark-tokens",
        "~/.worktrees/carbon-ui/feat-dark-tokens",
    )
    .elapsed(23)
    .stages(vec![
        WorktreeStage::new("Create worktree", StageStatus::Done).elapsed(2),
        WorktreeStage::new("Install dependencies", StageStatus::Running),
        WorktreeStage::new("Run setup script", StageStatus::Pending),
    ])
    .output(vec![
        "Lockfile is up to date".into(),
        "Progress: resolved 812, reused 806, downloaded 6".into(),
    ])
}

/// The reference preview (`cu-stack`, gap 12, max 640px).
fn reference() -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_04)
        .w_full()
        .max_w(gpui_kit::rems(40.))
        .child(running("wt-ref-0"))
        .child(
            WorktreeSetupCard::new(
                "wt-ref-1",
                WorktreeState::ScriptFailed,
                "fix/date-picker",
                "~/.worktrees/carbon-ui/fix-date-picker",
            )
            .elapsed(41),
        )
}

impl Render for WorktreePage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let state = WorktreeState::ALL[self.state];
        let stage = |s: StageStatus| match state {
            WorktreeState::Running => s,
            WorktreeState::Ready => StageStatus::Done,
            WorktreeState::ScriptFailed | WorktreeState::Failed => {
                if s == StageStatus::Done {
                    s
                } else {
                    StageStatus::Failed
                }
            }
            WorktreeState::Cancelled => {
                if s == StageStatus::Done {
                    s
                } else {
                    StageStatus::Skipped
                }
            }
        };
        let card = WorktreeSetupCard::new(
            SharedString::from(format!("wt-live-{}", self.state)),
            state,
            "feat/dark-tokens",
            "~/.worktrees/carbon-ui/feat-dark-tokens",
        )
        .elapsed(23)
        .stages(vec![
            WorktreeStage::new("Create worktree", StageStatus::Done).elapsed(2),
            WorktreeStage::new("Install dependencies", stage(StageStatus::Running)),
            WorktreeStage::new("Run setup script", stage(StageStatus::Pending)),
        ])
        .output(vec![
            "Lockfile is up to date".into(),
            "Progress: resolved 812, reused 806, downloaded 6".into(),
        ]);
        let states = WorktreeState::ALL.map(|s| s.name());
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "state",
                controls::choices(
                    "wt-state",
                    &states,
                    self.state,
                    controls::setter(&entity, |this: &mut Self, i| this.state = i),
                ),
            ));
        Body::new()
            .playground(div().w_full().max_w(gpui_kit::rems(40.)).child(card))
            .hint("Click the head to fold the card.")
            .config(config)
            .example(controls::block("Reference preview", reference()))
    }
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(gpui_kit::rems(15.))
        .child(
            WorktreeSetupCard::new(
                "wt-th",
                WorktreeState::Ready,
                "feat/dark-tokens",
                "~/.worktrees",
            )
            .elapsed(41),
        )
        .into_any_element()
}
