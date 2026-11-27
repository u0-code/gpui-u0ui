//! T3 · Pull requests & issues: the PR vocabulary ([`PullRequestStateGlyph`],
//! [`ChecksGlyph`], [`ReviewDecisionBadge`], [`LabelChip`]) and the cards
//! built from it ([`PullRequestRow`], [`PullRequestStack`], [`ChecksList`],
//! [`GuideRail`], [`PullRequestDetailHeader`], [`ReactionBar`],
//! [`PullRequestTimeline`], [`ReviewForm`]) — `src/t3.css` "PR vocabulary".
use super::diff::DiffStat;
use crate::{
    components::{AvatarColor, Badge, BadgeSize, BadgeVariant, Button, ButtonSize, ButtonVariant},
    icons::{CarbonIcon, Icon},
    t3::craft::{self, AvatarTone, focus_handle, focus_visible, mono, press, rem, sans},
    theme::color,
    tokens::ColorRole,
};
use gpui_kit::{
    AnyElement, App, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, Window, div, prelude::FluentBuilder as _, px, relative,
};
use std::rc::Rc;

/// A pull request's state (`PRS`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PullRequestState {
    /// `open`, the default.
    #[default]
    Open,
    /// `draft`.
    Draft,
    /// `merged`.
    Merged,
    /// `closed`.
    Closed,
}

impl PullRequestState {
    /// Every state.
    pub const ALL: [PullRequestState; 4] = [Self::Open, Self::Draft, Self::Merged, Self::Closed];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Draft => "draft",
            Self::Merged => "merged",
            Self::Closed => "closed",
        }
    }

    /// The reference label (`Open`, `Draft`, `Merged`, `Closed`).
    pub const fn label(self) -> &'static str {
        match self {
            Self::Open => "Open",
            Self::Draft => "Draft",
            Self::Merged => "Merged",
            Self::Closed => "Closed",
        }
    }

    /// `Merge` for merged, `PullRequest` otherwise.
    pub const fn icon(self) -> CarbonIcon {
        match self {
            Self::Merged => CarbonIcon::Merge,
            _ => CarbonIcon::PullRequest,
        }
    }

    /// `.t3-prglyph.t3-pr--open { color: support-success }`, `--draft`
    /// `icon-secondary`, `--merged` `tag-color-purple`, `--closed` `support-error`.
    pub fn ink(self) -> Hsla {
        color(match self {
            Self::Open => ColorRole::SupportSuccess,
            Self::Draft => ColorRole::IconSecondary,
            Self::Merged => ColorRole::TagColorPurple,
            Self::Closed => ColorRole::SupportError,
        })
    }

    /// The detail header's tag (`merged` purple, `open` green, else gray).
    pub const fn badge(self) -> BadgeVariant {
        match self {
            Self::Merged => BadgeVariant::Purple,
            Self::Open => BadgeVariant::Green,
            _ => BadgeVariant::Gray,
        }
    }
}

fn glyph(icon: impl Into<Icon>, ink: Hsla, size: f32) -> gpui_kit::Div {
    div()
        .flex()
        .flex_none()
        .child(icon.into().size_length(rem(size)).color(ink))
}

/// The PR state icon (`.t3-prglyph`), 16px or 20px.
#[derive(IntoElement, Clone, Copy, Debug, PartialEq)]
pub struct PullRequestStateGlyph {
    state: PullRequestState,
    size: f32,
}

impl PullRequestStateGlyph {
    /// A 16px glyph.
    pub fn new(state: PullRequestState) -> Self {
        Self { state, size: 16. }
    }

    /// The icon size in px.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }
}

impl RenderOnce for PullRequestStateGlyph {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        glyph(self.state.icon(), self.state.ink(), self.size)
    }
}

/// CI checks status (`CHK`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ChecksStatus {
    /// `passed`, the default.
    #[default]
    Passed,
    /// `failed`.
    Failed,
    /// `running`.
    Running,
    /// `none`.
    None,
}

impl ChecksStatus {
    /// Every status.
    pub const ALL: [ChecksStatus; 4] = [Self::Passed, Self::Failed, Self::Running, Self::None];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Running => "running",
            Self::None => "none",
        }
    }

    /// The reference's accessible name.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Passed => "All checks passed",
            Self::Failed => "Some checks were not successful",
            Self::Running => "Some checks haven't completed yet",
            Self::None => "No checks reported",
        }
    }

    /// `CheckmarkFilled`, `ErrorFilled`, `InProgress`, `CircleDash`.
    pub const fn icon(self) -> CarbonIcon {
        match self {
            Self::Passed => CarbonIcon::CheckmarkFilled,
            Self::Failed => CarbonIcon::ErrorFilled,
            Self::Running => CarbonIcon::InProgress,
            Self::None => CarbonIcon::CircleDash,
        }
    }

    /// `.t3-checks--passed` `support-success`, `--failed` `support-error`,
    /// `--running` `support-warning`, `--none` `icon-secondary`.
    pub fn ink(self) -> Hsla {
        color(match self {
            Self::Passed => ColorRole::SupportSuccess,
            Self::Failed => ColorRole::SupportError,
            Self::Running => ColorRole::SupportWarning,
            Self::None => ColorRole::IconSecondary,
        })
    }
}

