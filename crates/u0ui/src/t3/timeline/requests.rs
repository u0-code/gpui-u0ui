//! The timeline's request cards ("Cards" in `t3.css`):
//!
//! - [`UserInputCard`] (`.t3-input`): the agent's questions, one at a time,
//!   with numbered options, an optional free answer and Back / Skip / Next.
//!   Keys: 1…n pick an option, n+1 focuses the free answer, ↑/↓ move the
//!   choice, Enter submits (Shift+Enter does not).
//! - [`SecretRequestCard`] (`.t3-secret`): password fields on `field-02`.
//! - [`ProposedPlanCard`] (`.t3-plan`): its steps (an `<ol>` without markers,
//!   or each with the [`PlanStatus`] glyph of the step's state), how far it
//!   has got beside its title, copy / download / save icon buttons, a note
//!   before Keep planning and Implement plan; each button is drawn with its
//!   handler (a plan only followed has neither Keep planning nor Implement).
//! - [`RequestCard`] (`.t3-request`): a generic ask with a preview well.
use super::card;
use crate::LayerLevel;
use crate::components::button::{Button, ButtonKind, ButtonSize, ButtonVariant};
use crate::components::floating::Align;
use crate::components::{Input, InputState};
use crate::t3::action::{ActionButton, ActionKind, KeyCap, Part};
use crate::t3::base::Spin;
use crate::t3::press::{self, Handler, rem};
use crate::u0::thread::PlanStatus;
use crate::{
    components::form::FieldSize,
    icons::{CarbonIcon, Icon},
    theme::color,
    tokens::{ColorRole, FontFamilyKey},
    typography::font,
};
use gpui_kit::Focusable as _;
use gpui_kit::component::Sizable as _;
use gpui_kit::{
    AnyElement, App, AppContext as _, BoxShadow, ClickEvent, CursorStyle, ElementId, Entity,
    FocusHandle, FontWeight, InteractiveElement as _, IntoElement, KeyDownEvent, KeyboardButton,
    KeyboardClickEvent, MouseButton, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement as _, Styled, Window, div, prelude::FluentBuilder as _, px,
};
use std::collections::BTreeMap;
use std::rc::Rc;

/// One option of a question (`{ label, description, default }`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct InputOption {
    /// `label`.
    pub label: SharedString,
    /// `description`.
    pub description: Option<SharedString>,
    /// `default`: chosen when the card opens.
    pub default: bool,
}

impl InputOption {
    /// An option labelled `label`.
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            ..Default::default()
        }
    }

    /// `description`.
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// `default`.
    pub fn default_choice(mut self, default: bool) -> Self {
        self.default = default;
        self
    }
}

/// One question (`{ question, options, allowOther }`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct InputQuestion {
    /// `question`.
    pub question: SharedString,
    /// `options`.
    pub options: Vec<InputOption>,
    /// `allowOther`: a free answer after the options.
    pub allow_other: bool,
}

impl InputQuestion {
    /// A question with `options`.
    pub fn new(question: impl Into<SharedString>, options: Vec<InputOption>) -> Self {
        Self {
            question: question.into(),
            options,
            allow_other: false,
        }
    }

    /// `allowOther`.
    pub fn allow_other(mut self, allow: bool) -> Self {
        self.allow_other = allow;
        self
    }
}

/// A chosen answer: an option's label or the free answer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answer {
    /// Option label.
    Option(SharedString),
    /// The free answer (`__other`).
    Other,
}

/// `UserInputCard`'s strings (the reference's English defaults).
#[derive(Clone)]
pub struct UserInputTexts {
    /// One question's head (`title` overrides it).
    pub title: SharedString,
    /// `"Question {i} of {n}"`.
    pub progress: Rc<dyn Fn(usize, usize) -> SharedString>,
    /// Free-answer placeholder (`otherPlaceholder`).
    pub other_placeholder: SharedString,
    /// Back.
    pub back: SharedString,
    /// Skip (`skipLabel`).
    pub skip: SharedString,
    /// Next question.
    pub next: SharedString,
    /// Submit answer (one question).
    pub submit_one: SharedString,
    /// Submit answers (several).
    pub submit_all: SharedString,
}

impl Default for UserInputTexts {
    fn default() -> Self {
        Self {
            title: "The agent has a question".into(),
            progress: Rc::new(|i, n| format!("Question {i} of {n}").into()),
            other_placeholder: "Something else…".into(),
            back: "Back".into(),
            skip: "Skip".into(),
            next: "Next question".into(),
            submit_one: "Submit answer".into(),
            submit_all: "Submit answers".into(),
        }
    }
}

/// The card's walk through its questions.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct InputProgress {
    /// The question shown.
    pub index: usize,
    /// Answers so far, by question.
    pub answers: BTreeMap<usize, Answer>,
}

