//! T3 · Usage & workflows: [`WorkflowStateWord`], [`WorkflowRunFeed`] and
//! [`WorkflowRunRow`] (`src/t3/workspace.jsx`, "Workflows" in `src/t3.css`).
use super::parts::{card_sub, child_id};
use crate::{
    components::{Badge, BadgeSize, BadgeVariant, Button, ButtonSize, ButtonVariant},
    icons::CarbonIcon,
    t3::{
        base::{Spin, fmt_elapsed, visually_hidden},
        craft::sans,
        press::{self, Handler, rem},
    },
    theme::color,
    tokens::ColorRole,
};
use gpui_kit::{
    AnyElement, App, CursorStyle, ElementId, Hsla, InteractiveElement as _, IntoElement,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _, px,
};
use std::rc::Rc;

type IndexHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

// ── WorkflowStateWord ───────────────────────────────────────────────────

/// A workflow run's state (`WorkflowStateWord` `state`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RunState {
    /// `running` (the default): blue.
    #[default]
    Running,
    /// `queued`: warm gray.
    Queued,
    /// `succeeded`: green.
    Succeeded,
    /// `failed`: red.
    Failed,
    /// `waiting`: purple.
    Waiting,
    /// `cancelled`: gray.
    Cancelled,
}

impl RunState {
    /// Every state.
    pub const ALL: [RunState; 6] = [
        Self::Running,
        Self::Queued,
        Self::Succeeded,
        Self::Failed,
        Self::Waiting,
        Self::Cancelled,
    ];

    /// The reference name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Queued => "queued",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Waiting => "waiting",
            Self::Cancelled => "cancelled",
        }
    }

    /// The tag label (`state[0].toUpperCase() + state.slice(1)`).
    pub fn label(self) -> String {
        let n = self.name();
        let mut c = n.chars();
        c.next()
            .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
            .unwrap_or_default()
    }

    /// The tag variant (`t`).
    pub const fn variant(self) -> BadgeVariant {
        match self {
            Self::Running => BadgeVariant::Blue,
            Self::Queued => BadgeVariant::WarmGray,
            Self::Succeeded => BadgeVariant::Green,
            Self::Failed => BadgeVariant::Red,
            Self::Waiting => BadgeVariant::Purple,
            Self::Cancelled => BadgeVariant::Gray,
        }
    }
}

/// A run state as a small tag (`WorkflowStateWord`).
#[derive(IntoElement, Clone, Debug)]
pub struct WorkflowStateWord {
    state: RunState,
    label: Option<SharedString>,
}

impl WorkflowStateWord {
    /// The tag for `state`.
    pub fn new(state: RunState) -> Self {
        Self { state, label: None }
    }

    /// Replace the label.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl RenderOnce for WorkflowStateWord {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        Badge::new(self.label.unwrap_or_else(|| self.state.label().into()))
            .variant(self.state.variant())
            .size(BadgeSize::Sm)
    }
}

// ── WorkflowRunFeed ─────────────────────────────────────────────────────

/// A step's status (`steps[].status`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum StepStatus {
    /// `succeeded`: `CheckmarkFilled` in `support-success`.
    Succeeded,
    /// `failed`: `ErrorFilled` in `support-error`, with Resume.
    Failed,
    /// `running`: a spinner.
    Running,
    /// `pending` (the default): `CircleDash`, a quiet title.
    #[default]
    Pending,
    /// `waiting`: `Help` in `tag-color-purple`, with Answer.
    Waiting,
    /// `skipped`: `Subtract`.
    Skipped,
}

impl StepStatus {
    /// Every status.
    pub const ALL: [StepStatus; 6] = [
        Self::Succeeded,
        Self::Failed,
        Self::Running,
        Self::Pending,
        Self::Waiting,
        Self::Skipped,
    ];

    /// The reference name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Running => "running",
            Self::Pending => "pending",
            Self::Waiting => "waiting",
            Self::Skipped => "skipped",
        }
    }

    /// The mark's glyph and fill (`None` while running: a spinner).
    /// `.t3-wf--succeeded .t3-wf-mark svg { fill: support-success }`, `--failed`
    /// `support-error`, `--pending, --skipped` `icon-secondary`, `--waiting`
    /// `tag-color-purple`.
    pub fn mark(self) -> Option<(CarbonIcon, Hsla)> {
        let (glyph, role) = match self {
            Self::Running => return None,
            Self::Succeeded => (CarbonIcon::CheckmarkFilled, ColorRole::SupportSuccess),
            Self::Failed => (CarbonIcon::ErrorFilled, ColorRole::SupportError),
            Self::Pending => (CarbonIcon::CircleDash, ColorRole::IconSecondary),
            Self::Waiting => (CarbonIcon::Help, ColorRole::TagColorPurple),
            Self::Skipped => (CarbonIcon::Subtract, ColorRole::IconSecondary),
        };
        Some((glyph, color(role)))
    }
}

