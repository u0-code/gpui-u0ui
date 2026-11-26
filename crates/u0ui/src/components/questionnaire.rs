//! Questionnaire: a step-by-step form of single-choice, multiple-choice and
//! free-text questions (`.cu-quiz*` in `custom.css`, `Questionnaire` in
//! `index.jsx`), composed of u0ui's [`Progress`] (small), [`RadioGroup`],
//! [`Checkbox`], [`Textarea`] and [`Button`].
//!
//! - **Behavior** (the reference's): one question at a time; Next refuses to
//!   advance past an unanswered required question and shows the error row;
//!   Skip appears on optional questions but the last; Previous is disabled on
//!   the first; the last step's primary button submits every answer. Answers
//!   persist when moving back and forth. Kit's `questionnaire` is a different
//!   survey widget (its own layout, controls and copy), so it is not used.
//! - **Layout**: `.cu-quiz { flex column; gap: spacing-05; max-width: 32rem;
//!   padding: spacing-05; background: layer-01 }`; a 12px `text-secondary`
//!   kicker (the title); the small progress bar labelled by
//!   [`QuestionnaireTexts::progress`]; the question in `heading-03` (20/28,
//!   `margin-top: spacing-03`) and an optional `body-01` description in
//!   `text-secondary`; the control; the error row (`margin-top: spacing-03`,
//!   4px gap, 16px `ErrorFilled` in `support-error`, 12px `text-error`);
//!   actions at the end, 1px apart: ghost `md` Previous / Skip, primary `md`
//!   Next / Submit. Controls sit on `layer-01`, so fields use layer two.
//! - **Copy**: every string comes from the caller ([`QuestionnaireTexts`]).
use super::button::{Button, ButtonSize, ButtonVariant};
use super::checkbox::Checkbox;
use super::form::rem;
use super::progress::{Progress, ProgressSize};
use super::radio_group::{RadioGroup, RadioGroupItem, RadioOrientation};
use super::textarea::{Textarea, TextareaState};
use crate::{
    icons::{CarbonIcon, Icon, IconSize},
    theme::{LayerLevel, color},
    tokens::{ColorRole, SPACING_02, SPACING_03, SPACING_05, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    AnyElement, App, AppContext as _, ElementId, Entity, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, Window, div, prelude::FluentBuilder as _, px,
};
use std::{collections::HashMap, rc::Rc};

/// `QuestionnaireItem.type`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum QuestionKind {
    /// One choice (radio group).
    Single,
    /// Any number of choices (checkboxes).
    Multiple,
    /// Free text (a 3-row textarea).
    Text,
}

/// One question (`QuestionnaireItem`).
#[derive(Clone, Debug)]
pub struct QuestionnaireItem {
    /// The answer key.
    pub name: SharedString,
    /// The question.
    pub title: SharedString,
    /// Extra text under the question.
    pub description: Option<SharedString>,
    /// Its kind.
    pub kind: QuestionKind,
    /// The choices (single / multiple).
    pub choices: Vec<SharedString>,
    /// Must be answered to continue.
    pub required: bool,
}

impl QuestionnaireItem {
    /// A question of `kind` keyed `name`.
    pub fn new(
        name: impl Into<SharedString>,
        title: impl Into<SharedString>,
        kind: QuestionKind,
    ) -> Self {
        Self {
            name: name.into(),
            title: title.into(),
            description: None,
            kind,
            choices: Vec::new(),
            required: false,
        }
    }

    /// The description.
    pub fn description(mut self, text: impl Into<SharedString>) -> Self {
        self.description = Some(text.into());
        self
    }

    /// The choices.
    pub fn choices<S: Into<SharedString>>(mut self, choices: impl IntoIterator<Item = S>) -> Self {
        self.choices = choices.into_iter().map(Into::into).collect();
        self
    }

    /// Mark it required.
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }
}

/// An answer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answer {
    /// The chosen value.
    Single(SharedString),
    /// The checked values, in the order checked.
    Multiple(Vec<SharedString>),
    /// The text.
    Text(SharedString),
}

impl Answer {
    /// Whether it counts as answered (`v == null || v === '' || !v.length`).
    pub fn is_answered(answer: Option<&Answer>) -> bool {
        match answer {
            None => false,
            Some(Answer::Single(v)) | Some(Answer::Text(v)) => !v.is_empty(),
            Some(Answer::Multiple(v)) => !v.is_empty(),
        }
    }
}

/// Every user-facing string (the reference hard-codes English ones).
#[derive(Clone)]
pub struct QuestionnaireTexts {
    /// The progress label for question `i` (1-based) of `n`, e.g. "Question 1 of 3".
    pub progress: Rc<dyn Fn(usize, usize) -> SharedString>,
    /// "Previous".
    pub previous: SharedString,
    /// "Skip".
    pub skip: SharedString,
    /// "Next".
    pub next: SharedString,
    /// "Submit".
    pub submit: SharedString,
    /// "Answer this question to continue."
    pub required: SharedString,
}

/// What a step does on the primary button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Advance {
    /// Show the required error.
    Refuse,
    /// Go to the next question.
    Next,
    /// Submit.
    Submit,
}

