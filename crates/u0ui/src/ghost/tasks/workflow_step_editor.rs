//! WorkflowStepEditor: a workflow's steps as numbered cards
//! (`WorkflowStepEditor` in `src/ghost/admin.jsx`, `.g-wfe` in
//! `src/ghost.css`): agent steps (prompt, agent, mode), command steps
//! (command, continue on failure) and question steps (question, answers).
//!
//! - **Behavior**: each card moves up, down (the keyboard alternative to drag
//!   ordering) or is removed; the row underneath adds an Agent, Command or
//!   Question step. Text edits use Kit's editors; the agent and mode are
//!   ghost menu buttons; answers are a [`TagInput`]. The steps are the
//!   editor's own state, seeded from [`WorkflowStepEditor::steps`]; every
//!   change reports the whole list ([`WorkflowStepEditor::on_change`]).
//!
//! ```ignore
//! WorkflowStepEditor::new("wfe").steps([WorkflowStep::agent("Claude", "Summarise…")])
//! ```
use crate::{
    components::{
        Button, ButtonSize, ButtonVariant, DropdownMenu, DropdownMenuKind, DropdownMenuSize,
        MenuItem,
    },
    ghost::{
        fields::{self, InlineCheck},
        primitives::TagInput,
        rows as row,
    },
    icons::{CarbonIcon, Icon},
    t3::craft::{rem, sans},
    theme::color,
    tokens::{self, ColorRole},
};
use gpui_kit::{
    AnyElement, App, AppContext as _, Context, ElementId, Entity, IntoElement, ParentElement,
    RenderOnce, SharedString, Styled, Subscription, Window,
    component::input::{InputEvent, InputState, TextareaState},
    div,
};
use std::rc::Rc;

/// A step's type (`type`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum StepType {
    /// `agent`: a prompt for an agent.
    #[default]
    Agent,
    /// `command`: a shell command.
    Command,
    /// `question`: ask the user.
    Question,
}

impl StepType {
    /// Every type, in the add row's order.
    pub const ALL: [Self; 3] = [Self::Agent, Self::Command, Self::Question];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Agent => "agent",
            Self::Command => "command",
            Self::Question => "question",
        }
    }

    /// `STEP[type][0]`: `Bot`, `Terminal`, `Help`.
    pub const fn icon(self) -> CarbonIcon {
        match self {
            Self::Agent => CarbonIcon::Bot,
            Self::Command => CarbonIcon::Terminal,
            Self::Question => CarbonIcon::Help,
        }
    }
}

/// One step (`steps[]`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WorkflowStep {
    /// `type`.
    pub kind: StepType,
    /// `agent` (agent steps; default the first agent).
    pub agent: Option<SharedString>,
    /// `prompt` (agent steps).
    pub prompt: SharedString,
    /// `mode` (agent steps; default the first mode).
    pub mode: Option<SharedString>,
    /// `command` (command steps).
    pub command: SharedString,
    /// `continueOnError` (command steps).
    pub continue_on_error: bool,
    /// `question` (question steps).
    pub question: SharedString,
    /// `answers` (question steps).
    pub answers: Vec<SharedString>,
}

impl WorkflowStep {
    /// An empty step of `kind` (the add row's `withId({ type })`).
    pub fn new(kind: StepType) -> Self {
        Self {
            kind,
            ..Self::default()
        }
    }

    /// An agent step.
    pub fn agent(agent: impl Into<SharedString>, prompt: impl Into<SharedString>) -> Self {
        Self {
            agent: Some(agent.into()),
            prompt: prompt.into(),
            ..Self::new(StepType::Agent)
        }
    }

    /// A command step.
    pub fn command(command: impl Into<SharedString>) -> Self {
        Self {
            command: command.into(),
            ..Self::new(StepType::Command)
        }
    }

    /// A question step.
    pub fn question(
        question: impl Into<SharedString>,
        answers: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        Self {
            question: question.into(),
            answers: answers.into_iter().map(Into::into).collect(),
            ..Self::new(StepType::Question)
        }
    }

    /// With a mode (agent steps).
    pub fn with_mode(mut self, mode: impl Into<SharedString>) -> Self {
        self.mode = Some(mode.into());
        self
    }

    /// With continue-on-failure (command steps).
    pub fn with_continue_on_error(mut self, on: bool) -> Self {
        self.continue_on_error = on;
        self
    }
}