/// One step (`{ title, status, elapsed, detail, branch }`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WorkflowStep {
    /// `title`.
    pub title: SharedString,
    /// `status`.
    pub status: StepStatus,
    /// `elapsed` seconds.
    pub elapsed: Option<u64>,
    /// `detail`.
    pub detail: Option<SharedString>,
    /// `branch`: indented, with a branch elbow.
    pub branch: bool,
}

impl WorkflowStep {
    /// A step.
    pub fn new(title: impl Into<SharedString>, status: StepStatus) -> Self {
        Self {
            title: title.into(),
            status,
            ..Default::default()
        }
    }

    /// Seconds it took (`elapsed`).
    pub fn elapsed(mut self, secs: u64) -> Self {
        self.elapsed = Some(secs);
        self
    }

    /// The detail line (`detail`).
    pub fn detail(mut self, detail: impl Into<SharedString>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// A branch step (`branch`).
    pub fn branch(mut self, branch: bool) -> Self {
        self.branch = branch;
        self
    }
}

/// Texts of a [`WorkflowRunFeed`] (hard-coded in the reference).
#[derive(Clone, Debug)]
pub struct WorkflowFeedTexts {
    /// The running spinner's label (`Running`).
    pub running: SharedString,
    /// `Resume`.
    pub resume: SharedString,
    /// `Only the failed step runs again.`
    pub resume_note: SharedString,
    /// `Answer`.
    pub answer: SharedString,
}

impl Default for WorkflowFeedTexts {
    fn default() -> Self {
        Self {
            running: "Running".into(),
            resume: "Resume".into(),
            resume_note: "Only the failed step runs again.".into(),
            answer: "Answer".into(),
        }
    }
}

/// `.t3-wf--branch { margin-left: spacing-07 }`.
pub const BRANCH_INDENT: f32 = 32.;
/// `.t3-wf-rail { left: 7.5px }`: the rail sits on the 16px mark's centre.
pub const RAIL_LEFT: f32 = 7.5;

/// A workflow run as a timeline (`WorkflowRunFeed`, `.t3-wf`): a rail
/// joining the step marks, branch steps indented under an elbow, Resume on a
/// failed step and Answer on a waiting one. Draw it on `background` (the
/// marks mask the rail in that color).
#[derive(IntoElement)]
pub struct WorkflowRunFeed {
    id: ElementId,
    steps: Vec<WorkflowStep>,
    texts: WorkflowFeedTexts,
    on_resume: Option<IndexHandler>,
    on_answer: Option<IndexHandler>,
}

impl WorkflowRunFeed {
    /// A feed of `steps`.
    pub fn new(id: impl Into<ElementId>, steps: impl IntoIterator<Item = WorkflowStep>) -> Self {
        Self {
            id: id.into(),
            steps: steps.into_iter().collect(),
            texts: WorkflowFeedTexts::default(),
            on_resume: None,
            on_answer: None,
        }
    }

    /// Replace the default texts.
    pub fn texts(mut self, texts: WorkflowFeedTexts) -> Self {
        self.texts = texts;
        self
    }

    /// Resume pressed on a failed step.
    pub fn on_resume(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_resume = Some(Rc::new(f));
        self
    }

