//! A Carbon button whose content and padding the T3 craft pass reshapes:
//! the composer's labelled Stop, the running Queue / Stop pair, and the
//! approval card's neutral-outline Decline. The plain [`Button`] keeps
//! Carbon's fixed anatomy (label, icon at the end), so these lay out their
//! own row of icon, text and key cap, and take their colors from the same
//! [`Look`] (or the T3 neutral outline).
//!
//! [`Button`]: crate::components::Button
use crate::components::button::{ButtonKind, ButtonState, Look};
use crate::components::floating::Align;
use crate::t3::press::{Handler, inset_frame, rem};
use crate::{
    components::Tooltip,
    icons::{CarbonIcon, Icon},
    motion::{self, Motion},
    theme::color,
    tokens::{ColorRole, FontFamilyKey},
    typography::font,
};
use gpui_kit::{
    AnyElement, App, CursorStyle, ElementId, Hsla, InteractiveElement as _, IntoElement,
    MouseButton, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled,
    Window, div, prelude::FluentBuilder as _, px,
};

/// The fill family of an [`ActionButton`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ActionKind {
    /// A Carbon kind's own look.
    Carbon(ButtonKind),
    /// `background: transparent; color: text-primary; box-shadow: inset 0 0 0
    /// 1px border-strong-01`, hover `layer-hover-01` (`.t3-approval
    /// .t3-card-actions .cds--btn--secondary`, `.t3-primary .t3-stop`).
    NeutralOutline,
}

/// Colors for `kind` in `state`: (background, text/icon, frame, focus ring, focus inset).
pub(crate) fn action_look(
    kind: ActionKind,
    state: ButtonState,
) -> (Hsla, Hsla, Option<Hsla>, Option<Hsla>, Option<Hsla>) {
    match kind {
        ActionKind::Carbon(kind) => {
            let look = Look::of(kind, state, false);
            (
                look.background,
                look.text,
                None,
                look.focus_ring,
                look.focus_inset,
            )
        }
        ActionKind::NeutralOutline => {
            let base = Look::of(ButtonKind::Secondary, state, false);
            (
                if state.hovered && !state.disabled {
                    color(ColorRole::LayerHover01)
                } else {
                    gpui_kit::transparent_black()
                },
                color(if state.disabled {
                    ColorRole::TextDisabled
                } else {
                    ColorRole::TextPrimary
                }),
                Some(color(ColorRole::BorderStrong01)),
                base.focus_ring,
                base.focus_inset,
            )
        }
    }
}

/// One piece of an [`ActionButton`]'s row.
pub(crate) enum Part {
    Icon(CarbonIcon),
    Text(SharedString),
    /// A key cap in the button's ink: (label, style).
    Key(SharedString, KeyCap),
}

/// The in-button key cap variants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum KeyCap {
    /// `.t3-approval .t3-card-actions .cds--btn .cu-kbd`: 18px tall, 1px
    /// `currentColor` frame, 11px sans, opacity .7, 8px after the label.
    Framed,
    /// `.t3-primary--steer .cds--btn .cu-kbd` (r2-gal): no frame or padding,
    /// 12px, opacity .75, 8px after the label.
    Bare,
}

/// A Carbon-colored button with a free row of parts. See the module docs.
#[derive(IntoElement)]
pub(crate) struct ActionButton {
    id: ElementId,
    label: SharedString,
    kind: ActionKind,
    height: f32,
    padding: (f32, f32),
    gap: f32,
    parts: Vec<Part>,
    disabled: bool,
    tooltip: Option<SharedString>,
    on_click: Option<Handler>,
    square: bool,
    ink: Option<Hsla>,
    tooltip_align: Align,
}

impl ActionButton {
    /// A `kind` button named `label`, 32px tall with Carbon's 15/16 padding.
    pub(crate) fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        kind: ActionKind,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind,
            height: 32.,
            padding: (15., 16.),
            gap: 0.,
            parts: Vec::new(),
            disabled: false,
            tooltip: None,
            on_click: None,
            square: false,
            ink: None,
            tooltip_align: Align::Top,
        }
    }

    pub(crate) fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// (start, end) padding in px.
    pub(crate) fn padding(mut self, start: f32, end: f32) -> Self {
        self.padding = (start, end);
        self
    }

    pub(crate) fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    pub(crate) fn part(mut self, part: Part) -> Self {
        self.parts.push(part);
        self
    }

    pub(crate) fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// A Carbon tooltip with `text` (the reference's `title` / icon label).
    pub(crate) fn tooltip(mut self, text: impl Into<SharedString>) -> Self {
        self.tooltip = Some(text.into());
        self
    }

    pub(crate) fn on_click(mut self, f: Option<Handler>) -> Self {
        self.on_click = f;
        self
    }

    /// An icon-only square (`inline-size: block-size`), content centred.
    /// The text and icon color instead of the kind's (e.g. `icon-secondary`).
    pub(crate) fn ink(mut self, ink: Hsla) -> Self {
        self.ink = Some(ink);
        self
    }

    /// Where the tooltip opens (default above).
    pub(crate) fn tooltip_align(mut self, align: Align) -> Self {
        self.tooltip_align = align;
        self
    }

    pub(crate) fn square(mut self, square: bool) -> Self {
        self.square = square;
        self
    }
}