/// The checks status icon (`.t3-checks`, 16px).
#[derive(IntoElement, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChecksGlyph {
    status: ChecksStatus,
}

impl ChecksGlyph {
    /// A glyph.
    pub fn new(status: ChecksStatus) -> Self {
        Self { status }
    }
}

impl RenderOnce for ChecksGlyph {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        glyph(self.status.icon(), self.status.ink(), 16.)
    }
}

/// A review decision (`REV`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ReviewDecision {
    /// `approved`, the default.
    #[default]
    Approved,
    /// `changes`.
    Changes,
    /// `awaiting`.
    Awaiting,
}

impl ReviewDecision {
    /// Every decision.
    pub const ALL: [ReviewDecision; 3] = [Self::Approved, Self::Changes, Self::Awaiting];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Approved => "approved",
            Self::Changes => "changes",
            Self::Awaiting => "awaiting",
        }
    }

    /// The reference label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Approved => "Approved",
            Self::Changes => "Changes requested",
            Self::Awaiting => "Awaiting review",
        }
    }

    /// `CheckmarkOutline`, `Edit`, `View`.
    pub const fn icon(self) -> CarbonIcon {
        match self {
            Self::Approved => CarbonIcon::CheckmarkOutline,
            Self::Changes => CarbonIcon::Edit,
            Self::Awaiting => CarbonIcon::View,
        }
    }

    /// `.t3-review--approved` `tag-color-green`, `--changes` `text-error`,
    /// `--awaiting` `text-secondary`.
    pub fn ink(self) -> Hsla {
        color(match self {
            Self::Approved => ColorRole::TagColorGreen,
            Self::Changes => ColorRole::TextError,
            Self::Awaiting => ColorRole::TextSecondary,
        })
    }
}

/// The review decision: a 12px icon and 12px label in the decision's ink
/// (`.t3-review`).
#[derive(IntoElement, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReviewDecisionBadge {
    decision: ReviewDecision,
}

impl ReviewDecisionBadge {
    /// A badge.
    pub fn new(decision: ReviewDecision) -> Self {
        Self { decision }
    }
}

impl RenderOnce for ReviewDecisionBadge {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let ink = self.decision.ink();
        sans(div(), 12., 400)
            .flex()
            .flex_none()
            .items_center()
            .gap(rem(4.))
            .whitespace_nowrap()
            .text_color(ink)
            .child(
                Icon::carbon(self.decision.icon())
                    .size_length(rem(12.))
                    .color(ink),
            )
            .child(self.decision.label())
    }
}

/// A host-platform label: an 8px dot in the label's color and the name in
/// `text-primary` (`.t3-label`: a 20px pill with a 1px `border-subtle-01`
/// inset ring). The host color only paints the dot, so contrast holds.
#[derive(IntoElement, Clone, Debug)]
pub struct LabelChip {
    name: SharedString,
    color: Hsla,
}

impl LabelChip {
    /// A label `name` whose host color is `color`.
    pub fn new(name: impl Into<SharedString>, color: impl Into<Hsla>) -> Self {
        Self {
            name: name.into(),
            color: color.into(),
        }
    }
}

impl RenderOnce for LabelChip {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        sans(div(), 12., 400)
            .flex()
            .flex_none()
            .items_center()
            .gap(rem(4.))
            .h(rem(20.))
            .px(rem(8.))
            .rounded(rem(16.))
            .border_1()
            .border_color(color(ColorRole::BorderSubtle01))
            .whitespace_nowrap()
            .text_color(color(ColorRole::TextPrimary))
            .child(div().size(px(8.)).rounded_full().bg(self.color))
            .child(self.name)
    }
}

/// A label as data (`labels[]`).
#[derive(Clone, Debug, PartialEq)]
pub struct Label {
    /// `name`.
    pub name: SharedString,
    /// `color`.
    pub color: Hsla,
}

impl Label {
    /// A label.
    pub fn new(name: impl Into<SharedString>, color: impl Into<Hsla>) -> Self {
        Self {
            name: name.into(),
            color: color.into(),
        }
    }

    fn chip(&self) -> LabelChip {
        LabelChip::new(self.name.clone(), self.color)
    }
}

/// `.t3-prrow-c`: a 14px icon and a 12px `text-secondary` count.
pub(crate) fn icon_count(icon: CarbonIcon, text: impl Into<SharedString>) -> gpui_kit::Div {
    let ink = color(ColorRole::TextSecondary);
    sans(div(), 12., 400)
        .flex()
        .flex_none()
        .items_center()
        .gap(rem(4.))
        .text_color(ink)
        .child(Icon::carbon(icon).size_length(rem(14.)).color(ink))
        .child(text.into())
}

