//! Chat & AI › Questionnaire: the reference "Workspace setup" survey (required
//! single choice, multiple choice, free text) with the submitted answers shown
//! below, and a variant whose questions are all optional and described.
use crate::controls;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, SharedString,
    Styled, Window, div,
};
use std::rc::Rc;
use u0ui::components::questionnaire::{
    Answer, QuestionKind, Questionnaire, QuestionnaireItem, QuestionnaireTexts,
};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| QuizPage {
        submitted: None,
        required: true,
        descriptions: false,
        generation: 0,
    })
    .into()
}

struct QuizPage {
    submitted: Option<String>,
    required: bool,
    descriptions: bool,
    generation: usize,
}

/// The reference's English copy, supplied by the app.
fn texts() -> QuestionnaireTexts {
    QuestionnaireTexts {
        progress: Rc::new(|i, n| SharedString::from(format!("Question {i} of {n}"))),
        previous: "Previous".into(),
        skip: "Skip".into(),
        next: "Next".into(),
        submit: "Submit".into(),
        required: "Answer this question to continue.".into(),
    }
}

/// `preview.html` items.
fn items(required: bool, descriptions: bool) -> Vec<QuestionnaireItem> {
    let describe = |item: QuestionnaireItem, text: &'static str| {
        if descriptions {
            item.description(text)
        } else {
            item
        }
    };
    vec![
        describe(
            QuestionnaireItem::new("team", "How big is your team?", QuestionKind::Single)
                .required(required)
                .choices(["Just me", "2–10", "11–50", "50+"]),
            "We size the default quotas from this.",
        ),
        describe(
            QuestionnaireItem::new("use", "What will you build?", QuestionKind::Multiple)
                .choices(["APIs", "Data pipelines", "AI apps"]),
            "Pick all that apply.",
        ),
        describe(
            QuestionnaireItem::new("notes", "Anything else?", QuestionKind::Text),
            "Optional notes for the onboarding team.",
        ),
    ]
}

fn describe_answers(answers: &std::collections::HashMap<SharedString, Answer>) -> String {
    let mut keys: Vec<_> = answers.keys().collect();
    keys.sort();
    keys.iter()
        .map(|k| {
            let v = match &answers[*k] {
                Answer::Single(v) | Answer::Text(v) => v.to_string(),
                Answer::Multiple(v) => v
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            };
            format!("{k}: {v}")
        })
        .collect::<Vec<_>>()
        .join(" · ")
}

impl Render for QuizPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let on_submit = {
            let entity = entity.clone();
            move |answers: &std::collections::HashMap<SharedString, Answer>,
                  _: &mut Window,
                  cx: &mut App| {
                let text = describe_answers(answers);
                entity.update(cx, |this, cx| {
                    this.submitted = Some(text);
                    cx.notify();
                })
            }
        };

        let reference = Questionnaire::new("quiz-ref", items(true, false), texts())
            .title("Workspace setup")
            .on_submit(on_submit.clone());

        let live = Questionnaire::new(
            (
                "quiz-live",
                self.generation * 4 + self.required as usize * 2 + self.descriptions as usize,
            ),
            items(self.required, self.descriptions),
            texts(),
        )
        .title("Workspace setup")
        .on_submit(on_submit);

        let result = div()
            .type_style(TypeStyle::Label01)
            .text_color(color(ColorRole::TextSecondary))
            .child(
                self.submitted
                    .clone()
                    .map(|s| format!("Submitted: {s}"))
                    .unwrap_or_else(|| "Submit to see the answers here.".into()),
            );

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::toggle(
                "quiz-required",
                "first question required",
                self.required,
                controls::setter(&entity, |this: &mut QuizPage, on| this.required = on),
            ))
            .child(controls::toggle(
                "quiz-descriptions",
                "descriptions",
                self.descriptions,
                controls::setter(&entity, |this: &mut QuizPage, on| this.descriptions = on),
            ))
            .child(
                div().flex().child(
                    Button::new("quiz-reset", "Restart live")
                        .variant(ButtonVariant::Tertiary)
                        .size(ButtonSize::Sm)
                        .on_click({
                            let entity = entity.clone();
                            move |_, _, cx| {
                                entity.update(cx, |this, cx| {
                                    this.generation += 1;
                                    this.submitted = None;
                                    cx.notify();
                                })
                            }
                        }),
                ),
            );

        controls::page_body(
            div()
                .flex()
                .flex_col()
                .gap(tokens::SPACING_07)
                .child(controls::block("Reference preview", reference))
                .child(controls::block("Live", live))
                .child(result),
            Some(config.into_any_element()),
        )
    }
}
