//! ColorPicker: saturation/brightness plane, hue bar, hex field and preset
//! swatches (`ColorPicker` in `src/ghost/primitives.jsx`). For theme editors
//! and project colors. Carbon has no color picker; sizes and rules follow
//! Carbon fields.
//!
//! - **Behavior**: the plane is a 2-D slider (`role="slider"`): press or drag
//!   to pick, ←/→ saturation and ↑/↓ brightness by 2%. The hue bar is a range
//!   0–359 (drag, ←/→ by 1, Home/End). The hex field (Kit [`InputState`])
//!   validates `#rrggbb` (the `#` optional) on blur or Enter and lower-cases
//!   it. A swatch press sets its color. Written here because Kit has no 2-D
//!   slider; pointer math is in pure functions below.
//! - **Appearance** (`.g-color-*`): 15rem wide column, 8px gaps; an 8.5rem
//!   plane (white → hue, under transparent → black) with a 14px knob (2px
//!   white ring, 1px 50% black halo) filled with the color; a 12px rainbow
//!   bar with a 6×16 white thumb (1px dark border); a `density-field-md`
//!   chip beside a mono hex field (`field-01`, `border-strong-01` rule, 2px
//!   `focus` outline); 24px swatches, the current one ringed 2px
//!   `background` + 2px `border-interactive`.
use super::parts::{self, rem};
use crate::{
    components::form,
    density,
    theme::color,
    tokens::{ColorRole, FontFamilyKey},
};
use gpui_kit::{
    App, AppContext as _, Bounds, BoxShadow, Context, CursorStyle, ElementId, Entity,
    Focusable as _, Hsla, InteractiveElement, IntoElement, KeyDownEvent, MouseButton,
    ParentElement, Pixels, Point, RenderOnce, Rgba, SharedString, StatefulInteractiveElement as _,
    Styled, Subscription, Window, black, canvas,
    component::input::{InputEvent, InputState},
    div, hsla, linear_color_stop, linear_gradient,
    prelude::FluentBuilder as _,
    px, relative, white,
};
use std::{cell::Cell, rc::Rc};

/// The reference's default swatches (Carbon blue 60, purple 60, teal 60,
/// magenta 60, red 60, yellow 30, green 50, gray 100).
pub const DEFAULT_SWATCHES: [&str; 8] = [
    "#0f62fe", "#8a3ffc", "#007d79", "#d02670", "#da1e28", "#f1c21b", "#24a148", "#161616",
];

/// `/^#[0-9a-f]{6}$/i`.
pub fn is_hex(v: &str) -> bool {
    v.len() == 7 && v.starts_with('#') && v[1..].chars().all(|c| c.is_ascii_hexdigit())
}

/// The hex field's commit: `/^#?[0-9a-f]{6}$/i` → lower-case `#rrggbb`.
pub fn normalize_hex(input: &str) -> Option<String> {
    let t = input.strip_prefix('#').unwrap_or(input);
    (t.len() == 6 && t.chars().all(|c| c.is_ascii_hexdigit()))
        .then(|| format!("#{}", t.to_ascii_lowercase()))
}

/// `hex2hsv`: hue in degrees (0–360), saturation and value 0–1.
pub fn hex_to_hsv(hex: &str) -> (f32, f32, f32) {
    let n = u32::from_str_radix(hex.trim_start_matches('#'), 16).unwrap_or(0);
    let r = ((n >> 16) & 255) as f32 / 255.;
    let g = ((n >> 8) & 255) as f32 / 255.;
    let b = (n & 255) as f32 / 255.;
    let mx = r.max(g).max(b);
    let mn = r.min(g).min(b);
    let d = mx - mn;
    let mut hh = 0.;
    if d > 0. {
        hh = if mx == r {
            ((g - b) / d) % 6.
        } else if mx == g {
            (b - r) / d + 2.
        } else {
            (r - g) / d + 4.
        };
    }
    (
        (hh * 60. + 360.) % 360.,
        if mx > 0. { d / mx } else { 0. },
        mx,
    )
}

/// `hsv2hex`.
pub fn hsv_to_hex(h: f32, s: f32, v: f32) -> String {
    let f = |n: f32| {
        let k = (n + h / 60.) % 6.;
        v - v * s * k.min(4. - k).clamp(0., 1.)
    };
    let byte = |x: f32| (x * 255.).round().clamp(0., 255.) as u8;
    format!("#{:02x}{:02x}{:02x}", byte(f(5.)), byte(f(3.)), byte(f(1.)))
}