/// A 3px start bar (`box-shadow: inset 3px 0 0 <color>`).
pub(crate) fn start_bar(c: Hsla) -> AnyElement {
    div()
        .absolute()
        .left_0()
        .top_0()
        .bottom_0()
        .w(px(3.))
        .bg(c)
        .into_any_element()
}

type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

/// A PR list row (`.t3-prrow`): state glyph; title, `#number` and labels;
/// repo, author, `head → base` and time; then conflicts, review, checks,
/// comments and `+/−`.
#[derive(IntoElement)]
pub struct PullRequestRow {
    id: ElementId,
    number: u32,
    title: SharedString,
    state: PullRequestState,
    repo: Option<SharedString>,
    author: SharedString,
    head: SharedString,
    base: SharedString,
    labels: Vec<Label>,
    checks: Option<ChecksStatus>,
    review: Option<ReviewDecision>,
    stat: Option<(usize, usize)>,
    comments: Option<usize>,
    time: SharedString,
    conflict: bool,
    conflict_label: SharedString,
    unread: bool,
    selected: bool,
    on_click: Option<Handler>,
}

impl PullRequestRow {
    /// A row for PR `number` titled `title`.
    pub fn new(id: impl Into<ElementId>, number: u32, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            number,
            title: title.into(),
            state: PullRequestState::Open,
            repo: None,
            author: SharedString::default(),
            head: SharedString::default(),
            base: SharedString::default(),
            labels: Vec::new(),
            checks: None,
            review: None,
            stat: None,
            comments: None,
            time: SharedString::default(),
            conflict: false,
            conflict_label: "Conflicts".into(),
            unread: false,
            selected: false,
            on_click: None,
        }
    }

    /// The state (default open).
    pub fn state(mut self, state: PullRequestState) -> Self {
        self.state = state;
        self
    }

    /// The repository (`repo`).
    pub fn repo(mut self, repo: impl Into<SharedString>) -> Self {
        self.repo = Some(repo.into());
        self
    }

    /// The author login.
    pub fn author(mut self, author: impl Into<SharedString>) -> Self {
        self.author = author.into();
        self
    }

    /// Head and base branches.
    pub fn branches(
        mut self,
        head: impl Into<SharedString>,
        base: impl Into<SharedString>,
    ) -> Self {
        self.head = head.into();
        self.base = base.into();
        self
    }

    /// Labels.
    pub fn labels(mut self, labels: impl IntoIterator<Item = Label>) -> Self {
        self.labels = labels.into_iter().collect();
        self
    }

    /// Checks status.
    pub fn checks(mut self, checks: ChecksStatus) -> Self {
        self.checks = Some(checks);
        self
    }

    /// Review decision.
    pub fn review(mut self, review: ReviewDecision) -> Self {
        self.review = Some(review);
        self
    }

    /// Added and removed lines.
    pub fn stat(mut self, added: usize, removed: usize) -> Self {
        self.stat = Some((added, removed));
        self
    }

    /// Comment count (shown when non-zero).
    pub fn comments(mut self, comments: usize) -> Self {
        self.comments = Some(comments);
        self
    }

    /// Relative time.
    pub fn time(mut self, time: impl Into<SharedString>) -> Self {
        self.time = time.into();
        self
    }

    /// Has merge conflicts.
    pub fn conflict(mut self, conflict: bool) -> Self {
        self.conflict = conflict;
        self
    }

    /// The conflicts text (`Conflicts`).
    pub fn conflict_label(mut self, label: impl Into<SharedString>) -> Self {
        self.conflict_label = label.into();
        self
    }

    /// Unread: semibold title.
    pub fn unread(mut self, unread: bool) -> Self {
        self.unread = unread;
        self
    }

    /// Selected: `layer-selected-01` and a 3px `border-interactive` bar.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Activation (click, Enter, Space).
    pub fn on_click(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
}

/// `.t3-prrow-meta > span`: inline-flex, 4px gap.
fn meta_item() -> gpui_kit::Div {
    div().flex().flex_none().items_center().gap(rem(4.))
}

/// `<Avatar size="sm"><AvatarFallback color=…>` inside a row whose rule
/// shrinks the disc (`.t3-prrow-meta .cu-avatar--sm { 1rem }`, `.t3-issue
/// .cu-avatar--sm { 1.25rem }`); the fallback text stays 12px.
pub(crate) fn small_avatar(name: &str, c: AvatarColor, diameter: f32) -> gpui_kit::Div {
    craft::initials(craft::two_letters(name), AvatarTone::Avatar(c), diameter)
}

