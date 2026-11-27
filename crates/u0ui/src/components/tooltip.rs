//! Tooltip: shadcn `Tooltip` → Carbon `Tooltip` (`.cds--tooltip`, a
//! high-contrast `.cds--popover` with a caret).
//!
//! - **Behavior**: shows while the pointer rests on the trigger (after
//!   Carbon's `enterDelayMs` 100ms; hides `leaveDelayMs` 300ms after it leaves)
//!   or while focus that arrived by keyboard navigation (Tab, arrows) is
//!   inside the trigger. Escape hides it until the pointer enters again or
//!   focus arrives again by navigation. Pressing the trigger (pointer, Enter
//!   or Space) hides it the same way only with
//!   [`Tooltip::close_on_activation`] (Carbon `closeOnActivation`: off for a
//!   standalone `Tooltip`, on for `IconButton`, so [`super::Button`]'s
//!   icon-only tooltip turns it on); then it never covers the popover or
//!   menu the press opened, nor comes back when that surface hands focus
//!   back on close. Kit's
//!   `component::tooltip` is a builder for GPUI's single window tooltip slot,
//!   which cannot place a caret or honor Carbon's 12 alignments, so the
//!   surface is drawn here and positioned by Base's `Popup`.
//! - **Appearance**: `background-inverse` surface, `text-inverse` body-01
//!   text, 1rem padding, `max-inline-size: 18rem`, 2px radius, a 12×6 caret
//!   and a 12px offset (`.cds--tooltip { --popover-offset: 12px }`).
//! - **Motion**: none; it shows and hides at once (see [`super::floating`]).
//!
//! ```ignore
//! Tooltip::new("copy", Button::new("c", "Copy").icon(CarbonIcon::Copy).icon_only(true))
//!     .content("Copy to clipboard")
//! ```
use super::floating::{self, Align, TOOLTIP_OFFSET};
use super::overlay::rem;
use crate::{
    theme::color,
    tokens::{ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    AnyElement, App, ElementId, Hsla, InteractiveElement, IntoElement, KeyDownEvent, ParentElement,
    RenderOnce, Role, SharedString, StatefulInteractiveElement as _, Styled, TestSupportExt as _,
    Window, div, prelude::FluentBuilder as _,
};
use std::cell::Cell;
use std::time::Duration;

/// Carbon Tooltip `enterDelayMs` default.
pub const ENTER_DELAY: Duration = Duration::from_millis(100);
/// Carbon Tooltip `leaveDelayMs` default.
pub const LEAVE_DELAY: Duration = Duration::from_millis(300);

thread_local! {
    /// Whether the last key pressed in any window moves focus (Tab, arrows,
    /// Home/End, Page Up/Down), recorded by [`init`].
    static NAVIGATED: Cell<bool> = const { Cell::new(false) };
}

/// Whether `key` (a GPUI key name) moves focus.
pub fn navigation_key(key: &str) -> bool {
    matches!(
        key,
        "tab" | "up" | "down" | "left" | "right" | "home" | "end" | "pageup" | "pagedown"
    )
}

/// Watch keystrokes so a tooltip can tell focus that arrived by keyboard
/// navigation (shown, like `:focus-visible`) from focus handed back by a
/// closing popover or dialog (not shown). Called by [`crate::init`].
pub(crate) fn init(cx: &mut App) {
    cx.observe_keystrokes(|event, _, _| {
        NAVIGATED.with(|n| n.set(navigation_key(&event.keystroke.key)));
    })
    .detach();
}

/// Per-tooltip visibility state besides hover intent.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TipState {
    /// Focus was inside the trigger last frame.
    pub was_focused: bool,
    /// Focus is inside the trigger and arrived there by keyboard navigation.
    pub focus_shown: bool,
    /// Hidden by a press or Escape until the next deliberate arrival.
    pub dismissed: bool,
    /// The pointer is on the trigger.
    pub pointer_in: bool,
    /// The dismissal holds until the pointer leaves: it was on the trigger
    /// when pressed, so a hover report arriving with or after the press is
    /// not a fresh arrival.
    pub held: bool,
}

impl TipState {
    /// This frame's focus: `focused` inside the trigger, and whether focus
    /// that just arrived came by keyboard navigation.
    pub fn focus(mut self, focused: bool, by_navigation: bool) -> Self {
        if focused && !self.was_focused {
            self.focus_shown = by_navigation;
            if by_navigation {
                self.dismissed = false;
            }
        }
        if !focused {
            self.focus_shown = false;
        }
        self.was_focused = focused;
        self
    }

