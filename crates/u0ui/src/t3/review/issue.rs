//! T3 · Pull requests & issues: [`IssueStateGlyph`] and [`IssueRow`]
//! (`src/t3.css` "Issues").
use super::pull_request::{Label, LabelChip, icon_count};
use crate::{
    components::AvatarColor,
    icons::{CarbonIcon, Icon},
    t3::craft::{self, focus_handle, focus_visible, mono, press, rem, sans},
    theme::color,
    tokens::ColorRole,
};
use gpui_kit::{
    App, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, Window, div, prelude::FluentBuilder as _, px,
};
use std::rc::Rc;

/// An issue's state (`IS`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum IssueState {
    /// `open`, the default.
    #[default]
    Open,
    /// `progress`.
    Progress,
    /// `done`.
    Done,
    /// `cancelled`.
    Cancelled,
    /// `backlog`.
    Backlog,
}

impl IssueState {
    /// Every state.
    pub const ALL: [IssueState; 5] = [
        Self::Open,
        Self::Progress,
        Self::Done,
        Self::Cancelled,
        Self::Backlog,
    ];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Progress => "progress",
            Self::Done => "done",
            Self::Cancelled => "cancelled",
            Self::Backlog => "backlog",
        }
    }

    /// The reference label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Open => "Open",
            Self::Progress => "In progress",
            Self::Done => "Done",
            Self::Cancelled => "Cancelled",
            Self::Backlog => "Backlog",
        }
    }

    /// `CircleDash`, `InProgress`, `CheckmarkFilled`, `Close`.
    pub const fn icon(self) -> CarbonIcon {
        match self {
            Self::Open | Self::Backlog => CarbonIcon::CircleDash,
            Self::Progress => CarbonIcon::InProgress,
            Self::Done => CarbonIcon::CheckmarkFilled,
            Self::Cancelled => CarbonIcon::Close,
        }
    }

    /// `.t3-is--open, --backlog` `icon-secondary`; `--progress`
    /// `support-warning`; `--done` `tag-color-purple`; `--cancelled`
    /// `text-disabled`.
    pub fn ink(self) -> Hsla {
        color(match self {
            Self::Open | Self::Backlog => ColorRole::IconSecondary,
            Self::Progress => ColorRole::SupportWarning,
            Self::Done => ColorRole::TagColorPurple,
            Self::Cancelled => ColorRole::TextDisabled,
        })
    }
}

/// The issue state icon (`.t3-isglyph`, 16px).
#[derive(IntoElement, Clone, Copy, Debug, PartialEq, Eq)]
pub struct IssueStateGlyph {
    state: IssueState,
}

impl IssueStateGlyph {
    /// A glyph.
    pub fn new(state: IssueState) -> Self {
        Self { state }
    }
}

impl RenderOnce for IssueStateGlyph {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div().flex().flex_none().child(
            Icon::carbon(self.state.icon())
                .size_length(rem(16.))
                .color(self.state.ink()),
        )
    }
}

/// Issue priority (`priority`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Priority {
    /// `low`: one bar.
    Low,
    /// `medium`: two bars.
    Medium,
    /// `high`: three bars.
    High,
    /// `urgent`: three red bars.
    Urgent,
}

impl Priority {
    /// Every priority.
    pub const ALL: [Priority; 4] = [Self::Low, Self::Medium, Self::High, Self::Urgent];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Urgent => "urgent",
        }
    }
}

/// The three priority bars' fills (`.t3-issue-pri i`: 3px wide, 5/8/12px
/// tall; `layer-accent-01`, lit `icon-primary`, urgent `support-error`).
pub fn priority_bars(priority: Option<Priority>) -> [Hsla; 3] {
    let lit = match priority {
        None => 0,
        Some(Priority::Low) => 1,
        Some(Priority::Medium) => 2,
        Some(Priority::High | Priority::Urgent) => 3,
    };
    std::array::from_fn(|i| {
        if priority == Some(Priority::Urgent) {
            color(ColorRole::SupportError)
        } else if i < lit {
            color(ColorRole::IconPrimary)
        } else {
            color(ColorRole::LayerAccent01)
        }
    })
}

/// `.t3-issue-pri i:nth-child(n) { height }`.
pub const PRIORITY_HEIGHTS: [f32; 3] = [5., 8., 12.];

type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

/// An issue list row (`.t3-issue`, 40px): priority bars, mono id, state,
/// title, labels, linked-thread count, assignee and time.
#[derive(IntoElement)]
pub struct IssueRow {
    id: ElementId,
    key: SharedString,
    title: SharedString,
    state: IssueState,
    priority: Option<Priority>,
    labels: Vec<Label>,
    assignee: Option<SharedString>,
    time: SharedString,
    linked_threads: usize,
    on_click: Option<Handler>,
}