impl RenderOnce for PullRequestRow {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let focus = focus_handle(&id, window, cx);
        let ring = focus_visible(&focus, window);
        let on_click = self.on_click.clone();
        let secondary = color(ColorRole::TextSecondary);
        let title = sans(div(), 14., if self.unread { 600 } else { 400 })
            .flex()
            .flex_wrap()
            .items_center()
            .gap(rem(8.))
            .child(self.title.clone())
            .child(
                sans(div(), 14., 400)
                    .text_color(secondary)
                    .child(format!("#{}", self.number)),
            )
            .children(self.labels.iter().map(Label::chip));
        let meta = sans(div(), 12., 400)
            .flex()
            .flex_wrap()
            .items_center()
            .gap(rem(12.))
            .text_color(secondary)
            .when_some(self.repo.clone(), |this, repo| {
                this.child(meta_item().child(repo))
            })
            .child(
                meta_item()
                    .child(small_avatar(
                        if self.author.is_empty() {
                            "?"
                        } else {
                            &self.author
                        },
                        AvatarColor::Teal,
                        16.,
                    ))
                    .child(self.author.clone()),
            )
            .child(
                mono(meta_item(), 12., 400)
                    .child(
                        Icon::carbon(CarbonIcon::Branch)
                            .size_length(rem(12.))
                            .color(secondary),
                    )
                    .child(format!("{} \u{2192} {}", self.head, self.base)),
            )
            .child(meta_item().child(self.time.clone()));
        let side = div()
            .flex()
            .flex_none()
            .items_center()
            .gap(rem(12.))
            .when(self.conflict, |this| {
                // `.t3-conflict { 12px; text-primary } svg { fill: support-warning }`.
                this.child(
                    sans(div(), 12., 400)
                        .flex()
                        .items_center()
                        .gap(rem(4.))
                        .text_color(color(ColorRole::TextPrimary))
                        .child(
                            Icon::carbon(CarbonIcon::WarningAltFilled)
                                .size_length(rem(14.))
                                .color(color(ColorRole::SupportWarning)),
                        )
                        .child(self.conflict_label.clone()),
                )
            })
            .when_some(self.review, |this, r| {
                this.child(ReviewDecisionBadge::new(r))
            })
            .when_some(self.checks, |this, c| this.child(ChecksGlyph::new(c)))
            .when_some(self.comments.filter(|n| *n > 0), |this, n| {
                this.child(icon_count(CarbonIcon::Chat, n.to_string()))
            })
            .when_some(self.stat, |this, (a, r)| this.child(DiffStat::new(a, r)));
        press(id, &focus)
            .accessibility_label(self.title.clone())
            .w_full()
            .items_start()
            .gap(rem(12.))
            .px(rem(16.))
            .py(rem(12.))
            .border_b_1()
            .border_color(color(ColorRole::BorderSubtle01))
            .text_color(color(ColorRole::TextPrimary))
            .when(self.selected, |this| {
                this.bg(color(ColorRole::LayerSelected01))
            })
            // `.t3-prrow:hover` (0,2,0) wins over `.t3-prrow--on` (0,1,0).
            .hover(|s| s.bg(color(ColorRole::LayerHover01)))
            .when_some(on_click, |this, f| this.on_click(move |_, w, cx| f(w, cx)))
            .when(self.selected, |this| {
                this.child(start_bar(color(ColorRole::BorderInteractive)))
            })
            // `.t3-prrow > .t3-prglyph { margin-top: 2px }`.
            .child(
                div()
                    .mt(px(2.))
                    .child(PullRequestStateGlyph::new(self.state)),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .gap(rem(4.))
                    .child(title)
                    .child(meta),
            )
            .child(side)
            .when(ring, |this| this.child(craft::focus_ring()))
    }
}

/// One layer of a stack (`layers[]`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StackLayer {
    /// `number`.
    pub number: u32,
    /// `title`.
    pub title: SharedString,
    /// `state`.
    pub state: PullRequestState,
    /// `checks`.
    pub checks: Option<ChecksStatus>,
}

impl StackLayer {
    /// A layer.
    pub fn new(number: u32, title: impl Into<SharedString>, state: PullRequestState) -> Self {
        Self {
            number,
            title: title.into(),
            state,
            checks: None,
        }
    }

    /// Its checks status.
    pub fn checks(mut self, checks: ChecksStatus) -> Self {
        self.checks = Some(checks);
        self
    }
}

/// The start indent of stack layer `depth`: `margin-left: calc(var(--depth)
/// * var(--spacing-05))`.
pub fn stack_indent(depth: usize) -> f32 {
    depth as f32 * 16.
}

/// Stacked pull requests (`.t3-stack`): each layer 40px on `layer-01`,
/// indented 16px per depth; the current one selected with a 3px bar.
#[derive(IntoElement, Clone, Debug, Default)]
pub struct PullRequestStack {
    layers: Vec<StackLayer>,
    current: Option<u32>,
}

impl PullRequestStack {
    /// An empty stack.
    pub fn new() -> Self {
        Self::default()
    }

    /// The layers, base first.
    pub fn layers(mut self, layers: impl IntoIterator<Item = StackLayer>) -> Self {
        self.layers = layers.into_iter().collect();
        self
    }

    /// The current PR number.
    pub fn current(mut self, number: u32) -> Self {
        self.current = Some(number);
        self
    }
}

