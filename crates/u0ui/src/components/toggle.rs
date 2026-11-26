//! Toggle: shadcn `Toggle`, a two-state button. Carbon has no core toggle
//! button; the reference draws a ghost button whose pressed state is
//! `layer-selected-01` (`.cu-toggle` in `src/custom.css`).
//!
//! - **Behavior** is Kit's [`gpui_kit::base::Toggle`]: focus, Enter/Space and
//!   pointer activation, `aria-pressed`, disabled.
//! - **Appearance**: transparent, `text-secondary`; hover `background-hover` +
//!   `text-primary`; pressed `layer-selected-01` + `text-primary`; `outline`
//!   adds a 1px `border-strong-01` (`border-inverse` when pressed);
//!   `:focus-visible` 2px inset `focus`; disabled `text-disabled`.
//! - **Motion**: `transition: background 70ms cubic-bezier(0.2,0,0.38,0.9)`.
//!
//! ```ignore
//! Toggle::new("bold").aria_label("Bold").default_pressed(true).child("B")
//! ```
use super::form::{self, rem};
use crate::{
    motion::{self, Curve, Motion, Speed},
    theme::color,
    tokens::{self, ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    AnyElement, App, CursorStyle, ElementId, Hsla, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement as _, Styled, Window, prelude::FluentBuilder as _,
};
use std::rc::Rc;

/// `.cu-toggle { transition: background 70ms cubic-bezier(0.2,0,0.38,0.9) }`.
pub const TOGGLE_MOTION: Motion = Motion::new(Curve::ProductiveStandard, Speed::Fast01);

/// `index.d.ts` `ToggleProps.variant`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ToggleVariant {
    /// Borderless (`.cu-toggle--default`).
    #[default]
    Default,
    /// 1px `border-strong-01` (`.cu-toggle--outline`).
    Outline,
}

impl ToggleVariant {
    /// Both variants.
    pub const ALL: [Self; 2] = [Self::Default, Self::Outline];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Outline => "outline",
        }
    }
}

/// `index.d.ts` `Size` for the toggle: `sm`/`md`/`lg` = `size-sm/md/lg`
/// (`.cu-toggle--sm { height: var(--size-sm); min-width: var(--size-sm) }`).
pub use super::form::FieldSize as ToggleSize;

/// Toggle height (and min width): Carbon's 32/40/48 in every density
/// (`.cu-toggle { height: var(--size-md) }`, `--sm`/`--lg`); the reference's
/// `[data-density="compact"]` rules (`src/ghost.css`) do not touch `.cu-toggle`.
pub fn toggle_height(size: ToggleSize) -> f32 {
    size.carbon_height()
}

/// The state a toggle look is computed for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ToggleState {
    /// `aria-pressed`.
    pub pressed: bool,
    /// Pointer over it.
    pub hovered: bool,
    /// Disabled.
    pub disabled: bool,
}

/// The colors of a toggle in one state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToggleLook {
    /// Fill.
    pub background: Hsla,
    /// 1px border.
    pub border: Hsla,
    /// Text and icon color.
    pub text: Hsla,
}

impl ToggleLook {
    /// The look of `variant` in `state` (`.cu-toggle*` rules).
    pub fn of(variant: ToggleVariant, state: ToggleState) -> Self {
        let clear = Hsla::transparent_black();
        let outline = variant == ToggleVariant::Outline;
        if state.disabled {
            // `.cu-toggle:disabled { color: var(--text-disabled); background: transparent }`.
            return Self {
                background: clear,
                border: if outline {
                    color(ColorRole::BorderStrong01)
                } else {
                    clear
                },
                text: color(ColorRole::TextDisabled),
            };
        }
        let background = if state.pressed {
            // `.cu-toggle--on { background: var(--layer-selected-01) }` (after `:hover`).
            color(ColorRole::LayerSelected01)
        } else if state.hovered {
            // `.cu-toggle:hover { background: var(--background-hover) }`.
            color(ColorRole::BackgroundHover)
        } else {
            clear
        };
        let border = match (outline, state.pressed) {
            (false, _) => clear,
            // `.cu-toggle--outline.cu-toggle--on { border-color: var(--border-inverse) }`.
            (true, true) => color(ColorRole::BorderInverse),
            (true, false) => color(ColorRole::BorderStrong01),
        };
        let text = if state.pressed || state.hovered {
            color(ColorRole::TextPrimary)
        } else {
            color(ColorRole::TextSecondary)
        };
        Self {
            background,
            border,
            text,
        }
    }
}

type ChangeHandler = Rc<dyn Fn(bool, &mut Window, &mut App)>;

/// A two-state button. See the module docs.
#[derive(IntoElement)]
pub struct Toggle {
    id: ElementId,
    pressed: Option<bool>,
    default_pressed: bool,
    variant: ToggleVariant,
    size: ToggleSize,
    disabled: bool,
    aria_label: Option<SharedString>,
    children: Vec<AnyElement>,
    on_pressed_change: Option<ChangeHandler>,
}

