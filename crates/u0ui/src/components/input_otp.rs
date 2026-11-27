//! InputOTP: a one-time code entry. Carbon has none; the reference
//! (`.cu-otp*` in `src/custom.css`) draws Carbon field slots over one hidden
//! native input.
//!
//! - **Behavior** is Kit's: one transparent [`InputState`] editor covers the
//!   slots and owns typing, paste, IME, deletion and focus. The component
//!   installs a validator that keeps the text at most `length` characters,
//!   each matching `pattern` (digits by default). Read the code from the state
//!   (subscribe to its `InputEvent::Change` for `onValueChange`).
//! - **Appearance**: 40×48 slots (`size-md` × `size-lg`) on `field` with a
//!   1px `border-strong` rule, 1px apart, the characters in IBM Plex Mono
//!   20px; groups are split by an en dash in `text-secondary`. While focused
//!   the next empty slot (or the last when full) gets the 2px inset `focus`
//!   outline and an empty one shows a 1×20px caret blinking 1s in steps
//!   (`@keyframes cu-blink`; steady under reduced motion). `invalid` outlines
//!   every slot in `support-error`; `disabled` dims the text and drops the rule.
//!
//! ```ignore
//! let state = cx.new(|cx| InputState::new(window, cx).default_value("482"));
//! InputOtp::new("code", &state).length(6).groups([3, 3]).label("One-time code")
//! ```
use super::form::{self, rem};
use super::input::InputState;
use crate::{
    motion,
    theme::color,
    tokens::{self, ColorRole, TypeStyle},
    typography::{self, TypeStyled as _},
};
use gpui_kit::{
    Animation, AnimationExt as _, App, CursorStyle, ElementId, Entity, Focusable as _, Hsla,
    InteractiveElement as _, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    component::{Sizable as _, Size, input::Input as KitInput},
    div,
    prelude::FluentBuilder as _,
};
use std::time::Duration;

/// Slot width (`width: var(--size-md)`).
pub const SLOT_WIDTH: f32 = 40.;
/// Slot height (`height: var(--size-lg)`).
pub const SLOT_HEIGHT: f32 = 48.;

/// The index of the slot that shows focus: the next empty one, or the last
/// when the code is full (`k === v.length || (k === length - 1 && v.length === length)`).
pub fn active_slot(len: usize, length: usize) -> usize {
    len.min(length.saturating_sub(1))
}

/// Whether `text` is an acceptable (partial) code.
pub fn accepts(text: &str, length: usize, pattern: fn(char) -> bool) -> bool {
    text.chars().count() <= length && text.chars().all(pattern)
}

/// The colors of one slot.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SlotLook {
    /// Fill (`field`).
    pub background: Hsla,
    /// Bottom rule (`border-strong`, transparent when disabled).
    pub border: Hsla,
    /// Character color.
    pub text: Hsla,
    /// Outline: `support-error` when invalid, else `focus` on the active slot.
    pub outline: Option<Hsla>,
}

impl SlotLook {
    /// The look of a slot (`.cu-otp-slot*`).
    pub fn of(active: bool, invalid: bool, disabled: bool) -> Self {
        Self {
            background: color(ColorRole::Field01),
            border: if disabled {
                Hsla::transparent_black()
            } else {
                color(ColorRole::BorderStrong01)
            },
            text: color(if disabled {
                ColorRole::TextDisabled
            } else {
                ColorRole::TextPrimary
            }),
            // `.cu-otp--invalid .cu-otp-slot` comes after `.cu-otp-slot--active`, so it wins.
            outline: if invalid {
                Some(color(ColorRole::SupportError))
            } else if active {
                Some(color(ColorRole::Focus))
            } else {
                None
            },
        }
    }
}

/// A one-time code input. See the module docs.
#[derive(IntoElement)]
pub struct InputOtp {
    id: ElementId,
    state: Entity<InputState>,
    length: usize,
    groups: Option<Vec<usize>>,
    pattern: fn(char) -> bool,
    invalid: bool,
    disabled: bool,
    label: Option<SharedString>,
}

impl InputOtp {
    /// A 6-digit code input over the caller's `state`.
    pub fn new(id: impl Into<ElementId>, state: &Entity<InputState>) -> Self {
        Self {
            id: id.into(),
            state: state.clone(),
            length: 6,
            groups: None,
            pattern: |c| c.is_ascii_digit(),
            invalid: false,
            disabled: false,
            label: None,
        }
    }

    /// `length` (default 6).
    pub fn length(mut self, length: usize) -> Self {
        self.length = length.max(1);
        self
    }

    /// `groups`, e.g. `[3, 3]` (default one group of `length`).
    pub fn groups(mut self, groups: impl IntoIterator<Item = usize>) -> Self {
        self.groups = Some(groups.into_iter().collect());
        self
    }

    /// `pattern`: which characters are allowed (default ASCII digits).
    pub fn pattern(mut self, pattern: fn(char) -> bool) -> Self {
        self.pattern = pattern;
        self
    }

