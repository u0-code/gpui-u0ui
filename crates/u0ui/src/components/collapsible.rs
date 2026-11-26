//! Collapsible: one disclosure with a single trigger (`.cu-collapsible*` in
//! `custom.css`; Carbon has no direct equivalent).
//!
//! - **Behavior**: the trigger is a Base `Button` (focus, Enter/Space,
//!   pointer) that toggles the content, controlled or uncontrolled.
//! - **Appearance**: the trigger is an inline `size-md` row (0.5rem start,
//!   1rem end padding, 0.5rem gap) in `text-primary`, `background-hover` on
//!   hover, a 2px inset `focus` outline on keyboard focus only
//!   (`.cu-collapsible-trigger:focus-visible`); a 16px `ChevronRight` in
//!   `icon-primary` before the label; content padded `0.5rem 0 0.5rem 2rem`.
//! - **Motion**: the chevron turns 90° on open in 110ms
//!   (`.cu-collapsible-chev { transition: transform 110ms cubic-bezier(0.2, 0, 0.38, 0.9) }`;
//!   a cut under reduced motion). The content mounts and unmounts at once,
//!   as the reference's `{open && <div className="cu-collapsible-content">}`.
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
    trace: bool,
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
            trace: false,
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

impl Collapsible {
    /// Restyle as a Trace disclosure (`.cu-trace .cu-collapsible-trigger
    /// { min-height: 0; padding: 0; gap: spacing-02; font-size: 0.75rem; color:
    /// link-primary }`, `.cu-trace .cu-collapsible-chev { fill: link-primary }`,
    /// `.cu-trace .cu-collapsible-content { padding: spacing-03 0 0 }`). Used by
    /// `TraceCollapsible`.
    pub fn trace(mut self, trace: bool) -> Self {
        self.trace = trace;
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
        // `:focus-visible`: keyboard focus only.
        let focused = focus.is_focused(window) && window.last_input_was_keyboard();
        let trace = self.trace;
        let (text, chevron) = if trace {
            (color(ColorRole::LinkPrimary), color(ColorRole::LinkPrimary))
        } else {
            (color(ColorRole::TextPrimary), color(ColorRole::IconPrimary))
        };

        let trigger = gpui_kit::base::Button::new((id.clone(), "trigger"))
            .track_focus(&focus)
            .accessibility_label(self.trigger.clone())
            .relative()
            .flex()
            .flex_row()
            .items_center()
            .map(|this| {
                if trace {
                    this.gap(rem(4.)).type_style(TypeStyle::Label01)
                } else {
                    this.gap(rem(8.))
                        .min_h(SIZE_MD.rems())
                        .pl(rem(8.))
                        .pr(rem(16.))
                        .type_style(TypeStyle::BodyCompact01)
                }
            })
            .bg(bg)
            .text_color(text)
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
                    .color(chevron)
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

        let content = open.then(|| {
            // `.cu-collapsible-content { padding: spacing-03 0 spacing-03 spacing-07 }`.
            div()
                .w_full()
                .flex()
                .flex_col()
                .pt(rem(8.))
                .when(!trace, |this| this.pb(rem(8.)).pl(rem(32.)))
                .children(self.children)
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