impl RenderOnce for PullRequestStack {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap(px(2.))
            .w(rem(384.))
            .max_w_full()
            .children(self.layers.iter().enumerate().map(|(i, l)| {
                let on = Some(l.number) == self.current;
                sans(div(), 14., 400)
                    .relative()
                    .flex()
                    .items_center()
                    .gap(rem(8.))
                    .h(rem(40.))
                    .ml(rem(stack_indent(i)))
                    .px(rem(12.))
                    .bg(color(if on {
                        ColorRole::LayerSelected01
                    } else {
                        ColorRole::Layer01
                    }))
                    .text_color(color(ColorRole::TextPrimary))
                    .when(on, |this| {
                        this.child(start_bar(color(ColorRole::BorderInteractive)))
                    })
                    .child(PullRequestStateGlyph::new(l.state))
                    .child(
                        div()
                            .flex_none()
                            .text_color(color(ColorRole::TextSecondary))
                            .child(format!("#{}", l.number)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .text_ellipsis()
                            .child(l.title.clone()),
                    )
                    .when_some(l.checks, |this, c| this.child(ChecksGlyph::new(c)))
            }))
    }
}

/// One CI check (`checks[]`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Check {
    /// `name`.
    pub name: SharedString,
    /// `status`.
    pub status: ChecksStatus,
    /// `duration`.
    pub duration: SharedString,
}

impl Check {
    /// A check.
    pub fn new(
        name: impl Into<SharedString>,
        status: ChecksStatus,
        duration: impl Into<SharedString>,
    ) -> Self {
        Self {
            name: name.into(),
            status,
            duration: duration.into(),
        }
    }
}

/// The summary of a check list: failed if any failed, else running if any
/// runs, else passed; and the head text ("1 of 4 failing" / "4 checks").
pub fn checks_summary(checks: &[Check]) -> (ChecksStatus, String) {
    let failed = checks
        .iter()
        .filter(|c| c.status == ChecksStatus::Failed)
        .count();
    let status = if failed > 0 {
        ChecksStatus::Failed
    } else if checks.iter().any(|c| c.status == ChecksStatus::Running) {
        ChecksStatus::Running
    } else {
        ChecksStatus::Passed
    };
    let text = if failed > 0 {
        format!("{failed} of {} failing", checks.len())
    } else {
        format!("{} checks", checks.len())
    };
    (status, text)
}

type CheckHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// The CI checks list (`.t3-checklist` on `layer-01`; flat and full width per
/// `app-r3`): a summary head, then one 40px row per check with its glyph,
/// name, duration and a Details link.
#[derive(IntoElement)]
pub struct ChecksList {
    id: ElementId,
    checks: Vec<Check>,
    details_label: SharedString,
    on_details: Option<CheckHandler>,
}

impl ChecksList {
    /// A checks list.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            checks: Vec::new(),
            details_label: "Details".into(),
            on_details: None,
        }
    }

    /// The checks.
    pub fn checks(mut self, checks: impl IntoIterator<Item = Check>) -> Self {
        self.checks = checks.into_iter().collect();
        self
    }

    /// The link text (`Details`).
    pub fn details_label(mut self, label: impl Into<SharedString>) -> Self {
        self.details_label = label.into();
        self
    }

    /// A Details link was activated (check index).
    pub fn on_details(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_details = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for ChecksList {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let (status, head) = checks_summary(&self.checks);
        let id = self.id.clone();
        sans(div(), 14., 400)
            .flex()
            .flex_col()
            .bg(color(ColorRole::Layer01))
            .text_color(color(ColorRole::TextPrimary))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(rem(8.))
                    .p(rem(12.))
                    .border_b_1()
                    .border_color(color(ColorRole::BorderSubtle01))
                    .child(ChecksGlyph::new(status))
                    .child(sans(div(), 14., 600).child(head)),
            )
            .children(self.checks.iter().enumerate().map(|(i, c)| {
                let on_details = self.on_details.clone();
                div()
                    .id(craft::sub(&id, "row", i))
                    .flex()
                    .items_center()
                    .gap(rem(8.))
                    .min_h(rem(40.))
                    .px(rem(12.))
                    .hover(|s| s.bg(color(ColorRole::LayerHover01)))
                    .child(ChecksGlyph::new(c.status))
                    .child(div().flex_1().min_w_0().child(c.name.clone()))
                    .child(craft::text(
                        c.duration.clone(),
                        12.,
                        ColorRole::TextSecondary,
                    ))
                    .child(
                        Button::new(craft::sub(&id, "details", i), self.details_label.clone())
                            .variant(ButtonVariant::Link)
                            .when_some(on_details, |b, f| b.on_click(move |_, w, cx| f(i, w, cx))),
                    )
            }))
    }
}

/// One guide stop (`stops[]`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuideStop {
    /// `title`.
    pub title: SharedString,
    /// `files`.
    pub files: usize,
    /// `note`.
    pub note: SharedString,
}

impl GuideStop {
    /// A stop.
    pub fn new(
        title: impl Into<SharedString>,
        files: usize,
        note: impl Into<SharedString>,
    ) -> Self {
        Self {
            title: title.into(),
            files,
            note: note.into(),
        }
    }
}

