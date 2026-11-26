//! Popover: a click-opened floating surface with interactive content →
//! Carbon `Popover` + `PopoverContent` (`.cds--popover`, `.cu-popover-content`).
//!
//! - **Behavior**: pressing the trigger (pointer, Enter or Space) toggles it; a
//!   press outside or Escape closes it; focus moves into the surface on open
//!   and back to where it was on close. Positioning is Base's `Popup` via
//!   [`super::floating`]. Base's own `Popover` is not used because it unmounts
//!   its content the frame it closes, which would cut the exit fade.
//! - **Appearance**: `layer-01` surface, 2px radius
//!   (`--popover-border-radius: 2px`), the floating shadow when `drop_shadow`,
//!   a 12×6 caret and a 0.625rem offset when `caret`, content padded 1rem,
//!   12–22rem wide, body-01.
//!
//! ```ignore
//! Popover::new("dims", Button::new("t", "Dimensions").variant(ButtonVariant::Outline))
//!     .align(Align::BottomLeft)
//!     .child(div().child("Set the layer size."))
//! ```
use super::floating::{self, Align, FLOAT_ENTER, FLOAT_EXIT, POPOVER_OFFSET};
use super::overlay::{self, OpenChange, rem};
use crate::{
    theme::color,
    tokens::{ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    AnyElement, App, ElementId, Hsla, InteractiveElement, IntoElement, KeyDownEvent, MouseButton,
    ParentElement, RenderOnce, Styled, Window, div, prelude::FluentBuilder as _, px,
};
use std::rc::Rc;

/// The popover's colors.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PopoverLook {
    /// `--popover-background-color: var(--layer)` (layer-01 on the page).
    pub background: Hsla,
    /// `--popover-text-color: var(--text-primary)`.
    pub text: Hsla,
}

impl PopoverLook {
    /// The look on the active theme.
    pub fn current() -> Self {
        Self {
            background: color(ColorRole::Layer01),
            text: color(ColorRole::TextPrimary),
        }
    }
}

/// `.cu-popover-content { min-width: 12rem; max-width: 22rem; padding: spacing-05 }`.
pub const CONTENT_MIN_WIDTH: f32 = 192.;
/// See [`CONTENT_MIN_WIDTH`].
pub const CONTENT_MAX_WIDTH: f32 = 352.;
/// `--popover-border-radius: 2px`.
pub const RADIUS: f32 = 2.;

/// A Carbon popover. See the module docs.
#[derive(IntoElement)]
pub struct Popover {
    id: ElementId,
    trigger: AnyElement,
    children: Vec<AnyElement>,
    align: Align,
    open: Option<bool>,
    default_open: bool,
    on_open_change: Option<OpenChange>,
    caret: bool,
    drop_shadow: bool,
}

impl Popover {
    /// A popover with a stable `id`, opened by `trigger`.
    pub fn new(id: impl Into<ElementId>, trigger: impl IntoElement) -> Self {
        Self {
            id: id.into(),
            trigger: trigger.into_any_element(),
            children: Vec::new(),
            align: Align::Bottom,
            open: None,
            default_open: false,
            on_open_change: None,
            caret: true,
            drop_shadow: true,
        }
    }

    /// One of Carbon's 12 alignments (default `bottom`).
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
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

    /// Called whenever the popover asks to open or close.
    pub fn on_open_change(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_open_change = Some(Rc::new(f));
        self
    }

    /// Draw the caret (default true).
    pub fn caret(mut self, caret: bool) -> Self {
        self.caret = caret;
        self
    }

    /// Draw the floating shadow (default true).
    pub fn drop_shadow(mut self, shadow: bool) -> Self {
        self.drop_shadow = shadow;
        self
    }
}

impl ParentElement for Popover {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Popover {
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
        let progress = overlay::presence(
            (id.clone(), "fade"),
            open,
            FLOAT_ENTER,
            FLOAT_EXIT,
            window,
            cx,
        );
        let focus = overlay::surface_focus(&id, open, None, window, cx);

        let trigger = {
            let set_open = set_open.clone();
            overlay::trigger_wrapper(
                (id.clone(), "trigger"),
                self.trigger,
                Rc::new(move |window, cx| set_open(!open, window, cx)),
            )
            .into_any_element()
        };
        let look = PopoverLook::current();
        let surface = (open || progress > 0.).then(|| {
            let set_escape = set_open.clone();
            let set_outside = set_open.clone();
            (
                div()
                    .id((id.clone(), "surface"))
                    .track_focus(&focus)
                    .tab_group()
                    .rounded(px(RADIUS))
                    .bg(look.background)
                    .text_color(look.text)
                    .when(self.drop_shadow, |this| {
                        this.shadow(overlay::floating_shadow())
                    })
                    .min_w(rem(CONTENT_MIN_WIDTH))
                    .max_w(rem(CONTENT_MAX_WIDTH))
                    .p(rem(16.))
                    .type_style(TypeStyle::Body01)
                    .on_key_down(move |event: &KeyDownEvent, window, cx| {
                        if event.keystroke.key == "escape" && open {
                            set_escape(false, window, cx);
                            cx.stop_propagation();
                        }
                    })
                    .on_mouse_down_out(move |_, window, cx| {
                        if open {
                            set_outside(false, window, cx);
                        }
                    })
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .children(self.children),
                progress,
            )
        });
        floating::anchored(
            (id, "popup"),
            trigger,
            surface,
            self.align,
            self.caret.then_some(look.background),
            POPOVER_OFFSET,
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
    fn look_and_geometry_follow_popover_css() {
        crate::theme::tests_set(CarbonTheme::White);
        assert_eq!(hex(PopoverLook::current().background), "#f4f4f4");
        crate::theme::tests_set(CarbonTheme::G90);
        assert_eq!(hex(PopoverLook::current().background), "#393939");
        assert_eq!(
            (CONTENT_MIN_WIDTH, CONTENT_MAX_WIDTH, RADIUS),
            (192., 352., 2.)
        );
        let p = Popover::new("p", div());
        assert_eq!(
            (p.align, p.caret, p.drop_shadow),
            (Align::Bottom, true, true)
        );
    }
}