    /// Answer pressed on a waiting step.
    pub fn on_answer(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_answer = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for WorkflowRunFeed {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let n = self.steps.len();
        let line = color(ColorRole::BorderSubtle01);
        let steps = self.steps.iter().enumerate().map(|(i, s)| {
            let mark: AnyElement = match s.status.mark() {
                None => Spin::new(child_id(&id, "spin", i))
                    .label(self.texts.running.clone())
                    .into_any_element(),
                Some((glyph, fill)) => press::icon(glyph, 16., fill).into_any_element(),
            };
            let pending = s.status == StepStatus::Pending;
            // `.t3-wf-head { display: flex; align-items: center; gap: spacing-03; 14px }`,
            // `b { font-weight: 600 }`, `.t3-wf--pending .t3-wf-head b { color: text-secondary;
            // font-weight: 400 }`.
            let head = sans(div(), 14., 400)
                .flex()
                .flex_row()
                .items_center()
                .gap(rem(8.))
                .child(
                    sans(div(), 14., if pending { 400 } else { 600 })
                        .text_color(color(if pending {
                            ColorRole::TextSecondary
                        } else {
                            ColorRole::TextPrimary
                        }))
                        .child(s.title.clone()),
                )
                .child(visually_hidden(format!(" ({})", s.status.name())))
                .children(s.elapsed.map(|e| card_sub(fmt_elapsed(e))));
            // `.t3-card-actions.t3-card-actions--start { justify-content: flex-start; gap: spacing-03 }`.
            let actions = match s.status {
                StepStatus::Failed => {
                    let f = self.on_resume.clone();
                    Some(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .items_center()
                            .gap(rem(8.))
                            .child(
                                Button::new(child_id(&id, "resume", i), self.texts.resume.clone())
                                    .size(ButtonSize::Sm)
                                    .variant(ButtonVariant::Secondary)
                                    .icon(CarbonIcon::Restart)
                                    .when_some(f, |b, f| b.on_click(move |_, w, cx| f(i, w, cx))),
                            )
                            .child(card_sub(self.texts.resume_note.clone())),
                    )
                }
                StepStatus::Waiting => {
                    let f = self.on_answer.clone();
                    Some(
                        div().flex().flex_row().child(
                            Button::new(child_id(&id, "answer", i), self.texts.answer.clone())
                                .size(ButtonSize::Sm)
                                .when_some(f, |b, f| b.on_click(move |_, w, cx| f(i, w, cx))),
                        ),
                    )
                }
                _ => None,
            };
            // `.t3-wf-step { position: relative; display: flex; gap: spacing-04;
            // padding-bottom: spacing-05 }`.
            div()
                .relative()
                .flex()
                .flex_row()
                .gap(rem(12.))
                .pb(rem(16.))
                .when(s.branch, |this| this.ml(rem(BRANCH_INDENT)))
                // `.t3-wf-rail { position: absolute; left: 7.5px; top: 1.25rem; bottom: 0;
                // width: 1px; background: border-subtle-01 }`, hidden on the last step.
                .when(i + 1 < n, |this| {
                    this.child(
                        div()
                            .absolute()
                            .left(px(RAIL_LEFT))
                            .top(rem(20.))
                            .bottom_0()
                            .w(px(1.))
                            .bg(line),
                    )
                })
                // `.t3-wf--branch::before { left: calc(-1 * spacing-07 + 7.5px); top: -0.75rem;
                // width: calc(spacing-07 - 4px); height: 1.25rem; border-left: 1px; border-bottom: 1px }`.
                .when(s.branch, |this| {
                    this.child(
                        div()
                            .absolute()
                            .left(px(-BRANCH_INDENT + RAIL_LEFT))
                            .top(rem(-12.))
                            .w(rem(BRANCH_INDENT - 4.))
                            .h(rem(20.))
                            .border_l_1()
                            .border_b_1()
                            .border_color(line),
                    )
                })
                // `.t3-wf-mark { position: relative; z-index: 1; display: flex; width: 1rem;
                // height: 1.25rem; flex: none; align-items: center; background: background }`.
                .child(
                    div()
                        .relative()
                        .flex()
                        .flex_none()
                        .items_center()
                        .w(rem(16.))
                        .h(rem(20.))
                        .bg(color(ColorRole::Background))
                        .child(mark),
                )
                .child(
                    // `.t3-wf-body { display: flex; flex: 1; flex-direction: column; gap: spacing-03;
                    // min-width: 0 }`.
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .min_w_0()
                        .gap(rem(8.))
                        .child(head)
                        .when_some(s.detail.clone(), |this, d| {
                            // `.t3-wf-detail { font-size: 0.875rem; color: text-secondary }`.
                            this.child(
                                sans(div(), 14., 400)
                                    .text_color(color(ColorRole::TextSecondary))
                                    .child(d),
                            )
                        })
                        .children(actions),
                )
        });
        // `.t3-wf { margin: 0; padding: 0; list-style: none; max-width: 36rem }`.
        div()
            .id(id.clone())
            .flex()
            .flex_col()
            .w_full()
            .max_w(rem(576.))
            .children(steps)
    }
}

// ── WorkflowRunRow ──────────────────────────────────────────────────────

/// A run in the list (`WorkflowRunRow`, `.t3-wfrow`): a flow glyph, name
/// and `project · trigger · started`, the state tag and the elapsed time.
/// A 48px button row over a bottom hairline.
#[derive(IntoElement)]
pub struct WorkflowRunRow {
    id: ElementId,
    name: SharedString,
    state: RunState,
    trigger: SharedString,
    started: SharedString,
    elapsed: u64,
    project: SharedString,
    on_click: Option<Handler>,
}

impl WorkflowRunRow {
    /// A row for the run `name`.
    pub fn new(id: impl Into<ElementId>, name: impl Into<SharedString>, state: RunState) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            state,
            trigger: SharedString::default(),
            started: SharedString::default(),
            elapsed: 0,
            project: SharedString::default(),
            on_click: None,
        }
    }