impl Toggle {
    /// A toggle with a stable `id`. Add its content with `child`.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            pressed: None,
            default_pressed: false,
            variant: ToggleVariant::Default,
            size: ToggleSize::Md,
            disabled: false,
            aria_label: None,
            children: Vec::new(),
            on_pressed_change: None,
        }
    }

    /// Controlled `pressed`.
    pub fn pressed(mut self, pressed: bool) -> Self {
        self.pressed = Some(pressed);
        self
    }

    /// `defaultPressed` for an uncontrolled toggle.
    pub fn default_pressed(mut self, pressed: bool) -> Self {
        self.default_pressed = pressed;
        self
    }

    /// `onPressedChange`.
    pub fn on_pressed_change(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_pressed_change = Some(Rc::new(handler));
        self
    }

    /// `variant`.
    pub fn variant(mut self, variant: ToggleVariant) -> Self {
        self.variant = variant;
        self
    }

    /// `size` (default `md`).
    pub fn size(mut self, size: ToggleSize) -> Self {
        self.size = size;
        self
    }

    /// Disabled.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// `aria-label` (required for icon-only toggles).
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }
}

impl ParentElement for Toggle {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Toggle {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let height = toggle_height(self.size);
        let (pressed, store) = form::controlled(
            (id.clone(), "pressed"),
            self.pressed,
            self.default_pressed,
            window,
            cx,
        );
        let hovered = window.use_keyed_state((id.clone(), "hover"), cx, |_, _| false);
        let focus = window
            .use_keyed_state((id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let focus_visible = focus.is_focused(window) && window.last_input_was_keyboard();
        let look = ToggleLook::of(
            self.variant,
            ToggleState {
                pressed,
                hovered: *hovered.read(cx) && !self.disabled,
                disabled: self.disabled,
            },
        );
        let background = motion::animate_color(
            (id.clone(), "bg"),
            look.background,
            TOGGLE_MOTION,
            window,
            cx,
        );
        let on_change = self.on_pressed_change.clone();

        gpui_kit::base::Toggle::new(id.clone())
            .track_focus(&focus)
            .pressed(pressed)
            .disabled(self.disabled)
            .when_some(self.aria_label.clone(), |this, label| {
                this.accessibility_label(label)
            })
            .relative()
            .flex()
            .flex_shrink_0()
            .items_center()
            .justify_center()
            .gap(tokens::SPACING_03)
            .h(rem(height))
            .min_w(rem(height))
            .px(tokens::SPACING_04)
            .border_1()
            .border_color(look.border)
            .bg(background)
            .type_style(TypeStyle::BodyCompact01)
            .text_color(look.text)
            .cursor(if self.disabled {
                CursorStyle::OperationNotAllowed
            } else {
                CursorStyle::PointingHand
            })
            .on_hover(move |over, _, cx| {
                hovered.update(cx, |h, cx| {
                    if *h != *over {
                        *h = *over;
                        cx.notify();
                    }
                })
            })
            .on_change(move |next, _, window, cx| {
                store.update(cx, |p, cx| {
                    *p = next;
                    cx.notify();
                });
                if let Some(handler) = on_change.as_ref() {
                    handler(next, window, cx);
                }
            })
            .children(self.children)
            // `.cu-toggle:focus-visible { outline: 2px solid var(--focus); outline-offset: -2px }`.
            .when(focus_visible, |this| {
                this.child(form::inset_outline(2., color(ColorRole::Focus)))
            })
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
    fn looks_follow_cu_toggle_css() {
        crate::theme::tests_set(CarbonTheme::White);
        let rest = ToggleLook::of(ToggleVariant::Default, ToggleState::default());
        assert_eq!(rest.background.a, 0.);
        assert_eq!(hex(rest.text), "#525252");
        let on = ToggleLook::of(
            ToggleVariant::Default,
            ToggleState {
                pressed: true,
                ..Default::default()
            },
        );
        // `.cu-toggle--on { background: var(--layer-selected-01) }`.
        assert_eq!(hex(on.background), "#e0e0e0");
        assert_eq!(hex(on.text), "#161616");
        let outline_on = ToggleLook::of(
            ToggleVariant::Outline,
            ToggleState {
                pressed: true,
                ..Default::default()
            },
        );
        assert_eq!(hex(outline_on.border), "#161616");
        let outline = ToggleLook::of(ToggleVariant::Outline, ToggleState::default());
        assert_eq!(hex(outline.border), "#8d8d8d");
        let hover = ToggleLook::of(
            ToggleVariant::Default,
            ToggleState {
                hovered: true,
                ..Default::default()
            },
        );
        assert_eq!(hover.background, color(ColorRole::BackgroundHover));
        let disabled = ToggleLook::of(
            ToggleVariant::Default,
            ToggleState {
                disabled: true,
                pressed: true,
                hovered: true,
            },
        );
        assert_eq!(disabled.background.a, 0.);
    }

    #[test]
    fn heights_follow_size_tokens_in_every_density() {
        // `.cu-toggle--sm/md/lg { height: var(--size-sm/md/lg) }`; no density rule.
        let heights = ToggleSize::ALL.map(toggle_height);
        assert_eq!(heights, [32., 40., 48.]);
    }
}
