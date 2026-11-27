//! T3 · Usage & workflows: UsageWindowBar (and UsageShareBar),
//! WorkflowRunFeed and WorkflowRunRow (and WorkflowStateWord).
use super::{note, row, stack};
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, prelude::FluentBuilder as _, px,
};
use u0ui::t3::workspace::{
    RunState, ShareSegment, StepStatus, UsageShareBar, UsageWindowBar, WorkflowRunFeed,
    WorkflowRunRow, WorkflowStateWord, WorkflowStep,
};

// ── UsageWindowBar ──────────────────────────────────────────────────────

struct UsagePage {
    used: usize,
    elapsed: usize,
    account: bool,
    limit: bool,
}

const PERCENTS: [&str; 6] = ["10", "31", "48", "55", "72", "95"];

/// Build the UsageWindowBar page.
pub fn build_usage_window_bar(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| UsagePage {
        used: 4,
        elapsed: 2,
        account: true,
        limit: false,
    })
    .into()
}

fn shares() -> [ShareSegment; 3] {
    [
        ShareSegment::new("Claude", 58.),
        ShareSegment::new("Codex", 27.),
        ShareSegment::new("Cursor", 15.),
    ]
}

/// The overview tile.
pub fn usage_window_bar_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(px(240.))
        .child(
            UsageWindowBar::new("ubar-thumb", "5-hour window")
                .used(72)
                .elapsed(48)
                .resets("in 2h 36m"),
        )
        .into_any_element()
}

impl Render for UsagePage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let pct = |i: usize| PERCENTS[i].parse::<u32>().unwrap_or(0);
        let mut live = UsageWindowBar::new("ubar-live", "5-hour window")
            .used(pct(self.used))
            .elapsed(pct(self.elapsed))
            .resets("in 2h 36m");
        if self.account {
            live = live.account("jd@acme.dev");
        }
        if self.limit {
            live = live.limit_at("13:10");
        }
        let config = stack(24.)
            .child(controls::field(
                "used",
                controls::choices(
                    "ubar-used",
                    &PERCENTS,
                    self.used,
                    controls::setter(&entity, |this: &mut UsagePage, i| this.used = i),
                ),
            ))
            .child(controls::field(
                "elapsed",
                controls::choices(
                    "ubar-elapsed",
                    &PERCENTS,
                    self.elapsed,
                    controls::setter(&entity, |this: &mut UsagePage, i| this.elapsed = i),
                ),
            ))
            .child(controls::toggle(
                "ubar-account",
                "account",
                self.account,
                controls::setter(&entity, |this: &mut UsagePage, v| this.account = v),
            ))
            .child(controls::toggle(
                "ubar-limit",
                "limitAt (13:10)",
                self.limit,
                controls::setter(&entity, |this: &mut UsagePage, v| this.limit = v),
            ));
        // `components/UsageWindowBar/preview.html` (`cu-stack`, gap 20, max-width 520).
        let reference = stack(20.)
            .max_w(px(520.))
            .child(
                UsageWindowBar::new("ubar-ref-0", "5-hour window")
                    .used(72)
                    .elapsed(48)
                    .resets("in 2h 36m")
                    .account("jd@acme.dev"),
            )
            .child(
                UsageWindowBar::new("ubar-ref-1", "Weekly")
                    .used(31)
                    .elapsed(55)
                    .resets("Mon 09:00"),
            )
            .child(UsageShareBar::new("share-ref", shares()));
        Body::new()
            .playground(div().w(px(480.)).max_w_full().child(live))
            .hint("Hover the pace label or the even-pace line for its tooltip")
            .config(config)
            .example(controls::block("Reference preview", reference))
            .example(controls::block(
                "UsageShareBar",
                UsageShareBar::new(
                    "share-more",
                    [
                        ShareSegment::new("Claude", 40.),
                        ShareSegment::new("Codex", 25.),
                        ShareSegment::new("Cursor", 20.),
                        ShareSegment::new("Gemini", 10.),
                        ShareSegment::new("Kiro", 5.),
                    ],
                ),
            ))
    }
}

// ── WorkflowRunFeed ─────────────────────────────────────────────────────

fn feed_steps() -> Vec<WorkflowStep> {
    vec![
        WorkflowStep::new("Checkout and install", StepStatus::Succeeded).elapsed(41),
        WorkflowStep::new("Generate tokens", StepStatus::Succeeded)
            .elapsed(12)
            .detail("217 tokens, 4 themes"),
        WorkflowStep::new("Contrast check", StepStatus::Failed)
            .elapsed(8)
            .detail("text-helper on layer-02 (g90): 4.21"),
        WorkflowStep::new("Ask: ship anyway?", StepStatus::Waiting).branch(true),
        WorkflowStep::new("Open pull request", StepStatus::Pending),
    ]
}

struct FeedPage {
    third: usize,
    branch: bool,
    log: Option<SharedString>,
}

/// Build the WorkflowRunFeed page.
pub fn build_workflow_run_feed(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| FeedPage {
        third: 1,
        branch: true,
        log: None,
    })
    .into()
}

