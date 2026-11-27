//! `NumberInput`: Carbon's number field as the pattern cards draw it
//! (`K.NumberInput`, `.cds--number`), full width with its − / + buttons at
//! the right edge. (u0's [`crate::u0::primitives::Stepper`] is the
//! compact fixed-width variant.)
//!
//! - **Behavior**: Kit's [`InputState`] owns editing; the draft is parsed
//!   and clamped to `min`/`max` on blur or Enter; ↑/↓ step the draft. The
//!   buttons step the value and stay out of the Tab order.
//! - **Appearance** (`.cds--number`): `inline-size: 100%`; the input fills
//!   the row on `field`, a 1px `border-strong` bottom rule, padding
//!   `0 calc(2 × layout-size-height-local + 1rem) 0 1rem`; the controls
//!   (`.cds--number__controls`) take `2 × layout-size-height-local` at the
//!   end, one 2.5rem button each (down first), a 1×16px `border-subtle` rule
//!   between them; focus outlines the whole field in 2px `focus`.
//! - **Read-only** (`.cds--number--readonly`): transparent fill,
//!   `border-subtle` rule, the buttons `icon-disabled` and inert; the value
//!   still takes focus and can be copied.
//!
//! ```ignore
//! NumberInput::new("parallel").label("Max parallel agents").default_value(3.).readonly(true)
//! ```
use crate::{
    components::{
        form::{self, FieldSize, inset_outline},
        input::{FieldLook, FieldState},
    },
    density::{self, Density},
    icons::{CarbonIcon, Icon},
    theme::{LayerLevel, color},
    tokens::{self, ColorRole},
    u0::primitives::stepper::{clamp, format_number, parse_float, stepped},
};
use gpui_kit::{
    App, AppContext as _, Context, CursorStyle, ElementId, Entity, Focusable as _,
    InteractiveElement as _, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    Subscription, Window,
    component::{
        Sizable as _, Size,
        input::{Input as KitInput, InputEvent, InputState, MoveDown, MoveUp},
    },
    div,
    prelude::FluentBuilder as _,
    rems,
};

fn rem(v: f32) -> gpui_kit::Rems {
    rems(v / tokens::REM_PX)
}

/// One control button's width: `layout-size-height-local` of the `md`
/// field, 2.5rem in either density (`inline-size: calc(2 *
/// var(--layout-size-height-local))` for the pair).
pub const CONTROL_WIDTH: f32 = 40.;
/// `.cds--number__rule-divider { block-size: 1rem; inline-size: 0.0625rem }`.
pub const DIVIDER: (f32, f32) = (1., 16.);
/// The input's end padding: both controls plus 1rem.
pub const INPUT_END_PADDING: f32 = 2. * CONTROL_WIDTH + 16.;

struct Model {
    value: f64,
    input: Entity<InputState>,
    editing: bool,
    min: f64,
    max: f64,
    step: f64,
    _subscription: Subscription,
}

impl Model {
    fn set(&mut self, v: f64, window: &mut Window, cx: &mut Context<Self>) {
        self.value = v;
        self.editing = false;
        let text = format_number(v);
        self.input.update(cx, |s, cx| s.set_value(text, window, cx));
        cx.notify();
    }

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
        let v = parse_float(&self.input.read(cx).value())
            .map_or(self.value, |n| clamp(n, self.min, self.max, self.step));
        self.set(v, window, cx);
    }
}

/// A Carbon number input. See the module docs.
#[derive(IntoElement)]
pub struct NumberInput {
    id: ElementId,
    label: Option<SharedString>,
    default_value: f64,
    min: f64,
    max: f64,
    step: f64,
    disabled: bool,
    readonly: bool,
    level: LayerLevel,
    density: Option<Density>,
    decrease_label: SharedString,
    increase_label: SharedString,
}

