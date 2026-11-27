//! ComposerCompact: the one-line side-chat input (`ComposerCompact`,
//! `.g-ccompact*`).
//!
//! A `density-field-md` field (`field-01`, `border-strong-01` bottom rule,
//! 2px `focus` outline while focused within): the agent's 16px provider
//! mark, the input, a 24px attach button and a 24px `button-primary` send
//! square (`button-disabled` while the text is blank). Enter sends (Kit's
//! input does not report Enter mid-composition) and clears the field; a
//! multi-line message belongs in the full composer.
use crate::{
    density,
    ghost::primitives::{
        IconButton,
        parts::{editor, focus_ring, focus_visible, keyed_focus},
    },
    icons::{CarbonIcon, Icon},
    t3::{
        craft::{rem, sans},
        shell::{IdentitySize, ProviderIcon},
    },
    theme::color,
    tokens::{self, ColorRole},
};
use gpui_kit::{
    App, AppContext as _, Context, CursorStyle, ElementId, Entity, Focusable as _, Hsla,
    InteractiveElement as _, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    Subscription, Window,
    component::input::{InputEvent, InputState},
    div,
    prelude::FluentBuilder as _,
};
use std::rc::Rc;

/// The send square's (fill, glyph): `button-primary` / `text-on-color`,
/// disabled `button-disabled` / `text-on-color-disabled`
/// (`.g-ccompact-send`, `:disabled`).
pub fn send_colors(enabled: bool) -> (Hsla, Hsla) {
    if enabled {
        (
            color(ColorRole::ButtonPrimary),
            color(ColorRole::TextOnColor),
        )
    } else {
        (
            color(ColorRole::ButtonDisabled),
            color(ColorRole::TextOnColorDisabled),
        )
    }
}

/// The input's user-facing strings (the reference's defaults).
#[derive(Clone, Debug)]
pub struct ComposerCompactTexts {
    /// `"Ask on the side…"`.
    pub placeholder: SharedString,
    /// The input's name: `"Side chat message"`.
    pub message: SharedString,
    /// `"Attach"`.
    pub attach: SharedString,
    /// `"Send"`.
    pub send: SharedString,
}

impl Default for ComposerCompactTexts {
    fn default() -> Self {
        Self {
            placeholder: "Ask on the side…".into(),
            message: "Side chat message".into(),
            attach: "Attach".into(),
            send: "Send".into(),
        }
    }
}

type SendHandler = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;
type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

struct Model {
    input: Entity<InputState>,
    on_send: Option<SendHandler>,
    _subscription: Subscription,
}

impl Model {
    /// `onSend(t); setT('')` when the text is not blank.
    fn send(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let text = self.input.read(cx).value().to_string();
        if text.trim().is_empty() {
            return;
        }
        if let Some(f) = self.on_send.clone() {
            f(text.into(), window, cx);
        }
        self.input.update(cx, |s, cx| s.set_value("", window, cx));
        cx.notify();
    }
}

/// The side-chat input. See the module docs.
#[derive(IntoElement)]
pub struct ComposerCompact {
    id: ElementId,
    agent: SharedString,
    on_send: Option<SendHandler>,
    on_attach: Option<Handler>,
    texts: ComposerCompactTexts,
}

impl ComposerCompact {
    /// An input talking to `agent` (`agent`, the reference's "Agent").
    pub fn new(id: impl Into<ElementId>, agent: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            agent: agent.into(),
            on_send: None,
            on_attach: None,
            texts: ComposerCompactTexts::default(),
        }
    }

    /// The placeholder (`placeholder`).
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.texts.placeholder = placeholder.into();
        self
    }

    /// A message was sent (`onSend(text)`).
    pub fn on_send(mut self, f: impl Fn(SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_send = Some(Rc::new(f));
        self
    }

    /// Attach pressed.
    pub fn on_attach(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_attach = Some(Rc::new(f));
        self
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: ComposerCompactTexts) -> Self {
        self.texts = texts;
        self
    }
}

