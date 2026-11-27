//! TaskRow: one scheduled task in a list (`TaskRow` in `src/ghost/admin.jsx`,
//! `.g-task` in `src/ghost.css`): the `Time` glyph, name, a meta line
//! (schedule · agent · project), the last run with its result and the next
//! run, "Run now" and an on/off switch.
//!
//! The switch keeps its own state seeded from [`TaskRow::enabled`] (the
//! reference's `useState(enabled)`), and follows `enabled` again whenever
//! the caller changes it. A paused task greys its name and shows "Paused"
//! instead of the next run.
//!
//! ```ignore
//! TaskRow::new("t-triage", "Morning triage")
//!     .schedule("Weekdays at 09:00")
//!     .agent("Claude")
//!     .project("carbon-ui")
//!     .last_run("today 09:00")
//!     .last_status(TaskStatus::Ok)
//!     .next_run("tomorrow 09:00")
//!     .on_toggle(|on, _, _| {})
//!     .on_run(|_, _| {})
//! ```
use super::row;
use crate::{
    components::{Button, ButtonSize, ButtonVariant, Switch, SwitchSize},
    ghost::primitives::parts,
    icons::{CarbonIcon, Icon},
    t3::{
        IdentitySize, ProjectMonogram, ProviderIcon,
        base::visually_hidden,
        craft::{rem, sans},
    },
    theme::color,
    tokens::{self, ColorRole},
};
use gpui_kit::{
    AnyElement, App, ClickEvent, CursorStyle, ElementId, Hsla, IntoElement, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _, px,
};
use std::rc::Rc;

/// The last run's result (`lastStatus`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TaskStatus {
    /// `ok`: succeeded.
    Ok,
    /// `failed`.
    Failed,
    /// `running`: currently in progress.
    Running,
}

impl TaskStatus {
    /// Every status, in `index.d.ts` order plus `running`.
    pub const ALL: [Self; 3] = [Self::Ok, Self::Failed, Self::Running];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Failed => "failed",
            Self::Running => "running",
        }
    }

    /// `I.CheckmarkFilled` / `I.ErrorFilled` / `I.InProgress`.
    pub const fn icon(self) -> CarbonIcon {
        match self {
            Self::Ok => CarbonIcon::CheckmarkFilled,
            Self::Failed => CarbonIcon::ErrorFilled,
            Self::Running => CarbonIcon::InProgress,
        }
    }

    /// `svg.t3-okicon { fill: var(--support-success) }`, `svg.t3-err { fill:
    /// var(--support-error) }` (`src/ghost.css`), `interactive` when running.
    pub fn ink(self) -> Hsla {
        color(match self {
            Self::Ok => ColorRole::SupportSuccess,
            Self::Failed => ColorRole::SupportError,
            Self::Running => ColorRole::Interactive,
        })
    }
}

/// The row's user-facing strings (the reference's English defaults).
#[derive(Clone)]
pub struct TaskRowTexts {
    /// The last run: `"Last {lastRun}"`.
    pub last: Rc<dyn Fn(&str) -> SharedString>,
    /// The next run: `"Next {nextRun}"`.
    pub next: Rc<dyn Fn(&str) -> SharedString>,
    /// Instead of the next run while off: `"Paused"`.
    pub paused: SharedString,
    /// Hidden text after the `ok` glyph: `"Succeeded"`.
    pub succeeded: SharedString,
    /// Hidden text after the `failed` glyph: `"Failed"`.
    pub failed: SharedString,
    /// Hidden text after the `running` glyph: `"Running"`.
    pub running: SharedString,
    /// The run button's label (tooltip): `"Run now"`.
    pub run_now: SharedString,
    /// The switch's accessible name: `"{name} enabled"`.
    pub enabled: Rc<dyn Fn(&str) -> SharedString>,
}

impl Default for TaskRowTexts {
    fn default() -> Self {
        Self {
            last: Rc::new(|t| format!("Last {t}").into()),
            next: Rc::new(|t| format!("Next {t}").into()),
            paused: "Paused".into(),
            succeeded: "Succeeded".into(),
            failed: "Failed".into(),
            running: "Running".into(),
            run_now: "Run now".into(),
            enabled: Rc::new(|name| format!("{name} enabled").into()),
        }
    }
}

/// The switch state: the `enabled` prop it was last seeded from and the
/// current position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EnabledState {
    prop: bool,
    on: bool,
}

impl EnabledState {
    /// Seeded from the `enabled` prop.
    pub const fn new(enabled: bool) -> Self {
        Self {
            prop: enabled,
            on: enabled,
        }
    }

    /// Re-seed when the caller's `enabled` changed since the last render;
    /// otherwise keep the user's position.
    pub fn sync(&mut self, enabled: bool) {
        if self.prop != enabled {
            *self = Self::new(enabled);
        }
    }