/// A `#rrggbb` string as a color (black when malformed).
pub fn hex_color(hex: &str) -> Hsla {
    let n = u32::from_str_radix(hex.trim_start_matches('#'), 16).unwrap_or(0);
    Rgba {
        r: ((n >> 16) & 255) as f32 / 255.,
        g: ((n >> 8) & 255) as f32 / 255.,
        b: (n & 255) as f32 / 255.,
        a: 1.,
    }
    .into()
}

/// Saturation and value under a point of the plane (`(x − left) / width`,
/// `1 − (y − top) / height`, clamped).
pub fn plane_at(p: Point<Pixels>, b: Bounds<Pixels>) -> (f32, f32) {
    let w = f32::from(b.size.width).max(1.);
    let h = f32::from(b.size.height).max(1.);
    (
        ((f32::from(p.x) - f32::from(b.origin.x)) / w).clamp(0., 1.),
        (1. - (f32::from(p.y) - f32::from(b.origin.y)) / h).clamp(0., 1.),
    )
}

/// The plane's arrow-key step (`{ ArrowLeft: [-0.02, 0], … }`).
pub fn plane_key(key: &str, s: f32, v: f32) -> Option<(f32, f32)> {
    let (ds, dv) = match key {
        "left" => (-0.02, 0.),
        "right" => (0.02, 0.),
        "up" => (0., 0.02),
        "down" => (0., -0.02),
        _ => return None,
    };
    Some(((s + ds).clamp(0., 1.), (v + dv).clamp(0., 1.)))
}

/// The hue (0–359, whole degrees) under `x` on a bar whose 6px thumb
/// travels inside its ends, as a native range's does.
pub fn hue_at(x: f32, left: f32, width: f32) -> f32 {
    let travel = (width - HUE_THUMB).max(1.);
    (((x - left - HUE_THUMB / 2.) / travel).clamp(0., 1.) * 359.).round()
}

/// The hue thumb width (`::-webkit-slider-thumb { width: 6px; height: 1rem }`).
pub const HUE_THUMB: f32 = 6.;

/// The hue bar's six segments (`linear-gradient(to right, #f00, #ff0, #0f0,
/// #0ff, #00f, #f0f, #f00)`), as hue pairs.
const HUE_STOPS: [f32; 7] = [0., 60., 120., 180., 240., 300., 360.];

type Handler = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;

struct Model {
    value: SharedString,
    hsv: (f32, f32, f32),
    hex: Entity<InputState>,
    editing: bool,
    controlled: bool,
    on_change: Option<Handler>,
    _subscription: Subscription,
}

impl Model {
    fn set(&mut self, v: SharedString, window: &mut Window, cx: &mut Context<Self>) {
        if !self.controlled {
            self.value = v.clone();
        }
        if !self.editing {
            let shown = v.clone();
            self.hex.update(cx, |s, cx| s.set_value(shown, window, cx));
        }
        if let Some(f) = self.on_change.clone() {
            f(v, window, cx);
        }
        cx.notify();
    }

    /// `upd(n)`: keep the HSV (so hue survives at zero saturation) and set its hex.
    fn upd(&mut self, hsv: (f32, f32, f32), window: &mut Window, cx: &mut Context<Self>) {
        self.hsv = hsv;
        self.set(hsv_to_hex(hsv.0, hsv.1, hsv.2).into(), window, cx);
    }

    fn commit_hex(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.editing {
            return;
        }
        self.editing = false;
        let text = self.hex.read(cx).value().to_string();
        if let Some(hex) = normalize_hex(&text) {
            self.set(hex.into(), window, cx);
        } else {
            let v = self.value.clone();
            self.hex.update(cx, |s, cx| s.set_value(v, window, cx));
            cx.notify();
        }
    }
}

/// A color picker. See the module docs.
#[derive(IntoElement)]
pub struct ColorPicker {
    id: ElementId,
    label: Option<SharedString>,
    value: Option<SharedString>,
    default_value: SharedString,
    swatches: Vec<SharedString>,
    labels: [SharedString; 4],
    on_value_change: Option<Handler>,
}