    /// `invalid`.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// `disabled`.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// `label`: the accessible name of the hidden input.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl RenderOnce for InputOtp {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let length = self.length;
        let pattern = self.pattern;
        // Install the validator once per (state, length, pattern).
        let key = (self.state.entity_id(), length, pattern as usize);
        let applied = window.use_keyed_state((id.clone(), "validator"), cx, |_, _| None);
        if *applied.read(cx) != Some(key) {
            applied.update(cx, |a, _| *a = Some(key));
            self.state.update(cx, |s, cx| {
                s.set_validator(move |text, _| accepts(text, length, pattern), cx)
            });
        }
        // The caret marks the next slot (`input-otp` places it on focus), so
        // Tab into the slots does not select the whole code.
        super::form::keep_caret_on_tab_in(&self.state);
        let value: Vec<char> = self.state.read(cx).value().chars().collect();
        let focus = self.state.read(cx).focus_handle(cx);
        let focused = focus.is_focused(window) && !self.disabled;
        let active = active_slot(value.len(), length);
        let blink = !motion::reduced_motion(cx);
        let groups = self
            .groups
            .clone()
            .filter(|g| !g.is_empty())
            .unwrap_or_else(|| vec![length]);

        let mut index = 0;
        let mut parts = Vec::new();
        for (gi, size) in groups.iter().enumerate() {
            if gi > 0 {
                // `.cu-otp-sep { color: var(--text-secondary) }`.
                parts.push(
                    div()
                        .text_color(color(ColorRole::TextSecondary))
                        .child("–")
                        .into_any_element(),
                );
            }
            let slots = (0..*size).map(|_| {
                let k = index;
                index += 1;
                let is_active = focused && k == active;
                let look = SlotLook::of(is_active, self.invalid, self.disabled);
                let ch = value.get(k).copied();
                div()
                    .relative()
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(rem(SLOT_WIDTH))
                    .h(rem(SLOT_HEIGHT))
                    .bg(look.background)
                    .pb(gpui_kit::px(form::FIELD_RULE))
                    .font(typography::font(tokens::FontFamilyKey::Mono, 400))
                    .text_size(rem(20.))
                    .text_color(look.text)
                    .when_some(ch, |this, ch| {
                        this.child(SharedString::from(ch.to_string()))
                    })
                    .when(is_active && ch.is_none(), |this| {
                        let caret = div()
                            .w(gpui_kit::px(1.))
                            .h(rem(20.))
                            .bg(color(ColorRole::TextPrimary));
                        if blink {
                            this.child(caret.with_animation(
                                (id.clone(), "caret"),
                                Animation::new(Duration::from_secs(1)).repeat(),
                                // `@keyframes cu-blink { 50% { opacity: 0 } }` with `steps(1)`.
                                |caret, t| caret.opacity(if t < 0.5 { 1. } else { 0. }),
                            ))
                        } else {
                            this.child(caret)
                        }
                    })
                    .child(form::bottom_rule(look.border))
                    .when_some(look.outline, |this, outline| {
                        this.child(form::inset_outline(2., outline))
                    })
            });
            // `.cu-otp-group { display: flex; gap: 1px }`.
            parts.push(
                div()
                    .flex()
                    .flex_row()
                    .gap(gpui_kit::px(1.))
                    .children(slots)
                    .into_any_element(),
            );
        }

        let editor = KitInput::new(&self.state)
            .appearance(false)
            .with_size(Size::XSmall)
            .disabled(self.disabled)
            .when_some(self.label.clone(), |this, label| this.aria_label(label))
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .opacity(0.);

        div()
            .id(id.clone())
            .relative()
            .flex()
            .flex_row()
            .items_center()
            .type_style(TypeStyle::BodyCompact01)
            .cursor(if self.disabled {
                gpui_kit::CursorStyle::OperationNotAllowed
            } else {
                CursorStyle::IBeam
            })
            // `onClick={() => ref.current.focus()}`: a click anywhere focuses the
            // hidden editor, which then takes the text at its end.
            .when(!self.disabled, |this| {
                let state = self.state.clone();
                this.on_mouse_down(gpui_kit::MouseButton::Left, move |_, window, cx| {
                    focus.focus(window, cx);
                    state.update(cx, |s, cx| {
                        let end = s.value().len();
                        s.set_selected_range(end..end, cx);
                    });
                })
            })
            // `.cu-otp { gap: var(--spacing-03) }` between the groups only.
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(tokens::SPACING_03)
                    .children(parts),
            )
            // `.cu-otp-input { position: absolute; inset: 0 }`: Kit's input keeps an
            // in-flow root, so an absolute box holds it and the row stays as wide
            // as its slots.
            .child(div().absolute().top_0().left_0().size_full().child(editor))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    #[test]
    fn active_slot_follows_the_reference() {
        assert_eq!(active_slot(0, 6), 0);
        assert_eq!(active_slot(3, 6), 3);
        // Full: the last slot stays active.
        assert_eq!(active_slot(6, 6), 5);
    }

    #[test]
    fn validator_limits_length_and_pattern() {
        let digits: fn(char) -> bool = |c| c.is_ascii_digit();
        assert!(accepts("482", 6, digits));
        assert!(!accepts("48a", 6, digits));
        assert!(!accepts("1234567", 6, digits));
    }

    #[test]
    fn slot_colors_follow_cu_otp_css() {
        crate::theme::tests_set(CarbonTheme::White);
        let active = SlotLook::of(true, false, false);
        assert_eq!(active.outline, Some(color(ColorRole::Focus)));
        let invalid = SlotLook::of(true, true, false);
        assert_eq!(invalid.outline, Some(color(ColorRole::SupportError)));
        let disabled = SlotLook::of(false, false, true);
        assert_eq!(disabled.border.a, 0.);
        assert_eq!(SLOT_WIDTH, tokens::SIZE_MD.px_value());
        assert_eq!(SLOT_HEIGHT, tokens::SIZE_LG.px_value());
    }
}