    /// The switch was flipped.
    pub fn set(&mut self, on: bool) {
        self.on = on;
    }

    /// On?
    pub const fn on(self) -> bool {
        self.on
    }
}

/// The name's ink: `text-primary`, `text-secondary` while off
/// (`.g-task--off .g-task-main b { color: var(--text-secondary) }`).
pub fn name_ink(on: bool) -> Hsla {
    color(if on {
        ColorRole::TextPrimary
    } else {
        ColorRole::TextSecondary
    })
}

/// Row surface and selection bar colors: `(background, left_bar)`.
pub fn row_look(selected: bool, hovered: bool) -> (Hsla, Option<Hsla>) {
    let bg = if selected {
        color(ColorRole::LayerSelected01)
    } else if hovered {
        color(ColorRole::LayerHover01)
    } else {
        color(ColorRole::Layer01)
    };
    let bar = selected.then(|| color(ColorRole::BorderInteractive));
    (bg, bar)
}

type Handler = Rc<dyn Fn(&mut Window, &mut App)>;
type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;
type ToggleHandler = Rc<dyn Fn(bool, &mut Window, &mut App)>;

/// A scheduled-task row. See the module docs.
#[derive(IntoElement)]
pub struct TaskRow {
    id: ElementId,
    name: SharedString,
    icon: CarbonIcon,
    schedule: Option<SharedString>,
    agent: Option<SharedString>,
    project: Option<SharedString>,
    show_monograms: bool,
    last_run: Option<SharedString>,
    last_status: Option<TaskStatus>,
    next_run: Option<SharedString>,
    enabled: bool,
    selected: bool,
    texts: TaskRowTexts,
    on_click: Option<ClickHandler>,
    on_toggle: Option<ToggleHandler>,
    on_run: Option<Handler>,
}

impl TaskRow {
    /// A row for task `name` (`name`), enabled.
    pub fn new(id: impl Into<ElementId>, name: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            icon: CarbonIcon::Time,
            schedule: None,
            agent: None,
            project: None,
            show_monograms: true,
            last_run: None,
            last_status: None,
            next_run: None,
            enabled: true,
            selected: false,
            texts: TaskRowTexts::default(),
            on_click: None,
            on_toggle: None,
            on_run: None,
        }
    }

    /// Leading icon (default [`CarbonIcon::Time`]).
    pub fn icon(mut self, icon: CarbonIcon) -> Self {
        self.icon = icon;
        self
    }

    /// Show the 14px agent/project monograms in the metadata line (default `true`).
    pub fn show_monograms(mut self, show: bool) -> Self {
        self.show_monograms = show;
        self
    }

    /// Highlight as the selected task (`layer-selected-01` + 3px `border-interactive` start bar).
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Row body pressed (does not intercept the Run button or Switch).
    pub fn on_click(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }

    /// When it runs, in words (`schedule`, e.g. `ScheduleEditor`'s summary).
    pub fn schedule(mut self, schedule: impl Into<SharedString>) -> Self {
        self.schedule = Some(schedule.into());
        self
    }

    /// The agent that runs it (`agent`), drawn with its `ProviderIcon`.
    pub fn agent(mut self, agent: impl Into<SharedString>) -> Self {
        self.agent = Some(agent.into());
        self
    }

    /// The project (`project`), drawn with its `ProjectMonogram`.
    pub fn project(mut self, project: impl Into<SharedString>) -> Self {
        self.project = Some(project.into());
        self
    }

    /// When it last ran (`lastRun`).
    pub fn last_run(mut self, last_run: impl Into<SharedString>) -> Self {
        self.last_run = Some(last_run.into());
        self
    }

    /// How the last run ended (`lastStatus`); no glyph without one.
    pub fn last_status(mut self, status: TaskStatus) -> Self {
        self.last_status = Some(status);
        self
    }

    /// When it runs next (`nextRun`).
    pub fn next_run(mut self, next_run: impl Into<SharedString>) -> Self {
        self.next_run = Some(next_run.into());
        self
    }

    /// Whether the task is on (`enabled`, default `true`).
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: TaskRowTexts) -> Self {
        self.texts = texts;
        self
    }

    /// The switch was flipped (`onToggle`).
    pub fn on_toggle(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Rc::new(f));
        self
    }

    /// "Run now" pressed (`onRun`).
    pub fn on_run(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_run = Some(Rc::new(f));
        self
    }

    /// The second line of the runs column: `on ? "Next {nextRun}" :
    /// "Paused"` (nothing while on without a next run).
    pub fn upcoming(&self, on: bool) -> Option<SharedString> {
        if on {
            self.next_run.as_deref().map(|t| (self.texts.next)(t))
        } else {
            Some(self.texts.paused.clone())
        }
    }

    /// The meta line: schedule, then agent and project with their marks.
    fn meta(&self) -> Vec<AnyElement> {
        let mut runs: Vec<AnyElement> = Vec::new();
        if let Some(s) = &self.schedule {
            runs.push(row::meta_text(s.clone()));
        }
        // `<ProviderIcon name={agent} size="xs" />{agent}` and
        // `<ProjectMonogram name={project} size="xs" />{project}`: mark and
        // name are siblings in the meta flex, so the 8px column gap parts them.
        if let Some(agent) = &self.agent {
            if self.show_monograms {
                runs.push(mark_and_name(
                    ProviderIcon::new(agent.clone())
                        .id((self.id.clone(), "agent"))
                        .size(IdentitySize::Xs)
                        .into_any_element(),
                    agent.clone(),
                ));
            } else {
                runs.push(row::meta_text(agent.clone()));
            }
        }
        if let Some(project) = &self.project {
            if self.show_monograms {
                runs.push(mark_and_name(
                    ProjectMonogram::new(project.clone())
                        .size(IdentitySize::Xs)
                        .into_any_element(),
                    project.clone(),
                ));
            } else {
                runs.push(row::meta_text(project.clone()));
            }
        }
        row::with_dots(runs)
    }
}