impl ColorPicker {
    /// A picker with a stable `id` (uncontrolled, `#0f62fe`).
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            value: None,
            default_value: "#0f62fe".into(),
            swatches: DEFAULT_SWATCHES
                .iter()
                .map(|s| SharedString::from(*s))
                .collect(),
            // The reference's `aria-label`s; override per locale.
            labels: [
                "Saturation and brightness".into(),
                "Hue".into(),
                "Hex colour".into(),
                "Swatches".into(),
            ],
            on_value_change: None,
        }
    }

    /// The label above.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Control the value (`#rrggbb`).
    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// The initial value when uncontrolled.
    pub fn default_value(mut self, value: impl Into<SharedString>) -> Self {
        self.default_value = value.into();
        self
    }

    /// Preset swatches (`#rrggbb`).
    pub fn swatches(mut self, swatches: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.swatches = swatches.into_iter().map(Into::into).collect();
        self
    }

    /// Accessible names: plane, hue bar, hex field, swatch group.
    pub fn labels(mut self, labels: [SharedString; 4]) -> Self {
        self.labels = labels;
        self
    }

    /// Called with each new `#rrggbb`.
    pub fn on_value_change(
        mut self,
        f: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_value_change = Some(Rc::new(f));
        self
    }
}

fn bounds_cell(
    id: &ElementId,
    key: &'static str,
    window: &mut Window,
    cx: &mut App,
) -> Rc<Cell<Bounds<Pixels>>> {
    window
        .use_keyed_state((id.clone(), key), cx, |_, _| {
            Rc::new(Cell::new(Bounds::default()))
        })
        .read(cx)
        .clone()
}

fn measure(cell: &Rc<Cell<Bounds<Pixels>>>) -> impl IntoElement {
    let cell = cell.clone();
    canvas(move |b, _, _| cell.set(b), |_, _, _, _| {})
        .absolute()
        .size_full()
}

impl RenderOnce for ColorPicker {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let initial = self.value.clone().unwrap_or(self.default_value.clone());
        let model = window.use_keyed_state((id.clone(), "color"), cx, |window, cx| {
            let shown = initial.clone();
            let hex = cx.new(|cx| InputState::new(window, cx).default_value(shown));
            let subscription =
                cx.subscribe_in(&hex, window, |this: &mut Model, _, e, window, cx| match e {
                    InputEvent::Change => this.editing = true,
                    InputEvent::Blur | InputEvent::PressEnter { .. } => this.commit_hex(window, cx),
                    InputEvent::Focus => {}
                });
            Model {
                hsv: hex_to_hsv(&initial),
                value: initial,
                hex,
                editing: false,
                controlled: false,
                on_change: None,
                _subscription: subscription,
            }
        });
        model.update(cx, |m, cx| {
            m.on_change = self.on_value_change.clone();
            m.controlled = self.value.is_some();
            if let Some(v) = &self.value
                && *v != m.value
            {
                m.value = v.clone();
                if !m.editing {
                    let shown = v.clone();
                    m.hex.update(cx, |s, cx| s.set_value(shown, window, cx));
                }
            }
            // `useEffect([v])`: follow a valid value the HSV does not produce.
            if is_hex(&m.value)
                && hsv_to_hex(m.hsv.0, m.hsv.1, m.hsv.2) != m.value.to_ascii_lowercase()
            {
                m.hsv = hex_to_hsv(&m.value);
            }
        });
        let (value, (h, s, v), hex) = {
            let m = model.read(cx);
            (m.value.clone(), m.hsv, m.hex.clone())
        };
        let current = hex_color(&value);
        let [plane_label, hue_label, hex_label, swatch_label] = self.labels.clone();
        let field_h = density::current().field().px_value();

