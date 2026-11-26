//! Tooltip: shadcn `Tooltip` → Carbon `Tooltip` (`.cds--tooltip`, a
//! high-contrast `.cds--popover` with a caret).
//!
//! - **Behavior**: shows while the pointer rests on the trigger (after
//!   Carbon's `enterDelayMs` 100ms; hides `leaveDelayMs` 300ms after it leaves)
//!   or while keyboard focus is inside the trigger; Escape hides it. Kit's
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
    RenderOnce, SharedString, StatefulInteractiveElement as _, Styled, Window, div,
};
use std::time::Duration;

/// Carbon Tooltip `enterDelayMs` default.
pub const ENTER_DELAY: Duration = Duration::from_millis(100);
/// Carbon Tooltip `leaveDelayMs` default.
pub const LEAVE_DELAY: Duration = Duration::from_millis(300);

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

    /// Show on arrival until the pointer has come and gone.
    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }
}

impl RenderOnce for Tooltip {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let hover = floating::hover_intent((id.clone(), "hover"), self.default_open, window, cx);
        let within = window
            .use_keyed_state((id.clone(), "within"), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let dismissed = window.use_keyed_state((id.clone(), "dismissed"), cx, |_, _| false);
        let focused = within.contains_focused(window, cx);
        if !focused && *dismissed.read(cx) && !hover.read(cx).open {
            dismissed.update(cx, |d, _| *d = false);
        }
        let open = (hover.read(cx).open || focused) && !*dismissed.read(cx);

        let trigger = div()
            .id((id.clone(), "trigger"))
            .flex_shrink_0()
            .track_focus(&within.clone().tab_stop(false))
            .on_hover({
                let hover = hover.clone();
                move |hovered, _, cx| {
                    floating::schedule(
                        &hover,
                        *hovered,
                        if *hovered { ENTER_DELAY } else { LEAVE_DELAY },
                        cx,
                    )
                }
            })
            .on_key_down({
                let dismissed = dismissed.clone();
                let hover = hover.clone();
                move |event: &KeyDownEvent, _, cx| {
                    if event.keystroke.key == "escape" {
                        dismissed.update(cx, |d, cx| {
                            *d = true;
                            cx.notify();
                        });
                        floating::schedule(&hover, false, Duration::ZERO, cx);
                    }
                }
            })
            .child(self.trigger)
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
                .child(self.content.clone())
        });
        floating::anchored(
            (id, "popup"),
            trigger,
            surface,
            self.align,
            Some(look.background),
            TOOLTIP_OFFSET,
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
    fn carbon_delays() {
        assert_eq!(ENTER_DELAY.as_millis(), 100);
        assert_eq!(LEAVE_DELAY.as_millis(), 300);
        assert_eq!(Tooltip::new("t", div()).align, Align::Top);
    }
}
