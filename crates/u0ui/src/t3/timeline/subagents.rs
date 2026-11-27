//! Subagents ("Subagents" in `t3.css` and the r4-desk fixes):
//! [`SubagentStatusDot`] (`.t3-sdot`), [`SubagentRow`] (`.t3-subagent`) and
//! [`SubagentBar`] (`.t3-subbar`).
//!
//! A row keeps one 16px status slot (dot or 14px spinner), fixed 4.25rem
//! state and 3rem time columns, Open chat, and — while running — a framed
//! Stop after it; other rows reserve the Stop slot so the columns line up.
use crate::components::button::{Button, ButtonKind, ButtonSize, ButtonVariant};
use crate::t3::action::{ActionButton, ActionKind, Part};
use crate::t3::base::{EASE, Spin, fmt_elapsed, pulse};
use crate::t3::press::{Handler, rem};
use crate::t3::shell::{IdentitySize, ProviderIcon};
use crate::{icons::CarbonIcon, theme::color, tokens::ColorRole};
use gpui_kit::{
    AnyElement, App, BoxShadow, ElementId, FontWeight, Hsla, IntoElement, ParentElement,
    RenderOnce, SharedString, Styled, Window, div, prelude::FluentBuilder as _, px,
};
use std::rc::Rc;

/// A subagent's status.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SubagentStatus {
    /// Running (spinner in rows, a pulsing dot in the bar).
    #[default]
    Running,
    /// Queued.
    Queued,
    /// Waiting on something.
    Waiting,
    /// Idle and resumable.
    Idle,
    /// Completed.
    Completed,
    /// Failed.
    Failed,
    /// Stopped.
    Stopped,
}

impl SubagentStatus {
    /// Every status.
    pub const ALL: [SubagentStatus; 7] = [
        Self::Running,
        Self::Queued,
        Self::Waiting,
        Self::Idle,
        Self::Completed,
        Self::Failed,
        Self::Stopped,
    ];

    /// The reference name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Queued => "queued",
            Self::Waiting => "waiting",
            Self::Idle => "idle",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Stopped => "stopped",
        }
    }

    /// The row's state text.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Running => "Running",
            Self::Queued => "Queued",
            Self::Waiting => "Waiting",
            Self::Idle => "Idle · resumable",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
            Self::Stopped => "Stopped",
        }
    }

    /// The dot: (fill, ring). `.t3-sdot { background: icon-secondary }`;
    /// running `interactive`, completed `support-success`, failed
    /// `support-error`, waiting `support-warning`; queued / idle / stopped a
    /// 1.5px `icon-secondary` ring on nothing.
    pub fn dot(self) -> (Hsla, Option<Hsla>) {
        match self {
            Self::Running => (color(ColorRole::Interactive), None),
            Self::Completed => (color(ColorRole::SupportSuccess), None),
            Self::Failed => (color(ColorRole::SupportError), None),
            Self::Waiting => (color(ColorRole::SupportWarning), None),
            Self::Queued | Self::Idle | Self::Stopped => (
                gpui_kit::transparent_black(),
                Some(color(ColorRole::IconSecondary)),
            ),
        }
    }
}

/// The 8px status dot; running pulses (`t3-pulse 1.2s infinite`).
#[derive(IntoElement)]
pub struct SubagentStatusDot {
    id: ElementId,
    status: SubagentStatus,
}

impl SubagentStatusDot {
    /// The dot for `status`.
    pub fn new(id: impl Into<ElementId>, status: SubagentStatus) -> Self {
        Self {
            id: id.into(),
            status,
        }
    }
}

impl RenderOnce for SubagentStatusDot {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let (fill, ring) = self.status.dot();
        let dot = div()
            .flex_none()
            .size(px(8.))
            .rounded_full()
            .bg(fill)
            .when_some(ring, |this, c| {
                this.shadow(vec![
                    BoxShadow::new(px(0.), px(0.), c)
                        .spread_radius(px(1.5))
                        .inset(),
                ])
            });
        if self.status == SubagentStatus::Running {
            pulse(dot, self.id, EASE).into_any_element()
        } else {
            dot.into_any_element()
        }
    }
}

/// `SubagentRow`'s strings.
#[derive(Clone)]
pub struct SubagentTexts {
    /// Open chat.
    pub open: SharedString,
    /// Stop subagent.
    pub stop: SharedString,
    /// The state labels, by status.
    pub label: Rc<dyn Fn(SubagentStatus) -> SharedString>,
}

impl Default for SubagentTexts {
    fn default() -> Self {
        Self {
            open: "Open chat".into(),
            stop: "Stop subagent".into(),
            label: Rc::new(|s| s.label().into()),
        }
    }
}

/// One subagent. See the module docs.
#[derive(IntoElement)]
pub struct SubagentRow {
    id: ElementId,
    name: SharedString,
    task: SharedString,
    status: SubagentStatus,
    elapsed: u64,
    provider: Option<SharedString>,
    texts: SubagentTexts,
    on_open: Option<Handler>,
    on_stop: Option<Handler>,
}

