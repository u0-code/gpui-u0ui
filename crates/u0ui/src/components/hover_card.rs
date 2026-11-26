//! HoverCard: a preview of what sits behind a link, shown on hover → Carbon
//! `Popover` without a caret (`index.jsx` `HoverCard`).
//!
//! - **Behavior**: opens `open_delay` (300ms) after the pointer rests on the
//!   trigger, or while keyboard focus is inside it; closes 150ms after the
//!   pointer leaves both the trigger and the card (so the pointer can travel
//!   onto the card); Escape closes it. Kit's `component::hover_card` renders
//!   its own rounded surface, so the surface is drawn here and positioned by
//!   Base's `Popup` via [`super::floating`].
//! - **Appearance**: the Popover's (`layer-01`, 2px radius, floating shadow,
//!   `.cu-popover-content` padding), aligned `bottom-start` by default with a
//!   4px gap (no caret).
//!
//! ```ignore
//! HoverCard::new("profile", Button::new("l", "@carbon-design").variant(ButtonVariant::Link))
//!     .child(profile_summary)
//! ```
use super::floating::{self, Align};
use super::overlay::{self, rem};
use super::popover::{CONTENT_MAX_WIDTH, CONTENT_MIN_WIDTH, PopoverLook, RADIUS};
use crate::{tokens::TypeStyle, typography::TypeStyled as _};
use gpui_kit::{
    AnyElement, App, ElementId, InteractiveElement, IntoElement, KeyDownEvent, ParentElement,
    RenderOnce, StatefulInteractiveElement as _, Styled, Window, div, px,
};
use std::time::Duration;

/// `openDelay` default (`index.jsx`: `openDelay = 300`).
pub const OPEN_DELAY: Duration = Duration::from_millis(300);
/// The close delay (`setTimeout(() => set(defaultOpen), 150)`).
pub const CLOSE_DELAY: Duration = Duration::from_millis(150);

/// A hover card. See the module docs.
#[derive(IntoElement)]
pub struct HoverCard {
    id: ElementId,
    trigger: AnyElement,
    children: Vec<AnyElement>,
    align: Align,
    default_open: bool,
    open_delay: Duration,
}

impl HoverCard {
    /// A hover card with a stable `id` for `trigger`.
    pub fn new(id: impl Into<ElementId>, trigger: impl IntoElement) -> Self {
        Self {
            id: id.into(),
            trigger: trigger.into_any_element(),
            children: Vec::new(),
            align: Align::BottomLeft,
            default_open: false,
            open_delay: OPEN_DELAY,
        }
    }

    /// One of Carbon's 12 alignments (default `bottom-start`).
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    /// Show on arrival until the pointer has come and gone.
    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }

    /// Delay before opening on hover (default 300ms).
    pub fn open_delay(mut self, delay: Duration) -> Self {
        self.open_delay = delay;
        self
    }
}

impl ParentElement for HoverCard {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for HoverCard {
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
        let open_delay = self.open_delay;
        let on_hover = {
            let hover = hover.clone();
            move |hovered: &bool, _: &mut Window, cx: &mut App| {
                floating::schedule(
                    &hover,
                    *hovered,
                    if *hovered { open_delay } else { CLOSE_DELAY },
                    cx,
                )
            }
        };

        let trigger = div()
            .id((id.clone(), "trigger"))
            .flex_shrink_0()
            .track_focus(&within.clone().tab_stop(false))
            .on_hover(on_hover.clone())
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

        let look = PopoverLook::current();
        let surface = open.then(|| {
            div()
                .id((id.clone(), "surface"))
                .on_hover(on_hover)
                .rounded(px(RADIUS))
                .bg(look.background)
                .text_color(look.text)
                .shadow(overlay::floating_shadow())
                .min_w(rem(CONTENT_MIN_WIDTH))
                .max_w(rem(CONTENT_MAX_WIDTH))
                .p(rem(16.))
                .type_style(TypeStyle::Body01)
                .children(self.children)
        });
        floating::anchored((id, "popup"), trigger, surface, self.align, None, 0.)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_follow_index_jsx() {
        let card = HoverCard::new("h", div());
        assert_eq!(card.align, Align::BottomLeft);
        assert_eq!(card.open_delay.as_millis(), 300);
        assert_eq!(CLOSE_DELAY.as_millis(), 150);
        // No caret: 4px gap, start edge 1rem before the trigger center.
        let placement = floating::Placement::of(card.align, false, 0.);
        assert_eq!((placement.gap, placement.cross_px), (4., -16.));
    }
}