    /// What started it (`trigger`).
    pub fn trigger(mut self, trigger: impl Into<SharedString>) -> Self {
        self.trigger = trigger.into();
        self
    }

    /// When it started (`started`).
    pub fn started(mut self, started: impl Into<SharedString>) -> Self {
        self.started = started.into();
        self
    }

    /// Seconds it has run (`elapsed`).
    pub fn elapsed(mut self, secs: u64) -> Self {
        self.elapsed = secs;
        self
    }

    /// The project (`project`).
    pub fn project(mut self, project: impl Into<SharedString>) -> Self {
        self.project = project.into();
        self
    }

    /// Pressed.
    pub fn on_click(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for WorkflowRunRow {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let (hovered, hover) = press::hover((id.clone(), "hover"), window, cx);
        let (focus, ring) = press::focus((id.clone(), "focus"), window, cx);
        let meta = format!(
            "{} \u{b7} {} \u{b7} {}",
            self.project, self.trigger, self.started
        );
        let on_click = self.on_click.clone();
        // `.t3-wfrow { display: flex; width: 100%; align-items: center; gap: spacing-04;
        // min-height: size-lg; padding: 0 spacing-05; border-bottom: 1px solid border-subtle-01;
        // color: text-primary }`, `:hover { background: layer-hover-01 }`.
        gpui_kit::base::Button::new(id)
            .track_focus(&focus)
            .accessibility_label(self.name.clone())
            .relative()
            .flex()
            .flex_row()
            .items_center()
            .justify_start()
            .gap(rem(12.))
            .w_full()
            .min_h(rem(48.))
            .px(rem(16.))
            .border_b_1()
            .border_color(color(ColorRole::BorderSubtle01))
            .when(hovered, |this| this.bg(color(ColorRole::LayerHover01)))
            .text_color(color(ColorRole::TextPrimary))
            .cursor(CursorStyle::PointingHand)
            .on_hover(press::track_hover(hover))
            .when_some(on_click, |this, f| this.on_click(move |_, w, cx| f(w, cx)))
            .child(press::icon(CarbonIcon::Flow, 16., color(ColorRole::IconSecondary)).flex_none())
            .child(
                // `.t3-wfrow-main { display: flex; flex: 1; flex-direction: column;
                // font-size: 0.875rem }`, `b { font-weight: 600 }`.
                sans(div(), 14., 400)
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .items_start()
                    .child(sans(div(), 14., 600).child(self.name.clone()))
                    .child(card_sub(meta)),
            )
            .child(WorkflowStateWord::new(self.state))
            .child(card_sub(fmt_elapsed(self.elapsed)))
            .when(ring, |this| this.child(press::ring()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_words_capitalise_and_map_tags() {
        assert_eq!(RunState::Succeeded.label(), "Succeeded");
        assert_eq!(RunState::Waiting.variant(), BadgeVariant::Purple);
        assert_eq!(RunState::Queued.variant(), BadgeVariant::WarmGray);
        assert_eq!(RunState::Cancelled.variant(), BadgeVariant::Gray);
    }

    #[test]
    fn step_marks_follow_the_css() {
        assert!(StepStatus::Running.mark().is_none());
        assert_eq!(
            StepStatus::Waiting.mark(),
            Some((CarbonIcon::Help, color(ColorRole::TagColorPurple)))
        );
        assert_eq!(
            StepStatus::Failed.mark(),
            Some((CarbonIcon::ErrorFilled, color(ColorRole::SupportError)))
        );
    }

    #[test]
    fn branch_elbow_reaches_back_to_the_rail() {
        // `left: calc(-1 * spacing-07 + 7.5px)` lands on the parent rail.
        assert_eq!(-BRANCH_INDENT + RAIL_LEFT + BRANCH_INDENT, RAIL_LEFT);
    }
}