impl SubagentRow {
    /// `name` working on `task`.
    pub fn new(
        id: impl Into<ElementId>,
        name: impl Into<SharedString>,
        task: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            task: task.into(),
            status: SubagentStatus::Running,
            elapsed: 0,
            provider: None,
            texts: SubagentTexts::default(),
            on_open: None,
            on_stop: None,
        }
    }

    /// `status`.
    pub fn status(mut self, status: SubagentStatus) -> Self {
        self.status = status;
        self
    }

    /// `elapsed` seconds.
    pub fn elapsed(mut self, seconds: u64) -> Self {
        self.elapsed = seconds;
        self
    }

    /// `provider`.
    pub fn provider(mut self, provider: impl Into<SharedString>) -> Self {
        self.provider = Some(provider.into());
        self
    }

    /// Replace the strings.
    pub fn texts(mut self, texts: SubagentTexts) -> Self {
        self.texts = texts;
        self
    }

    /// `onOpen`.
    pub fn on_open(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_open = Some(Rc::new(f));
        self
    }

    /// `onStop`.
    pub fn on_stop(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_stop = Some(Rc::new(f));
        self
    }
}

/// `.t3-subagent-state { width: 4rem }`, `.t3-subagent-time { width: 2.75rem }`
/// (the later d-fix rule over `4.25rem` / `3rem`) and the reserved Stop slot
/// `2rem + spacing-03`, in px.
pub const SUBAGENT_COLUMNS: (f32, f32, f32) = (64., 44., 40.);

impl RenderOnce for SubagentRow {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let running = self.status == SubagentStatus::Running;
        let texts = self.texts.clone();
        // One 16px slot: a 14px spinner with 1px margins, or the 8px dot with 4px margins (r4-desk).
        let slot: AnyElement = if running {
            div()
                .flex_none()
                .mx(px(1.))
                .child(
                    Spin::new((id.clone(), "spin"))
                        .size(14.)
                        .label(texts.label.as_ref()(self.status)),
                )
                .into_any_element()
        } else {
            div()
                .flex_none()
                .mx(px(4.))
                .child(SubagentStatusDot::new((id.clone(), "dot"), self.status))
                .into_any_element()
        };
        let open = self.on_open.clone();
        let stop = self.on_stop.clone();
        // `.t3-subagent { display: flex; align-items: center; gap: spacing-03; min-height: size-md;
        // padding: 0 spacing-02 0 spacing-04; background: layer-01; font-size: 0.875rem }`.
        div()
            .flex()
            .flex_row()
            .items_center()
            .w_full()
            .min_w_0()
            .gap(rem(8.))
            .min_h(rem(40.))
            .pl(rem(12.))
            .pr(rem(4.))
            .bg(color(ColorRole::Layer01))
            .text_size(rem(14.))
            .line_height(rem(18.))
            .text_color(color(ColorRole::TextPrimary))
            .child(slot)
            .when_some(self.provider.clone(), |this, p| {
                this.child(ProviderIcon::new(p).size(IdentitySize::Xs))
            })
            // `.t3-subagent-main { display: flex; flex: 1; min-width: 0 }` and the d-fix
            // `{ flex-direction: column; align-items: flex-start; gap: 0; padding-block: spacing-02 }`:
            // the task sits under the name.
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .items_start()
                    .py(rem(4.))
                    .child(
                        div()
                            .flex_none()
                            .whitespace_nowrap()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(self.name.clone()),
                    )
                    // `.t3-subagent-main span { max-width: 100%; font-size: 0.75rem; ellipsis;
                    // color: text-secondary }`, keeping the row's 1.28572 line height.
                    .child(
                        div()
                            .max_w_full()
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .text_ellipsis()
                            .text_size(rem(12.))
                            .line_height(gpui_kit::relative(18. / 14.))
                            .text_color(color(ColorRole::TextSecondary))
                            .child(self.task.clone()),
                    ),
            )
            // `.t3-subagent-state { font-size: 0.75rem; color: text-secondary; flex: none; width: 4.25rem }`,
            // failed `text-error`.
            .child(
                div()
                    .flex_none()
                    .w(rem(SUBAGENT_COLUMNS.0))
                    .whitespace_nowrap()
                    .text_size(rem(12.))
                    .text_color(color(if self.status == SubagentStatus::Failed {
                        ColorRole::TextError
                    } else {
                        ColorRole::TextSecondary
                    }))
                    .child(texts.label.as_ref()(self.status)),
            )
            // `.t3-subagent-time { font-size: 0.75rem; color: text-helper; flex: none; width: 3rem }`.
            .child(
                div()
                    .flex_none()
                    .w(rem(SUBAGENT_COLUMNS.1))
                    .text_size(rem(12.))
                    .text_color(color(ColorRole::TextHelper))
                    .child(SharedString::from(fmt_elapsed(self.elapsed))),
            )
            .child(
                div()
                    .flex_none()
                    .when(!running, |this| this.mr(rem(SUBAGENT_COLUMNS.2)))
                    .child(
                        Button::new((id.clone(), "open"), texts.open.clone())
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::Sm)
                            .when_some(open, |b, f| b.on_click(move |_, w, cx| f(w, cx))),
                    ),
            )
            .when(running, |this| {
                // gemini-A: `.t3-subagent--running .cds--btn--icon-only { box-shadow: inset 0 0 0 1px
                // border-subtle-01 }`, `svg { fill: icon-secondary }`.
                this.child(
                    div()
                        .relative()
                        .flex_none()
                        .child(
                            ActionButton::new(
                                (id.clone(), "stop"),
                                texts.stop.clone(),
                                ActionKind::Carbon(ButtonKind::Ghost),
                            )
                            .padding(0., 0.)
                            .square(true)
                            .ink(color(ColorRole::IconSecondary))
                            .part(Part::Icon(CarbonIcon::StopFilled))
                            .tooltip(texts.stop.clone())
                            .on_click(stop),
                        )
                        .child(
                            div()
                                .absolute()
                                .inset_0()
                                .border_1()
                                .border_color(color(ColorRole::BorderSubtle01)),
                        ),
                )
            })
    }
}