impl InputProgress {
    /// Seeded with each question's `default` option.
    pub fn new(questions: &[InputQuestion]) -> Self {
        let answers = questions
            .iter()
            .enumerate()
            .filter_map(|(k, q)| {
                q.options
                    .iter()
                    .find(|o| o.default)
                    .map(|o| (k, Answer::Option(o.label.clone())))
            })
            .collect();
        Self { index: 0, answers }
    }

    /// Whether question `i` can be submitted: answered, and a free answer
    /// is not blank.
    pub fn ready(&self, i: usize, other: &str) -> bool {
        match self.answers.get(&i) {
            Some(Answer::Option(_)) => true,
            Some(Answer::Other) => !other.trim().is_empty(),
            None => false,
        }
    }
}

type SubmitHandler = Rc<dyn Fn(Vec<(usize, SharedString)>, &mut Window, &mut App)>;

/// The agent's questions. See the module docs.
#[derive(IntoElement)]
pub struct UserInputCard {
    id: ElementId,
    questions: Vec<InputQuestion>,
    title: Option<SharedString>,
    texts: UserInputTexts,
    on_submit: Option<SubmitHandler>,
    on_skip: Option<Handler>,
}

impl UserInputCard {
    /// A card asking `questions`.
    pub fn new(id: impl Into<ElementId>, questions: Vec<InputQuestion>) -> Self {
        Self {
            id: id.into(),
            questions,
            title: None,
            texts: UserInputTexts::default(),
            on_submit: None,
            on_skip: None,
        }
    }

    /// `title` for a single question.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Replace the strings (`otherPlaceholder`, `skipLabel`, …).
    pub fn texts(mut self, texts: UserInputTexts) -> Self {
        self.texts = texts;
        self
    }

    /// `onSubmit(answers)`: (question index, answer text) pairs.
    pub fn on_submit(
        mut self,
        f: impl Fn(Vec<(usize, SharedString)>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_submit = Some(Rc::new(f));
        self
    }

    /// `onSkip`: adds the Skip button.
    pub fn on_skip(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_skip = Some(Rc::new(f));
        self
    }
}

/// The free-answer fields, one per question, and the options' focus handles.
struct InputFields {
    other: Vec<Entity<InputState>>,
    focus: Vec<FocusHandle>,
}

/// `.t3-opt` colors: (fill, frame) for hover × chosen.
pub fn option_colors(hovered: bool, chosen: bool) -> (gpui_kit::Hsla, Option<gpui_kit::Hsla>) {
    (
        // `.t3-opt { background: layer-02 }`, `:hover { background: layer-hover-02 }`.
        color(if hovered {
            ColorRole::LayerHover02
        } else {
            ColorRole::Layer02
        }),
        // `.t3-opt--on { box-shadow: inset 0 0 0 2px border-interactive }`.
        chosen.then(|| color(ColorRole::BorderInteractive)),
    )
}

/// What a key does in the card: pick option `n` (0-based), focus the free
/// answer, submit, or move the choice by ±1.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputKey {
    /// Choose option n.
    Pick(usize),
    /// Choose and focus the free answer.
    Other,
    /// Submit (Enter without Shift).
    Submit,
    /// Arrow: move the choice.
    Move(isize),
}

/// The `onKey` map of `UserInputCard` for `options` options; `in_other`
/// while typing the free answer (only Enter counts there).
pub fn input_key(
    key: &str,
    shift: bool,
    modified: bool,
    options: usize,
    allow_other: bool,
    in_other: bool,
) -> Option<InputKey> {
    if modified {
        return None;
    }
    if key == "enter" && !shift {
        return Some(InputKey::Submit);
    }
    if in_other {
        return None;
    }
    match key {
        "down" | "right" => return Some(InputKey::Move(1)),
        "up" | "left" => return Some(InputKey::Move(-1)),
        _ => {}
    }
    let n: usize = key.parse().ok()?;
    if n >= 1 && n <= options {
        Some(InputKey::Pick(n - 1))
    } else if allow_other && n == options + 1 {
        Some(InputKey::Other)
    } else {
        None
    }
}

impl RenderOnce for UserInputCard {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let questions = Rc::new(self.questions);
        let count = questions.len();
        let progress = window.use_keyed_state((id.clone(), "progress"), cx, {
            let questions = questions.clone();
            move |_, _| InputProgress::new(&questions)
        });
        let placeholder = self.texts.other_placeholder.clone();
        let fields = window.use_keyed_state((id.clone(), "fields"), cx, {
            let questions = questions.clone();
            move |window, cx| InputFields {
                other: questions
                    .iter()
                    .map(|_| cx.new(|cx| InputState::new(window, cx)))
                    .collect(),
                focus: (0..12).map(|_| cx.focus_handle()).collect(),
            }
        });
        let Some(question) = questions.get(progress.read(cx).index.min(count.saturating_sub(1)))
        else {
            return div().into_any_element();
        };
        let question = question.clone();
        let i = progress.read(cx).index.min(count - 1);
        let last = i + 1 == count;
        let other_state = fields.read(cx).other[i].clone();
        let option_focus = fields.read(cx).focus.clone();
        let other_text = other_state.read(cx).value().to_string();
        let ready = progress.read(cx).ready(i, &other_text);
        let chosen = progress.read(cx).answers.get(&i).cloned();
        let other_focused = other_state.read(cx).focus_handle(cx).is_focused(window);
        let texts = self.texts.clone();

