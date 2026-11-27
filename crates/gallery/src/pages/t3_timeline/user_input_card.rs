//! T3 · Timeline › UserInputCard: two questions, the first with a free
//! answer (`components/UserInputCard/preview.html`). Number keys pick, Enter
//! submits, and the answers show under the card.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div,
};
use u0ui::prelude::*;
use u0ui::t3::timeline::{InputOption, InputQuestion, UserInputCard};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| InputPage {
        single: false,
        skip: false,
        generation: 0,
        answers: "".into(),
    })
    .into()
}

struct InputPage {
    single: bool,
    skip: bool,
    generation: usize,
    answers: SharedString,
}

/// The preview's questions.
fn questions() -> Vec<InputQuestion> {
    vec![
        InputQuestion::new(
            "Which theme should be the default for new users?",
            vec![
                InputOption::new("White").description("Matches the marketing site"),
                InputOption::new("G10").description("Slightly softer page background"),
                InputOption::new("Follow the OS")
                    .description("Light or dark based on system setting"),
            ],
        )
        .allow_other(true),
        InputQuestion::new(
            "Ship behind a flag?",
            vec![InputOption::new("Yes"), InputOption::new("No")],
        ),
    ]
}

impl Render for InputPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let mut qs = questions();
        if self.single {
            qs.truncate(1);
        }
        let mut card = UserInputCard::new(
            SharedString::from(format!("input-live-{}-{}", self.generation, self.single)),
            qs,
        )
        .on_submit({
            let entity = entity.clone();
            move |answers, _, cx| {
                entity.update(cx, |this, cx| {
                    this.answers = answers
                        .iter()
                        .map(|(i, a)| format!("{}: {a}", i + 1))
                        .collect::<Vec<_>>()
                        .join(" · ")
                        .into();
                    cx.notify();
                })
            }
        });
        if self.skip {
            let entity = entity.clone();
            card = card.on_skip(move |_, cx| {
                entity.update(cx, |this, cx| {
                    this.answers = "onSkip".into();
                    cx.notify();
                })
            });
        }
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .w_full()
            .max_w(gpui_kit::rems(40.))
            .child(card)
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(self.answers.clone()),
            );
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::toggle(
                "input-single",
                "One question",
                self.single,
                controls::setter(&entity, |this: &mut Self, on| this.single = on),
            ))
            .child(controls::toggle(
                "input-skip",
                "onSkip",
                self.skip,
                controls::setter(&entity, |this: &mut Self, on| this.skip = on),
            ))
            .child(
                Button::new("input-reset", "Start over")
                    .variant(ButtonVariant::Tertiary)
                    .size(ButtonSize::Sm)
                    .on_click({
                        let entity = entity.clone();
                        move |_, _, cx| {
                            entity.update(cx, |this, cx| {
                                this.generation += 1;
                                this.answers = "".into();
                                cx.notify();
                            })
                        }
                    }),
            );
        Body::new()
            .playground(live)
            .hint("Click an option, then 1–4 pick, ↑/↓ move, Enter submits.")
            .config(config)
            .example(controls::block(
                "Reference preview",
                div()
                    .w_full()
                    .max_w(gpui_kit::rems(40.))
                    .child(UserInputCard::new("input-ref", questions())),
            ))
    }
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(gpui_kit::rems(15.))
        .child(UserInputCard::new(
            "input-th",
            vec![InputQuestion::new(
                "Ship behind a flag?",
                vec![InputOption::new("Yes"), InputOption::new("No")],
            )],
        ))
        .into_any_element()
}
