//! Stepper: a compact number input (`Stepper` in `src/ghost/primitives.jsx`,
//! Carbon `NumberInput` with its buttons on the right and a controllable width).
//!
//! - **Behavior**: Kit's [`InputState`] owns editing; the draft is parsed and
//!   clamped to `min`/`max` on blur (Enter blurs). ↑/↓ add or subtract `step`
//!   from the draft (or the value). The −/+ buttons are out of the Tab order
//!   (`tabIndex={-1}`) and disable at the bounds.
//! - **Appearance** (`.g-stepper`): `field-01` fill, 1px `border-strong-01`
//!   bottom rule, 2px inset `focus` outline while editing, `support-error`
//!   outline when invalid; buttons `density-field-md` square with a 1px
//!   `border-subtle-01` left rule and `field-hover-01` hover; disabled at 50%
//!   opacity without the rule.
//! - **Read-only** (`.cds--number--readonly`): transparent fill, `border-subtle`
//!   rule, the value focusable and copyable, the buttons `icon-disabled` and inert.
//!
//! ```ignore
//! Stepper::new("parallel").label("Max parallel agents").default_value(3.).min(1.).max(8.).helper_text("1–8")
//! Stepper::new("timeout").label("Timeout").default_value(30.).min(5.).step(5.).unit("min").width(rems(8.))
//! ```
use super::parts::{self, rem};
use crate::{
    components::form,
    density,
    icons::{CarbonIcon, Icon},
    theme::color,
    tokens::ColorRole,
};
use gpui_kit::{
    App, AppContext as _, Context, CursorStyle, ElementId, Entity, Focusable as _, Hsla,
    InteractiveElement, IntoElement, MouseButton, ParentElement, Rems, RenderOnce, SharedString,
    StatefulInteractiveElement as _, Styled, Subscription, Window,
    component::input::{InputEvent, InputState, MoveDown, MoveUp},
    div,
    prelude::FluentBuilder as _,
    px,
};
use std::rc::Rc;

/// The number of decimals in `step` (`String(step).split('.')[1].length`).
pub fn step_decimals(step: f64) -> usize {
    let s = format_number(step);
    s.split_once('.').map_or(0, |(_, frac)| frac.len())
}

/// `clamp`: `+Math.min(max, Math.max(min, n)).toFixed(dec)`.
pub fn clamp(n: f64, min: f64, max: f64, step: f64) -> f64 {
    let v = n.max(min).min(max);
    let dec = step_decimals(step) as i32;
    let f = 10f64.powi(dec);
    (v * f).round() / f
}

/// `String(v)` for the values a stepper holds (`3`, `0.5`, `-2.25`).
pub fn format_number(v: f64) -> String {
    if v == v.trunc() && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

/// `parseFloat(draft)`: the longest leading decimal number (after leading
/// whitespace), or `None` (`NaN`).
pub fn parse_float(text: &str) -> Option<f64> {
    let t = text.trim_start();
    let bytes = t.as_bytes();
    let mut end = 0;
    let mut seen_dot = false;
    let mut best = None;
    if end < bytes.len() && (bytes[end] == b'-' || bytes[end] == b'+') {
        end += 1;
    }
    while end < bytes.len() {
        match bytes[end] {
            b'0'..=b'9' => {
                end += 1;
                best = Some(end);
            }
            b'.' if !seen_dot => {
                seen_dot = true;
                end += 1;
            }
            _ => break,
        }
    }
    best.and_then(|e| t[..e].parse().ok())
}

/// The value a key or button press sets: `clamp(base ± step)`.
pub fn stepped(base: f64, dir: f64, min: f64, max: f64, step: f64) -> f64 {
    clamp(base + dir * step, min, max, step)
}

/// The `.g-stepper` box colors: (fill, bottom rule, outline).
pub fn box_look(focused: bool, invalid: bool, disabled: bool) -> (Hsla, Hsla, Hsla) {
    let clear = Hsla::transparent_black();
    let fill = color(ColorRole::Field01);
    // `.g-stepper--disabled { border-bottom-color: transparent }`.
    let rule = if disabled {
        clear
    } else {
        color(ColorRole::BorderStrong01)
    };
    // `.g-stepper:focus-within { outline-color: focus }`, then the later
    // `.g-field--invalid .g-stepper { outline-color: support-error }` wins.
    let outline = if invalid {
        color(ColorRole::SupportError)
    } else if focused {
        color(ColorRole::Focus)
    } else {
        clear
    };
    (fill, rule, outline)
}

/// The read-only box colors (fill, bottom rule, outline): `.cds--number--readonly
/// input { background: transparent; border-block-end-color: var(--border-subtle) }`,
/// still outlined in `focus` while focused (read-only fields take focus).
pub fn readonly_box_look(focused: bool) -> (Hsla, Hsla, Hsla) {
    let clear = Hsla::transparent_black();
    let outline = if focused {
        color(ColorRole::Focus)
    } else {
        clear
    };
    (clear, color(ColorRole::BorderSubtle01), outline)
}

type ValueHandler = Rc<dyn Fn(f64, &mut Window, &mut App)>;

struct Model {
    value: f64,
    input: Entity<InputState>,
    editing: bool,
    min: f64,
    max: f64,
    step: f64,
    controlled: bool,
    on_change: Option<ValueHandler>,
    _subscription: Subscription,
}

impl Model {
    fn set(&mut self, v: f64, window: &mut Window, cx: &mut Context<Self>) {
        if !self.controlled {
            self.value = v;
        }
        self.editing = false;
        let text = format_number(if self.controlled { v } else { self.value });
        self.input.update(cx, |s, cx| s.set_value(text, window, cx));
        if let Some(f) = self.on_change.clone() {
            f(v, window, cx);
        }
        cx.notify();
    }

    /// `base`: the parsed draft while editing, else the value.
    fn base(&self, cx: &App) -> f64 {
        if self.editing {
            parse_float(&self.input.read(cx).value()).unwrap_or(self.value)
        } else {
            self.value
        }
    }

    fn commit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.editing {
            return;
        }
        // `onBlur: const n = parseFloat(draft); if (!isNaN(n)) set(clamp(n)); setDraft(null)`.
        match parse_float(&self.input.read(cx).value()) {
            Some(n) => self.set(clamp(n, self.min, self.max, self.step), window, cx),
            None => {
                self.editing = false;
                let text = format_number(self.value);
                self.input.update(cx, |s, cx| s.set_value(text, window, cx));
                cx.notify();
            }
        }
    }
}