        // Submit: advance, or hand back every answer.
        let submit: Handler = {
            let progress = progress.clone();
            let fields = fields.clone();
            let on_submit = self.on_submit.clone();
            Rc::new(move |window, cx| {
                let other: Vec<String> = fields
                    .read(cx)
                    .other
                    .iter()
                    .map(|s| s.read(cx).value().to_string())
                    .collect();
                let p = progress.read(cx).clone();
                if !p.ready(p.index, &other[p.index]) {
                    return;
                }
                if p.index + 1 < count {
                    progress.update(cx, |p, cx| {
                        p.index += 1;
                        cx.notify();
                    });
                    return;
                }
                let out = p
                    .answers
                    .iter()
                    .map(|(k, a)| {
                        (
                            *k,
                            match a {
                                Answer::Option(label) => label.clone(),
                                Answer::Other => other[*k].clone().into(),
                            },
                        )
                    })
                    .collect();
                if let Some(f) = on_submit.as_ref() {
                    f(out, window, cx)
                }
            })
        };
        let choose = {
            let progress = progress.clone();
            move |answer: Answer, cx: &mut App| {
                progress.update(cx, |p, cx| {
                    p.answers.insert(i, answer);
                    cx.notify();
                })
            }
        };

        let head_text = if count > 1 {
            (texts.progress)(i + 1, count)
        } else {
            self.title.clone().unwrap_or_else(|| texts.title.clone())
        };
        let mut options: Vec<AnyElement> = Vec::new();
        for (k, option) in question.options.iter().enumerate() {
            let on = chosen == Some(Answer::Option(option.label.clone()));
            let (hovered, hover) = press::hover(
                (id.clone(), SharedString::from(format!("opt-hover-{i}-{k}"))),
                window,
                cx,
            );
            let focus = option_focus[k.min(option_focus.len() - 1)].clone();
            let choose = choose.clone();
            let label = option.label.clone();
            options.push(
                option_row(
                    div()
                        .id((id.clone(), SharedString::from(format!("opt-{i}-{k}"))))
                        // The options are one native radio group (`<input
                        // type="radio" name=…>`): the checked one is its tab stop,
                        // the first when none is. A tracked handle carries its
                        // own tab-stop flag.
                        .track_focus(
                            &focus
                                .clone()
                                .tab_index(0)
                                .tab_stop(on || (chosen.is_none() && k == 0)),
                        )
                        .on_hover(press::track_hover(hover))
                        // Space picks the focused radio (GPUI's keyboard click);
                        // Enter is the card's submit key, so its keyboard click
                        // must not also pick the option under focus.
                        .on_click(move |e, window, cx| {
                            if matches!(
                                e,
                                ClickEvent::Keyboard(KeyboardClickEvent {
                                    button: KeyboardButton::Enter,
                                    ..
                                })
                            ) {
                                return;
                            }
                            window.focus(&focus, cx);
                            choose(Answer::Option(label.clone()), cx)
                        }),
                    k + 1,
                    hovered,
                    on,
                    option_focus[k.min(option_focus.len() - 1)].is_focused(window)
                        && crate::focus::focus_visible_mode(window),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .min_w_0()
                        // `.t3-opt b { display: block; font-weight: 600 }`.
                        .child(
                            div()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(option.label.clone()),
                        )
                        // `.t3-opt-desc { display: block; margin-top: 2px; color: text-secondary; font-size: 0.75rem }`.
                        .when_some(option.description.clone(), |this, d| {
                            this.child(
                                div()
                                    .mt(px(2.))
                                    .text_size(rem(12.))
                                    .line_height(rem(16.))
                                    .text_color(color(ColorRole::TextSecondary))
                                    .child(d),
                            )
                        }),
                )
                .into_any_element(),
            );
        }
        if question.allow_other {
            let k = question.options.len();
            let on = chosen == Some(Answer::Other);
            let (hovered, hover) = press::hover(
                (
                    id.clone(),
                    SharedString::from(format!("opt-hover-{i}-other")),
                ),
                window,
                cx,
            );
            let state = other_state.clone();
            let choose_click = choose.clone();
            options.push(
                option_row(
                    div()
                        .id((id.clone(), SharedString::from(format!("opt-{i}-other"))))
                        .on_hover(press::track_hover(hover))
                        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                            choose_click(Answer::Other, cx);
                            state.update(cx, |s, cx| s.focus(window, cx));
                        }),
                    k + 1,
                    hovered,
                    on,
                    // `.t3-opt:focus-within { outline: 2px solid focus }`.
                    other_focused,
                )
                .child(
                    // `.t3-opt-other { flex: 1; border-bottom: 1px solid border-strong-01;
                    // background: transparent; color: text-primary }`.
                    div()
                        .relative()
                        .flex_1()
                        .min_w_0()
                        .border_b_1()
                        .border_color(color(ColorRole::BorderStrong01))
                        .when(other_text.is_empty(), |this| {
                            this.child(
                                div()
                                    .absolute()
                                    .left_0()
                                    .top_0()
                                    .text_color(color(ColorRole::TextPlaceholder))
                                    .child(placeholder.clone()),
                            )
                        })
                        .child(
                            gpui_kit::component::input::Input::new(&other_state)
                                .appearance(false)
                                .with_size(gpui_kit::component::Size::XSmall)
                                .p_0()
                                .w_full()
                                .text_size(rem(14.))
                                .line_height(rem(18.)),
                        ),
                )
                .into_any_element(),
            );
        }

        let on_back = {
            let progress = progress.clone();
            move |_: &gpui_kit::ClickEvent, _: &mut Window, cx: &mut App| {
                progress.update(cx, |p, cx| {
                    p.index = p.index.saturating_sub(1);
                    cx.notify();
                })
            }
        };
        let submit_label = if !last {
            texts.next.clone()
        } else if count > 1 {
            texts.submit_all.clone()
        } else {
            texts.submit_one.clone()
        };
        let actions = card::actions()
            .when(i > 0, |this| {
                this.child(
                    Button::new((id.clone(), "back"), texts.back.clone())
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .on_click(on_back),
                )
            })
            .when_some(self.on_skip.clone(), |this, f| {
                this.child(
                    Button::new((id.clone(), "skip"), texts.skip.clone())
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .on_click(move |_, w, cx| f(w, cx)),
                )
            })
            .child(
                ActionButton::new(
                    (id.clone(), "submit"),
                    submit_label.clone(),
                    ActionKind::Carbon(ButtonKind::Primary),
                )
                .part(Part::Text(submit_label))
                .part(Part::Key("↵".into(), KeyCap::Framed))
                .disabled(!ready)
                .on_click(Some(submit.clone())),
            );

        let option_count = question.options.len();
        let allow_other = question.allow_other;
        let labels: Vec<SharedString> = question.options.iter().map(|o| o.label.clone()).collect();
        let key_progress = progress.clone();
        let key_other = other_state.clone();
        let key_focus = option_focus.clone();
        card::frame()
            .id(id.clone())
            .on_key_down(move |e: &KeyDownEvent, window, cx| {
                let m = &e.keystroke.modifiers;
                let in_other = key_other.read(cx).focus_handle(cx).is_focused(window);
                let Some(action) = input_key(
                    &e.keystroke.key,
                    m.shift,
                    m.platform || m.control || m.alt,
                    option_count,
                    allow_other,
                    in_other,
                ) else {
                    return;
                };
                cx.stop_propagation();
                let set = |answer: Answer, cx: &mut App| {
                    key_progress.update(cx, |p, cx| {
                        p.answers.insert(i, answer);
                        cx.notify();
                    })
                };
                match action {
                    InputKey::Submit => submit(window, cx),
                    InputKey::Pick(n) => set(Answer::Option(labels[n].clone()), cx),
                    InputKey::Other => {
                        set(Answer::Other, cx);
                        key_other.update(cx, |s, cx| s.focus(window, cx));
                    }
                    InputKey::Move(d) => {
                        let total = option_count + usize::from(allow_other);
                        if total == 0 {
                            return;
                        }
                        let current = match key_progress.read(cx).answers.get(&i) {
                            Some(Answer::Option(l)) => labels.iter().position(|x| x == l),
                            Some(Answer::Other) => Some(option_count),
                            None => None,
                        };
                        let next = match current {
                            Some(c) => (c as isize + d).rem_euclid(total as isize) as usize,
                            None => 0,
                        };
                        if next < option_count {
                            set(Answer::Option(labels[next].clone()), cx);
                            window.focus(&key_focus[next.min(key_focus.len() - 1)], cx);
                        } else {
                            set(Answer::Other, cx);
                            key_other.update(cx, |s, cx| s.focus(window, cx));
                        }
                    }
                }
            })
            .child(
                card::head()
                    .child(card::head_icon(
                        CarbonIcon::Help,
                        color(ColorRole::IconPrimary),
                    ))
                    .child(card::head_title(head_text)),
            )
            // `.t3-input-q { font-size: 1rem; line-height: 1.5 }`.
            .child(
                div()
                    .text_size(rem(16.))
                    .line_height(rem(24.))
                    .child(question.question.clone()),
            )
            // `.t3-input-opts { display: flex; flex-direction: column; gap: 1px }`.
            .child(div().flex().flex_col().gap(px(1.)).children(options))
            .child(actions)
            .into_any_element()
    }
}

