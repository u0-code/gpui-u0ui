//! WorkflowRow: one workflow definition in a list (`WorkflowRow` in
//! `src/ghost/admin.jsx`, `.g-wfrow` in `src/ghost.css`): the `Flow` glyph,
//! name, a meta line (description · step count · trigger), the last run and
//! a tertiary small "Start" button. A running instance is T3's
//! `WorkflowRunRow`, not this row.
//!
//! ```ignore
//! WorkflowRow::new("wf-release", "Release")
//!     .description("Tests, notes, tag")
//!     .steps(4)
//!     .trigger("Manual")
//!     .last_run("2d ago")
//!     .on_start(|_, _| {})
//! ```
use super::row;
use crate::{
    components::{Button, ButtonSize, ButtonVariant},
    icons::CarbonIcon,
    theme::color,
    tokens::ColorRole,
};
use gpui_kit::{
    App, ElementId, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
    prelude::FluentBuilder as _,
};
use std::rc::Rc;

/// The row's user-facing strings (the reference's English defaults).
#[derive(Clone)]
pub struct WorkflowRowTexts {
    /// The step count: `"{n} steps"`.
    pub steps: Rc<dyn Fn(usize) -> SharedString>,
    /// The button: `"Start"`.
    pub start: SharedString,
}

impl Default for WorkflowRowTexts {
    fn default() -> Self {
        Self {
            steps: Rc::new(|n| format!("{n} steps").into()),
            start: "Start".into(),
        }
    }
}

type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

/// A workflow definition row. See the module docs.
#[derive(IntoElement)]
pub struct WorkflowRow {
    id: ElementId,
    name: SharedString,
    description: Option<SharedString>,
    steps: Option<usize>,
    trigger: Option<SharedString>,
    last_run: Option<SharedString>,
    texts: WorkflowRowTexts,
    on_start: Option<Handler>,
}

impl WorkflowRow {
    /// A row for workflow `name` (`name`).
    pub fn new(id: impl Into<ElementId>, name: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            steps: None,
            trigger: None,
            last_run: None,
            texts: WorkflowRowTexts::default(),
            on_start: None,
        }
    }

    /// What the workflow does (`description`).
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// The number of steps (`steps`), shown through [`WorkflowRowTexts::steps`].
    pub fn steps(mut self, steps: usize) -> Self {
        self.steps = Some(steps);
        self
    }

    /// How it starts (`trigger`): "Manual", "On webhook", a schedule.
    pub fn trigger(mut self, trigger: impl Into<SharedString>) -> Self {
        self.trigger = Some(trigger.into());
        self
    }

    /// When it last ran (`lastRun`), e.g. "2d ago".
    pub fn last_run(mut self, last_run: impl Into<SharedString>) -> Self {
        self.last_run = Some(last_run.into());
        self
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: WorkflowRowTexts) -> Self {
        self.texts = texts;
        self
    }

    /// "Start" pressed (`onStart`).
    pub fn on_start(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_start = Some(Rc::new(f));
        self
    }

    /// The meta line's runs in order: description, step count, trigger
    /// (`{description}·{steps} steps·{trigger}`; a run left unset is skipped
    /// with its dot).
    pub fn meta_runs(&self) -> Vec<SharedString> {
        [
            self.description.clone(),
            self.steps.map(|n| (self.texts.steps)(n)),
            self.trigger.clone(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

impl RenderOnce for WorkflowRow {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let meta = row::with_dots(self.meta_runs().into_iter().map(row::meta_text));
        let on_start = self.on_start.clone();
        row::frame()
            .child(row::lead_icon(CarbonIcon::Flow))
            .child(row::main(
                self.name.clone(),
                color(ColorRole::TextPrimary),
                meta,
            ))
            .when_some(self.last_run.clone(), |this, last| {
                // `.g-dim { color: var(--text-secondary) }` at the row's 14px.
                this.child(
                    div()
                        .flex_none()
                        .whitespace_nowrap()
                        .text_color(color(ColorRole::TextSecondary))
                        .child(last),
                )
            })
            .child(
                // `<Cb.Button kind="tertiary" size="sm" renderIcon={I.Play}>`;
                // `.g-wfrow .cds--btn { flex: none }`.
                div().flex_none().child(
                    Button::new((self.id.clone(), "start"), self.texts.start.clone())
                        .variant(ButtonVariant::Tertiary)
                        .size(ButtonSize::Sm)
                        .icon(CarbonIcon::Play)
                        .when_some(on_start, |b, f| b.on_click(move |_, w, cx| f(w, cx))),
                ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meta_line_follows_the_reference_order() {
        let row = WorkflowRow::new("w", "Release")
            .description("Tests, notes, tag")
            .steps(4)
            .trigger("Manual");
        assert_eq!(
            row.meta_runs(),
            vec![
                SharedString::from("Tests, notes, tag"),
                "4 steps".into(),
                "Manual".into()
            ]
        );
    }

    #[test]
    fn unset_runs_are_skipped() {
        let row = WorkflowRow::new("w", "Triage").trigger("On webhook");
        assert_eq!(row.meta_runs(), vec![SharedString::from("On webhook")]);
        let row = WorkflowRow::new("w", "Triage");
        assert!(row.meta_runs().is_empty());
    }

    #[test]
    fn step_count_comes_from_the_texts() {
        let row = WorkflowRow::new("w", "Release")
            .steps(2)
            .texts(WorkflowRowTexts {
                steps: Rc::new(|n| format!("{n} 步").into()),
                start: "开始".into(),
            });
        assert_eq!(row.meta_runs(), vec![SharedString::from("2 步")]);
    }
}