/// The primary button's effect on question `index` of `count`.
pub fn advance(index: usize, count: usize, required: bool, answered: bool) -> Advance {
    if required && !answered {
        Advance::Refuse
    } else if index + 1 >= count {
        Advance::Submit
    } else {
        Advance::Next
    }
}

/// The progress value (`((i + 1) / items.length) * 100`).
pub fn progress_value(index: usize, count: usize) -> f32 {
    (index + 1) as f32 / count.max(1) as f32 * 100.
}

/// Whether Skip shows (`!it.required && !last`).
pub fn shows_skip(index: usize, count: usize, required: bool) -> bool {
    !required && index + 1 < count
}

#[derive(Default)]
struct QuizState {
    index: usize,
    answers: HashMap<SharedString, Answer>,
    error: bool,
    texts: HashMap<SharedString, Entity<TextareaState>>,
}

type SubmitHandler = Rc<dyn Fn(&HashMap<SharedString, Answer>, &mut Window, &mut App)>;

/// The questionnaire. See the module docs.
#[derive(IntoElement)]
pub struct Questionnaire {
    id: ElementId,
    title: Option<SharedString>,
    items: Vec<QuestionnaireItem>,
    texts: QuestionnaireTexts,
    on_submit: Option<SubmitHandler>,
}

impl Questionnaire {
    /// A questionnaire over `items`, with the caller's copy.
    pub fn new(
        id: impl Into<ElementId>,
        items: Vec<QuestionnaireItem>,
        texts: QuestionnaireTexts,
    ) -> Self {
        Self {
            id: id.into(),
            title: None,
            items,
            texts,
            on_submit: None,
        }
    }

    /// The kicker above the progress bar.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Called with every answer on submit.
    pub fn on_submit(
        mut self,
        f: impl Fn(&HashMap<SharedString, Answer>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_submit = Some(Rc::new(f));
        self
    }
}

/// The answers with free-text fields read from their editors.
fn collect(state: &Entity<QuizState>, cx: &App) -> HashMap<SharedString, Answer> {
    let s = state.read(cx);
    let mut answers = s.answers.clone();
    for (name, editor) in &s.texts {
        let text = editor.read(cx).value();
        answers.insert(name.clone(), Answer::Text(text));
    }
    answers
}

impl RenderOnce for Questionnaire {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let count = self.items.len();
        let state = window.use_keyed_state((id.clone(), "quiz"), cx, |_, _| QuizState::default());
        let index = state.read(cx).index.min(count.saturating_sub(1));
        let Some(item) = self.items.get(index).cloned() else {
            return div().into_any_element();
        };
        let name = item.name.clone();

        // Free text lives in an editor kept per question.
        if item.kind == QuestionKind::Text && !state.read(cx).texts.contains_key(&name) {
            let editor = cx.new(|cx| TextareaState::new(window, cx));
            state.update(cx, |s, _| {
                s.texts.insert(name.clone(), editor);
            });
        }
        let answers = collect(&state, cx);
        let current = answers.get(&name).cloned();
        let answered = Answer::is_answered(current.as_ref());
        // The error clears once the question is answered (`set` clears `err`).
        let show_error = state.read(cx).error && !answered;

        let set_answer = {
            let state = state.clone();
            let name = name.clone();
            move |answer: Answer, cx: &mut App| {
                state.update(cx, |s, cx| {
                    s.error = false;
                    s.answers.insert(name.clone(), answer);
                    cx.notify();
                })
            }
        };

        let control: AnyElement = match item.kind {
            QuestionKind::Single => {
                let set_answer = set_answer.clone();
                let mut group = RadioGroup::new((id.clone(), format!("radio-{name}")))
                    .orientation(RadioOrientation::Vertical)
                    .on_value_change(move |v, _, cx| set_answer(Answer::Single(v.clone()), cx));
                if let Some(Answer::Single(v)) = &current {
                    group = group.value(v.clone());
                }
                for choice in &item.choices {
                    group = group.item(RadioGroupItem::new(choice.clone()).label(choice.clone()));
                }
                group.into_any_element()
            }
            QuestionKind::Multiple => {
                let checked = match &current {
                    Some(Answer::Multiple(v)) => v.clone(),
                    _ => Vec::new(),
                };
                // `.cds--checkbox-wrapper { margin-block-end: 0.375rem }`, last 0.1875rem.
                div()
                    .flex()
                    .flex_col()
                    .gap(rem(6.))
                    .pb(rem(3.))
                    .children(item.choices.iter().enumerate().map(|(k, choice)| {
                        let on = checked.contains(choice);
                        let checked = checked.clone();
                        let choice_value = choice.clone();
                        let set_answer = set_answer.clone();
                        Checkbox::new((id.clone(), format!("check-{name}-{k}")))
                            .label(choice.clone())
                            .checked(on)
                            .on_checked_change(move |now, _, cx| {
                                let mut next: Vec<SharedString> = checked
                                    .iter()
                                    .filter(|c| **c != choice_value)
                                    .cloned()
                                    .collect();
                                if now {
                                    next.push(choice_value.clone());
                                }
                                set_answer(Answer::Multiple(next), cx)
                            })
                    }))
                    .into_any_element()
            }
            QuestionKind::Text => {
                let editor = state.read(cx).texts.get(&name).cloned();
                match editor {
                    Some(editor) => Textarea::new((id.clone(), format!("text-{name}")), &editor)
                        .label(item.title.clone())
                        .hide_label(true)
                        .rows(3)
                        .layer(LayerLevel::Two)
                        .into_any_element(),
                    None => div().into_any_element(),
                }
            }
        };