/// A `.t3-opt` row: `display: flex; align-items: flex-start; gap: spacing-04;
/// padding: spacing-04; font-size: 0.875rem`, with its number key.
fn option_row(
    row: gpui_kit::Stateful<gpui_kit::Div>,
    number: usize,
    hovered: bool,
    on: bool,
    focused: bool,
) -> gpui_kit::Stateful<gpui_kit::Div> {
    let (fill, frame) = option_colors(hovered, on);
    let mut shadows: Vec<BoxShadow> = Vec::new();
    if let Some(frame) = frame {
        shadows.push(
            BoxShadow::new(px(0.), px(0.), frame)
                .spread_radius(px(2.))
                .inset(),
        );
    }
    row.relative()
        .flex()
        .flex_row()
        .items_start()
        .gap(rem(12.))
        .p(rem(12.))
        .bg(fill)
        .shadow(shadows)
        .cursor(CursorStyle::PointingHand)
        .text_size(rem(14.))
        .line_height(rem(18.))
        .text_color(color(ColorRole::TextPrimary))
        // `.t3-opt-key { width: 1.25rem; height: 1.25rem; background: layer-accent-02;
        // font-family: mono; font-size: 0.75rem }`; r3-desk: chosen = interactive / text-on-color.
        .child(
            div()
                .flex()
                .flex_none()
                .items_center()
                .justify_center()
                .size(rem(20.))
                .bg(color(if on {
                    ColorRole::Interactive
                } else {
                    ColorRole::LayerAccent02
                }))
                .text_color(color(if on {
                    ColorRole::TextOnColor
                } else {
                    ColorRole::TextPrimary
                }))
                .font(font(FontFamilyKey::Mono, 400))
                .text_size(rem(12.))
                .child(SharedString::from(number.to_string())),
        )
        .when(focused, |this| this.child(press::ring()))
}