/// The overview tile.
pub fn workflow_run_feed_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(px(240.))
        .child(WorkflowRunFeed::new(
            "wf-thumb",
            feed_steps().into_iter().take(3),
        ))
        .into_any_element()
}

impl Render for FeedPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let mut steps = feed_steps();
        steps[2].status = StepStatus::ALL[self.third];
        steps[3].branch = self.branch;
        let titles: Vec<SharedString> = steps.iter().map(|s| s.title.clone()).collect();
        let live = stack(12.)
            .child(
                WorkflowRunFeed::new("wf-live", steps)
                    .on_resume({
                        let entity = entity.clone();
                        let titles = titles.clone();
                        move |i, _, cx| {
                            let t = titles[i].clone();
                            entity.update(cx, |this, cx| {
                                this.log = Some(format!("Resume {t}").into());
                                this.third = 2; // running
                                cx.notify();
                            })
                        }
                    })
                    .on_answer({
                        let entity = entity.clone();
                        move |i, _, cx| {
                            let t = titles[i].clone();
                            entity.update(cx, |this, cx| {
                                this.log = Some(format!("Answer {t}").into());
                                cx.notify();
                            })
                        }
                    }),
            )
            .when_some(self.log.clone(), |this, l| this.child(note(l)));
        let statuses = StepStatus::ALL.map(|s| s.name());
        let config = stack(24.)
            .child(controls::field(
                "third step status",
                controls::choices(
                    "wf-third",
                    &statuses,
                    self.third,
                    controls::setter(&entity, |this: &mut FeedPage, i| this.third = i),
                ),
            ))
            .child(controls::toggle(
                "wf-branch",
                "fourth step is a branch",
                self.branch,
                controls::setter(&entity, |this: &mut FeedPage, v| this.branch = v),
            ));
        Body::new()
            .playground(live)
            .hint("Resume re-runs the failed step (it turns to running)")
            .config(config)
            .example(controls::block(
                "Reference preview",
                WorkflowRunFeed::new("wf-ref", feed_steps()),
            ))
    }
}

// ── WorkflowRunRow ──────────────────────────────────────────────────────

struct RunRowPage {
    state: usize,
    opened: Option<SharedString>,
}

/// Build the WorkflowRunRow page.
pub fn build_workflow_run_row(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| RunRowPage {
        state: 0,
        opened: None,
    })
    .into()
}

/// The overview tile.
pub fn workflow_run_row_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    row()
        .gap(px(8.))
        .children(
            [RunState::Running, RunState::Succeeded, RunState::Failed].map(WorkflowStateWord::new),
        )
        .into_any_element()
}

fn runs() -> [(
    &'static str,
    RunState,
    &'static str,
    &'static str,
    &'static str,
    u64,
); 4] {
    [
        (
            "Nightly token sync",
            RunState::Running,
            "carbon-ui",
            "Scheduled",
            "10m ago",
            610,
        ),
        (
            "Release notes",
            RunState::Succeeded,
            "carbon-ui",
            "Manual",
            "1h ago",
            95,
        ),
        (
            "Dependency bump",
            RunState::Failed,
            "api-gateway",
            "Webhook",
            "3h ago",
            44,
        ),
        (
            "Weekly triage",
            RunState::Queued,
            "carbon-ui",
            "Scheduled",
            "\u{2014}",
            0,
        ),
    ]
}

impl Render for RunRowPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let live = stack(12.)
            .child(
                div().max_w(px(720.)).child(
                    WorkflowRunRow::new(
                        "wfrow-live",
                        "Nightly token sync",
                        RunState::ALL[self.state],
                    )
                    .project("carbon-ui")
                    .trigger("Scheduled")
                    .started("10m ago")
                    .elapsed(610)
                    .on_click({
                        let entity = entity.clone();
                        move |_, cx| {
                            entity.update(cx, |this, cx| {
                                this.opened = Some("Opened Nightly token sync".into());
                                cx.notify();
                            })
                        }
                    }),
                ),
            )
            .when_some(self.opened.clone(), |this, o| this.child(note(o)));
        let states = RunState::ALL.map(|s| s.name());
        let config = controls::field(
            "state",
            controls::choices(
                "wfrow-state",
                &states,
                self.state,
                controls::setter(&entity, |this: &mut RunRowPage, i| this.state = i),
            ),
        );
        // `components/WorkflowRunRow/preview.html` (`max-width: 720`).
        let reference =
            div()
                .flex()
                .flex_col()
                .max_w(px(720.))
                .children(runs().into_iter().enumerate().map(
                    |(i, (name, state, project, trigger, started, e))| {
                        WorkflowRunRow::new(
                            SharedString::from(format!("wfrow-ref-{i}")),
                            name,
                            state,
                        )
                        .project(project)
                        .trigger(trigger)
                        .started(started)
                        .elapsed(e)
                    },
                ));
        Body::new()
            .playground(div().w_full().child(live))
            .config(config)
            .example(controls::block("Reference preview", reference))
            .example(controls::block(
                "WorkflowStateWord",
                row()
                    .gap(px(8.))
                    .children(RunState::ALL.map(WorkflowStateWord::new)),
            ))
    }
}