/// Where a guide stop is relative to the current one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StopState {
    /// Before the current stop (`--done`: green bullet with a check).
    Done,
    /// The current stop (`--on`: blue bullet, selected row).
    Current,
    /// After it (a neutral numbered bullet).
    Ahead,
}

impl StopState {
    /// The state of stop `i` when `current` is shown.
    pub fn of(i: usize, current: usize) -> Self {
        use std::cmp::Ordering::*;
        match i.cmp(&current) {
            Less => Self::Done,
            Equal => Self::Current,
            Greater => Self::Ahead,
        }
    }

    /// (bullet fill, bullet ink): `layer-accent-01`/`text-primary`, current
    /// `interactive`/`text-on-color`, done `support-success`/`text-on-color`.
    pub fn bullet(self) -> (Hsla, Hsla) {
        match self {
            Self::Ahead => (
                color(ColorRole::LayerAccent01),
                color(ColorRole::TextPrimary),
            ),
            Self::Current => (color(ColorRole::Interactive), color(ColorRole::TextOnColor)),
            Self::Done => (
                color(ColorRole::SupportSuccess),
                color(ColorRole::TextOnColor),
            ),
        }
    }
}

type IndexHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// The PR guide route (`.t3-guide`, 18rem): one button per stop with a 20px
/// bullet joined to the next by a 1px rule; done stops show a check.
#[derive(IntoElement)]
pub struct GuideRail {
    id: ElementId,
    stops: Vec<GuideStop>,
    current: usize,
    on_select: Option<IndexHandler>,
    files_label: SharedString,
}

impl GuideRail {
    /// A guide rail.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            stops: Vec::new(),
            current: 0,
            on_select: None,
            files_label: "files".into(),
        }
    }

    /// The stops.
    pub fn stops(mut self, stops: impl IntoIterator<Item = GuideStop>) -> Self {
        self.stops = stops.into_iter().collect();
        self
    }

    /// The current stop index.
    pub fn current(mut self, current: usize) -> Self {
        self.current = current;
        self
    }

    /// A stop was chosen (`onSelect`).
    pub fn on_select(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }

    /// The word after the file count (`files`).
    pub fn files_label(mut self, label: impl Into<SharedString>) -> Self {
        self.files_label = label.into();
        self
    }
}

impl RenderOnce for GuideRail {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let n = self.stops.len();
        let rows: Vec<AnyElement> = self
            .stops
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let state = StopState::of(i, self.current);
                let row_id = craft::sub(&self.id, "stop", i);
                let focus = focus_handle(&row_id, window, cx);
                let ring = focus_visible(&focus, window);
                let (fill, ink) = state.bullet();
                let on_select = self.on_select.clone();
                let bullet = sans(div(), 11., 400)
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .size(rem(20.))
                    .rounded_full()
                    .bg(fill)
                    .text_color(ink)
                    .map(|this| match state {
                        StopState::Done => this.child(
                            Icon::carbon(CarbonIcon::Checkmark)
                                .size_length(rem(12.))
                                .color(ink),
                        ),
                        _ => this.child((i + 1).to_string()),
                    });
                press(row_id, &focus)
                    .w_full()
                    .items_start()
                    .gap(rem(12.))
                    .px(rem(12.))
                    .py(rem(8.))
                    .text_color(color(if state == StopState::Current {
                        ColorRole::TextPrimary
                    } else {
                        ColorRole::TextSecondary
                    }))
                    .when(state == StopState::Current, |this| {
                        this.bg(color(ColorRole::LayerSelected01))
                    })
                    .hover(|s| s.bg(color(ColorRole::LayerHover01)))
                    .when_some(on_select, |this, f| {
                        this.on_click(move |_, w, cx| f(i, w, cx))
                    })
                    // `::after { left: calc(12px + 9.5px); top: 2rem; bottom: -0.5rem; width: 1px }`.
                    .when(i + 1 < n, |this| {
                        this.child(
                            div()
                                .absolute()
                                .left(px(21.5))
                                .top(rem(32.))
                                .bottom(rem(-8.))
                                .w(px(1.))
                                .bg(color(ColorRole::BorderSubtle01)),
                        )
                    })
                    .child(bullet)
                    .child(
                        sans(div(), 14., 400)
                            .flex()
                            .flex_col()
                            .gap(px(2.))
                            .child(sans(div(), 14., 600).child(s.title.clone()))
                            .child(craft::text(
                                format!("{} {} \u{b7} {}", s.files, self.files_label, s.note),
                                12.,
                                ColorRole::TextSecondary,
                            )),
                    )
                    .when(ring, |this| this.child(craft::focus_ring()))
                    .into_any_element()
            })
            .collect();
        div()
            .flex()
            .flex_col()
            .w(rem(288.))
            .max_w_full()
            .children(rows)
    }
}

/// Texts of a [`PullRequestDetailHeader`] (hard-coded in the reference).
#[derive(Clone, Debug)]
pub struct DetailHeaderTexts {
    /// `Back`.
    pub back: SharedString,
    /// `Edit title`.
    pub edit_title: SharedString,
    /// ` wants to merge `.
    pub wants_to_merge: SharedString,
    /// ` into `.
    pub into: SharedString,
}

