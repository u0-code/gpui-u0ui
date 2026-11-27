//! [`WorktreeSetupCard`] (`.t3-wt`): progress of a new worktree. The head
//! is a button that folds the body (open unless `ready`); the body lists the
//! branch and path, the setup stages with their status icons, the output
//! tail on the terminal ground, and Cancel / Open terminal / Work locally.
//! A running head shimmers; the rail follows the state (ready green,
//! script-failed yellow, failed red).
use super::card;
use super::work::shimmer;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::t3::base::{Spin, fmt_elapsed, warning_glyph};
use crate::t3::press::{self, Handler, rem};
use crate::{
    icons::CarbonIcon,
    theme::color,
    tokens::{ColorRole, FontFamilyKey},
    typography::font,
};
use gpui_kit::{
    AnyElement, App, ElementId, FontWeight, Hsla, InteractiveElement as _, IntoElement,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _, px,
};
use std::rc::Rc;

/// `WorktreeSetupCard` `state`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum WorktreeState {
    /// Setting up.
    #[default]
    Running,
    /// Ready.
    Ready,
    /// Ready, but the setup script failed.
    ScriptFailed,
    /// Failed.
    Failed,
    /// Cancelled.
    Cancelled,
}

impl WorktreeState {
    /// Every state.
    pub const ALL: [WorktreeState; 5] = [
        Self::Running,
        Self::Ready,
        Self::ScriptFailed,
        Self::Failed,
        Self::Cancelled,
    ];

    /// The reference name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Ready => "ready",
            Self::ScriptFailed => "script-failed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    /// The default head.
    pub const fn head(self) -> &'static str {
        match self {
            Self::Running => "Setting up worktree…",
            Self::Ready => "Worktree ready",
            Self::ScriptFailed => "Worktree ready, setup script failed",
            Self::Failed => "Worktree setup failed",
            Self::Cancelled => "Worktree setup cancelled",
        }
    }

    /// (rail, head icon color): `.t3-wt--failed / --script-failed / --ready`
    /// rails; `.t3-wt-head > svg:first-child` success, warning, error, or
    /// `icon-secondary` when cancelled.
    pub fn look(self) -> (Option<Hsla>, Hsla) {
        match self {
            Self::Running => (None, color(ColorRole::Interactive)),
            Self::Ready => (
                Some(color(ColorRole::SupportSuccess)),
                color(ColorRole::SupportSuccess),
            ),
            Self::ScriptFailed => (
                Some(color(ColorRole::SupportWarning)),
                color(ColorRole::SupportWarning),
            ),
            Self::Failed => (
                Some(color(ColorRole::SupportError)),
                color(ColorRole::SupportError),
            ),
            Self::Cancelled => (None, color(ColorRole::IconSecondary)),
        }
    }
}

/// A setup stage's status (`STAGE_ICON`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum StageStatus {
    /// Done.
    Done,
    /// Running.
    Running,
    /// Not started.
    #[default]
    Pending,
    /// Failed.
    Failed,
    /// Skipped.
    Skipped,
}

impl StageStatus {
    /// (glyph, glyph color, text color); `None` glyph = spinner.
    pub fn look(self) -> (Option<CarbonIcon>, Hsla, Hsla) {
        let ink = color(ColorRole::TextPrimary);
        match self {
            // `.t3-stage--done svg { fill: support-success }`.
            Self::Done => (
                Some(CarbonIcon::CheckmarkFilled),
                color(ColorRole::SupportSuccess),
                ink,
            ),
            Self::Running => (None, color(ColorRole::Interactive), ink),
            // `.t3-stage--pending { color: text-secondary }`, `svg { fill: icon-secondary }`.
            Self::Pending => (
                Some(CarbonIcon::CircleDash),
                color(ColorRole::IconSecondary),
                color(ColorRole::TextSecondary),
            ),
            Self::Failed => (
                Some(CarbonIcon::ErrorFilled),
                color(ColorRole::SupportError),
                ink,
            ),
            Self::Skipped => (
                Some(CarbonIcon::Subtract),
                color(ColorRole::IconSecondary),
                ink,
            ),
        }
    }
}