        let texts = self.texts.clone();
        let on_submit = self.on_submit.clone();
        let primary = {
            let state = state.clone();
            let required = item.required;
            move |_: &gpui_kit::ClickEvent, window: &mut Window, cx: &mut App| {
                let answers = collect(&state, cx);
                let answered = Answer::is_answered(answers.get(&name));
                match advance(index, count, required, answered) {
                    Advance::Refuse => state.update(cx, |s, cx| {
                        s.error = true;
                        cx.notify();
                    }),
                    Advance::Next => state.update(cx, |s, cx| {
                        s.error = false;
                        s.index = index + 1;
                        cx.notify();
                    }),
                    Advance::Submit => {
                        if let Some(f) = on_submit.as_ref() {
                            f(&answers, window, cx);
                        }
                    }
                }
            }
        };
        let go = |to: usize| {
            let state = state.clone();
            move |_: &gpui_kit::ClickEvent, _: &mut Window, cx: &mut App| {
                state.update(cx, |s, cx| {
                    s.error = false;
                    s.index = to;
                    cx.notify();
                })
            }
        };
        let last = index + 1 >= count;

        div()
            .flex()
            .flex_col()
            .gap(SPACING_05)
            .max_w(rem(512.))
            .p(SPACING_05)
            .bg(color(ColorRole::Layer01))
            .when_some(self.title, |this, title| {
                this.child(
                    div()
                        .type_style(TypeStyle::Label01)
                        .text_color(color(ColorRole::TextSecondary))
                        .child(title),
                )
            })
            .child(
                Progress::new((id.clone(), "progress"))
                    .label((texts.progress)(index + 1, count))
                    .value(progress_value(index, count))
                    .size(ProgressSize::Small)
                    .layer(LayerLevel::Two),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .mt(SPACING_03)
                            .type_style(TypeStyle::Heading03)
                            .text_color(color(ColorRole::TextPrimary))
                            .child(item.title.clone()),
                    )
                    .when_some(item.description.clone(), |this, d| {
                        this.child(
                            div()
                                .type_style(TypeStyle::Body01)
                                .text_color(color(ColorRole::TextSecondary))
                                .child(d),
                        )
                    }),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .child(control)
                    .when(show_error, |this| {
                        this.child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(SPACING_02)
                                .mt(SPACING_03)
                                .text_size(rem(12.))
                                .text_color(color(ColorRole::TextError))
                                .child(
                                    Icon::carbon(CarbonIcon::ErrorFilled)
                                        .size(IconSize::S16)
                                        .color(color(ColorRole::SupportError)),
                                )
                                .child(texts.required.clone()),
                        )
                    }),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_end()
                    .gap(px(1.))
                    .child(
                        Button::new((id.clone(), "previous"), texts.previous.clone())
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::Md)
                            .disabled(index == 0)
                            .on_click(go(index.saturating_sub(1))),
                    )
                    .when(shows_skip(index, count, item.required), |this| {
                        this.child(
                            Button::new((id.clone(), "skip"), texts.skip.clone())
                                .variant(ButtonVariant::Ghost)
                                .size(ButtonSize::Md)
                                .on_click(go(index + 1)),
                        )
                    })
                    .child(
                        Button::new(
                            (id.clone(), "primary"),
                            if last {
                                texts.submit.clone()
                            } else {
                                texts.next.clone()
                            },
                        )
                        .variant(ButtonVariant::Primary)
                        .size(ButtonSize::Md)
                        .on_click(primary),
                    ),
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_follow_the_reference_logic() {
        assert_eq!(advance(0, 3, true, false), Advance::Refuse);
        assert_eq!(advance(0, 3, true, true), Advance::Next);
        assert_eq!(advance(1, 3, false, false), Advance::Next);
        assert_eq!(advance(2, 3, false, false), Advance::Submit);
        assert!(shows_skip(1, 3, false));
        assert!(!shows_skip(0, 3, true));
        assert!(!shows_skip(2, 3, false));
        assert!((progress_value(0, 3) - 100. / 3.).abs() < 1e-4);
        assert_eq!(progress_value(2, 3), 100.);
    }

    #[test]
    fn empty_answers_do_not_count() {
        assert!(!Answer::is_answered(None));
        assert!(!Answer::is_answered(Some(&Answer::Text("".into()))));
        assert!(!Answer::is_answered(Some(&Answer::Multiple(vec![]))));
        assert!(Answer::is_answered(Some(&Answer::Single("2–10".into()))));
    }
}