/// One secret field (`{ label, placeholder }`).
#[derive(Clone, Debug, PartialEq)]
pub struct SecretField {
    /// `label`.
    pub label: SharedString,
    /// `placeholder` (default `••••••••`).
    pub placeholder: SharedString,
}

impl SecretField {
    /// A field labelled `label`.
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            placeholder: "••••••••".into(),
        }
    }

    /// `placeholder`.
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }
}

/// `SecretRequestCard`'s strings.
#[derive(Clone)]
pub struct SecretTexts {
    /// The head's note.
    pub requested: SharedString,
    /// Decline.
    pub decline: SharedString,
    /// Save securely.
    pub save: SharedString,
}

impl Default for SecretTexts {
    fn default() -> Self {
        Self {
            requested: "Requested by the agent".into(),
            decline: "Decline".into(),
            save: "Save securely".into(),
        }
    }
}

type SaveHandler = Rc<dyn Fn(Vec<SharedString>, &mut Window, &mut App)>;

/// A request for a secret value. See the module docs.
#[derive(IntoElement)]
pub struct SecretRequestCard {
    id: ElementId,
    name: SharedString,
    description: Option<SharedString>,
    fields: Vec<SecretField>,
    texts: SecretTexts,
    on_save: Option<SaveHandler>,
    on_decline: Option<Handler>,
}

impl SecretRequestCard {
    /// A request for `name` (the reference default: "API key") with one
    /// `Value` field.
    pub fn new(id: impl Into<ElementId>, name: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            fields: vec![SecretField::new("Value")],
            texts: SecretTexts::default(),
            on_save: None,
            on_decline: None,
        }
    }

    /// `description`.
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// `fields`.
    pub fn fields(mut self, fields: Vec<SecretField>) -> Self {
        self.fields = fields;
        self
    }

    /// Replace the strings.
    pub fn texts(mut self, texts: SecretTexts) -> Self {
        self.texts = texts;
        self
    }

    /// `onSave`: the values, in field order.
    pub fn on_save(
        mut self,
        f: impl Fn(Vec<SharedString>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_save = Some(Rc::new(f));
        self
    }

    /// `onDecline`.
    pub fn on_decline(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_decline = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for SecretRequestCard {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let n = self.fields.len();
        let states = window
            .use_keyed_state((id.clone(), "fields"), cx, move |window, cx| {
                (0..n)
                    .map(|_| cx.new(|cx| InputState::new(window, cx)))
                    .collect::<Vec<_>>()
            })
            .read(cx)
            .clone();
        let texts = self.texts.clone();
        let on_save = self.on_save.clone();
        let save_states = states.clone();
        card::frame()
            .child(
                card::head()
                    .child(card::head_icon(
                        CarbonIcon::Password,
                        color(ColorRole::IconPrimary),
                    ))
                    .child(card::head_title(self.name.clone()))
                    .child(card::sub(texts.requested.clone())),
            )
            .when_some(self.description.clone(), |this, d| {
                this.child(card::note(d))
            })
            .children(self.fields.iter().enumerate().filter_map(|(i, f)| {
                states.get(i).map(|state| {
                    // `.t3-secret-field .cds--text-input { background: field-02 }`, `size="md"`.
                    Input::new(
                        (id.clone(), SharedString::from(format!("field-{i}"))),
                        state,
                    )
                    .label(f.label.clone())
                    .placeholder(f.placeholder.clone())
                    .password(true)
                    .size(FieldSize::Md)
                    .layer(LayerLevel::Two)
                })
            }))
            .child(
                card::actions()
                    .child(
                        Button::new((id.clone(), "decline"), texts.decline.clone())
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::Sm)
                            .when_some(self.on_decline.clone(), |b, f| {
                                b.on_click(move |_, w, cx| f(w, cx))
                            }),
                    )
                    .child(
                        Button::new((id.clone(), "save"), texts.save.clone())
                            .size(ButtonSize::Sm)
                            .icon(CarbonIcon::Locked)
                            .on_click(move |_, window, cx| {
                                let values = save_states
                                    .iter()
                                    .map(|s| SharedString::from(s.read(cx).value().to_string()))
                                    .collect();
                                if let Some(f) = on_save.as_ref() {
                                    f(values, window, cx)
                                }
                            }),
                    ),
            )
    }
}