        // ── Plane ──
        let plane_bounds = bounds_cell(&id, "plane-bounds", window, cx);
        let plane_focus = parts::keyed_focus((id.clone(), "plane-focus"), window, cx);
        let plane_drag = window.use_keyed_state((id.clone(), "plane-drag"), cx, |_, _| false);
        let plane_ring = parts::focus_visible(&plane_focus, window);
        let pick = {
            let model = model.clone();
            let b = plane_bounds.clone();
            Rc::new(move |p: Point<Pixels>, window: &mut Window, cx: &mut App| {
                let (s, v) = plane_at(p, b.get());
                model.update(cx, |m, cx| {
                    let h = m.hsv.0;
                    m.upd((h, s, v), window, cx)
                });
            })
        };
        let hue_color = hsla(h / 360., 1., 0.5, 1.);
        let clear = Hsla::transparent_black();
        let plane = div()
            .id((id.clone(), "plane"))
            // A tracked handle carries its own tab-stop flag; the div's
            // `.tab_index` does not reach it.
            .track_focus(&plane_focus.clone().tab_index(0).tab_stop(true))
            .role(gpui_kit::Role::Slider)
            .aria_label(plane_label)
            .relative()
            .h(rem(136.))
            // `linear-gradient(to right, #fff, hsl(h 100% 50%))`.
            .bg(linear_gradient(
                90.,
                linear_color_stop(white(), 0.),
                linear_color_stop(hue_color, 1.),
            ))
            .cursor(CursorStyle::Crosshair)
            .child(measure(&plane_bounds))
            // `linear-gradient(to top, #000, transparent)` above it.
            .child(div().absolute().size_full().bg(linear_gradient(
                0.,
                linear_color_stop(black(), 0.),
                linear_color_stop(clear, 1.),
            )))
            .child(
                // `.g-color-knob { width: 14px; height: 14px; margin: -7px 0 0 -7px;
                // border: 2px solid #fff; border-radius: 50%; box-shadow: 0 0 0 1px rgba(0,0,0,.5) }`.
                div()
                    .absolute()
                    .left(relative(s))
                    .top(relative(1. - v))
                    .ml(px(-7.))
                    .mt(px(-7.))
                    .size(px(14.))
                    .rounded_full()
                    .border_2()
                    .border_color(white())
                    .bg(current)
                    .shadow(vec![BoxShadow {
                        color: hsla(0., 0., 0., 0.5),
                        offset: gpui_kit::point(px(0.), px(0.)),
                        blur_radius: px(0.),
                        spread_radius: px(1.),
                        inset: false,
                    }]),
            )
            .when(plane_ring, |d| d.child(parts::focus_ring()))
            .on_key_down({
                let model = model.clone();
                move |e: &KeyDownEvent, window, cx| {
                    let hsv = model.read(cx).hsv;
                    if let Some((s, v)) = plane_key(&e.keystroke.key, hsv.1, hsv.2) {
                        cx.stop_propagation();
                        model.update(cx, |m, cx| m.upd((hsv.0, s, v), window, cx));
                    }
                }
            })
            .on_mouse_down(MouseButton::Left, {
                let pick = pick.clone();
                let drag = plane_drag.clone();
                let focus = plane_focus.clone();
                move |e, window, cx| {
                    focus.focus(window, cx);
                    drag.update(cx, |d, _| *d = true);
                    pick(e.position, window, cx);
                }
            })
            .on_mouse_move({
                let pick = pick.clone();
                let drag = plane_drag.clone();
                move |e, window, cx| {
                    if *drag.read(cx) && e.pressed_button == Some(MouseButton::Left) {
                        pick(e.position, window, cx);
                    }
                }
            })
            .on_mouse_up_out(MouseButton::Left, {
                let drag = plane_drag.clone();
                move |_, _, cx| drag.update(cx, |d, _| *d = false)
            })
            .on_mouse_up(MouseButton::Left, {
                let drag = plane_drag.clone();
                move |_, _, cx| drag.update(cx, |d, _| *d = false)
            });