impl Default for DetailHeaderTexts {
    fn default() -> Self {
        Self {
            back: "Back".into(),
            edit_title: "Edit title".into(),
            wants_to_merge: "wants to merge".into(),
            into: "into".into(),
        }
    }
}

/// `.t3-ref`: mono 12px on `layer-accent-01`, `1px 4px` padding.
pub(crate) fn code_ref(text: impl Into<SharedString>) -> gpui_kit::Div {
    mono(div(), 12., 400)
        .flex_none()
        .px(rem(4.))
        .py(px(1.))
        .bg(color(ColorRole::LayerAccent01))
        .text_color(color(ColorRole::TextPrimary))
        .child(text.into())
}

type TabHandler = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;

/// The PR detail page header (`.t3-prhead`): Back + repo / `#number`
/// crumbs, state glyph + title + Edit, state tag + "author wants to merge
/// head into base" + primary actions, and the page tabs.
#[derive(IntoElement)]
pub struct PullRequestDetailHeader {
    id: ElementId,
    number: u32,
    title: SharedString,
    state: PullRequestState,
    repo: SharedString,
    author: SharedString,
    head: SharedString,
    base: SharedString,
    tabs: Vec<SharedString>,
    tab: Option<SharedString>,
    primary: Option<AnyElement>,
    on_tab: Option<TabHandler>,
    on_back: Option<Handler>,
    on_edit: Option<Handler>,
    on_number: Option<Handler>,
    texts: DetailHeaderTexts,
}

impl PullRequestDetailHeader {
    /// A header for PR `number` titled `title`.
    pub fn new(id: impl Into<ElementId>, number: u32, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            number,
            title: title.into(),
            state: PullRequestState::Open,
            repo: SharedString::default(),
            author: SharedString::default(),
            head: SharedString::default(),
            base: SharedString::default(),
            tabs: ["Summary", "Code", "Timeline", "Guide"]
                .map(SharedString::from)
                .to_vec(),
            tab: None,
            primary: None,
            on_tab: None,
            on_back: None,
            on_edit: None,
            on_number: None,
            texts: DetailHeaderTexts::default(),
        }
    }

    /// The state.
    pub fn state(mut self, state: PullRequestState) -> Self {
        self.state = state;
        self
    }

    /// The repository.
    pub fn repo(mut self, repo: impl Into<SharedString>) -> Self {
        self.repo = repo.into();
        self
    }

    /// The author.
    pub fn author(mut self, author: impl Into<SharedString>) -> Self {
        self.author = author.into();
        self
    }

    /// Head and base branches.
    pub fn branches(
        mut self,
        head: impl Into<SharedString>,
        base: impl Into<SharedString>,
    ) -> Self {
        self.head = head.into();
        self.base = base.into();
        self
    }

    /// The tabs (default Summary, Code, Timeline, Guide).
    pub fn tabs<S: Into<SharedString>>(mut self, tabs: impl IntoIterator<Item = S>) -> Self {
        self.tabs = tabs.into_iter().map(Into::into).collect();
        self
    }

    /// The selected tab (default the first).
    pub fn tab(mut self, tab: impl Into<SharedString>) -> Self {
        self.tab = Some(tab.into());
        self
    }

    /// Primary actions at the meta row's end.
    pub fn primary(mut self, primary: impl IntoElement) -> Self {
        self.primary = Some(primary.into_any_element());
        self
    }

    /// A tab was chosen (`onTab`).
    pub fn on_tab(mut self, f: impl Fn(SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_tab = Some(Rc::new(f));
        self
    }

    /// Back pressed.
    pub fn on_back(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_back = Some(Rc::new(f));
        self
    }

    /// Edit title pressed.
    pub fn on_edit(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_edit = Some(Rc::new(f));
        self
    }

    /// The `#number` link was activated.
    pub fn on_number(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_number = Some(Rc::new(f));
        self
    }

    /// Replace the default texts.
    pub fn texts(mut self, texts: DetailHeaderTexts) -> Self {
        self.texts = texts;
        self
    }
}

pub(crate) fn icon_button(
    id: impl Into<ElementId>,
    label: SharedString,
    icon: CarbonIcon,
    handler: Option<Handler>,
) -> Button {
    Button::new(id, label)
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::Sm)
        .icon(icon)
        .icon_only(true)
        .when_some(handler, |b, f| b.on_click(move |_, w, cx| f(w, cx)))
}

