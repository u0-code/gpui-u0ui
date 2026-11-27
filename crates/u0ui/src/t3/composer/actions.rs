//! [`ComposerPrimaryActions`] (`.t3-primary`): the composer's send area.
//!
//! - `idle`: a primary `md` Send icon button.
//! - `running`: blue stays the forward action, so Queue is a primary button
//!   with the queue icon and "Queue ↵", and Stop is a neutral outlined
//!   square, 8px apart (craft pass, gemini-A).
//! - `running` with Steer: Steer (ghost, ⌘↵), Queue (the one primary, ↵) and
//!   a labelled dark Stop, all 32px (r2-gal); key caps follow the button's
//!   ink (gemini-G).
//! - `answering`: a primary `md` text button ("Submit answer").
//! - `submitting`: the same, disabled ("Submitting...").
use crate::components::button::{Button, ButtonKind, ButtonSize, ButtonVariant};
use crate::t3::action::{ActionButton, ActionKind, KeyCap, Part};
use crate::t3::press::{Handler, rem};
use crate::{density, icons::CarbonIcon};
use gpui_kit::{
    App, ElementId, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
};
use std::rc::Rc;

/// `ComposerPrimaryActions` `state`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ComposerState {
    /// Ready to send.
    #[default]
    Idle,
    /// The agent is running.
    Running,
    /// Answering the agent's question.
    Answering,
    /// Sending the answer.
    Submitting,
}

impl ComposerState {
    /// Every state.
    pub const ALL: [ComposerState; 4] =
        [Self::Idle, Self::Running, Self::Answering, Self::Submitting];

    /// The reference name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Running => "running",
            Self::Answering => "answering",
            Self::Submitting => "submitting",
        }
    }
}

/// The primary actions' strings (the reference's English defaults).
#[derive(Clone)]
pub struct PrimaryActionTexts {
    /// Send (↵): the idle icon button's tooltip.
    pub send: SharedString,
    /// Queue message (⌘↵): the running Queue's tooltip.
    pub queue_message: SharedString,
    /// The running Queue's text ("Queue ↵").
    pub queue_inline: SharedString,
    /// Stop (Esc): the running Stop's tooltip.
    pub stop_short: SharedString,
    /// Steer.
    pub steer: SharedString,
    /// Steer's tooltip.
    pub steer_title: SharedString,
    /// Queue.
    pub queue: SharedString,
    /// Queue's tooltip.
    pub queue_title: SharedString,
    /// Stop.
    pub stop: SharedString,
    /// The labelled Stop's tooltip.
    pub stop_title: SharedString,
    /// Submit answer (`answerLabel` overrides it).
    pub answer: SharedString,
    /// Submitting...
    pub submitting: SharedString,
}

impl Default for PrimaryActionTexts {
    fn default() -> Self {
        Self {
            send: "Send (↵)".into(),
            queue_message: "Queue message (⌘↵)".into(),
            queue_inline: "Queue ↵".into(),
            stop_short: "Stop (Esc)".into(),
            steer: "Steer".into(),
            steer_title: "Interrupt the current step and send now".into(),
            queue: "Queue".into(),
            queue_title: "Send after this turn finishes".into(),
            stop: "Stop".into(),
            stop_title: "Stop the agent (Esc)".into(),
            answer: "Submit answer".into(),
            submitting: "Submitting...".into(),
        }
    }
}

/// The composer's send / queue / stop area. See the module docs.
#[derive(IntoElement)]
pub struct ComposerPrimaryActions {
    id: ElementId,
    state: ComposerState,
    disabled: bool,
    empty: bool,
    answer_label: Option<SharedString>,
    texts: PrimaryActionTexts,
    on_send: Option<Handler>,
    on_stop: Option<Handler>,
    on_queue: Option<Handler>,
    on_steer: Option<Handler>,
}

impl ComposerPrimaryActions {
    /// The actions for `state`.
    pub fn new(id: impl Into<ElementId>, state: ComposerState) -> Self {
        Self {
            id: id.into(),
            state,
            disabled: false,
            empty: false,
            answer_label: None,
            texts: PrimaryActionTexts::default(),
            on_send: None,
            on_stop: None,
            on_queue: None,
            on_steer: None,
        }
    }

    /// `disabled`: the idle Send.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// `empty`: nothing typed, so Steer and Queue are disabled.
    pub fn empty(mut self, empty: bool) -> Self {
        self.empty = empty;
        self
    }

    /// `answerLabel`.
    pub fn answer_label(mut self, label: impl Into<SharedString>) -> Self {
        self.answer_label = Some(label.into());
        self
    }

    /// Replace the strings.
    pub fn texts(mut self, texts: PrimaryActionTexts) -> Self {
        self.texts = texts;
        self
    }

    /// `onSend` (idle and answering).
    pub fn on_send(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_send = Some(Rc::new(f));
        self
    }