/// A mark and its name, `spacing-03` apart (the meta line's column gap).
fn mark_and_name(mark: AnyElement, name: SharedString) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap(tokens::SPACING_03)
        .child(mark)
        .child(row::meta_text(name))
        .into_any_element()
}

impl RenderOnce for TaskRow {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let enabled = self.enabled;
        let store = window.use_keyed_state((self.id.clone(), "enabled"), cx, |_, _| {
            EnabledState::new(enabled)
        });
        let on = store.update(cx, |s, _| {
            s.sync(enabled);
            s.on()
        });
        let meta = self.meta();
        let upcoming = self.upcoming(on);
        let status = self.last_status.map(|s| {
            let hidden = match s {
                TaskStatus::Ok => self.texts.succeeded.clone(),
                TaskStatus::Failed => self.texts.failed.clone(),
                TaskStatus::Running => self.texts.running.clone(),
            };
            div()
                .relative()
                .flex()
                .flex_none()
                .child(Icon::carbon(s.icon()).size_length(rem(12.)).color(s.ink()))
                .child(visually_hidden(hidden))
        });
        let last = self.last_run.as_deref().map(|t| (self.texts.last)(t));
        let on_run = self.on_run.clone();
        let on_toggle = self.on_toggle.clone();
        let on_click = self.on_click.clone();
        let switch_label = (self.texts.enabled)(&self.name);
        let hovered_state = window.use_keyed_state((self.id.clone(), "hover"), cx, |_, _| false);
        let is_hovered = on_click.is_some() && *hovered_state.read(cx);
        let (bg, bar) = row_look(self.selected, is_hovered);

