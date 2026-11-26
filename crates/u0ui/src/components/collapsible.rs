//! Collapsible: one disclosure with a single trigger (`.cu-collapsible*` in
//! `custom.css`; Carbon has no direct equivalent).
//!
//! - **Behavior**: the trigger is a Base `Button` (focus, Enter/Space,
//!   pointer) that toggles the content, controlled or uncontrolled; the
//!   content sits in Base's `Collapsible` and is revealed with Base's
//!   `MotionReveal`.
//! - **Appearance**: the trigger is an inline `size-md` row (0.5rem start,
//!   1rem end padding, 0.5rem gap) in `text-primary`, `background-hover` on
//!   hover, a 2px inset `focus` outline; a 16px `ChevronRight` in
//!   `icon-primary` before the label; content padded `0.5rem 0 0.5rem 2rem`.
//! - **Motion**: the chevron turns 90° on open in 110ms
//!   (`.cu-collapsible-chev { transition: transform 110ms cubic-bezier(0.2, 0, 0.38, 0.9) }`;
//!   a cut under reduced motion). The reference mounts the content at once;
//!   here it reveals and fades with the Accordion's 110ms panel motion, and
//!   collapses before unmounting (reduced motion: fade only).
use super::accordion::REVEAL;
use super::overlay::{self, OpenChange, rem};
use crate::{
    icons::{CarbonIcon, Icon, IconSize},
    motion::{self, Motion},
    theme::color,
    tokens::{ColorRole, SIZE_MD, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    AnyElement, App, CursorStyle, ElementId, Hsla, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _,
};
use std::rc::Rc;

/// `.cu-collapsible--open .cu-collapsible-chev { transform: rotate(90deg) }`.
pub const OPEN_ROTATION: f32 = std::f32::consts::FRAC_PI_2;

/// A collapsible region. See the module docs.
#[derive(IntoElement)]
pub struct Collapsible {
    id: ElementId,
    trigger: SharedString,
    open: Option<bool>,
    default_open: bool,
    on_open_change: Option<OpenChange>,
    children: Vec<AnyElement>,
}

impl Collapsible {
    /// A collapsible with a stable `id` and a trigger label
    /// (`CollapsibleTrigger` children).
    pub fn new(id: impl Into<ElementId>, trigger: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            trigger: trigger.into(),
            open: None,
            default_open: false,
            on_open_change: None,
            children: Vec::new(),
        }
    }

    /// Controlled open state.
    pub fn open(mut self, open: bool) -> Self {
        self.open = Some(open);
        self
    }

    /// Initial open state when uncontrolled.
    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }

    /// Called when the trigger toggles.
    pub fn on_open_change(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_open_change = Some(Rc::new(f));
        self
    }
}

impl ParentElement for Collapsible {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Collapsible {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let openness = overlay::openness(
            &id,
            self.open,
            self.default_open,
            self.on_open_change.clone(),
            window,
            cx,
        );
        let open = openness.open;
        let set_open = openness.setter();
        let rotation = motion::animate(
            (id.clone(), "chevron"),
            if open { OPEN_ROTATION } else { 0. },
            Motion::TOGGLE,
            window,
            cx,
        );
        let progress = overlay::presence((id.clone(), "reveal"), open, REVEAL, REVEAL, window, cx);
        let reduced = motion::reduced_motion(cx);
        let hover = window.use_keyed_state((id.clone(), "hover"), cx, |_, _| false);
        let bg = motion::animate_color(
            (id.clone(), "bg"),
            if *hover.read(cx) {
                color(ColorRole::BackgroundHover)
            } else {
                Hsla::transparent_black()
            },
            Motion::HOVER,
            window,
            cx,
        );
        let focus = window
            .use_keyed_state((id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let focused = focus.is_focused(window);

        let trigger = gpui_kit::base::Button::new((id.clone(), "trigger"))
            .track_focus(&focus)
            .accessibility_label(self.trigger.clone())
            .relative()
            .flex()
            .flex_row()
            .items_center()
            .gap(rem(8.))
            .min_h(SIZE_MD.rems())
            .pl(rem(8.))
            .pr(rem(16.))
            .bg(bg)
            .text_color(color(ColorRole::TextPrimary))
            .type_style(TypeStyle::BodyCompact01)
            .cursor(CursorStyle::PointingHand)
            .on_hover(move |hovered, _, cx| {
                hover.update(cx, |h, cx| {
                    *h = *hovered;
                    cx.notify();
                })
            })
            .on_click(move |_, window, cx| set_open(!open, window, cx))
            .child(
                Icon::carbon(CarbonIcon::ChevronRight)
                    .size(IconSize::S16)
                    .color(color(ColorRole::IconPrimary))
                    .rotate(gpui_kit::radians(rotation)),
            )
            .child(self.trigger.clone())
            .when(focused, |this| {
                this.child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .border_2()
                        .border_color(color(ColorRole::Focus)),
                )
            });

        let content = (open || progress > 0.).then(|| {
            gpui_kit::base::Collapsible::new()
                .open(true)
                .reveal(
                    (id.clone(), "content"),
                    if reduced {
                        if open { 1. } else { 0. }
                    } else {
                        progress
                    },
                )
                .content(
                    // `.cu-collapsible-content { padding: spacing-03 0 spacing-03 spacing-07 }`.
                    div()
                        .flex()
                        .flex_col()
                        .pt(rem(8.))
                        .pb(rem(8.))
                        .pl(rem(32.))
                        .opacity(progress)
                        .children(self.children),
                )
        });
        div()
            .flex()
            .flex_col()
            .items_start()
            .child(trigger)
            .when_some(content, |this, content| {
                this.child(div().w_full().child(content))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chevron_turns_a_quarter() {
        assert_eq!(OPEN_ROTATION, std::f32::consts::FRAC_PI_2);
        assert_eq!(Motion::TOGGLE.speed.millis(), 110);
        assert_eq!(Motion::TOGGLE.curve.points(), (0.2, 0., 0.38, 0.9));
        assert_eq!(SIZE_MD.px_value(), 40.);
    }
}
