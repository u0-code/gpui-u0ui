//! T3 · Pull requests & issues: the PR vocabulary ([`PullRequestStateGlyph`],
//! [`ChecksGlyph`], [`ReviewDecisionBadge`], [`LabelChip`]) and the cards
//! built from it ([`PullRequestRow`], [`PullRequestStack`], [`ChecksList`],
//! [`GuideRail`], [`PullRequestDetailHeader`], [`ReactionBar`],
//! [`PullRequestTimeline`], [`ReviewForm`]) — `src/t3.css` "PR vocabulary".
use super::diff::DiffStat;
use crate::{
    components::{AvatarColor, BadgeVariant, Button, ButtonVariant},
    icons::{CarbonIcon, Icon},
    t3::craft::{self, AvatarTone, focus_handle, focus_visible, mono, press, rem, sans},
    theme::color,
    tokens::ColorRole,
};
use gpui_kit::{
    AnyElement, App, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _, px,
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

    /// `(background, text)` of the sidebar's mini badge (`.t3-prmini`):
    /// `.t3-pr--open` green tag, `--draft` gray, `--merged` purple,
    /// `--closed` red.
    pub fn tag_colors(self) -> (Hsla, Hsla) {
        let (bg, fg) = match self {
            Self::Open => (ColorRole::TagBackgroundGreen, ColorRole::TagColorGreen),
            Self::Draft => (ColorRole::TagBackgroundGray, ColorRole::TagColorGray),
            Self::Merged => (ColorRole::TagBackgroundPurple, ColorRole::TagColorPurple),
            Self::Closed => (ColorRole::TagBackgroundRed, ColorRole::TagColorRed),
        };
        (color(bg), color(fg))
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

/// A status icon with its `title` (`role="img" aria-label title`).
pub(crate) fn glyph(
    icon: impl Into<Icon>,
    ink: Hsla,
    size: f32,
    title: &'static str,
) -> gpui_kit::Stateful<gpui_kit::Div> {
    craft::titled(
        div()
            .flex()
            .flex_none()
            .child(icon.into().size_length(rem(size)).color(ink)),
        "glyph",
        title,
    )
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
        glyph(
            self.state.icon(),
            self.state.ink(),
            self.size,
            self.state.label(),
        )
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
        glyph(
            self.status.icon(),
            self.status.ink(),
            16.,
            self.status.label(),
        )
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
    conflict_title: SharedString,
    unread: bool,
    selected: bool,
    divided: bool,
    show_avatar: bool,
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
            conflict_title: "Has conflicts".into(),
            unread: false,
            selected: false,
            divided: true,
            show_avatar: true,
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

    /// The conflicts text and its tooltip (`Conflicts`, `Has conflicts`).
    pub fn conflict_label(
        mut self,
        label: impl Into<SharedString>,
        title: impl Into<SharedString>,
    ) -> Self {
        self.conflict_label = label.into();
        self.conflict_title = title.into();
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

    /// Show the bottom divider rule (`true` by default; `false` inside role-grouped lists).
    pub fn divided(mut self, divided: bool) -> Self {
        self.divided = divided;
        self
    }

    /// Show the 16px author avatar in the metadata row (`true` by default).
    pub fn show_avatar(mut self, show: bool) -> Self {
        self.show_avatar = show;
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
                    .when(self.show_avatar, |m| {
                        m.child(small_avatar(
                            if self.author.is_empty() {
                                "?"
                            } else {
                                &self.author
                            },
                            AvatarColor::Teal,
                            16.,
                        ))
                    })
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
                    craft::titled(
                        sans(div(), 12., 400),
                        "conflict",
                        self.conflict_title.clone(),
                    )
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
            .when(self.divided, |this| {
                this.border_b_1()
                    .border_color(color(ColorRole::BorderSubtle01))
            })
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

type StackSelectHandler = Rc<dyn Fn(u32, &mut Window, &mut App)>;

/// Stacked pull requests (`.t3-stack`): each layer 40px on `layer-01`,
/// indented 16px per depth; the current one selected with a 3px bar.
#[derive(IntoElement, Clone, Default)]
pub struct PullRequestStack {
    layers: Vec<StackLayer>,
    current: Option<u32>,
    flat: bool,
    on_select: Option<StackSelectHandler>,
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

    /// Flat sidebar layout (`margin-left: 0` and wrapping multi-line titles).
    pub fn flat(mut self, flat: bool) -> Self {
        self.flat = flat;
        self
    }

    /// Layer selected (`onSelect(number)`).
    pub fn on_select(mut self, f: impl Fn(u32, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for PullRequestStack {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let flat = self.flat;
        let on_select = self.on_select;
        div()
            .flex()
            .flex_col()
            .gap(px(2.))
            .map(|d| {
                if flat {
                    d.w_full()
                } else {
                    d.w(rem(384.)).max_w_full()
                }
            })
            .children(self.layers.iter().enumerate().map(|(i, l)| {
                let on = Some(l.number) == self.current;
                let num = l.number;
                let cb = on_select.clone();
                let mut row = sans(div(), 14., 400)
                    .id(SharedString::from(format!("t3-stack-{num}")))
                    .relative()
                    .flex()
                    .gap(rem(8.))
                    .px(rem(12.))
                    .bg(color(if on {
                        ColorRole::LayerSelected01
                    } else {
                        ColorRole::Layer01
                    }))
                    .text_color(color(ColorRole::TextPrimary));
                row = if flat {
                    row.items_start().min_h(rem(40.)).py(rem(8.))
                } else {
                    row.items_center().h(rem(40.)).ml(rem(stack_indent(i)))
                };
                if let Some(f) = cb {
                    row = row
                        .cursor_pointer()
                        .hover(|s| s.bg(color(ColorRole::LayerHover01)))
                        .on_click(move |_, w, cx| f(num, w, cx));
                }
                row.when(on, |this| {
                    this.child(start_bar(color(ColorRole::BorderInteractive)))
                })
                .child(
                    div()
                        .when(flat, |d| d.mt(px(2.)))
                        .child(PullRequestStateGlyph::new(l.state)),
                )
                .child(
                    div()
                        .flex_none()
                        .text_color(color(ColorRole::TextSecondary))
                        .child(format!("#{}", l.number)),
                )
                .child(if flat {
                    div().flex_1().min_w_0().child(l.title.clone())
                } else {
                    div()
                        .flex_1()
                        .min_w_0()
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .text_ellipsis()
                        .child(l.title.clone())
                })
                .when_some(l.checks, |this, c| {
                    this.child(
                        div()
                            .when(flat, |d| d.mt(px(2.)))
                            .child(ChecksGlyph::new(c)),
                    )
                })
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
    show_header: bool,
    compact: bool,
    details_failed_only: bool,
    on_details: Option<CheckHandler>,
}

impl ChecksList {
    /// A checks list.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            checks: Vec::new(),
            details_label: "Details".into(),
            show_header: true,
            compact: false,
            details_failed_only: false,
            on_details: None,
        }
    }

    /// Show the summary header row (`true` by default; `false` inside a titled card).
    pub fn show_header(mut self, show: bool) -> Self {
        self.show_header = show;
        self
    }

    /// Compact card layout (`px: 0`, `36px` rows, transparent background).
    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }

    /// Show the `Details` link only on failed checks (`false` by default).
    pub fn details_failed_only(mut self, failed_only: bool) -> Self {
        self.details_failed_only = failed_only;
        self
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
        let compact = self.compact;
        let failed_only = self.details_failed_only;
        sans(div(), 14., 400)
            .flex()
            .flex_col()
            .when(!compact, |d| d.bg(color(ColorRole::Layer01)))
            .text_color(color(ColorRole::TextPrimary))
            .when(self.show_header, |d| {
                d.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(rem(8.))
                        .p(rem(12.))
                        .border_b_1()
                        .border_color(color(ColorRole::BorderSubtle01))
                        .child(ChecksGlyph::new(status))
                        // A bare `<b>`: Carbon's reset (`b { font: inherit }`) keeps it 400.
                        .child(sans(div(), 14., 400).child(head)),
                )
            })
            .children(self.checks.iter().enumerate().map(|(i, c)| {
                let on_details = self.on_details.clone();
                let show_details = !failed_only || c.status == ChecksStatus::Failed;
                let details_btn = show_details.then(|| {
                    Button::new(craft::sub(&id, "details", i), self.details_label.clone())
                        .variant(ButtonVariant::Link)
                        .when_some(on_details, |b, f| b.on_click(move |_, w, cx| f(i, w, cx)))
                });
                let mut row = div()
                    .id(craft::sub(&id, "row", i))
                    .flex()
                    .items_center()
                    .gap(rem(8.))
                    .min_h(rem(if compact { 36. } else { 40. }))
                    .when(!compact, |r| r.px(rem(12.)))
                    .hover(|s| s.bg(color(ColorRole::LayerHover01)))
                    .child(ChecksGlyph::new(c.status));
                if failed_only && show_details {
                    row = row
                        .child(div().flex_none().child(c.name.clone()))
                        .children(details_btn)
                        .child(div().flex_1())
                        .child(craft::text(
                            c.duration.clone(),
                            12.,
                            ColorRole::TextSecondary,
                        ));
                } else {
                    row = row
                        .child(div().flex_1().min_w_0().child(c.name.clone()))
                        .child(craft::text(
                            c.duration.clone(),
                            12.,
                            ColorRole::TextSecondary,
                        ))
                        .children(details_btn);
                }
                row
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
                let bullet = sans(div(), 12., 400)
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
                            // `.t3-guide-text b`: no weight rule, so Carbon's
                            // `b { font: inherit }` keeps it 400.
                            .child(sans(div(), 14., 400).child(s.title.clone()))
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