/// One stage (`{ label, status, elapsed }`).
#[derive(Clone, Debug, PartialEq)]
pub struct WorktreeStage {
    /// `label`.
    pub label: SharedString,
    /// `status`.
    pub status: StageStatus,
    /// `elapsed` seconds.
    pub elapsed: Option<u64>,
}

impl WorktreeStage {
    /// A stage.
    pub fn new(label: impl Into<SharedString>, status: StageStatus) -> Self {
        Self {
            label: label.into(),
            status,
            elapsed: None,
        }
    }

    /// `elapsed`.
    pub fn elapsed(mut self, seconds: u64) -> Self {
        self.elapsed = Some(seconds);
        self
    }
}

/// `WorktreeSetupCard`'s strings.
#[derive(Clone)]
pub struct WorktreeTexts {
    /// The head per state.
    pub head: Rc<dyn Fn(WorktreeState) -> SharedString>,
    /// Branch.
    pub branch: SharedString,
    /// Path.
    pub path: SharedString,
    /// Cancel.
    pub cancel: SharedString,
    /// Open terminal.
    pub open_terminal: SharedString,
    /// Work locally.
    pub work_locally: SharedString,
}

impl Default for WorktreeTexts {
    fn default() -> Self {
        Self {
            head: Rc::new(|s| s.head().into()),
            branch: "Branch".into(),
            path: "Path".into(),
            cancel: "Cancel".into(),
            open_terminal: "Open terminal".into(),
            work_locally: "Work locally".into(),
        }
    }
}

/// The worktree setup card. See the module docs.
#[derive(IntoElement)]
pub struct WorktreeSetupCard {
    id: ElementId,
    state: WorktreeState,
    branch: SharedString,
    path: SharedString,
    stages: Vec<WorktreeStage>,
    output: Vec<SharedString>,
    elapsed: u64,
    texts: WorktreeTexts,
    on_cancel: Option<Handler>,
    on_open_terminal: Option<Handler>,
    on_work_locally: Option<Handler>,
}

impl WorktreeSetupCard {
    /// A `state` card for `branch` at `path`.
    pub fn new(
        id: impl Into<ElementId>,
        state: WorktreeState,
        branch: impl Into<SharedString>,
        path: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            state,
            branch: branch.into(),
            path: path.into(),
            stages: Vec::new(),
            output: Vec::new(),
            elapsed: 0,
            texts: WorktreeTexts::default(),
            on_cancel: None,
            on_open_terminal: None,
            on_work_locally: None,
        }
    }

    /// `stages`.
    pub fn stages(mut self, stages: Vec<WorktreeStage>) -> Self {
        self.stages = stages;
        self
    }

    /// `output`: the last lines.
    pub fn output(mut self, lines: Vec<SharedString>) -> Self {
        self.output = lines;
        self
    }

    /// `elapsed` seconds.
    pub fn elapsed(mut self, seconds: u64) -> Self {
        self.elapsed = seconds;
        self
    }

    /// Replace the strings.
    pub fn texts(mut self, texts: WorktreeTexts) -> Self {
        self.texts = texts;
        self
    }

    /// Cancel (running).
    pub fn on_cancel(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_cancel = Some(Rc::new(f));
        self
    }

    /// Open terminal (finished).
    pub fn on_open_terminal(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_open_terminal = Some(Rc::new(f));
        self
    }

    /// Work locally (failed).
    pub fn on_work_locally(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_work_locally = Some(Rc::new(f));
        self
    }
}

fn ghost(id: ElementId, label: SharedString, f: Option<Handler>) -> Button {
    Button::new(id, label)
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::Sm)
        .when_some(f, |b, f| b.on_click(move |_, w, cx| f(w, cx)))
}