/// The subagents summary bar: dots, "{n} subagents", "{k} running", View all.
#[derive(IntoElement)]
pub struct SubagentBar {
    id: ElementId,
    agents: Vec<SubagentStatus>,
    count: Rc<dyn Fn(usize) -> SharedString>,
    running: Rc<dyn Fn(usize) -> SharedString>,
    view_all: SharedString,
    on_view_all: Option<Handler>,
}

impl SubagentBar {
    /// A bar summarizing `agents`.
    pub fn new(id: impl Into<ElementId>, agents: Vec<SubagentStatus>) -> Self {
        Self {
            id: id.into(),
            agents,
            count: Rc::new(|n| format!("{n} subagents").into()),
            running: Rc::new(|n| format!("{n} running").into()),
            view_all: "View all".into(),
            on_view_all: None,
        }
    }

    /// The View all handler.
    pub fn on_view_all(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_view_all = Some(Rc::new(f));
        self
    }

    /// Replace the strings.
    pub fn texts(
        mut self,
        count: impl Fn(usize) -> SharedString + 'static,
        running: impl Fn(usize) -> SharedString + 'static,
        view_all: impl Into<SharedString>,
    ) -> Self {
        self.count = Rc::new(count);
        self.running = Rc::new(running);
        self.view_all = view_all.into();
        self
    }
}

impl RenderOnce for SubagentBar {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let running = self
            .agents
            .iter()
            .filter(|s| **s == SubagentStatus::Running)
            .count();
        let view = self.on_view_all.clone();
        // `.t3-subbar { display: flex; align-items: center; gap: spacing-03; min-height: size-md;
        // padding: 0 spacing-02 0 spacing-04; background: layer-01; font-size: 0.875rem }`.
        div()
            .flex()
            .flex_row()
            .items_center()
            .w_full()
            .gap(rem(8.))
            .min_h(rem(40.))
            .pl(rem(12.))
            .pr(rem(4.))
            .bg(color(ColorRole::Layer01))
            .text_size(rem(14.))
            .line_height(rem(18.))
            .text_color(color(ColorRole::TextPrimary))
            // `.t3-subbar-dots { display: flex; gap: 3px }`.
            .child(div().flex().flex_row().items_center().gap(px(3.)).children(
                self.agents.iter().enumerate().map(|(i, s)| {
                    SubagentStatusDot::new((id.clone(), SharedString::from(format!("dot-{i}"))), *s)
                }),
            ))
            // `<b>` with no weight rule of its own: the render draws it regular.
            .child(div().child((self.count)(self.agents.len())))
            .child(
                div()
                    .text_size(rem(12.))
                    .text_color(color(ColorRole::TextSecondary))
                    .child((self.running)(running)),
            )
            .child(div().flex_1())
            .child(
                Button::new((id.clone(), "all"), self.view_all.clone())
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm)
                    .icon(CarbonIcon::ChevronRight)
                    .when_some(view, |b, f| b.on_click(move |_, w, cx| f(w, cx))),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    #[test]
    fn dots_follow_the_status_vocabulary() {
        crate::theme::tests_set(CarbonTheme::White);
        assert_eq!(
            SubagentStatus::Running.dot().0,
            color(ColorRole::Interactive)
        );
        assert_eq!(
            SubagentStatus::Waiting.dot().0,
            color(ColorRole::SupportWarning)
        );
        let (fill, ring) = SubagentStatus::Idle.dot();
        assert_eq!(fill.a, 0.);
        assert_eq!(ring, Some(color(ColorRole::IconSecondary)));
        assert_eq!(SubagentStatus::Idle.label(), "Idle · resumable");
    }
}