impl RenderOnce for PullRequestDetailHeader {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let secondary = color(ColorRole::TextSecondary);
        let current = self
            .tab
            .clone()
            .or_else(|| self.tabs.first().cloned())
            .unwrap_or_default();
        let crumbs = sans(div(), 14., 400)
            .flex()
            .items_center()
            .gap(rem(4.))
            .text_color(secondary)
            .child(icon_button(
                (id.clone(), "back"),
                self.texts.back.clone(),
                CarbonIcon::ArrowLeft,
                self.on_back.clone(),
            ))
            .child(self.repo.clone())
            .child(div().text_color(color(ColorRole::TextHelper)).child("/"))
            .child({
                let f = self.on_number.clone();
                Button::new((id.clone(), "number"), format!("#{}", self.number))
                    .variant(ButtonVariant::Link)
                    .when_some(f, |b, f| b.on_click(move |_, w, cx| f(w, cx)))
            });
        let title = div()
            .flex()
            .items_center()
            .gap(rem(8.))
            .child(PullRequestStateGlyph::new(self.state).size(20.))
            .child(
                sans(div(), 20., 300)
                    .font_weight(gpui_kit::FontWeight(400.))
                    .line_height(relative(1.4))
                    .text_color(color(ColorRole::TextPrimary))
                    .child(self.title.clone()),
            )
            .child(icon_button(
                (id.clone(), "edit"),
                self.texts.edit_title.clone(),
                CarbonIcon::Edit,
                self.on_edit.clone(),
            ));
        let meta = sans(div(), 14., 400)
            .flex()
            .flex_wrap()
            .items_center()
            .gap(rem(8.))
            .text_color(secondary)
            .child(
                Badge::new(self.state.label())
                    .variant(self.state.badge())
                    .size(BadgeSize::Sm),
            )
            .child(
                sans(div(), 14., 600)
                    .text_color(color(ColorRole::TextPrimary))
                    .child(self.author.clone()),
            )
            .child(self.texts.wants_to_merge.clone())
            .child(code_ref(self.head.clone()))
            .child(self.texts.into.clone())
            .child(code_ref(self.base.clone()))
            .child(div().flex_1())
            .children(self.primary);
        let tabs = div()
            .flex()
            .children(self.tabs.iter().enumerate().map(|(i, t)| {
                let on = *t == current;
                let tab_id = craft::sub(&id, "tab", i);
                let focus = focus_handle(&tab_id, window, cx);
                let ring = focus_visible(&focus, window);
                let on_tab = self.on_tab.clone();
                let name = t.clone();
                // `.t3-prtab { height: 40px; padding: 0 16px; border-bottom: 2px }`.
                sans(press(tab_id, &focus), 14., if on { 600 } else { 400 })
                    .role(gpui_kit::Role::Tab)
                    .h(rem(40.))
                    .px(rem(16.))
                    .border_b_2()
                    .text_color(color(if on {
                        ColorRole::TextPrimary
                    } else {
                        ColorRole::TextSecondary
                    }))
                    .border_color(if on {
                        color(ColorRole::BorderInteractive)
                    } else {
                        Hsla::transparent_black()
                    })
                    .when(!on, |this| {
                        this.hover(|s| {
                            s.text_color(color(ColorRole::TextPrimary))
                                .border_color(color(ColorRole::BorderStrong01))
                        })
                    })
                    .when_some(on_tab, |this, f| {
                        this.on_click(move |_, w, cx| f(name.clone(), w, cx))
                    })
                    .child(t.clone())
                    .when(ring, |this| this.child(craft::focus_ring()))
            }));
        div()
            .id(id)
            .flex()
            .flex_col()
            .gap(rem(8.))
            .border_b_1()
            .border_color(color(ColorRole::BorderSubtle01))
            .child(crumbs)
            .child(title)
            .child(meta)
            .child(tabs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checks_summarise_like_the_reference() {
        let checks = [
            Check::new("build", ChecksStatus::Passed, "2m 10s"),
            Check::new("unit tests", ChecksStatus::Failed, "4m 02s"),
            Check::new("visual regression", ChecksStatus::Running, "1m"),
            Check::new("lint", ChecksStatus::Passed, "38s"),
        ];
        assert_eq!(
            checks_summary(&checks),
            (ChecksStatus::Failed, "1 of 4 failing".to_string())
        );
        assert_eq!(
            checks_summary(&checks[2..]),
            (ChecksStatus::Running, "2 checks".to_string())
        );
        assert_eq!(
            checks_summary(&checks[..1]),
            (ChecksStatus::Passed, "1 checks".to_string())
        );
    }

    #[test]
    fn stops_and_stack_geometry() {
        assert_eq!(StopState::of(0, 1), StopState::Done);
        assert_eq!(StopState::of(1, 1), StopState::Current);
        assert_eq!(StopState::of(2, 1), StopState::Ahead);
        assert_eq!(stack_indent(3), 48.);
    }

    #[test]
    fn vocabulary_colors() {
        crate::theme::tests_set(crate::tokens::CarbonTheme::White);
        assert_eq!(
            PullRequestState::Merged.ink(),
            color(ColorRole::TagColorPurple)
        );
        assert_eq!(PullRequestState::Merged.icon(), CarbonIcon::Merge);
        assert_eq!(
            ChecksStatus::Running.ink(),
            color(ColorRole::SupportWarning)
        );
        assert_eq!(ReviewDecision::Changes.ink(), color(ColorRole::TextError));
        assert_eq!(StopState::Done.bullet().0, color(ColorRole::SupportSuccess));
    }
}