/// One plan step (`{ title, detail }`).
#[derive(Clone, Debug, PartialEq)]
pub struct PlanStep {
    /// `title`.
    pub title: SharedString,
    /// `detail`.
    pub detail: Option<SharedString>,
    /// Where the step stands: its glyph in the gutter, as the
    /// [`crate::u0::thread::PlanChecklist`] draws it; none leaves the
    /// gutter empty.
    pub status: Option<PlanStatus>,
}

impl PlanStep {
    /// A step.
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            detail: None,
            status: None,
        }
    }

    /// `detail`.
    pub fn detail(mut self, detail: impl Into<SharedString>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Where the step stands.
    pub fn status(mut self, status: PlanStatus) -> Self {
        self.status = Some(status);
        self
    }
}

/// `ProposedPlanCard`'s strings.
#[derive(Clone)]
pub struct PlanTexts {
    /// Copy plan.
    pub copy: SharedString,
    /// Download as Markdown.
    pub download: SharedString,
    /// Save to workspace.
    pub save: SharedString,
    /// Keep planning.
    pub keep: SharedString,
    /// Implement plan.
    pub implement: SharedString,
}

impl Default for PlanTexts {
    fn default() -> Self {
        Self {
            copy: "Copy plan".into(),
            download: "Download as Markdown".into(),
            save: "Save to workspace".into(),
            keep: "Keep planning".into(),
            implement: "Implement plan".into(),
        }
    }
}

/// The agent's proposed plan. See the module docs.
#[derive(IntoElement)]
pub struct ProposedPlanCard {
    id: ElementId,
    title: SharedString,
    steps: Vec<PlanStep>,
    texts: PlanTexts,
    progress: Option<SharedString>,
    note: Option<SharedString>,
    on_implement: Option<Handler>,
    on_keep_planning: Option<Handler>,
    on_copy: Option<Handler>,
    on_download: Option<Handler>,
    on_save: Option<Handler>,
}

impl ProposedPlanCard {
    /// A plan titled `title` (the reference default: "Proposed plan").
    pub fn new(
        id: impl Into<ElementId>,
        title: impl Into<SharedString>,
        steps: Vec<PlanStep>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            steps,
            texts: PlanTexts::default(),
            progress: None,
            note: None,
            on_implement: None,
            on_keep_planning: None,
            on_copy: None,
            on_download: None,
            on_save: None,
        }
    }

    /// Replace the strings.
    pub fn texts(mut self, texts: PlanTexts) -> Self {
        self.texts = texts;
        self
    }

    /// How far the plan has got ("1 of 4 done"), beside the title.
    pub fn progress(mut self, progress: impl Into<SharedString>) -> Self {
        self.progress = Some(progress.into());
        self
    }

    /// A line before the actions, saying what implementing does.
    pub fn note(mut self, note: impl Into<SharedString>) -> Self {
        self.note = Some(note.into());
        self
    }

    /// `onImplement`.
    pub fn on_implement(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_implement = Some(Rc::new(f));
        self
    }

    /// Keep planning.
    pub fn on_keep_planning(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_keep_planning = Some(Rc::new(f));
        self
    }

    /// Copy plan.
    pub fn on_copy(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_copy = Some(Rc::new(f));
        self
    }

    /// Download as Markdown.
    pub fn on_download(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_download = Some(Rc::new(f));
        self
    }

    /// Save to workspace.
    pub fn on_save(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_save = Some(Rc::new(f));
        self
    }
}