    /// The pointer entered the trigger: a fresh arrival undoes a dismissal
    /// (one the pointer has left since; see [`TipState::held`]).
    pub fn pointer_entered(mut self) -> Self {
        self.pointer_in = true;
        if !self.held {
            self.dismissed = false;
        }
        self
    }

    /// The pointer left the trigger: the next entry is a fresh arrival.
    pub fn pointer_left(mut self) -> Self {
        self.pointer_in = false;
        self.held = false;
        self
    }

    /// A press on the trigger (pointer, Enter, Space) or Escape. A pointer
    /// press means the pointer is on the trigger even if its hover report
    /// has not arrived yet, so the dismissal holds until it leaves.
    pub fn dismiss(mut self, by_pointer: bool) -> Self {
        self.dismissed = true;
        self.held = by_pointer || self.pointer_in;
        self
    }

    /// Whether the tooltip shows, given the hover intent.
    pub fn open(self, hovered: bool) -> bool {
        (hovered || self.focus_shown) && !self.dismissed
    }
}

/// The tooltip's colors (`.cds--popover--high-contrast`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TooltipLook {
    /// `--popover-background-color: var(--background-inverse)`.
    pub background: Hsla,
    /// `--popover-text-color: var(--text-inverse)`.
    pub text: Hsla,
}

impl TooltipLook {
    /// The look on the active theme.
    pub fn current() -> Self {
        Self {
            background: color(ColorRole::BackgroundInverse),
            text: color(ColorRole::TextInverse),
        }
    }
}

/// A Carbon tooltip around a focusable trigger. See the module docs.
#[derive(IntoElement)]
pub struct Tooltip {
    id: ElementId,
    trigger: AnyElement,
    content: SharedString,
    align: Align,
    default_open: bool,
    close_on_activation: bool,
}

impl Tooltip {
    /// A tooltip with a stable `id` around `trigger` (which should take focus).
    pub fn new(id: impl Into<ElementId>, trigger: impl IntoElement) -> Self {
        Self {
            id: id.into(),
            trigger: trigger.into_any_element(),
            content: SharedString::default(),
            align: Align::Top,
            default_open: false,
            close_on_activation: false,
        }
    }

    /// The text (supplementary only; no interactive content).
    pub fn content(mut self, content: impl Into<SharedString>) -> Self {
        self.content = content.into();
        self
    }

    /// `side`/`align` (default `top`).
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    /// Hide when the trigger is pressed (pointer, Enter, Space) until the
    /// next fresh arrival (Carbon `closeOnActivation`; default false, as
    /// `Cb.Tooltip` has it; Carbon `IconButton` turns it on). Escape hides
    /// it either way.
    pub fn close_on_activation(mut self, close: bool) -> Self {
        self.close_on_activation = close;
        self
    }

    /// Show on arrival until the pointer has come and gone.
    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }
}