impl RenderOnce for ComposerCompact {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let model = window.use_keyed_state((id.clone(), "model"), cx, |window, cx| {
            let input = cx.new(|cx| InputState::new(window, cx));
            let subscription =
                cx.subscribe_in(&input, window, |m: &mut Model, _, e, window, cx| match e {
                    InputEvent::PressEnter { .. } => m.send(window, cx),
                    InputEvent::Change => cx.notify(),
                    _ => {}
                });
            Model {
                input,
                on_send: None,
                _subscription: subscription,
            }
        });
        model.update(cx, |m, _| m.on_send = self.on_send.clone());
        let input = model.read(cx).input.clone();
        let text = input.read(cx).value().to_string();
        let enabled = !text.trim().is_empty();
        let focused = input.read(cx).focus_handle(cx).is_focused(window);
        let send_focus = keyed_focus((id.clone(), "send-focus"), window, cx);
        let send_ring = focus_visible(&send_focus, window);
        let (fill, ink) = send_colors(enabled);
        let placeholder = text.is_empty().then(|| {
            sans(div(), 14., 400)
                .absolute()
                .left_0()
                .top_0()
                .bottom_0()
                .flex()
                .items_center()
                .text_color(color(ColorRole::TextPlaceholder))
                .child(self.texts.placeholder.clone())
        });
        let send_model = model.clone();
        let attach = self.on_attach.clone();
        // `.g-ccompact { display: flex; align-items: center; gap:
        // var(--spacing-02); height: var(--density-field-md); padding: 0
        // var(--spacing-02) 0 var(--spacing-03); background: var(--field-01);
        // border-bottom: 1px solid var(--border-strong-01) }`.
        div()
            .id(id.clone())
            .relative()
            .flex()
            .items_center()
            .w_full()
            .gap(tokens::SPACING_02)
            .h(density::current().field())
            .pl(tokens::SPACING_03)
            .pr(tokens::SPACING_02)
            .bg(color(ColorRole::Field01))
            .border_b_1()
            .border_color(color(ColorRole::BorderStrong01))
            .child(
                ProviderIcon::new(self.agent.clone())
                    .id((id.clone(), "agent"))
                    .size(IdentitySize::Xs),
            )
            .child(
                // `.g-ccompact-input { flex: 1; font-size: 0.875rem }`.
                div()
                    .relative()
                    .flex()
                    .items_center()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .children(placeholder)
                    .child(editor(&input)),
            )
            .child(
                // `.g-ccompact .g-iconbtn { width: 1.5rem; height: 1.5rem }`.
                IconButton::new(
                    (id.clone(), "attach"),
                    CarbonIcon::Add,
                    self.texts.attach.clone(),
                )
                .size_px(24.)
                .on_click(move |_, w, cx| {
                    if let Some(f) = &attach {
                        f(w, cx);
                    }
                }),
            )
            .child(
                // `.g-ccompact-send { width: 1.5rem; height: 1.5rem; background:
                // var(--button-primary); color: var(--text-on-color) }`.
                gpui_kit::base::Button::new((id.clone(), "send"))
                    .track_focus(&send_focus)
                    .disabled(!enabled)
                    .accessibility_label(self.texts.send.clone())
                    .relative()
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .size(rem(24.))
                    .bg(fill)
                    .cursor(if enabled {
                        CursorStyle::PointingHand
                    } else {
                        CursorStyle::OperationNotAllowed
                    })
                    .on_click(move |_, w, cx| send_model.update(cx, |m, cx| m.send(w, cx)))
                    .child(
                        Icon::carbon(CarbonIcon::SendFilled)
                            .size_length(rem(14.))
                            .color(ink),
                    )
                    .when(send_ring, |b| b.child(focus_ring())),
            )
            // `.g-ccompact:focus-within { outline: 2px solid var(--focus); outline-offset: -2px }`.
            .when(focused, |d| d.child(focus_ring()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CarbonTheme, theme::color_in};

    #[test]
    fn send_square_follows_the_css() {
        crate::theme::tests_set(CarbonTheme::White);
        let c = |r| color_in(CarbonTheme::White, r);
        assert_eq!(
            send_colors(true),
            (c(ColorRole::ButtonPrimary), c(ColorRole::TextOnColor))
        );
        assert_eq!(
            send_colors(false),
            (
                c(ColorRole::ButtonDisabled),
                c(ColorRole::TextOnColorDisabled)
            )
        );
    }
}