impl IssueRow {
    /// A row for issue `key` (e.g. `CUI-142`) titled `title`.
    pub fn new(
        id: impl Into<ElementId>,
        key: impl Into<SharedString>,
        title: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            key: key.into(),
            title: title.into(),
            state: IssueState::Open,
            priority: None,
            labels: Vec::new(),
            assignee: None,
            time: SharedString::default(),
            linked_threads: 0,
            on_click: None,
        }
    }

    /// The state.
    pub fn state(mut self, state: IssueState) -> Self {
        self.state = state;
        self
    }

    /// The priority.
    pub fn priority(mut self, priority: Priority) -> Self {
        self.priority = Some(priority);
        self
    }

    /// Labels.
    pub fn labels(mut self, labels: impl IntoIterator<Item = Label>) -> Self {
        self.labels = labels.into_iter().collect();
        self
    }

    /// The assignee's initials (shown as given).
    pub fn assignee(mut self, assignee: impl Into<SharedString>) -> Self {
        self.assignee = Some(assignee.into());
        self
    }

    /// Relative time.
    pub fn time(mut self, time: impl Into<SharedString>) -> Self {
        self.time = time.into();
        self
    }

    /// Number of linked threads (shown when non-zero).
    pub fn linked_threads(mut self, n: usize) -> Self {
        self.linked_threads = n;
        self
    }

    /// Activation.
    pub fn on_click(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for IssueRow {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let focus = focus_handle(&id, window, cx);
        let ring = focus_visible(&focus, window);
        let on_click = self.on_click.clone();
        let bars = priority_bars(self.priority);
        sans(press(id, &focus), 14., 400)
            .accessibility_label(self.title.clone())
            .w_full()
            .items_center()
            .gap(rem(12.))
            .h(rem(40.))
            .px(rem(16.))
            .border_b_1()
            .border_color(color(ColorRole::BorderSubtle01))
            .text_color(color(ColorRole::TextPrimary))
            .hover(|s| s.bg(color(ColorRole::LayerHover01)))
            .when_some(on_click, |this, f| this.on_click(move |_, w, cx| f(w, cx)))
            .child(
                div()
                    .flex()
                    .flex_none()
                    .items_end()
                    .gap(px(1.))
                    .h(px(12.))
                    .children(
                        (0..3).map(|i| div().w(px(3.)).h(px(PRIORITY_HEIGHTS[i])).bg(bars[i])),
                    ),
            )
            .child(
                mono(div(), 12., 400)
                    .flex_none()
                    .w(rem(72.))
                    .text_color(color(ColorRole::TextSecondary))
                    .child(self.key.clone()),
            )
            .child(IssueStateGlyph::new(self.state))
            .child(
                div()
                    .min_w_0()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .child(self.title.clone()),
            )
            .children(
                self.labels
                    .iter()
                    .map(|l| LabelChip::new(l.name.clone(), l.color)),
            )
            .child(div().flex_1())
            .when(self.linked_threads > 0, |this| {
                this.child(icon_count(
                    CarbonIcon::Link,
                    self.linked_threads.to_string(),
                ))
            })
            .when_some(self.assignee.clone(), |this, a| {
                // `.t3-issue .cu-avatar--sm { 1.25rem }`; the fallback text stays 12px.
                this.child(craft::initials(
                    a,
                    craft::AvatarTone::Avatar(AvatarColor::Blue),
                    20.,
                ))
            })
            .child(craft::text(self.time.clone(), 12., ColorRole::TextSecondary).flex_none())
            .when(ring, |this| this.child(craft::focus_ring()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn priority_bars_light_up() {
        crate::theme::tests_set(crate::tokens::CarbonTheme::White);
        let on = color(ColorRole::IconPrimary);
        let off = color(ColorRole::LayerAccent01);
        let red = color(ColorRole::SupportError);
        assert_eq!(priority_bars(None), [off; 3]);
        assert_eq!(priority_bars(Some(Priority::Low)), [on, off, off]);
        assert_eq!(priority_bars(Some(Priority::Medium)), [on, on, off]);
        assert_eq!(priority_bars(Some(Priority::High)), [on; 3]);
        assert_eq!(priority_bars(Some(Priority::Urgent)), [red; 3]);
    }

    #[test]
    fn state_glyphs() {
        crate::theme::tests_set(crate::tokens::CarbonTheme::White);
        assert_eq!(IssueState::Backlog.icon(), CarbonIcon::CircleDash);
        assert_eq!(IssueState::Done.ink(), color(ColorRole::TagColorPurple));
        assert_eq!(IssueState::Cancelled.ink(), color(ColorRole::TextDisabled));
    }
}