        // ── Hue ──
        let hue_bounds = bounds_cell(&id, "hue-bounds", window, cx);
        let hue_focus = parts::keyed_focus((id.clone(), "hue-focus"), window, cx);
        let hue_drag = window.use_keyed_state((id.clone(), "hue-drag"), cx, |_, _| false);
        let hue_ring = parts::focus_visible(&hue_focus, window);
        let set_hue = {
            let model = model.clone();
            Rc::new(move |hue: f32, window: &mut Window, cx: &mut App| {
                model.update(cx, |m, cx| {
                    let (_, s, v) = m.hsv;
                    m.upd((hue, s, v), window, cx)
                });
            })
        };
        let pick_hue = {
            let set_hue = set_hue.clone();
            let b = hue_bounds.clone();
            Rc::new(move |x: Pixels, window: &mut Window, cx: &mut App| {
                let b = b.get();
                set_hue(
                    hue_at(f32::from(x), f32::from(b.origin.x), f32::from(b.size.width)),
                    window,
                    cx,
                );
            })
        };
        let hue_value = h.round().min(359.);
        let segments = HUE_STOPS.windows(2).map(|w| {
            div().flex_1().h_full().bg(linear_gradient(
                90.,
                linear_color_stop(hsla(w[0] / 360. % 1., 1., 0.5, 1.), 0.),
                linear_color_stop(hsla(w[1] / 360. % 1., 1., 0.5, 1.), 1.),
            ))
        });
        let hue = div()
            .id((id.clone(), "hue"))
            // A tracked handle carries its own tab-stop flag; the div's
            // `.tab_index` does not reach it.
            .track_focus(&hue_focus.clone().tab_index(0).tab_stop(true))
            .role(gpui_kit::Role::Slider)
            .aria_label(hue_label)
            .relative()
            .w_full()
            .h(rem(12.))
            .flex()
            .cursor(CursorStyle::PointingHand)
            .children(segments)
            .child(measure(&hue_bounds))
            .child(
                // The thumb: 6px × 1rem, white, 1px #161616 border, centered on the bar.
                div()
                    .absolute()
                    .left(relative(hue_value / 359.))
                    .ml(px(-HUE_THUMB * hue_value / 359.))
                    .top(rem(-2.))
                    .w(px(HUE_THUMB))
                    .h(rem(16.))
                    .bg(white())
                    .border_1()
                    .border_color(crate::theme::color_in(
                        crate::CarbonTheme::White,
                        ColorRole::TextPrimary,
                    )),
            )
            // `.g-color-hue:focus-visible { outline: 2px solid var(--focus); outline-offset: 2px }`.
            .when(hue_ring, |d| {
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
                let set_hue = set_hue.clone();
                move |e: &KeyDownEvent, window, cx| {
                    let next = match e.keystroke.key.as_str() {
                        "left" | "down" => (hue_value - 1.).max(0.),
                        "right" | "up" => (hue_value + 1.).min(359.),
                        "home" => 0.,
                        "end" => 359.,
                        _ => return,
                    };
                    cx.stop_propagation();
                    set_hue(next, window, cx);
                }
            })
            .on_mouse_down(MouseButton::Left, {
                let pick = pick_hue.clone();
                let drag = hue_drag.clone();
                let focus = hue_focus.clone();
                move |e, window, cx| {
                    focus.focus(window, cx);
                    drag.update(cx, |d, _| *d = true);
                    pick(e.position.x, window, cx);
                }
            })
            .on_mouse_move({
                let pick = pick_hue.clone();
                let drag = hue_drag.clone();
                move |e, window, cx| {
                    if *drag.read(cx) && e.pressed_button == Some(MouseButton::Left) {
                        pick(e.position.x, window, cx);
                    }
                }
            })
            .on_mouse_up_out(MouseButton::Left, {
                let drag = hue_drag.clone();
                move |_, _, cx| drag.update(cx, |d, _| *d = false)
            })
            .on_mouse_up(MouseButton::Left, {
                let drag = hue_drag.clone();
                move |_, _, cx| drag.update(cx, |d, _| *d = false)
            });

        // ── Chip + hex ──
        let hex_focused = hex.read(cx).focus_handle(cx).is_focused(window);
        let inset_rule = |c: Hsla| {
            vec![BoxShadow {
                color: c,
                offset: gpui_kit::point(px(0.), px(0.)),
                blur_radius: px(0.),
                spread_radius: px(1.),
                inset: true,
            }]
        };
        let row = div()
            .flex()
            .items_center()
            .gap(rem(8.))
            .child(
                // `.g-color-chip { box-shadow: inset 0 0 0 1px var(--border-subtle-01) }`.
                div()
                    .flex_none()
                    .size(rem(field_h))
                    .bg(current)
                    .shadow(inset_rule(color(ColorRole::BorderSubtle01))),
            )
            .child(
                div()
                    .relative()
                    .flex()
                    .flex_1()
                    .min_w_0()
                    .items_center()
                    .h(rem(field_h))
                    .px(rem(12.))
                    .bg(color(ColorRole::Field01))
                    .on_mouse_down(MouseButton::Left, {
                        let focus = hex.read(cx).focus_handle(cx);
                        move |_, window, cx| focus.focus(window, cx)
                    })
                    .child(
                        parts::mono_editor(&hex)
                            .aria_label(hex_label)
                            .text_color(color(ColorRole::TextPrimary)),
                    )
                    .child(form::bottom_rule(color(ColorRole::BorderStrong01)))
                    .when(hex_focused, |d| d.child(parts::focus_ring())),
            );