/// `mv(i, d)`: move item `i` by `d` (−1 up, +1 down); out of range is a no-op.
pub fn move_step<T>(items: &mut Vec<T>, i: usize, d: isize) {
    let j = i as isize + d;
    if i < items.len() && j >= 0 && (j as usize) < items.len() {
        let x = items.remove(i);
        items.insert(j as usize, x);
    }
}

/// The editor's user-facing strings (the reference's English defaults).
#[derive(Clone)]
pub struct WorkflowStepEditorTexts {
    /// The step titles (`STEP[type][1]`): Agent / Command / Question step.
    pub titles: [SharedString; 3],
    /// The add buttons: Agent / Command / Question.
    pub add: [SharedString; 3],
    /// `"Add step"`.
    pub add_step: SharedString,
    /// `"Move up"`.
    pub move_up: SharedString,
    /// `"Move down"`.
    pub move_down: SharedString,
    /// `"Remove step"`.
    pub remove: SharedString,
    /// The mode menu's label: `"Mode: {mode}"`.
    pub mode: Rc<dyn Fn(&str) -> SharedString>,
    /// `"Continue on failure"`.
    pub continue_on_failure: SharedString,
    /// `"Answers"`.
    pub answers: SharedString,
    /// `"Add an answer…"`.
    pub answer_placeholder: SharedString,
}

impl Default for WorkflowStepEditorTexts {
    fn default() -> Self {
        Self {
            titles: [
                "Agent step".into(),
                "Command step".into(),
                "Question step".into(),
            ],
            add: ["Agent".into(), "Command".into(), "Question".into()],
            add_step: "Add step".into(),
            move_up: "Move up".into(),
            move_down: "Move down".into(),
            remove: "Remove step".into(),
            mode: Rc::new(|m| format!("Mode: {m}").into()),
            continue_on_failure: "Continue on failure".into(),
            answers: "Answers".into(),
            answer_placeholder: "Add an answer…".into(),
        }
    }
}

type ChangeHandler = Rc<dyn Fn(Vec<WorkflowStep>, &mut Window, &mut App)>;

struct StepModel {
    key: u64,
    data: WorkflowStep,
    prompt: Entity<TextareaState>,
    command: Entity<InputState>,
    question: Entity<InputState>,
    _subs: [Subscription; 3],
}

struct Model {
    steps: Vec<StepModel>,
    seq: u64,
    on_change: Option<ChangeHandler>,
}

impl Model {
    fn snapshot(&self, cx: &App) -> Vec<WorkflowStep> {
        self.steps
            .iter()
            .map(|s| WorkflowStep {
                prompt: s.prompt.read(cx).value(),
                command: s.command.read(cx).value(),
                question: s.question.read(cx).value(),
                ..s.data.clone()
            })
            .collect()
    }

    fn emit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(f) = self.on_change.clone() {
            let steps = self.snapshot(cx);
            f(steps, window, cx);
        }
        cx.notify();
    }

    fn push(&mut self, data: WorkflowStep, window: &mut Window, cx: &mut Context<Self>) {
        self.seq += 1;
        let prompt = cx.new(|cx| TextareaState::new(window, cx).default_value(data.prompt.clone()));
        let command = cx.new(|cx| InputState::new(window, cx).default_value(data.command.clone()));
        let question =
            cx.new(|cx| InputState::new(window, cx).default_value(data.question.clone()));
        let on_edit = |m: &mut Model, e: &InputEvent, w: &mut Window, cx: &mut Context<Model>| {
            if matches!(e, InputEvent::Change) {
                m.emit(w, cx);
            }
        };
        let subs = [
            cx.subscribe_in(&prompt, window, move |m, _, e, w, cx| on_edit(m, e, w, cx)),
            cx.subscribe_in(&command, window, move |m, _, e, w, cx| on_edit(m, e, w, cx)),
            cx.subscribe_in(&question, window, move |m, _, e, w, cx| {
                on_edit(m, e, w, cx)
            }),
        ];
        self.steps.push(StepModel {
            key: self.seq,
            data,
            prompt,
            command,
            question,
            _subs: subs,
        });
    }

    fn patch(
        &mut self,
        key: u64,
        f: impl FnOnce(&mut WorkflowStep),
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(s) = self.steps.iter_mut().find(|s| s.key == key) {
            f(&mut s.data);
        }
        self.emit(window, cx);
    }
}