#[derive(Default)]
struct PointerState {
    hovered: bool,
    pressed: bool,
}

impl RenderOnce for ActionButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let pointer =
            window.use_keyed_state((id.clone(), "pointer"), cx, |_, _| PointerState::default());
        let focus = window
            .use_keyed_state((id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let state = {
            let p = pointer.read(cx);
            ButtonState {
                hovered: p.hovered && !self.disabled,
                pressed: p.pressed && !self.disabled,
                focused: focus.is_focused(window),
                disabled: self.disabled,
            }
        };
        let (bg, ink, frame, ring, inset) = action_look(self.kind, state);
        let ink = if self.disabled {
            ink
        } else {
            self.ink.unwrap_or(ink)
        };
        // `.cds--btn { transition: background 70ms … }`.
        let bg = motion::animate_color((id.clone(), "bg"), bg, Motion::HOVER, window, cx);
        let parts: Vec<AnyElement> = self
            .parts
            .into_iter()
            .map(|part| match part {
                Part::Icon(icon) => Icon::carbon(icon)
                    .size_length(rem(16.))
                    .color(ink)
                    .into_any_element(),
                Part::Text(text) => div().child(text).into_any_element(),
                Part::Key(label, KeyCap::Framed) => div()
                    .flex()
                    .flex_none()
                    .items_center()
                    .ml(rem(8.))
                    .h(rem(18.))
                    .px(rem(4.))
                    .rounded(px(4.))
                    .border_1()
                    .border_color(ink.opacity(0.7))
                    .text_color(ink.opacity(0.7))
                    .font(font(FontFamilyKey::Sans, 400))
                    .text_size(rem(11.))
                    .child(label)
                    .into_any_element(),
                Part::Key(label, KeyCap::Bare) => div()
                    .flex_none()
                    .ml(rem(8.))
                    .text_color(ink.opacity(0.75))
                    .font(font(FontFamilyKey::Mono, 400))
                    .text_size(rem(12.))
                    .child(label)
                    .into_any_element(),
            })
            .collect();
        let focus_layers: Vec<AnyElement> = [ring.map(|c| (0., c)), inset.map(|c| (1., c))]
            .into_iter()
            .flatten()
            .map(|(at, c)| {
                div()
                    .absolute()
                    .top(px(at))
                    .left(px(at))
                    .right(px(at))
                    .bottom(px(at))
                    .border_1()
                    .border_color(c)
                    .into_any_element()
            })
            .collect();
        let on_click = self.on_click.clone();
        let button = gpui_kit::base::Button::new(id.clone())
            .track_focus(&focus)
            .disabled(self.disabled)
            .accessibility_label(self.label.clone())
            .relative()
            .flex()
            .flex_row()
            .flex_none()
            .items_center()
            .gap(rem(self.gap))
            .h(rem(self.height))
            .pl(rem(self.padding.0))
            .pr(rem(self.padding.1))
            .map(|this| {
                if self.square {
                    this.w(rem(self.height)).justify_center()
                } else {
                    this.justify_start()
                }
            })
            .bg(bg)
            .when_some(frame, |this, frame| this.shadow(inset_frame(frame)))
            .text_color(ink)
            .text_size(rem(14.))
            .line_height(rem(18.))
            .whitespace_nowrap()
            .cursor(if self.disabled {
                CursorStyle::OperationNotAllowed
            } else {
                CursorStyle::PointingHand
            })
            .on_hover({
                let pointer = pointer.clone();
                move |over, _, cx| {
                    pointer.update(cx, |p, cx| {
                        if p.hovered != *over {
                            p.hovered = *over;
                            if !over {
                                p.pressed = false;
                            }
                            cx.notify();
                        }
                    })
                }
            })
            .on_mouse_down(MouseButton::Left, {
                let pointer = pointer.clone();
                move |_, _, cx| {
                    pointer.update(cx, |p, cx| {
                        p.pressed = true;
                        cx.notify();
                    })
                }
            })
            .on_mouse_up(MouseButton::Left, {
                let pointer = pointer.clone();
                move |_, _, cx| {
                    pointer.update(cx, |p, cx| {
                        p.pressed = false;
                        cx.notify();
                    })
                }
            })
            .when_some(on_click, |this, f| {
                this.on_click(move |_, window, cx| f(window, cx))
            })
            .children(parts)
            .children(focus_layers);
        match self.tooltip {
            Some(text) => Tooltip::new((id, "tooltip"), button)
                .content(text)
                .align(self.tooltip_align)
                .into_any_element(),
            None => button.into_any_element(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    #[test]
    fn neutral_outline_is_a_strong_hairline_on_nothing() {
        crate::theme::tests_set(CarbonTheme::White);
        let (bg, ink, frame, ..) = action_look(ActionKind::NeutralOutline, ButtonState::default());
        assert_eq!(bg.a, 0.);
        assert_eq!(ink, color(ColorRole::TextPrimary));
        assert_eq!(frame, Some(color(ColorRole::BorderStrong01)));
        let hover = ButtonState {
            hovered: true,
            ..Default::default()
        };
        assert_eq!(
            action_look(ActionKind::NeutralOutline, hover).0,
            color(ColorRole::LayerHover01)
        );
    }
}