impl NumberInput {
    /// A number input with a stable `id` (uncontrolled, default 0).
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            default_value: 0.,
            min: f64::NEG_INFINITY,
            max: f64::INFINITY,
            step: 1.,
            disabled: false,
            readonly: false,
            level: LayerLevel::One,
            density: None,
            decrease_label: SharedString::default(),
            increase_label: SharedString::default(),
        }
    }

    /// `label`.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// `defaultValue`.
    pub fn default_value(mut self, value: f64) -> Self {
        self.default_value = value;
        self
    }

    /// `min`.
    pub fn min(mut self, min: f64) -> Self {
        self.min = min;
        self
    }

    /// `max`.
    pub fn max(mut self, max: f64) -> Self {
        self.max = max;
        self
    }

    /// `step` (default 1).
    pub fn step(mut self, step: f64) -> Self {
        self.step = step;
        self
    }

    /// `disabled`.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// `readOnly`.
    pub fn readonly(mut self, readonly: bool) -> Self {
        self.readonly = readonly;
        self
    }

    /// The layer the field sits on (`field-01/02/03`).
    pub fn layer(mut self, level: LayerLevel) -> Self {
        self.level = level;
        self
    }

    /// Override the app density (the field height).
    pub fn density(mut self, density: Density) -> Self {
        self.density = Some(density);
        self
    }

    /// The − / + buttons' accessible names (Carbon `iconDescription`s).
    pub fn button_labels(
        mut self,
        decrease: impl Into<SharedString>,
        increase: impl Into<SharedString>,
    ) -> Self {
        self.decrease_label = decrease.into();
        self.increase_label = increase.into();
        self
    }
}

impl RenderOnce for NumberInput {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let initial = self.default_value;
        let model = window.use_keyed_state((id.clone(), "number"), cx, |window, cx| {
            let input =
                cx.new(|cx| InputState::new(window, cx).default_value(format_number(initial)));
            let subscription = cx.subscribe_in(
                &input,
                window,
                |this: &mut Model, _, e, window, cx| match e {
                    InputEvent::Change => {
                        this.editing = true;
                        cx.notify();
                    }
                    InputEvent::Blur | InputEvent::PressEnter { .. } => this.commit(window, cx),
                    InputEvent::Focus => {}
                },
            );
            Model {
                value: initial,
                input,
                editing: false,
                min: f64::NEG_INFINITY,
                max: f64::INFINITY,
                step: 1.,
                _subscription: subscription,
            }
        });
        model.update(cx, |m, _| {
            m.min = self.min;
            m.max = self.max;
            m.step = self.step;
        });
        let (value, input) = {
            let m = model.read(cx);
            (m.value, m.input.clone())
        };
        crate::components::form::select_all_on_tab_in(&input, window, cx);
        let focused = input.read(cx).focus_handle(cx).is_focused(window);
        let level = self.level;
        let readonly = self.readonly && !self.disabled;
        let look = FieldLook::of(
            level,
            FieldState {
                focused,
                disabled: self.disabled,
                readonly,
            },
        );
        let density = self.density.unwrap_or_else(density::current);
        let height = FieldSize::Md.height(density);

