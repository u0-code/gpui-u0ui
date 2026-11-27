//! TurnFooter: the actions at the end of an agent turn (`TurnFooter`,
//! `.g-turnfoot` in `src/ghost.css`): Copy, Fork from here and Retry as
//! ghost `sm` icon buttons, then the model, token count and time.
//!
//! The row rests at 75% opacity and shows fully while hovered or while one
//! of its buttons has focus (`.g-turnfoot:hover, .g-turnfoot:focus-within`).
//!
//! ```ignore
//! TurnFooter::new("turn-3")
//!     .model("claude-sonnet")
//!     .tokens("12.4k tokens")
//!     .time("10:43")
//!     .on_copy(|_, _| copy_response())
//! ```
use crate::components::{Button, ButtonSize, ButtonVariant};
use crate::icons::{CarbonIcon, Icon};
use crate::t3::press;
use crate::{theme::color, tokens::ColorRole};
use gpui_kit::{
    App, ElementId, InteractiveElement as _, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement as _, Styled, Window, div, prelude::FluentBuilder as _,
};
use std::rc::Rc;

/// `.g-turnfoot { opacity: .75 }`.
pub const REST_OPACITY: f32 = 0.75;

/// The row's opacity: 1 while hovered or focused within
/// (`.g-turnfoot:hover, .g-turnfoot:focus-within { opacity: 1 }`), else
/// [`REST_OPACITY`].
pub fn turn_footer_opacity(hovered: bool, focus_within: bool) -> f32 {
    if hovered || focus_within {
        1.
    } else {
        REST_OPACITY
    }
}

/// The button names of a [`TurnFooter`] (hard-coded in the reference).
#[derive(Clone, Debug)]
pub struct TurnFooterTexts {
    /// `Copy response`.
    pub copy: SharedString,
    /// `Fork from here`.
    pub fork: SharedString,
    /// `Retry`.
    pub retry: SharedString,
}

impl Default for TurnFooterTexts {
    fn default() -> Self {
        Self {
            copy: "Copy response".into(),
            fork: "Fork from here".into(),
            retry: "Retry".into(),
        }
    }
}

type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

/// The actions and provenance under one agent turn. See the module docs.
#[derive(IntoElement)]
pub struct TurnFooter {
    id: ElementId,
    model: Option<SharedString>,
    tokens: Option<SharedString>,
    time: Option<SharedString>,
    on_copy: Option<Handler>,
    on_fork: Option<Handler>,
    on_retry: Option<Handler>,
    texts: TurnFooterTexts,
}

impl TurnFooter {
    /// An empty footer: the three buttons, no metadata.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            model: None,
            tokens: None,
            time: None,
            on_copy: None,
            on_fork: None,
            on_retry: None,
            texts: TurnFooterTexts::default(),
        }
    }

    /// `model`: the model that answered.
    pub fn model(mut self, model: impl Into<SharedString>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// `tokens`: the token count, already formatted (`12.4k tokens`).
    pub fn tokens(mut self, tokens: impl Into<SharedString>) -> Self {
        self.tokens = Some(tokens.into());
        self
    }

    /// `time`: when the turn ended.
    pub fn time(mut self, time: impl Into<SharedString>) -> Self {
        self.time = Some(time.into());
        self
    }

    /// `onCopy`.
    pub fn on_copy(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_copy = Some(Rc::new(f));
        self
    }

    /// `onFork`.
    pub fn on_fork(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_fork = Some(Rc::new(f));
        self
    }

    /// `onRetry`.
    pub fn on_retry(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_retry = Some(Rc::new(f));
        self
    }

    /// The button names.
    pub fn texts(mut self, texts: TurnFooterTexts) -> Self {
        self.texts = texts;
        self
    }
}

impl RenderOnce for TurnFooter {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id;
        let (hovered, hover) = press::hover((id.clone(), "hover"), window, cx);
        let actions = [
            ("copy", CarbonIcon::Copy, self.texts.copy, self.on_copy),
            ("fork", CarbonIcon::Fork, self.texts.fork, self.on_fork),
            ("retry", CarbonIcon::Renew, self.texts.retry, self.on_retry),
        ];
        let mut focus_within = false;
        let buttons: Vec<Button> = actions
            .into_iter()
            .map(|(key, icon, label, handler)| {
                let (focus, _) = press::focus((id.clone(), key), window, cx);
                focus_within |= focus.is_focused(window);
                // `<Cb.IconButton kind="ghost" size="sm">` with a 16px glyph.
                Button::new((id.clone(), key), label)
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm)
                    .icon(Icon::carbon(icon))
                    .icon_only(true)
                    .track_focus(&focus)
                    .when_some(handler, |b, f| {
                        b.on_click(move |_, window, cx| f(window, cx))
                    })
            })
            .collect();
        let meta: Vec<SharedString> = [self.model, self.tokens, self.time]
            .into_iter()
            .flatten()
            .collect();
        // `.g-turnfoot { display: flex; align-items: center; gap: 0 }`.
        div()
            .id(id)
            .flex()
            .flex_row()
            .items_center()
            .opacity(turn_footer_opacity(hovered, focus_within))
            .on_hover(press::track_hover(hover))
            .children(buttons)
            .when(!meta.is_empty(), |this| {
                // `.g-turnfoot-meta { display: inline-flex; gap: spacing-03; margin-left:
                // spacing-03; font-size: 0.75rem; color: text-helper }`.
                this.child(
                    div()
                        .flex()
                        .flex_row()
                        .gap(press::rem(8.))
                        .ml(press::rem(8.))
                        .text_size(press::rem(12.))
                        .line_height(press::rem(16.))
                        .text_color(color(ColorRole::TextHelper))
                        .whitespace_nowrap()
                        .children(meta),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rests_at_75_percent_until_hovered_or_focused() {
        assert_eq!(turn_footer_opacity(false, false), 0.75);
        assert_eq!(turn_footer_opacity(true, false), 1.);
        assert_eq!(turn_footer_opacity(false, true), 1.);
    }

    #[test]
    fn button_names_follow_the_reference() {
        let t = TurnFooterTexts::default();
        assert_eq!(t.copy, "Copy response");
        assert_eq!(t.fork, "Fork from here");
        assert_eq!(t.retry, "Retry");
    }
}