/// A ghost `sm` icon button with its label as a tooltip.
pub(crate) fn icon_button(
    id: impl Into<ElementId>,
    label: SharedString,
    icon: CarbonIcon,
    align: Align,
    on_click: Option<Handler>,
) -> Button {
    Button::new(id, label)
        .icon(Icon::carbon(icon))
        .icon_only(true)
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::Sm)
        .tooltip_align(align)
        .when_some(on_click, |b, f| b.on_click(move |_, w, cx| f(w, cx)))
}

impl RenderOnce for ProposedPlanCard {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let texts = self.texts.clone();
        let steps = self.steps.into_iter().enumerate().map(|(i, step)| {
            // `.t3-plan-steps { padding-left: spacing-06; font-size: 0.875rem; line-height: 1.42857 }`,
            // `li { margin-bottom: spacing-03 }`, `b { display: block; font-weight: 600 }`,
            // `span { color: text-secondary }`. Carbon's reset (`ol { list-style: none }`)
            // removes the markers, so the 24px gutter stays empty (the reference render),
            // unless the step says where it stands: then its glyph sits there.
            let glyph = step.status.map(|st| {
                let (_, ink, _, _) = st.look();
                let mark: AnyElement = if st == PlanStatus::InProgress {
                    Spin::new((id.clone(), SharedString::from(format!("spin-{i}"))))
                        .into_any_element()
                } else {
                    Icon::carbon(if st == PlanStatus::Done {
                        CarbonIcon::CheckmarkFilled
                    } else {
                        CarbonIcon::CircleDash
                    })
                    .size_length(rem(16.))
                    .color(ink)
                    .into_any_element()
                };
                div().absolute().left_0().top(px(2.)).child(mark)
            });
            let (ink, struck) = step.status.map_or((None, false), |st| {
                let (ink, _, struck, _) = st.look();
                (Some(ink), struck)
            });
            div()
                .relative()
                .flex()
                .flex_row()
                .mb(rem(8.))
                .pl(rem(24.))
                .children(glyph)
                .child(
                    div()
                        .when_some(ink, |d, ink| d.text_color(ink))
                        .when(struck, |d| d.line_through())
                        .flex()
                        .flex_col()
                        .min_w_0()
                        .child(div().font_weight(FontWeight::SEMIBOLD).child(step.title))
                        .when_some(step.detail, |this, d| {
                            this.child(div().text_color(color(ColorRole::TextSecondary)).child(d))
                        }),
                )
        });
        card::frame()
            .child(
                card::head()
                    .child(card::head_icon(
                        CarbonIcon::ListChecked,
                        color(ColorRole::IconPrimary),
                    ))
                    .child(card::head_title(self.title.clone()))
                    .children(self.progress.clone().map(card::sub))
                    .child(card::grow())
                    // Each drawn with its handler.
                    .children(
                        [
                            (
                                "copy",
                                texts.copy.clone(),
                                CarbonIcon::Copy,
                                Align::Bottom,
                                self.on_copy.clone(),
                            ),
                            (
                                "download",
                                texts.download.clone(),
                                CarbonIcon::Download,
                                Align::Bottom,
                                self.on_download.clone(),
                            ),
                            (
                                "save",
                                texts.save.clone(),
                                CarbonIcon::DocumentAdd,
                                Align::BottomRight,
                                self.on_save.clone(),
                            ),
                        ]
                        .into_iter()
                        .filter(|(.., f)| f.is_some())
                        .map(|(part, label, icon, align, f)| {
                            icon_button((id.clone(), part), label, icon, align, f)
                        }),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .text_size(rem(14.))
                    .line_height(rem(20.))
                    .children(steps),
            )
            // A plan only followed, not proposed, has no way on from here.
            .when(
                self.on_keep_planning.is_some() || self.on_implement.is_some(),
                |frame| {
                    frame.child(
                        card::actions()
                            .when_some(self.note.clone(), |row, n| {
                                // `.tpa-plan-note { margin-right: auto; font-size: 0.75rem;
                                // color: text-secondary }`.
                                row.child(card::sub(n).mr_auto())
                            })
                            .when_some(self.on_keep_planning.clone(), |row, f| {
                                row.child(
                                    Button::new((id.clone(), "keep"), texts.keep.clone())
                                        .variant(ButtonVariant::Ghost)
                                        .size(ButtonSize::Sm)
                                        .on_click(move |_, w, cx| f(w, cx)),
                                )
                            })
                            .when_some(self.on_implement.clone(), |row, f| {
                                row.child(
                                    Button::new((id.clone(), "implement"), texts.implement.clone())
                                        .size(ButtonSize::Sm)
                                        .on_click(move |_, w, cx| f(w, cx)),
                                )
                            }),
                    )
                },
            )
    }
}

/// A generic request from the agent. See the module docs.
#[derive(IntoElement)]
pub struct RequestCard {
    id: ElementId,
    icon: Icon,
    title: SharedString,
    body: Option<SharedString>,
    preview: Option<AnyElement>,
    primary: SharedString,
    secondary: SharedString,
    on_primary: Option<Handler>,
    on_secondary: Option<Handler>,
}