        // ── Swatches ──
        let swatches = div()
            .id((id.clone(), "swatches"))
            .aria_label(swatch_label)
            .flex()
            .flex_wrap()
            .gap(rem(4.))
            .children(self.swatches.iter().enumerate().map(|(i, sw)| {
                let on = sw.eq_ignore_ascii_case(&value);
                let focus = parts::keyed_focus(
                    (id.clone(), SharedString::from(format!("swatch-focus-{i}"))),
                    window,
                    cx,
                );
                let ring = parts::focus_visible(&focus, window);
                let model = model.clone();
                let pick: SharedString = sw.to_ascii_lowercase().into();
                gpui_kit::base::Button::new((id.clone(), SharedString::from(format!("swatch-{i}"))))
                    .track_focus(&focus)
                    .accessibility_label(sw.clone())
                    .relative()
                    .flex_none()
                    .size(rem(24.))
                    .bg(hex_color(sw))
                    .cursor(CursorStyle::PointingHand)
                    .map(|b| {
                        if on {
                            // `.g-color-swatch--on { box-shadow: 0 0 0 2px var(--background),
                            // 0 0 0 4px var(--border-interactive) }`.
                            b.shadow(vec![
                                BoxShadow {
                                    color: color(ColorRole::Background),
                                    offset: gpui_kit::point(px(0.), px(0.)),
                                    blur_radius: px(0.),
                                    spread_radius: px(2.),
                                    inset: false,
                                },
                                BoxShadow {
                                    color: color(ColorRole::BorderInteractive),
                                    offset: gpui_kit::point(px(0.), px(0.)),
                                    blur_radius: px(0.),
                                    spread_radius: px(4.),
                                    inset: false,
                                },
                            ])
                        } else {
                            b.shadow(inset_rule(color(ColorRole::BorderSubtle01)))
                        }
                    })
                    // `.g-color-swatch:focus-visible { outline-offset: 4px }` (2px wide).
                    .when(ring, |b| {
                        b.child(
                            div()
                                .absolute()
                                .top(px(-6.))
                                .bottom(px(-6.))
                                .left(px(-6.))
                                .right(px(-6.))
                                .border_2()
                                .border_color(color(ColorRole::Focus)),
                        )
                    })
                    .on_click(move |_, window, cx| {
                        let pick = pick.clone();
                        model.update(cx, |m, cx| m.set(pick, window, cx))
                    })
            }));

        div()
            .flex()
            .flex_col()
            .gap(rem(8.))
            .w(rem(240.))
            .when_some(self.label.clone(), |d, l| d.child(parts::label(l)))
            .child(plane)
            .child(hue)
            .child(row)
            .child(swatches)
            .font(crate::typography::font(FontFamilyKey::Sans, 400))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hsv_round_trips_the_swatches() {
        for hex in DEFAULT_SWATCHES {
            let (h, s, v) = hex_to_hsv(hex);
            assert_eq!(hsv_to_hex(h, s, v), hex, "{hex}");
        }
        let (h, s, v) = hex_to_hsv("#8a3ffc");
        assert!((h - 264.).abs() < 1., "{h}");
        assert!((s - 0.75).abs() < 0.01 && (v - 0.988).abs() < 0.01);
        assert_eq!(hsv_to_hex(0., 0., 1.), "#ffffff");
        assert_eq!(hsv_to_hex(120., 1., 1.), "#00ff00");
    }

    #[test]
    fn hex_validation_matches_the_reference() {
        assert!(is_hex("#0F62fe"));
        assert!(!is_hex("0f62fe"));
        assert_eq!(normalize_hex("0F62FE").as_deref(), Some("#0f62fe"));
        assert_eq!(normalize_hex("#abc"), None);
        assert_eq!(normalize_hex("#gggggg"), None);
    }

    #[test]
    fn pointer_and_keys() {
        let b = Bounds::new(
            gpui_kit::point(px(10.), px(20.)),
            gpui_kit::size(px(240.), px(136.)),
        );
        assert_eq!(plane_at(gpui_kit::point(px(130.), px(20.)), b), (0.5, 1.));
        assert_eq!(plane_at(gpui_kit::point(px(0.), px(500.)), b), (0., 0.));
        assert_eq!(plane_key("right", 0.99, 0.5), Some((1., 0.5)));
        assert_eq!(plane_key("down", 0.5, 0.01), Some((0.5, 0.)));
        assert_eq!(hue_at(3., 0., 240.), 0.);
        assert_eq!(hue_at(237., 0., 240.), 359.);
    }
}