/// A compact number input. See the module docs.
#[derive(IntoElement)]
pub struct Stepper {
    id: ElementId,
    label: Option<SharedString>,
    value: Option<f64>,
    default_value: f64,
    min: f64,
    max: f64,
    step: f64,
    unit: Option<SharedString>,
    disabled: bool,
    readonly: bool,
    invalid: bool,
    invalid_text: Option<SharedString>,
    helper_text: Option<SharedString>,
    width: Rems,
    decrease_label: SharedString,
    increase_label: SharedString,
    on_value_change: Option<ValueHandler>,
}

impl Stepper {
    /// A stepper with a stable `id` (uncontrolled, default value 0).
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            value: None,
            default_value: 0.,
            min: f64::NEG_INFINITY,
            max: f64::INFINITY,
            step: 1.,
            unit: None,
            disabled: false,
            readonly: false,
            invalid: false,
            invalid_text: None,
            helper_text: None,
            // `width = '7rem'`.
            width: gpui_kit::rems(7.),
            // The reference's `aria-label`s; override per locale.
            decrease_label: "Decrease".into(),
            increase_label: "Increase".into(),
            on_value_change: None,
        }
    }

    /// The label above the field.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Control the value (the caller updates it from [`Self::on_value_change`]).
    pub fn value(mut self, value: f64) -> Self {
        self.value = Some(value);
        self
    }

    /// The initial value when uncontrolled.
    pub fn default_value(mut self, value: f64) -> Self {
        self.default_value = value;
        self
    }

    /// Lower bound (default −∞).
    pub fn min(mut self, min: f64) -> Self {
        self.min = min;
        self
    }

    /// Upper bound (default +∞).
    pub fn max(mut self, max: f64) -> Self {
        self.max = max;
        self
    }

    /// Increment for ↑/↓ and the buttons (default 1); its decimals set the rounding.
    pub fn step(mut self, step: f64) -> Self {
        self.step = step;
        self
    }

    /// A unit after the number (`min`, `MB`).
    pub fn unit(mut self, unit: impl Into<SharedString>) -> Self {
        self.unit = Some(unit.into());
        self
    }

    /// Disabled.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Read-only (`readOnly`): the value can be focused, selected and copied
    /// but not changed; the buttons draw in `icon-disabled` and do nothing
    /// (`.cds--number--readonly .cds--number__control-btn { color: var(--icon-disabled);
    /// pointer-events: none }`).
    pub fn readonly(mut self, readonly: bool) -> Self {
        self.readonly = readonly;
        self
    }

    /// Invalid: `support-error` outline; shows `invalid_text` instead of the helper.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// The error message when invalid.
    pub fn invalid_text(mut self, text: impl Into<SharedString>) -> Self {
        self.invalid_text = Some(text.into());
        self
    }

    /// Helper text under the field.
    pub fn helper_text(mut self, text: impl Into<SharedString>) -> Self {
        self.helper_text = Some(text.into());
        self
    }

    /// The field width (default 7rem).
    pub fn width(mut self, width: Rems) -> Self {
        self.width = width;
        self
    }

    /// Accessible names of the −/+ buttons (default "Decrease" / "Increase").
    pub fn button_labels(
        mut self,
        decrease: impl Into<SharedString>,
        increase: impl Into<SharedString>,
    ) -> Self {
        self.decrease_label = decrease.into();
        self.increase_label = increase.into();
        self
    }

    /// Called with each new value.
    pub fn on_value_change(mut self, f: impl Fn(f64, &mut Window, &mut App) + 'static) -> Self {
        self.on_value_change = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for Stepper {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let initial = self.value.unwrap_or(self.default_value);
        let model = window.use_keyed_state((id.clone(), "stepper"), cx, |window, cx| {
            let input =
                cx.new(|cx| InputState::new(window, cx).default_value(format_number(initial)));
            let subscription =
                cx.subscribe_in(&input, window, |this: &mut Model, _, e, window, cx| {
                    match e {
                        InputEvent::Change => {
                            this.editing = true;
                            cx.notify();
                        }
                        InputEvent::Blur => this.commit(window, cx),
                        InputEvent::PressEnter { .. } => {
                            // `if (e.key === 'Enter') e.currentTarget.blur()`.
                            this.commit(window, cx);
                            window.blur(cx);
                        }
                        InputEvent::Focus => {}
                    }
                });
            Model {
                value: initial,
                input,
                editing: false,
                min: f64::NEG_INFINITY,
                max: f64::INFINITY,
                step: 1.,
                controlled: false,
                on_change: None,
                _subscription: subscription,
            }
        });
        // Refresh the props the model's handlers read.
        model.update(cx, |m, cx| {
            m.min = self.min;
            m.max = self.max;
            m.step = self.step;
            m.on_change = self.on_value_change.clone();
            m.controlled = self.value.is_some();
            if let Some(v) = self.value
                && (v != m.value)
            {
                m.value = v;
                if !m.editing {
                    let text = format_number(v);
                    m.input.update(cx, |s, cx| s.set_value(text, window, cx));
                }
            }
        });
        let (value, input) = {
            let m = model.read(cx);
            (m.value, m.input.clone())
        };
        let focused = input.read(cx).focus_handle(cx).is_focused(window);
        let (fill, rule, outline) = if self.readonly && !self.disabled {
            readonly_box_look(focused)
        } else {
            box_look(focused, self.invalid, self.disabled)
        };
        let readonly = self.readonly;
        let field_h = density::current().field().px_value();

        let mut button =
            |key: &'static str, icon: CarbonIcon, label: SharedString, dir: f64, at_bound: bool| {
                let model = model.clone();
                let inert = readonly && !self.disabled;
                let disabled = self.disabled || at_bound || inert;
                let hovered = window.use_keyed_state(
                    (ElementId::from((id.clone(), key)), "hover"),
                    cx,
                    |_, _| false,
                );
                let is_hovered = *hovered.read(cx) && !disabled;
                gpui_kit::base::Button::new((id.clone(), key))
                    .tab_stop(false)
                    .disabled(disabled)
                    .accessibility_label(label)
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .size(rem(field_h))
                    // `border-left: 1px solid var(--border-subtle-01)`.
                    .border_l(px(1.))
                    .border_color(color(ColorRole::BorderSubtle01))
                    // `:hover:not(:disabled) { background: var(--field-hover-01) }`.
                    .when(is_hovered, |b| b.bg(color(ColorRole::FieldHover01)))
                    .cursor(if inert {
                        CursorStyle::Arrow
                    } else if disabled {
                        CursorStyle::OperationNotAllowed
                    } else {
                        CursorStyle::PointingHand
                    })
                    .on_hover(move |h, _, cx| {
                        hovered.update(cx, |v, cx| {
                            if *v != *h {
                                *v = *h;
                                cx.notify();
                            }
                        })
                    })
                    .child(
                        Icon::carbon(icon)
                            .size_length(px(14.))
                            .color(color(if disabled {
                                ColorRole::IconDisabled
                            } else {
                                ColorRole::IconPrimary
                            })),
                    )
                    .on_click(move |_, window, cx| {
                        model.update(cx, |m, cx| {
                            // `onClick={() => set(clamp(v ± step))}` from the value.
                            let v = stepped(m.value, dir, m.min, m.max, m.step);
                            m.set(v, window, cx);
                        })
                    })
            };
        let dec = button(
            "dec",
            CarbonIcon::Subtract,
            self.decrease_label.clone(),
            -1.,
            value <= self.min,
        );
        let inc = button(
            "inc",
            CarbonIcon::Add,
            self.increase_label.clone(),
            1.,
            value >= self.max,
        );

        let key = |dir: f64| {
            let model = model.clone();
            move |window: &mut Window, cx: &mut App| {
                model.update(cx, |m, cx| {
                    let v = stepped(m.base(cx), dir, m.min, m.max, m.step);
                    m.set(v, window, cx);
                })
            }
        };
        let up = key(1.);
        let down = key(-1.);

        let body = div()
            .id((id.clone(), "box"))
            .relative()
            .flex()
            .items_center()
            .w(self.width)
            .min_h(rem(field_h))
            .bg(fill)
            .when(self.disabled, |d| d.opacity(0.5))
            .when(!self.disabled, |this| {
                this.when(!readonly, |this| {
                    this.capture_action(move |_: &MoveUp, window, cx| {
                        up(window, cx);
                        cx.stop_propagation();
                    })
                    .capture_action(move |_: &MoveDown, window, cx| {
                        down(window, cx);
                        cx.stop_propagation();
                    })
                })
                .on_mouse_down(MouseButton::Left, {
                    let focus = input.read(cx).focus_handle(cx);
                    move |_, window, cx| focus.focus(window, cx)
                })
            })
            .child(
                // `.g-stepper-input { padding: 0 var(--spacing-03) 0 var(--spacing-04);
                // font-variant-numeric: tabular-nums }`.
                div()
                    .flex()
                    .flex_1()
                    .min_w_0()
                    .h(rem(field_h))
                    .pl(rem(12.))
                    .pr(rem(8.))
                    .items_center()
                    .cursor(if self.disabled {
                        CursorStyle::OperationNotAllowed
                    } else {
                        CursorStyle::IBeam
                    })
                    .child(
                        parts::editor(&input)
                            .disabled(self.disabled)
                            .readonly(self.readonly)
                            .when_some(self.label.clone(), |e, l| e.aria_label(l))
                            .text_color(color(ColorRole::TextPrimary)),
                    ),
            )
            .when_some(self.unit.clone(), |this, unit| {
                // `.g-stepper-unit { padding-right: spacing-03; font-size: 0.75rem; color: text-secondary }`.
                this.child(
                    div()
                        .flex_none()
                        .pr(rem(8.))
                        .text_size(rem(12.))
                        .text_color(color(ColorRole::TextSecondary))
                        .child(unit),
                )
            })
            .child(dec)
            .child(inc)
            .child(form::bottom_rule(rule))
            .child(form::inset_outline(2., outline));

        let message = if self.invalid && self.invalid_text.is_some() {
            self.invalid_text.clone().map(parts::error)
        } else {
            self.helper_text.clone().map(parts::helper)
        };
        parts::field()
            .when_some(self.label.clone(), |f, l| f.child(parts::label(l)))
            .child(body)
            .children(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CarbonTheme, theme::color_in};

    #[test]
    fn clamp_rounds_to_the_step_decimals() {
        assert_eq!(step_decimals(1.), 0);
        assert_eq!(step_decimals(0.25), 2);
        assert_eq!(clamp(12., f64::NEG_INFINITY, 10., 1.), 10.);
        assert_eq!(clamp(0.1 + 0.2, 0., 1., 0.1), 0.3);
        assert_eq!(clamp(-3., 1., 8., 1.), 1.);
        assert_eq!(stepped(30., -1., 5., f64::INFINITY, 5.), 25.);
        assert_eq!(stepped(5., -1., 5., f64::INFINITY, 5.), 5.);
    }

    #[test]
    fn parse_float_matches_js() {
        assert_eq!(parse_float("42"), Some(42.));
        assert_eq!(parse_float("  3.5kg"), Some(3.5));
        assert_eq!(parse_float("-2"), Some(-2.));
        assert_eq!(parse_float(".5"), Some(0.5));
        assert_eq!(parse_float("abc"), None);
        assert_eq!(parse_float(""), None);
        assert_eq!(parse_float("-"), None);
        assert_eq!(format_number(3.), "3");
        assert_eq!(format_number(0.5), "0.5");
    }

    #[test]
    fn box_states_follow_g_stepper() {
        crate::theme::tests_set(CarbonTheme::G10);
        let c = |r| color_in(CarbonTheme::G10, r);
        let (fill, rule, outline) = box_look(false, false, false);
        assert_eq!(fill, c(ColorRole::Field01));
        assert_eq!(rule, c(ColorRole::BorderStrong01));
        assert_eq!(outline, Hsla::transparent_black());
        assert_eq!(box_look(true, false, false).2, c(ColorRole::Focus));
        // The invalid rule comes later in the cascade than `:focus-within`.
        assert_eq!(box_look(true, true, false).2, c(ColorRole::SupportError));
        assert_eq!(box_look(false, false, true).1, Hsla::transparent_black());
        let (fill, rule, outline) = readonly_box_look(false);
        assert_eq!(fill, Hsla::transparent_black());
        assert_eq!(rule, c(ColorRole::BorderSubtle01));
        assert_eq!(outline, Hsla::transparent_black());
        assert_eq!(readonly_box_look(true).2, c(ColorRole::Focus));
    }
}