/// A workflow step editor. See the module docs.
#[derive(IntoElement)]
pub struct WorkflowStepEditor {
    id: ElementId,
    steps: Vec<WorkflowStep>,
    agents: Vec<SharedString>,
    modes: Vec<SharedString>,
    texts: WorkflowStepEditorTexts,
    on_change: Option<ChangeHandler>,
}

impl WorkflowStepEditor {
    /// An editor with no steps, agents Claude / Codex / Gemini and modes
    /// Default / Plan / Auto-accept edits (the reference's defaults).
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            steps: Vec::new(),
            agents: vec!["Claude".into(), "Codex".into(), "Gemini".into()],
            modes: vec!["Default".into(), "Plan".into(), "Auto-accept edits".into()],
            texts: WorkflowStepEditorTexts::default(),
            on_change: None,
        }
    }

    /// The starting steps (`steps`).
    pub fn steps(mut self, steps: impl IntoIterator<Item = WorkflowStep>) -> Self {
        self.steps = steps.into_iter().collect();
        self
    }

    /// The agents an agent step can use (`agents`).
    pub fn agents(mut self, agents: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.agents = agents.into_iter().map(Into::into).collect();
        self
    }

    /// The modes an agent step can run in (`modes`).
    pub fn modes(mut self, modes: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.modes = modes.into_iter().map(Into::into).collect();
        self
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: WorkflowStepEditorTexts) -> Self {
        self.texts = texts;
        self
    }

    /// The steps changed (`onChange`).
    pub fn on_change(
        mut self,
        f: impl Fn(Vec<WorkflowStep>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
}

fn key_id(id: &ElementId, key: u64, part: &str) -> ElementId {
    (id.clone(), SharedString::from(format!("{part}-{key}"))).into()
}

impl RenderOnce for WorkflowStepEditor {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let seed = self.steps.clone();
        let model = window.use_keyed_state((id.clone(), "wfe"), cx, |window, cx| {
            let mut m = Model {
                steps: Vec::new(),
                seq: 0,
                on_change: None,
            };
            for s in seed {
                m.push(s, window, cx);
            }
            m
        });
        model.update(cx, |m, _| m.on_change = self.on_change.clone());
        let n = model.read(cx).steps.len();
        let texts = self.texts.clone();
        let first_agent = self.agents.first().cloned().unwrap_or_default();
        let first_mode = self.modes.first().cloned().unwrap_or_default();
        let fill = fields::field_fill(true);

        let cards: Vec<AnyElement> = (0..n)
            .map(|i| {
                let (key, data, prompt, command, question) = {
                    let s = &model.read(cx).steps[i];
                    (
                        s.key,
                        s.data.clone(),
                        s.prompt.clone(),
                        s.command.clone(),
                        s.question.clone(),
                    )
                };
                let kind_ix = StepType::ALL
                    .iter()
                    .position(|k| *k == data.kind)
                    .unwrap_or(0);
                let title = texts.titles[kind_ix].clone();
                let icon_button = |part: &str, label: SharedString, icon: CarbonIcon| {
                    Button::new(key_id(&id, key, part), label)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .icon(icon)
                        .icon_only(true)
                };
                let mv = |d: isize| {
                    let model = model.clone();
                    move |_: &gpui_kit::ClickEvent, w: &mut Window, cx: &mut App| {
                        model.update(cx, |m, cx| {
                            if let Some(i) = m.steps.iter().position(|s| s.key == key) {
                                move_step(&mut m.steps, i, d);
                            }
                            m.emit(w, cx);
                        })
                    }
                };
                let remove = {
                    let model = model.clone();
                    move |_: &gpui_kit::ClickEvent, w: &mut Window, cx: &mut App| {
                        model.update(cx, |m, cx| {
                            m.steps.retain(|s| s.key != key);
                            m.emit(w, cx);
                        })
                    }
                };
                // `.g-wfe-head { display: flex; align-items: center; gap:
                // var(--spacing-03); height: var(--density-row-lg); font-size:
                // 0.875rem }`, `> svg { fill: icon-secondary }`, `b { 600 }`.
                let head = sans(div(), 14., 400)
                    .flex()
                    .items_center()
                    .gap(tokens::SPACING_03)
                    .h(crate::density::current().row_lg())
                    .text_color(color(ColorRole::TextPrimary))
                    .child(
                        Icon::carbon(data.kind.icon())
                            .size_length(rem(16.))
                            .color(color(ColorRole::IconSecondary)),
                    )
                    .child(sans(div(), 14., 600).child(title))
                    .child(div().flex_1())
                    .child(
                        icon_button("up", texts.move_up.clone(), CarbonIcon::ChevronUp)
                            .disabled(i == 0)
                            .on_click(mv(-1)),
                    )
                    .child(
                        icon_button("down", texts.move_down.clone(), CarbonIcon::ChevronDown)
                            .disabled(i + 1 == n)
                            .on_click(mv(1)),
                    )
                    .child(
                        icon_button("rm", texts.remove.clone(), CarbonIcon::TrashCan)
                            .on_click(remove),
                    );

                // `.g-wfe-card .g-in, .g-wfe-card .g-ta { width: calc(100% -
                // var(--spacing-03)); background: var(--field-02) }`.
                let inset = |el: AnyElement| div().pr(tokens::SPACING_03).child(el);
                let body: Vec<AnyElement> = match data.kind {
                    StepType::Agent => {
                        let agent = data.agent.clone().unwrap_or_else(|| first_agent.clone());
                        let mode = data.mode.clone().unwrap_or_else(|| first_mode.clone());
                        let pick = |field: fn(&mut WorkflowStep, SharedString), v: SharedString| {
                            let model = model.clone();
                            move |w: &mut Window, cx: &mut App| {
                                let v = v.clone();
                                model.update(cx, |m, cx| m.patch(key, |s| field(s, v), w, cx))
                            }
                        };
                        let agents = self.agents.iter().map(|a| {
                            MenuItem::item(a.clone())
                                .on_select(pick(|s, v| s.agent = Some(v), a.clone()))
                        });
                        let modes = self.modes.iter().map(|m| {
                            MenuItem::item(m.clone())
                                .on_select(pick(|s, v| s.mode = Some(v), m.clone()))
                        });
                        vec![
                            inset(
                                fields::text_area(
                                    &prompt,
                                    2,
                                    fill,
                                    Some(format!("Step {} prompt", i + 1).into()),
                                    window,
                                    cx,
                                )
                                .into_any_element(),
                            )
                            .into_any_element(),
                            // `.g-wfe-opts { display: flex; gap: var(--spacing-03); flex-wrap: wrap }`.
                            div()
                                .flex()
                                .flex_wrap()
                                .gap(tokens::SPACING_03)
                                .child(
                                    DropdownMenu::new(key_id(&id, key, "agent"))
                                        .label(agent)
                                        .kind(DropdownMenuKind::Ghost)
                                        .size(DropdownMenuSize::Sm)
                                        .items(agents),
                                )
                                .child(
                                    DropdownMenu::new(key_id(&id, key, "mode"))
                                        .label((texts.mode)(&mode))
                                        .kind(DropdownMenuKind::Ghost)
                                        .size(DropdownMenuSize::Sm)
                                        .items(modes),
                                )
                                .into_any_element(),
                        ]
                    }
                    StepType::Command => {
                        let model = model.clone();
                        let on = data.continue_on_error;
                        vec![
                            inset(
                                fields::line_input(&command, true, fill, None, window, cx)
                                    .into_any_element(),
                            )
                            .into_any_element(),
                            div()
                                .flex()
                                .flex_wrap()
                                .gap(tokens::SPACING_03)
                                .child(
                                    InlineCheck::new(key_id(&id, key, "continue"))
                                        .checked(on)
                                        .label(texts.continue_on_failure.clone())
                                        .on_change(move |v, w, cx| {
                                            model.update(cx, |m, cx| {
                                                m.patch(key, |s| s.continue_on_error = v, w, cx)
                                            })
                                        }),
                                )
                                .into_any_element(),
                        ]
                    }
                    StepType::Question => {
                        let model = model.clone();
                        vec![
                            inset(
                                fields::line_input(&question, false, fill, None, window, cx)
                                    .into_any_element(),
                            )
                            .into_any_element(),
                            inset(
                                TagInput::new(key_id(&id, key, "answers"))
                                    .label(texts.answers.clone())
                                    .value(data.answers.clone())
                                    .placeholder(texts.answer_placeholder.clone())
                                    .fill(fill)
                                    .on_value_change(move |a, w, cx| {
                                        model.update(cx, |m, cx| {
                                            m.patch(key, |s| s.answers = a, w, cx)
                                        })
                                    })
                                    .into_any_element(),
                            )
                            .into_any_element(),
                        ]
                    }
                };

                // `.g-wfe-step { display: flex; gap: var(--spacing-04) }`.
                div()
                    .flex()
                    .gap(tokens::SPACING_04)
                    .child(
                        // `.g-wfe-n { width/height: 1.5rem; margin-top:
                        // var(--spacing-03); background: var(--layer-accent-01);
                        // font-size: 0.75rem; font-weight: 600; border-radius: 50% }`.
                        sans(div(), 12., 600)
                            .flex()
                            .flex_none()
                            .items_center()
                            .justify_center()
                            .size(rem(24.))
                            .mt(tokens::SPACING_03)
                            .rounded_full()
                            .bg(color(ColorRole::LayerAccent01))
                            .text_color(color(ColorRole::TextPrimary))
                            .child(format!("{}", i + 1)),
                    )
                    .child(
                        // `.g-wfe-card { display: flex; flex: 1; min-width: 0;
                        // flex-direction: column; gap: var(--spacing-03); padding:
                        // 0 var(--spacing-02) var(--spacing-04) var(--spacing-04);
                        // background: var(--layer-01) }`.
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_w_0()
                            .gap(tokens::SPACING_03)
                            .pr(tokens::SPACING_02)
                            .pb(tokens::SPACING_04)
                            .pl(tokens::SPACING_04)
                            .bg(color(ColorRole::Layer01))
                            .child(head)
                            .children(body),
                    )
                    .into_any_element()
            })
            .collect();

        let adds = StepType::ALL.into_iter().enumerate().map(|(k, kind)| {
            let model = model.clone();
            Button::new(
                (
                    id.clone(),
                    SharedString::from(format!("add-{}", kind.name())),
                ),
                texts.add[k].clone(),
            )
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Sm)
            .icon(kind.icon())
            .on_click(move |_, w, cx| {
                model.update(cx, |m, cx| {
                    m.push(WorkflowStep::new(kind), w, cx);
                    m.emit(w, cx);
                })
            })
        });

        // `.g-wfe { display: flex; flex-direction: column; gap: var(--spacing-03) }`.
        div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_03)
            .w_full()
            .children(cards)
            .child(
                // `.g-wfe-add { display: flex; align-items: center; gap:
                // var(--spacing-02); padding-left: 2.5rem; font-size: 0.75rem }`.
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap(tokens::SPACING_02)
                    .pl(rem(40.))
                    .child(row::dim(texts.add_step.clone(), 12.))
                    .children(adds),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_step_follows_mv() {
        let mut v = vec![1, 2, 3];
        move_step(&mut v, 0, 1);
        assert_eq!(v, vec![2, 1, 3]);
        move_step(&mut v, 2, -1);
        assert_eq!(v, vec![2, 3, 1]);
        move_step(&mut v, 0, -1);
        assert_eq!(v, vec![2, 3, 1], "first can't move up");
        move_step(&mut v, 2, 1);
        assert_eq!(v, vec![2, 3, 1], "last can't move down");
    }

    #[test]
    fn step_types_map_to_reference_glyphs() {
        assert_eq!(StepType::Agent.icon(), CarbonIcon::Bot);
        assert_eq!(StepType::Command.icon(), CarbonIcon::Terminal);
        assert_eq!(StepType::Question.icon(), CarbonIcon::Help);
        let t = WorkflowStepEditorTexts::default();
        assert_eq!((t.mode)("Plan"), SharedString::from("Mode: Plan"));
    }

    #[test]
    fn builders_fill_the_step() {
        let s = WorkflowStep::agent("Claude", "Summarise").with_mode("Plan");
        assert_eq!(s.kind, StepType::Agent);
        assert_eq!(s.mode.as_deref(), Some("Plan"));
        let s = WorkflowStep::question("Ship?", ["Yes", "Edit first"]);
        assert_eq!(s.answers.len(), 2);
    }
}