impl RenderOnce for Tooltip {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let hover =
            floating::hover_intent((id.clone(), "hover"), self.default_open, false, window, cx);
        let within = window
            .use_keyed_state((id.clone(), "within"), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let tip = window.use_keyed_state((id.clone(), "tip"), cx, |_, _| TipState::default());
        let focused = within.contains_focused(window, cx);
        let by_navigation = window.last_input_was_keyboard() && NAVIGATED.with(Cell::get);
        let next = tip.read(cx).focus(focused, by_navigation);
        if next != *tip.read(cx) {
            tip.update(cx, |t, _| *t = next);
        }
        let open = next.open(hover.read(cx).open);

        // A press hides the tooltip and also cancels a show still waiting out
        // `ENTER_DELAY` (the zero-delay request drops the pending task), so a
        // press inside the delay never lets it appear afterwards.
        let dismiss = {
            let tip = tip.clone();
            let hover = hover.clone();
            move |by_pointer: bool, cx: &mut App| {
                tip.update(cx, |t, cx| {
                    *t = t.dismiss(by_pointer);
                    cx.notify();
                });
                floating::schedule(&hover, false, Duration::ZERO, cx);
            }
        };
        let trigger = div()
            .id((id.clone(), "trigger"))
            .flex_shrink_0()
            .track_focus(&within.clone().tab_stop(false))
            .on_hover({
                let hover = hover.clone();
                let tip = tip.clone();
                move |hovered, _, cx| {
                    if *hovered {
                        tip.update(cx, |t, _| *t = t.pointer_entered());
                    } else {
                        tip.update(cx, |t, _| *t = t.pointer_left());
                    }
                    floating::pointer(
                        &hover,
                        *hovered,
                        if *hovered { ENTER_DELAY } else { LEAVE_DELAY },
                        cx,
                    )
                }
            })
            // Capture: the trigger (a button, a popover trigger) handles the
            // press itself and may stop it.
            .when(self.close_on_activation, |this| {
                let dismiss = dismiss.clone();
                this.capture_any_mouse_down(move |_, _, cx| dismiss(true, cx))
            })
            .capture_key_down({
                let close_on_activation = self.close_on_activation;
                move |event: &KeyDownEvent, _, cx| {
                    let key = event.keystroke.key.as_str();
                    if key == "escape" || (close_on_activation && matches!(key, "enter" | "space"))
                    {
                        dismiss(false, cx);
                    }
                }
            })
            .child(self.trigger)
            .child(floating::window_exit_guard(hover.clone(), LEAVE_DELAY))
            .test_support()
            .into_any_element();

        let look = TooltipLook::current();
        let surface = open.then(|| {
            div()
                .id((id.clone(), "surface"))
                // `.cds--tooltip-content`: body-01, 1rem padding, max 18rem.
                .max_w(rem(288.))
                .px(rem(16.))
                .py(rem(16.))
                .rounded(gpui_kit::px(2.))
                .bg(look.background)
                .text_color(look.text)
                .type_style(TypeStyle::Body01)
                .child(
                    div()
                        .id((id.clone(), "text"))
                        .role(Role::Tooltip)
                        .aria_label(self.content.clone())
                        .child(self.content.clone())
                        .test_support(),
                )
        });
        floating::anchored(
            (id, "popup"),
            trigger,
            surface,
            self.align,
            Some(look.background),
            TOOLTIP_OFFSET,
            window,
            cx,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    fn hex(c: Hsla) -> String {
        let c = c.to_rgb();
        let b = |v: f32| (v * 255.).round() as u8;
        format!("#{:02x}{:02x}{:02x}", b(c.r), b(c.g), b(c.b))
    }

    #[test]
    fn high_contrast_colors() {
        crate::theme::tests_set(CarbonTheme::White);
        let look = TooltipLook::current();
        assert_eq!(hex(look.background), "#393939");
        assert_eq!(hex(look.text), "#ffffff");
        crate::theme::tests_set(CarbonTheme::G100);
        assert_eq!(hex(TooltipLook::current().background), "#f4f4f4");
    }

    #[test]
    fn presses_and_escape_hide_until_a_fresh_arrival() {
        let rest = TipState::default();
        // Hover shows; a press hides while the pointer stays.
        assert!(rest.open(true));
        let pressed = rest.pointer_entered().focus(true, false).dismiss(true);
        assert!(!pressed.open(true));
        // The popover it opened takes focus, then hands it back on Escape:
        // still hidden (focus came back without navigation).
        let back = pressed.focus(false, false).focus(true, false);
        assert!(!back.open(false) && !back.open(true));
        // A hover report that arrives after the press (the pointer moved
        // and pressed within the enter delay) is not a fresh arrival.
        assert!(!back.pointer_entered().open(true));
        let late = TipState::default().dismiss(true).pointer_entered();
        assert!(!late.open(true));
        // The pointer leaves and enters again: shows.
        assert!(back.pointer_left().pointer_entered().open(true));
    }

    #[test]
    fn focus_shows_only_when_navigated_to() {
        let tabbed = TipState::default().focus(true, true);
        assert!(tabbed.open(false));
        // Escape hides it while focus stays.
        let escaped = tabbed.dismiss(false).focus(true, true);
        assert!(!escaped.open(false));
        // Clicked into focus: no focus tooltip (hover alone decides).
        let clicked = TipState::default().focus(true, false);
        assert!(!clicked.open(false));
        // Leaving and tabbing back shows again.
        assert!(escaped.focus(false, false).focus(true, true).open(false));
        assert!(navigation_key("tab") && navigation_key("down"));
        assert!(!navigation_key("escape") && !navigation_key("enter"));
    }

    #[test]
    fn carbon_delays() {
        assert_eq!(ENTER_DELAY.as_millis(), 100);
        assert_eq!(LEAVE_DELAY.as_millis(), 300);
        let tip = Tooltip::new("t", div());
        assert_eq!(tip.align, Align::Top);
        // `Cb.Tooltip` `closeOnActivation` defaults to false.
        assert!(!tip.close_on_activation);
        assert!(
            Tooltip::new("t", div())
                .close_on_activation(true)
                .close_on_activation
        );
    }
}