    /// `onStop`.
    pub fn on_stop(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_stop = Some(Rc::new(f));
        self
    }

    /// `onQueue`.
    pub fn on_queue(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_queue = Some(Rc::new(f));
        self
    }

    /// `onSteer`: switches the running layout to Steer / Queue / Stop.
    pub fn on_steer(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_steer = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for ComposerPrimaryActions {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let t = self.texts.clone();
        let md = ButtonSize::Md.height(density::current());
        let click = |h: Option<Handler>| {
            move |_: &gpui_kit::ClickEvent, w: &mut Window, cx: &mut App| {
                if let Some(f) = h.as_ref() {
                    f(w, cx)
                }
            }
        };
        match self.state {
            ComposerState::Running if self.on_steer.is_some() => div()
                .flex()
                .flex_row()
                .items_center()
                // `.t3-primary--steer { gap: spacing-03 }`.
                .gap(rem(8.))
                .child(
                    ActionButton::new(
                        (id.clone(), "steer"),
                        t.steer.clone(),
                        ActionKind::Carbon(ButtonKind::Ghost),
                    )
                    .part(Part::Text(t.steer.clone()))
                    .part(Part::Key("⌘↵".into(), KeyCap::Bare))
                    .tooltip(t.steer_title.clone())
                    .disabled(self.empty)
                    .on_click(self.on_steer.clone()),
                )
                .child(
                    ActionButton::new(
                        (id.clone(), "queue"),
                        t.queue.clone(),
                        ActionKind::Carbon(ButtonKind::Primary),
                    )
                    .part(Part::Text(t.queue.clone()))
                    .part(Part::Key("↵".into(), KeyCap::Bare))
                    .tooltip(t.queue_title.clone())
                    .disabled(self.empty)
                    .on_click(self.on_queue.clone()),
                )
                // `.t3-stop--labelled { background: button-secondary; gap: spacing-03;
                // padding-inline: spacing-04 spacing-05 }`.
                .child(
                    ActionButton::new(
                        (id.clone(), "stop"),
                        t.stop.clone(),
                        ActionKind::Carbon(ButtonKind::Secondary),
                    )
                    .padding(12., 16.)
                    .gap(8.)
                    .part(Part::Icon(CarbonIcon::StopFilled))
                    .part(Part::Text(t.stop.clone()))
                    .tooltip(t.stop_title.clone())
                    .on_click(self.on_stop.clone()),
                )
                .into_any_element(),
            ComposerState::Running => div()
                .flex()
                .flex_row()
                .items_center()
                // `.t3-primary { gap: spacing-03 }` (gemini-A).
                .gap(rem(8.))
                // `.t3-primary .cds--btn--secondary.cds--btn--icon-only { width: auto;
                // padding: 0 spacing-05 0 spacing-04; gap: spacing-03; background: button-primary }`
                // with `::after { content: "Queue ↵" }`.
                .child(
                    ActionButton::new(
                        (id.clone(), "queue"),
                        t.queue_message.clone(),
                        ActionKind::Carbon(ButtonKind::Primary),
                    )
                    .height(md)
                    .padding(12., 16.)
                    .gap(8.)
                    .part(Part::Icon(CarbonIcon::ListChecked))
                    .part(Part::Text(t.queue_inline.clone()))
                    .tooltip(t.queue_message.clone())
                    .on_click(self.on_queue.clone()),
                )
                // `.t3-primary .t3-stop { background: transparent; box-shadow: inset 0 0 0 1px border-strong-01 }`.
                .child(
                    ActionButton::new(
                        (id.clone(), "stop"),
                        t.stop_short.clone(),
                        ActionKind::NeutralOutline,
                    )
                    .height(md)
                    .padding(0., 0.)
                    .part(Part::Icon(CarbonIcon::StopFilled))
                    .square(true)
                    .tooltip(t.stop_short.clone())
                    .on_click(self.on_stop.clone()),
                )
                .into_any_element(),
            ComposerState::Answering => Button::new(
                (id.clone(), "answer"),
                self.answer_label.clone().unwrap_or(t.answer.clone()),
            )
            .size(ButtonSize::Md)
            .balanced(true)
            .on_click(click(self.on_send.clone()))
            .into_any_element(),
            ComposerState::Submitting => {
                Button::new((id.clone(), "submitting"), t.submitting.clone())
                    .size(ButtonSize::Md)
                    .balanced(true)
                    .disabled(true)
                    .into_any_element()
            }
            ComposerState::Idle => Button::new((id.clone(), "send"), t.send.clone())
                .variant(ButtonVariant::Primary)
                .icon(CarbonIcon::SendFilled)
                .icon_only(true)
                .size(ButtonSize::Md)
                .disabled(self.disabled)
                .on_click(click(self.on_send.clone()))
                .into_any_element(),
        }
    }
}