        let button = |key: &'static str, icon: CarbonIcon, label: SharedString, dir: f64| {
            let at_bound = if dir < 0. {
                value <= self.min
            } else {
                value >= self.max
            };
            let inert = self.disabled || readonly;
            let model = model.clone();
            gpui_kit::base::Button::new((id.clone(), key))
                .tab_stop(false)
                .disabled(inert || at_bound)
                .accessibility_label(label)
                .flex_none()
                .flex()
                .items_center()
                .justify_center()
                .w(rem(CONTROL_WIDTH))
                .h_full()
                // `.cds--number__control-btn:hover { background-color: var(--field-hover) }`.
                .when(!inert, |b| b.hover(|s| s.bg(color(level.field_hover()))))
                .cursor(if inert {
                    CursorStyle::Arrow
                } else {
                    CursorStyle::PointingHand
                })
                .child(
                    // `.cds--number--readonly .cds--number__control-btn { color:
                    // var(--icon-disabled) }`; disabled: `fill: icon-disabled`.
                    Icon::carbon(icon)
                        .size_length(rem(16.))
                        .color(color(if inert || at_bound {
                            ColorRole::IconDisabled
                        } else {
                            ColorRole::IconPrimary
                        })),
                )
                .on_click(move |_, window, cx| {
                    model.update(cx, |m, cx| {
                        let v = stepped(m.value, dir, m.min, m.max, m.step);
                        m.set(v, window, cx);
                    })
                })
        };
        let down = button(
            "down",
            CarbonIcon::Subtract,
            self.decrease_label.clone(),
            -1.,
        );
        let up = button("up", CarbonIcon::Add, self.increase_label.clone(), 1.);
        // `.cds--number__rule-divider`: between the buttons (the first one,
        // before "−", is transparent).
        let divider = div()
            .absolute()
            .left(rem(CONTROL_WIDTH - DIVIDER.0 / 2.))
            .top(rem((height - DIVIDER.1) / 2.))
            .w(rem(DIVIDER.0))
            .h(rem(DIVIDER.1))
            .bg(color(if self.disabled {
                ColorRole::IconDisabled
            } else {
                level.border_subtle()
            }));
        let controls = div()
            .absolute()
            .top_0()
            .right_0()
            .h_full()
            .w(rem(2. * CONTROL_WIDTH))
            .flex()
            .flex_row()
            .child(down)
            .child(up)
            .child(divider);

        let key = |dir: f64| {
            let model = model.clone();
            move |window: &mut Window, cx: &mut App| {
                model.update(cx, |m, cx| {
                    let v = stepped(m.base(cx), dir, m.min, m.max, m.step);
                    m.set(v, window, cx);
                })
            }
        };
        let (step_up, step_down) = (key(1.), key(-1.));

        let field = div()
            .id((id.clone(), "field"))
            .relative()
            .flex()
            .items_center()
            .w_full()
            .h(rem(height))
            .bg(look.background)
            .when(!self.disabled && !readonly, |this| {
                this.capture_action(move |_: &MoveUp, window, cx| {
                    step_up(window, cx);
                    cx.stop_propagation();
                })
                .capture_action(move |_: &MoveDown, window, cx| {
                    step_down(window, cx);
                    cx.stop_propagation();
                })
            })
            .child(
                div()
                    .flex()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .items_center()
                    .pl(rem(16.))
                    .pr(rem(INPUT_END_PADDING))
                    .cursor(if self.disabled {
                        CursorStyle::OperationNotAllowed
                    } else {
                        CursorStyle::IBeam
                    })
                    .child(
                        KitInput::new(&input)
                            .appearance(false)
                            .with_size(Size::XSmall)
                            .disabled(self.disabled)
                            .readonly(self.readonly)
                            .when_some(self.label.clone(), |e, l| e.aria_label(l))
                            .p_0()
                            .h_full()
                            .w_full()
                            .text_size(rem(14.))
                            .line_height(rem(18.))
                            .text_color(look.text),
                    ),
            )
            .child(controls)
            .child(form::bottom_rule(look.border_bottom))
            .when(focused, |this| {
                this.child(inset_outline(2., color(ColorRole::Focus)))
            });
        form::labeled(self.label.as_ref(), self.disabled, field, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry_follows_cds_number() {
        // `padding-inline: 1rem calc(2 * var(--layout-size-height-local) + 1rem)`
        // with the md local size (2.5rem): the reference's 96px end padding,
        // the controls 80px wide.
        assert_eq!(CONTROL_WIDTH, FieldSize::Md.carbon_height());
        assert_eq!(INPUT_END_PADDING, 96.);
        assert_eq!(DIVIDER, (1., 16.));
    }

    #[test]
    fn readonly_look_is_transparent_with_a_subtle_rule() {
        crate::theme::tests_set(crate::CarbonTheme::White);
        let ro = FieldLook::of(
            LayerLevel::One,
            FieldState {
                readonly: true,
                ..Default::default()
            },
        );
        assert_eq!(ro.background.a, 0.);
        assert_eq!(ro.border_bottom, color(ColorRole::BorderSubtle01));
        assert_eq!(ro.text, color(ColorRole::TextPrimary));
    }
}