impl RequestCard {
    /// A request titled `title` with `primary` / `secondary` actions (the
    /// reference defaults: "Allow" / "Decline") and the `Link` icon.
    pub fn new(
        id: impl Into<ElementId>,
        title: impl Into<SharedString>,
        primary: impl Into<SharedString>,
        secondary: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            icon: Icon::carbon(CarbonIcon::Link),
            title: title.into(),
            body: None,
            preview: None,
            primary: primary.into(),
            secondary: secondary.into(),
            on_primary: None,
            on_secondary: None,
        }
    }

    /// `icon`.
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = icon.into();
        self
    }

    /// `children`: the note under the head.
    pub fn body(mut self, body: impl Into<SharedString>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// `preview`: content in the `layer-02` well.
    pub fn preview(mut self, preview: impl IntoElement) -> Self {
        self.preview = Some(preview.into_any_element());
        self
    }

    /// `onPrimary`.
    pub fn on_primary(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_primary = Some(Rc::new(f));
        self
    }

    /// `onSecondary`.
    pub fn on_secondary(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_secondary = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for RequestCard {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        card::frame()
            .child(
                card::head()
                    .child(
                        self.icon
                            .size_length(rem(16.))
                            .color(color(ColorRole::IconPrimary)),
                    )
                    .child(card::head_title(self.title.clone())),
            )
            .when_some(self.body, |this, b| this.child(card::note(b)))
            // `.t3-request-preview { padding: spacing-04; background: layer-02; font-size: 0.875rem }`.
            .when_some(self.preview, |this, p| {
                this.child(
                    div()
                        .p(rem(12.))
                        .bg(color(ColorRole::Layer02))
                        .text_size(rem(14.))
                        .line_height(rem(18.))
                        .child(p),
                )
            })
            .child(
                card::actions()
                    .child(
                        Button::new((id.clone(), "secondary"), self.secondary.clone())
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::Sm)
                            .when_some(self.on_secondary.clone(), |b, f| {
                                b.on_click(move |_, w, cx| f(w, cx))
                            }),
                    )
                    .child(
                        Button::new((id.clone(), "primary"), self.primary.clone())
                            .size(ButtonSize::Sm)
                            .when_some(self.on_primary.clone(), |b, f| {
                                b.on_click(move |_, w, cx| f(w, cx))
                            }),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    #[test]
    fn a_plan_step_says_where_it_stands_only_when_told() {
        assert_eq!(PlanStep::new("Extract tokens").status, None);
        let done = PlanStep::new("Extract tokens").status(PlanStatus::Done);
        assert_eq!(done.status, Some(PlanStatus::Done));
        // A done step is struck through, as the checklist draws it.
        crate::theme::tests_set(CarbonTheme::White);
        assert!(PlanStatus::Done.look().2);
    }

    fn questions() -> Vec<InputQuestion> {
        vec![
            InputQuestion::new(
                "Theme?",
                vec![
                    InputOption::new("White"),
                    InputOption::new("G10").default_choice(true),
                ],
            )
            .allow_other(true),
            InputQuestion::new(
                "Flag?",
                vec![InputOption::new("Yes"), InputOption::new("No")],
            ),
        ]
    }

    #[test]
    fn defaults_seed_the_answers() {
        let p = InputProgress::new(&questions());
        assert_eq!(p.answers.get(&0), Some(&Answer::Option("G10".into())));
        assert!(p.ready(0, ""));
        assert!(!p.ready(1, ""));
        let mut p = p;
        p.answers.insert(0, Answer::Other);
        assert!(!p.ready(0, "   "));
        assert!(p.ready(0, "Sepia"));
    }

    #[test]
    fn number_keys_pick_and_enter_submits() {
        assert_eq!(
            input_key("1", false, false, 3, true, false),
            Some(InputKey::Pick(0))
        );
        assert_eq!(
            input_key("3", false, false, 3, true, false),
            Some(InputKey::Pick(2))
        );
        assert_eq!(
            input_key("4", false, false, 3, true, false),
            Some(InputKey::Other)
        );
        assert_eq!(input_key("4", false, false, 3, false, false), None);
        assert_eq!(
            input_key("enter", false, false, 3, true, true),
            Some(InputKey::Submit)
        );
        assert_eq!(input_key("enter", true, false, 3, true, false), None);
        // Typing the free answer: digits are text.
        assert_eq!(input_key("1", false, false, 3, true, true), None);
        assert_eq!(input_key("1", false, true, 3, true, false), None);
    }

    #[test]
    fn chosen_options_get_the_interactive_frame() {
        crate::theme::tests_set(CarbonTheme::White);
        let (fill, frame) = option_colors(false, true);
        assert_eq!(fill, color(ColorRole::Layer02));
        assert_eq!(frame, Some(color(ColorRole::BorderInteractive)));
        assert_eq!(option_colors(true, false).0, color(ColorRole::LayerHover02));
    }
}