impl RenderOnce for WorktreeSetupCard {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let state = self.state;
        let open_state = window.use_keyed_state((id.clone(), "open"), cx, move |_, _| {
            state != WorktreeState::Ready
        });
        let open = *open_state.read(cx);
        let (focus, ring) = press::focus((id.clone(), "focus"), window, cx);
        let (rail, icon_ink) = state.look();
        let texts = self.texts.clone();
        let head_text = (texts.head)(state);
        let icon: AnyElement = match state {
            WorktreeState::Running => Spin::new((id.clone(), "spin")).into_any_element(),
            WorktreeState::Ready => {
                press::icon(CarbonIcon::CheckmarkFilled, 16., icon_ink).into_any_element()
            }
            WorktreeState::Cancelled => {
                press::icon(CarbonIcon::Subtract, 16., icon_ink).into_any_element()
            }
            WorktreeState::ScriptFailed => warning_glyph(16.).into_any_element(),
            WorktreeState::Failed => {
                press::icon(CarbonIcon::WarningAltFilled, 16., icon_ink).into_any_element()
            }
        };
        let title: AnyElement = if state == WorktreeState::Running {
            div()
                .font_weight(FontWeight::SEMIBOLD)
                .child(shimmer((id.clone(), "shimmer"), head_text.clone(), 600, cx))
                .into_any_element()
        } else {
            card::head_title(head_text.clone()).into_any_element()
        };
        let head_state = open_state.clone();
        // `button.t3-card-head { width: 100% }`, `:focus-visible { outline: 2px solid focus; outline-offset: 2px }`.
        let head = gpui_kit::base::Button::new((id.clone(), "head"))
            .track_focus(&focus)
            .accessibility_label(head_text)
            .relative()
            .flex()
            .flex_row()
            .items_center()
            .justify_start()
            .w_full()
            .gap(rem(8.))
            .text_size(rem(14.))
            .line_height(rem(18.))
            .cursor_pointer()
            .on_click(move |_, _, cx| {
                head_state.update(cx, |o, cx| {
                    *o = !*o;
                    cx.notify();
                })
            })
            .child(icon)
            .child(title)
            .child(card::sub(fmt_elapsed(self.elapsed)))
            .child(card::grow())
            // `.t3-wt-head > svg:last-child { fill: icon-secondary }`, `.t3-rot` when closed.
            .child(
                press::icon(
                    CarbonIcon::ChevronDown,
                    16.,
                    color(ColorRole::IconSecondary),
                )
                .rotate(gpui_kit::radians(if open {
                    0.
                } else {
                    -std::f32::consts::FRAC_PI_2
                })),
            )
            .when(ring, |this| {
                this.child(
                    div()
                        .absolute()
                        .top(px(-4.))
                        .left(px(-4.))
                        .right(px(-4.))
                        .bottom(px(-4.))
                        .border_2()
                        .border_color(color(ColorRole::Focus)),
                )
            });
        let body = open.then(|| {
            let secondary = color(ColorRole::TextSecondary);
            // `.t3-kv { display: grid; grid-template-columns: max-content 1fr; gap: spacing-02 spacing-05;
            // font-size: 0.875rem }`, `dt { color: text-secondary }`.
            let kv = div()
                .flex()
                .flex_row()
                .gap(rem(16.))
                .text_size(rem(14.))
                .line_height(rem(18.))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_none()
                        .gap(rem(4.))
                        .text_color(secondary)
                        .child(texts.branch.clone())
                        .child(texts.path.clone()),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .min_w_0()
                        .gap(rem(4.))
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(rem(4.))
                                .min_w_0()
                                .child(press::icon(
                                    CarbonIcon::Branch,
                                    12.,
                                    color(ColorRole::IconSecondary),
                                ))
                                .child(
                                    div()
                                        .min_w_0()
                                        .overflow_hidden()
                                        .whitespace_nowrap()
                                        .text_ellipsis()
                                        .child(self.branch.clone()),
                                ),
                        )
                        // `.t3-mono-text { font-family: mono; font-size: 0.75rem }`.
                        .child(
                            div()
                                .min_w_0()
                                .overflow_hidden()
                                .whitespace_nowrap()
                                .text_ellipsis()
                                .font(font(FontFamilyKey::Mono, 400))
                                .text_size(rem(12.))
                                .child(self.path.clone()),
                        ),
                );
            // `.t3-stages { display: flex; flex-direction: column; gap: spacing-03; font-size: 0.875rem }`.
            let stages = div()
                .flex()
                .flex_col()
                .gap(rem(8.))
                .text_size(rem(14.))
                .line_height(rem(18.))
                .children(self.stages.iter().enumerate().map(|(i, s)| {
                    let (glyph, glyph_ink, ink) = s.status.look();
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(rem(8.))
                        .text_color(ink)
                        .child(match glyph {
                            Some(g) => press::icon(g, 16., glyph_ink).into_any_element(),
                            None => {
                                Spin::new((id.clone(), SharedString::from(format!("stage-{i}"))))
                                    .into_any_element()
                            }
                        })
                        .child(div().flex_1().min_w_0().child(s.label.clone()))
                        .when_some(s.elapsed, |this, e| this.child(card::sub(fmt_elapsed(e))))
                }));
            // `.t3-tail { max-height: 7rem; overflow: auto; padding: spacing-03 spacing-04;
            // background: terminal-background; color: terminal-text; font-family: mono;
            // font-size: 0.75rem; line-height: 1.33333 }`.
            let tail = (!self.output.is_empty()).then(|| {
                div()
                    .id((id.clone(), "tail"))
                    .max_h(rem(112.))
                    .overflow_y_scroll()
                    .py(rem(8.))
                    .px(rem(12.))
                    .bg(color(ColorRole::TerminalBackground))
                    .text_color(color(ColorRole::TerminalText))
                    .font(font(FontFamilyKey::Mono, 400))
                    .text_size(rem(12.))
                    .line_height(rem(16.))
                    .children(self.output.iter().cloned().map(|l| div().child(l)))
            });
            let actions = card::actions()
                .when(state == WorktreeState::Running, |this| {
                    this.child(ghost(
                        (id.clone(), "cancel").into(),
                        texts.cancel.clone(),
                        self.on_cancel.clone(),
                    ))
                })
                .when(state != WorktreeState::Running, |this| {
                    this.child(
                        ghost(
                            (id.clone(), "terminal").into(),
                            texts.open_terminal.clone(),
                            self.on_open_terminal.clone(),
                        )
                        .icon(CarbonIcon::Terminal),
                    )
                })
                .when(state == WorktreeState::Failed, |this| {
                    let f = self.on_work_locally.clone();
                    this.child(
                        Button::new((id.clone(), "local"), texts.work_locally.clone())
                            .variant(ButtonVariant::Secondary)
                            .size(ButtonSize::Sm)
                            .when_some(f, |b, f| b.on_click(move |_, w, cx| f(w, cx))),
                    )
                });
            vec![kv.into_any_element(), stages.into_any_element()]
                .into_iter()
                .chain(tail.map(|t| t.into_any_element()))
                .chain(std::iter::once(actions.into_any_element()))
                .collect::<Vec<_>>()
        });
        card::framed(rail)
            .child(head)
            .when_some(body, |this, body| this.children(body))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    #[test]
    fn rails_and_stage_glyphs_follow_the_css() {
        crate::theme::tests_set(CarbonTheme::White);
        assert_eq!(WorktreeState::Running.look().0, None);
        assert_eq!(
            WorktreeState::ScriptFailed.look().0,
            Some(color(ColorRole::SupportWarning))
        );
        assert_eq!(StageStatus::Running.look().0, None);
        assert_eq!(
            StageStatus::Pending.look().2,
            color(ColorRole::TextSecondary)
        );
        assert_eq!(WorktreeState::Failed.head(), "Worktree setup failed");
    }
}