        // `.g-task-runs { display: flex; flex-direction: column;
        // align-items: flex-end; gap: 2px; flex: none; font-size:
        // 0.75rem; color: var(--text-secondary) }`; each span
        // `inline-flex; align-items: center; gap: var(--spacing-02)`.
        let runs_col = sans(div(), 12., 400)
            .flex()
            .flex_col()
            .items_end()
            .flex_none()
            .gap(px(2.))
            .text_color(color(ColorRole::TextSecondary))
            .when(status.is_some() || last.is_some(), |this| {
                this.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(tokens::SPACING_02)
                        .whitespace_nowrap()
                        .children(status)
                        .children(last),
                )
            })
            .when_some(upcoming, |this, t| {
                this.child(div().whitespace_nowrap().child(t))
            });

        let run_button = div().flex_none().child(
            Button::new((self.id.clone(), "run"), self.texts.run_now.clone())
                .icon(CarbonIcon::Play)
                .icon_only(true)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .when_some(on_run, |b, f| b.on_click(move |_, w, cx| f(w, cx))),
        );

        let switch = div().flex_none().child(
            Switch::new((self.id.clone(), "toggle"))
                .size(SwitchSize::Sm)
                .label(switch_label)
                .hide_label(true)
                .checked(on)
                .on_checked_change(move |v, w, cx| {
                    store.update(cx, |s, cx| {
                        s.set(v);
                        cx.notify();
                    });
                    if let Some(f) = &on_toggle {
                        f(v, w, cx);
                    }
                }),
        );

        let accent_bar = bar.map(|bar| {
            div()
                .absolute()
                .top_0()
                .bottom_0()
                .left_0()
                .w(px(3.))
                .bg(bar)
        });

        match on_click {
            Some(f) => {
                let focus = parts::keyed_focus((self.id.clone(), "body-focus"), window, cx);
                let ring = parts::focus_visible(&focus, window);
                let body = gpui_kit::base::Button::new((self.id.clone(), "body"))
                    .track_focus(&focus)
                    .accessibility_label(self.name.clone())
                    .relative()
                    .flex()
                    .items_center()
                    .justify_start()
                    .gap(tokens::SPACING_04)
                    .flex_1()
                    .min_w_0()
                    .self_stretch()
                    .pl(tokens::SPACING_05)
                    .pt(tokens::SPACING_02)
                    .pb(tokens::SPACING_02)
                    .text_left()
                    .cursor(CursorStyle::PointingHand)
                    .on_hover(move |h, _, cx| {
                        hovered_state.update(cx, |v, cx| {
                            if *v != *h {
                                *v = *h;
                                cx.notify();
                            }
                        })
                    })
                    .on_click(move |e, w, cx| f(e, w, cx))
                    .child(row::lead_icon(self.icon))
                    .child(row::main(self.name.clone(), name_ink(on), meta))
                    .child(runs_col)
                    .when(ring, |b| b.child(parts::focus_ring()));

                row::frame()
                    .pl(px(0.))
                    .pt(px(0.))
                    .pb(px(0.))
                    .bg(bg)
                    .child(body)
                    .child(
                        div()
                            .flex()
                            .flex_none()
                            .items_center()
                            .gap(tokens::SPACING_04)
                            .pt(tokens::SPACING_02)
                            .pb(tokens::SPACING_02)
                            .child(run_button)
                            .child(switch),
                    )
                    .children(accent_bar)
            }
            None => row::frame()
                .bg(bg)
                .child(row::lead_icon(self.icon))
                .child(row::main(self.name.clone(), name_ink(on), meta))
                .child(runs_col)
                .child(run_button)
                .child(switch)
                .children(accent_bar),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CarbonTheme;

    #[test]
    fn status_glyphs_and_inks() {
        crate::theme::tests_set(CarbonTheme::G100);
        assert_eq!(TaskStatus::Ok.icon(), CarbonIcon::CheckmarkFilled);
        assert_eq!(TaskStatus::Failed.icon(), CarbonIcon::ErrorFilled);
        assert_eq!(TaskStatus::Running.icon(), CarbonIcon::InProgress);
        assert_eq!(TaskStatus::Ok.ink(), color(ColorRole::SupportSuccess));
        assert_eq!(TaskStatus::Failed.ink(), color(ColorRole::SupportError));
        assert_eq!(TaskStatus::Running.ink(), color(ColorRole::Interactive));
        assert_eq!(
            TaskStatus::ALL.map(TaskStatus::name),
            ["ok", "failed", "running"]
        );
    }

    #[test]
    fn selection_and_hover_colors_follow_carbon_layers() {
        crate::theme::tests_set(CarbonTheme::White);
        assert_eq!(row_look(false, false), (color(ColorRole::Layer01), None));
        assert_eq!(
            row_look(false, true),
            (color(ColorRole::LayerHover01), None)
        );
        assert_eq!(
            row_look(true, false),
            (
                color(ColorRole::LayerSelected01),
                Some(color(ColorRole::BorderInteractive)),
            )
        );
        let row = TaskRow::new("t", "Hook")
            .icon(CarbonIcon::Flow)
            .agent("Claude")
            .project("u0")
            .show_monograms(false)
            .selected(true);
        assert_eq!(row.icon, CarbonIcon::Flow);
        assert!(!row.show_monograms);
        assert!(row.selected);
        // 2 runs (`Claude` · `u0`) + 1 dot = 3 elements.
        assert_eq!(row.meta().len(), 3);
    }

    #[test]
    fn paused_greys_the_name() {
        crate::theme::tests_set(CarbonTheme::White);
        assert_eq!(name_ink(true), color(ColorRole::TextPrimary));
        assert_eq!(name_ink(false), color(ColorRole::TextSecondary));
    }

    #[test]
    fn upcoming_is_next_run_or_paused() {
        let row = TaskRow::new("t", "Morning triage").next_run("tomorrow 09:00");
        assert_eq!(row.upcoming(true), Some("Next tomorrow 09:00".into()));
        assert_eq!(row.upcoming(false), Some("Paused".into()));
        let row = TaskRow::new("t", "Release notes draft");
        assert_eq!(row.upcoming(true), None);
        assert_eq!(row.upcoming(false), Some("Paused".into()));
    }

    #[test]
    fn switch_follows_the_prop_but_keeps_the_users_flip() {
        let mut s = EnabledState::new(true);
        s.set(false);
        s.sync(true);
        assert!(!s.on(), "same prop: the user's flip stays");
        s.sync(false);
        assert!(!s.on());
        s.sync(true);
        assert!(s.on(), "prop changed: re-seeded");
    }
}
