//! SteppedSlider: a few named, discrete levels (`SteppedSlider` in
//! `src/ghost/primitives.jsx`), e.g. telemetry 0–3.
//!
//! - **Behavior**: the track is one focusable range (the reference's native
//!   `<input type="range" step=1>`): ←/↓ and →/↑ step by one level,
//!   Home/End jump to the ends, pressing or dragging on the track picks the
//!   nearest level. The tick names are pointer shortcuts (`tabIndex={-1}`).
//!   Written here because Kit's slider draws a continuous track whose
//!   geometry differs from this one.
//! - **Appearance** (`.g-sslider-*`): a 16px-high range with a 2px track,
//!   `border-interactive` up to the value and `border-subtle-01` after it, a
//!   14px `icon-primary` round thumb, a 2px `focus` outline 2px outside on
//!   keyboard focus; ticks (1×4 `border-strong-01` marks over 12px names in
//!   `text-secondary`, the current one `text-primary` 600) inset 7px so they
//!   line up with the thumb's travel; the description below in helper text.
use super::parts::{self, rem};
use crate::{
    theme::color,
    tokens::{ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    App, Bounds, CursorStyle, ElementId, FontWeight, InteractiveElement, IntoElement, KeyDownEvent,
    MouseButton, ParentElement, Pixels, RenderOnce, SharedString, StatefulInteractiveElement as _,
    Styled, Window, canvas, div, prelude::FluentBuilder as _, px, relative,
};
use std::{cell::Cell, rc::Rc};

/// The thumb diameter (`::-webkit-slider-thumb { width: 14px; height: 14px }`).
pub const THUMB: f32 = 14.;

/// The value's position along the track, 0–1 (`--pct: v / (steps.length - 1)`).
pub fn fraction(value: usize, steps: usize) -> f32 {
    if steps <= 1 {
        0.
    } else {
        value.min(steps - 1) as f32 / (steps - 1) as f32
    }
}

/// The level nearest `x` on a track `left..left+width` whose thumb center
/// travels `THUMB / 2` inside each end (a native range).
pub fn level_at(x: f32, left: f32, width: f32, steps: usize) -> usize {
    if steps <= 1 {
        return 0;
    }
    let travel = (width - THUMB).max(1.);
    let p = ((x - left - THUMB / 2.) / travel).clamp(0., 1.);
    (p * (steps - 1) as f32).round() as usize
}

/// The level after a key press, or `None` for other keys.
pub fn level_after_key(key: &str, value: usize, steps: usize) -> Option<usize> {
    let last = steps.saturating_sub(1);
    Some(match key {
        "left" | "down" | "pagedown" => value.saturating_sub(1),
        "right" | "up" | "pageup" => (value + 1).min(last),
        "home" => 0,
        "end" => last,
        _ => return None,
    })
}

type Handler = Rc<dyn Fn(usize, &mut Window, &mut App)>;
type Describe = Rc<dyn Fn(usize) -> SharedString>;

/// A slider over named levels. See the module docs.
#[derive(IntoElement)]
pub struct SteppedSlider {
    id: ElementId,
    label: Option<SharedString>,
    steps: Vec<SharedString>,
    value: Option<usize>,
    default_value: usize,
    describe: Option<Describe>,
    on_value_change: Option<Handler>,
}

impl SteppedSlider {
    /// A slider with a stable `id` over the level names `steps`.
    pub fn new(
        id: impl Into<ElementId>,
        steps: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        Self {
            id: id.into(),
            label: None,
            steps: steps.into_iter().map(Into::into).collect(),
            value: None,
            default_value: 0,
            describe: None,
            on_value_change: None,
        }
    }

    /// The label above.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Control the level (an index into `steps`).
    pub fn value(mut self, value: usize) -> Self {
        self.value = Some(value);
        self
    }

    /// The initial level when uncontrolled.
    pub fn default_value(mut self, value: usize) -> Self {
        self.default_value = value;
        self
    }

    /// `describe(i)`: the description shown under the current level.
    pub fn describe(mut self, f: impl Fn(usize) -> SharedString + 'static) -> Self {
        self.describe = Some(Rc::new(f));
        self
    }

    /// Called with each new level.
    pub fn on_value_change(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_value_change = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for SteppedSlider {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let n = self.steps.len();
        let own = window.use_keyed_state((id.clone(), "value"), cx, {
            let d = self.default_value;
            move |_, _| d
        });
        let dragging = window.use_keyed_state((id.clone(), "drag"), cx, |_, _| false);
        let bounds: Rc<Cell<Bounds<Pixels>>> = window
            .use_keyed_state((id.clone(), "bounds"), cx, |_, _| {
                Rc::new(Cell::new(Bounds::default()))
            })
            .read(cx)
            .clone();
        let value = self.value.unwrap_or(*own.read(cx)).min(n.saturating_sub(1));
        let set: Handler = {
            let own = own.clone();
            let f = self.on_value_change.clone();
            let controlled = self.value.is_some();
            Rc::new(move |v, window, cx| {
                if !controlled {
                    own.update(cx, |s, cx| {
                        if *s != v {
                            *s = v;
                            cx.notify();
                        }
                    });
                }
                if let Some(f) = &f {
                    f(v, window, cx);
                }
            })
        };
        let focus = parts::keyed_focus((id.clone(), "focus"), window, cx);
        let ring = parts::focus_visible(&focus, window);
        let pct = fraction(value, n);

        let pick = {
            let bounds = bounds.clone();
            let set = set.clone();
            move |x: Pixels, window: &mut Window, cx: &mut App| {
                let b = bounds.get();
                let v = level_at(
                    f32::from(x),
                    f32::from(b.origin.x),
                    f32::from(b.size.width),
                    n,
                );
                set(v, window, cx);
            }
        };
        let pick = Rc::new(pick);

        // `.g-sslider-range { height: 1rem; background: linear-gradient(to right,
        // var(--border-interactive) var(--pct), var(--border-subtle-01) var(--pct))
        // center / 100% 2px no-repeat }`. The thumb center travels 7px inside
        // each end, as a native range's does.
        let thumb_left = move |p: f32| {
            // left = p × (100% − 14px)
            (p, THUMB * p)
        };
        let (tp, toff) = thumb_left(pct);
        let range = div()
            .id((id.clone(), "range"))
            .track_focus(&focus)
            .tab_index(0)
            .relative()
            .w_full()
            .h(rem(16.))
            .cursor(CursorStyle::PointingHand)
            .child(
                canvas(
                    {
                        let bounds = bounds.clone();
                        move |b, _, _| bounds.set(b)
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .size_full(),
            )
            .child(
                div()
                    .absolute()
                    .left_0()
                    .right_0()
                    .top(rem(7.))
                    .h(px(2.))
                    .bg(color(ColorRole::BorderSubtle01))
                    .child(
                        div()
                            .absolute()
                            .top_0()
                            .bottom_0()
                            .left_0()
                            .w(relative(pct))
                            .bg(color(ColorRole::BorderInteractive)),
                    ),
            )
            .child(
                div()
                    .absolute()
                    .top(rem(1.))
                    .left(relative(tp))
                    .ml(px(-toff))
                    .size(rem(THUMB))
                    .rounded_full()
                    .bg(color(ColorRole::IconPrimary)),
            )
            // `:focus-visible { outline: 2px solid var(--focus); outline-offset: 2px }`.
            .when(ring, |d| {
                d.child(
                    div()
                        .absolute()
                        .top(px(-4.))
                        .bottom(px(-4.))
                        .left(px(-4.))
                        .right(px(-4.))
                        .border_2()
                        .border_color(color(ColorRole::Focus)),
                )
            })
            .on_key_down({
                let set = set.clone();
                move |e: &KeyDownEvent, window, cx| {
                    if let Some(v) = level_after_key(&e.keystroke.key, value, n) {
                        cx.stop_propagation();
                        set(v, window, cx);
                    }
                }
            })
            .on_mouse_down(MouseButton::Left, {
                let pick = pick.clone();
                let dragging = dragging.clone();
                let focus = focus.clone();
                move |e, window, cx| {
                    focus.focus(window, cx);
                    dragging.update(cx, |d, _| *d = true);
                    pick(e.position.x, window, cx);
                }
            })
            .on_mouse_move({
                let pick = pick.clone();
                let dragging = dragging.clone();
                move |e, window, cx| {
                    if *dragging.read(cx) && e.pressed_button == Some(MouseButton::Left) {
                        pick(e.position.x, window, cx);
                    }
                }
            })
            .on_mouse_up(MouseButton::Left, {
                let dragging = dragging.clone();
                move |_, _, cx| dragging.update(cx, |d, _| *d = false)
            })
            .on_mouse_up_out(MouseButton::Left, {
                let dragging = dragging.clone();
                move |_, _, cx| dragging.update(cx, |d, _| *d = false)
            });

        // `.g-sslider-ticks { position: relative; height: 1.5rem; margin: 0 7px }`.
        let ticks =
            div()
                .relative()
                .h(rem(24.))
                .mx(rem(7.))
                .children(self.steps.iter().enumerate().map(|(i, name)| {
                    let on = i == value;
                    let set = set.clone();
                    // `.g-sslider-tick { position: absolute; transform: translateX(-50%) }`:
                    // a zero-width anchor at the tick's position centers its column.
                    div()
                        .absolute()
                        .top_0()
                        .left(relative(fraction(i, n)))
                        .w_0()
                        .flex()
                        .justify_center()
                        .child(
                            div()
                                .id((id.clone(), SharedString::from(format!("tick-{i}"))))
                                .flex()
                                .flex_col()
                                .flex_none()
                                .items_center()
                                .gap(px(2.))
                                .type_style(TypeStyle::Label01)
                                .whitespace_nowrap()
                                .text_color(color(if on {
                                    ColorRole::TextPrimary
                                } else {
                                    ColorRole::TextSecondary
                                }))
                                .when(on, |d| d.font_weight(FontWeight::SEMIBOLD))
                                .cursor(CursorStyle::PointingHand)
                                .on_click(move |_, window, cx| set(i, window, cx))
                                .child(
                                    div()
                                        .w(px(1.))
                                        .h(px(4.))
                                        .bg(color(ColorRole::BorderStrong01)),
                                )
                                .child(name.clone()),
                        )
                }));

        let desc = self.describe.as_ref().map(|f| {
            // `.g-sslider-desc { min-height: 2rem }`.
            parts::helper(f(value)).min_h(rem(32.))
        });
        parts::field()
            // `.g-sslider { padding-bottom: var(--spacing-02) }`.
            .pb(rem(4.))
            .when_some(self.label.clone(), |f, l| f.child(parts::label(l)))
            .child(range)
            .child(ticks)
            .children(desc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fraction_and_hit_testing_follow_a_native_range() {
        assert_eq!(fraction(1, 4), 1. / 3.);
        assert_eq!(fraction(0, 1), 0.);
        // A 214px track: the thumb center travels from 7 to 207.
        assert_eq!(level_at(7., 0., 214., 4), 0);
        assert_eq!(level_at(207., 0., 214., 4), 3);
        assert_eq!(level_at(107., 0., 214., 4), 2); // 0.5 × 3 = 1.5 rounds up
        assert_eq!(level_at(-50., 0., 214., 4), 0);
        assert_eq!(level_at(500., 0., 214., 4), 3);
    }

    #[test]
    fn keys_step_and_jump() {
        assert_eq!(level_after_key("right", 1, 4), Some(2));
        assert_eq!(level_after_key("up", 3, 4), Some(3));
        assert_eq!(level_after_key("left", 0, 4), Some(0));
        assert_eq!(level_after_key("end", 0, 4), Some(3));
        assert_eq!(level_after_key("home", 2, 4), Some(0));
        assert_eq!(level_after_key("a", 2, 4), None);
    }
}
