//! Carbon motion: easing curves, duration tokens, reduced motion and helpers
//! that animate a value when a state changes.
//!
//! Carbon motion (carbondesignsystem.com › Elements › Motion; `@carbon/motion`):
//!
//! | Curve | Productive | Expressive |
//! | --- | --- | --- |
//! | standard | `cubic-bezier(0.2, 0, 0.38, 0.9)` | `cubic-bezier(0.4, 0.14, 0.3, 1)` |
//! | entrance | `cubic-bezier(0, 0, 0.38, 0.9)` | `cubic-bezier(0, 0, 0.3, 1)` |
//! | exit | `cubic-bezier(0.2, 0, 1, 0.9)` | `cubic-bezier(0.4, 0.14, 1, 1)` |
//!
//! Durations: fast-01 70ms, fast-02 110ms, moderate-01 150ms, moderate-02 240ms,
//! slow-01 400ms, slow-02 700ms.
//!
//! # Animating on state change
//!
//! Render code computes a *target* from state (hovered → `button-primary-hover`)
//! and asks [`animate`] / [`animate_color`] for the value to paint this frame.
//! The transition state is keyed by element id (Base's
//! [`gpui_kit::base::transition`]): it retargets from the current value when the
//! state flips mid-way and requests frames only while running.
//!
//! ```ignore
//! let target = if hovered { color(ButtonPrimaryHover) } else { color(ButtonPrimary) };
//! let bg = motion::animate_color((id.clone(), "bg"), target, Motion::HOVER, window, cx);
//! ```
//!
//! # Reduced motion
//!
//! Base reads the system preference at init (`App::reduce_motion`); use
//! [`set_reduced_motion`] for an explicit override. Under reduced motion,
//! [`Motion`]s marked spatial finish at once (movement becomes a cut) while
//! color/opacity fades keep playing; [`crate::icons::spin`] becomes a pulse.
use gpui_kit::base::{self as base, Easing, Interpolate, ReduceMotionPreference, TransitionId};
use gpui_kit::{Animation, App, Hsla, Rgba, Window};
use std::time::Duration;

/// A Carbon easing curve.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Curve {
    /// Productive standard `cubic-bezier(0.2, 0, 0.38, 0.9)`: in-place changes
    /// (hover, press, toggle). Carbon's default.
    ProductiveStandard,
    /// Productive entrance `cubic-bezier(0, 0, 0.38, 0.9)`: elements appearing.
    ProductiveEntrance,
    /// Productive exit `cubic-bezier(0.2, 0, 1, 0.9)`: elements leaving.
    ProductiveExit,
    /// Expressive standard `cubic-bezier(0.4, 0.14, 0.3, 1)`.
    ExpressiveStandard,
    /// Expressive entrance `cubic-bezier(0, 0, 0.3, 1)`.
    ExpressiveEntrance,
    /// Expressive exit `cubic-bezier(0.4, 0.14, 1, 1)`.
    ExpressiveExit,
}

impl Curve {
    /// Every curve.
    pub const ALL: [Curve; 6] = [
        Self::ProductiveStandard,
        Self::ProductiveEntrance,
        Self::ProductiveExit,
        Self::ExpressiveStandard,
        Self::ExpressiveEntrance,
        Self::ExpressiveExit,
    ];

    /// Control points `(x1, y1, x2, y2)`.
    pub const fn points(self) -> (f32, f32, f32, f32) {
        match self {
            Self::ProductiveStandard => (0.2, 0., 0.38, 0.9),
            Self::ProductiveEntrance => (0., 0., 0.38, 0.9),
            Self::ProductiveExit => (0.2, 0., 1., 0.9),
            Self::ExpressiveStandard => (0.4, 0.14, 0.3, 1.),
            Self::ExpressiveEntrance => (0., 0., 0.3, 1.),
            Self::ExpressiveExit => (0.4, 0.14, 1., 1.),
        }
    }

    /// Carbon's name, e.g. `productive standard`.
    pub const fn label(self) -> &'static str {
        match self {
            Self::ProductiveStandard => "Productive standard",
            Self::ProductiveEntrance => "Productive entrance",
            Self::ProductiveExit => "Productive exit",
            Self::ExpressiveStandard => "Expressive standard",
            Self::ExpressiveEntrance => "Expressive entrance",
            Self::ExpressiveExit => "Expressive exit",
        }
    }

    /// Eased progress at `t` (0..=1).
    pub fn ease(self, t: f32) -> f32 {
        let (x1, y1, x2, y2) = self.points();
        cubic_bezier_at(x1, y1, x2, y2, t)
    }

    /// The curve as a plain function, for GPUI's `Animation::with_easing`.
    pub fn ease_fn(self) -> impl Fn(f32) -> f32 + Copy + 'static {
        move |t| self.ease(t)
    }

    /// The curve as a Base [`Easing`], for Kit motion tokens and transitions.
    pub fn easing(self) -> Easing {
        let (x1, y1, x2, y2) = self.points();
        Easing::cubic_bezier(x1, y1, x2, y2).expect("Carbon curves are valid")
    }
}

/// Evaluate CSS `cubic-bezier(x1, y1, x2, y2)` at time `t`.
///
/// Solves x(s) = t for the curve parameter with Newton's method, falling back
/// to bisection, then returns y(s). `t` is clamped to 0..=1.
pub fn cubic_bezier_at(x1: f32, y1: f32, x2: f32, y2: f32, t: f32) -> f32 {
    let t = if t.is_nan() { 0. } else { t.clamp(0., 1.) };
    if t == 0. || t == 1. {
        return t;
    }
    // Polynomial coefficients of B(s) = 3(1-s)²s·p1 + 3(1-s)s²·p2 + s³.
    let (cx, cy) = (3. * x1, 3. * y1);
    let (bx, by) = (3. * (x2 - x1) - cx, 3. * (y2 - y1) - cy);
    let (ax, ay) = (1. - cx - bx, 1. - cy - by);
    let x = |s: f32| ((ax * s + bx) * s + cx) * s;
    let dx = |s: f32| (3. * ax * s + 2. * bx) * s + cx;
    let y = |s: f32| ((ay * s + by) * s + cy) * s;
    let mut s = t;
    for _ in 0..8 {
        let err = x(s) - t;
        if err.abs() < 1e-6 {
            return y(s);
        }
        let d = dx(s);
        if d.abs() < 1e-6 {
            break;
        }
        s -= err / d;
    }
    let (mut lo, mut hi) = (0f32, 1f32);
    s = t;
    for _ in 0..40 {
        let v = x(s);
        if (v - t).abs() < 1e-6 {
            break;
        }
        if v < t {
            lo = s;
        } else {
            hi = s;
        }
        s = (lo + hi) / 2.;
    }
    y(s)
}

/// A Carbon duration token.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Speed {
    /// `fast-01` 70ms: micro-interactions (button hover, toggle).
    Fast01,
    /// `fast-02` 110ms: small expansions, chevrons, short fades.
    Fast02,
    /// `moderate-01` 150ms: dropdowns, tooltips.
    Moderate01,
    /// `moderate-02` 240ms: panel / modal entrance.
    Moderate02,
    /// `slow-01` 400ms: large expansions.
    Slow01,
    /// `slow-02` 700ms: background dimming, large moves.
    Slow02,
}

impl Speed {
    /// Every duration token.
    pub const ALL: [Speed; 6] = [
        Self::Fast01,
        Self::Fast02,
        Self::Moderate01,
        Self::Moderate02,
        Self::Slow01,
        Self::Slow02,
    ];

    /// Milliseconds.
    pub const fn millis(self) -> u64 {
        match self {
            Self::Fast01 => 70,
            Self::Fast02 => 110,
            Self::Moderate01 => 150,
            Self::Moderate02 => 240,
            Self::Slow01 => 400,
            Self::Slow02 => 700,
        }
    }

    /// The token as a [`Duration`].
    pub const fn duration(self) -> Duration {
        Duration::from_millis(self.millis())
    }

    /// Token name, e.g. `fast-01`.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Fast01 => "fast-01",
            Self::Fast02 => "fast-02",
            Self::Moderate01 => "moderate-01",
            Self::Moderate02 => "moderate-02",
            Self::Slow01 => "slow-01",
            Self::Slow02 => "slow-02",
        }
    }
}

/// A transition policy: curve + duration, and whether it moves things.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Motion {
    /// The easing curve.
    pub curve: Curve,
    /// The duration token.
    pub speed: Speed,
    /// Spatial motion (position, size, rotation) finishes at once under reduced
    /// motion; non-spatial (color, opacity) keeps its timing.
    pub spatial: bool,
}

impl Motion {
    /// Button/control background, border and text color on hover or press:
    /// `.cds--btn { transition: background 70ms cubic-bezier(0, 0, 0.38, 0.9) }`
    /// (fast-01; the compiled Carbon CSS uses the productive *entrance* curve here).
    pub const HOVER: Motion = Motion::new(Curve::ProductiveEntrance, Speed::Fast01);
    /// Chevron rotation and small toggles: fast-02, productive standard
    /// (`.cds--side-nav__submenu-chevron > svg { transition: transform 110ms }`).
    pub const TOGGLE: Motion = Motion::new(Curve::ProductiveStandard, Speed::Fast02).spatial(true);
    /// Something opening (menus, panels): moderate-02 productive entrance.
    pub const ENTER: Motion = Motion::new(Curve::ProductiveEntrance, Speed::Moderate02);
    /// Something closing: fast-02 productive exit.
    pub const EXIT: Motion = Motion::new(Curve::ProductiveExit, Speed::Fast02);

    /// A non-spatial motion.
    pub const fn new(curve: Curve, speed: Speed) -> Self {
        Self {
            curve,
            speed,
            spatial: false,
        }
    }

    /// Mark the motion spatial (see [`Motion::spatial`]).
    pub const fn spatial(mut self, spatial: bool) -> Self {
        self.spatial = spatial;
        self
    }

    /// As a Base value transition.
    pub fn transition(self) -> base::Transition {
        base::Transition::new(self.speed.duration())
            .easing(self.curve.easing())
            .reduced_motion(if self.spatial {
                base::ReducedMotion::Finish
            } else {
                base::ReducedMotion::Continue
            })
    }

    /// As a GPUI one-shot [`Animation`] (for `with_animation`).
    pub fn animation(self) -> Animation {
        Animation::new(self.speed.duration()).with_easing(self.curve.ease_fn())
    }
}

/// The value to paint this frame for a transition toward `target`, keyed by `id`.
///
/// The first frame adopts `target`; later changes animate per `motion` from the
/// value showing at that instant. Call while rendering.
pub fn animate<T>(
    id: impl Into<TransitionId>,
    target: T,
    motion: Motion,
    window: &mut Window,
    cx: &mut App,
) -> T
where
    T: Interpolate + PartialEq + 'static,
{
    base::transition(id, target, motion.transition(), window, cx)
}

/// A color that interpolates like CSS (premultiplied sRGB), so fading from
/// `transparent` does not pass through black or swing hue.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnimatedColor(pub Rgba);

impl base::animation::Lerp for AnimatedColor {
    fn lerp(&self, to: &Self, t: f32) -> Self {
        let (a, b) = (self.0, to.0);
        let alpha = a.a + (b.a - a.a) * t;
        let channel = |x: f32, y: f32| {
            let premultiplied = x * a.a + (y * b.a - x * a.a) * t;
            if alpha > 0. { premultiplied / alpha } else { y }
        };
        AnimatedColor(Rgba {
            r: channel(a.r, b.r),
            g: channel(a.g, b.g),
            b: channel(a.b, b.b),
            a: alpha,
        })
    }
}

/// [`animate`] for a color.
pub fn animate_color(
    id: impl Into<TransitionId>,
    target: impl Into<Hsla>,
    motion: Motion,
    window: &mut Window,
    cx: &mut App,
) -> Hsla {
    let target: Rgba = target.into().to_rgb();
    animate(id, AnimatedColor(target), motion, window, cx)
        .0
        .into()
}

/// Rotation (radians) of an expand chevron: 0 closed, π open, animated with
/// [`Motion::TOGGLE`] (a cut under reduced motion).
pub fn chevron_rotation(
    id: impl Into<TransitionId>,
    expanded: bool,
    window: &mut Window,
    cx: &mut App,
) -> f32 {
    let target = if expanded { std::f32::consts::PI } else { 0. };
    animate(id, target, Motion::TOGGLE, window, cx)
}

/// Open progress (0 closed → 1 open): [`Motion::ENTER`] when opening,
/// [`Motion::EXIT`] when closing. Non-spatial, so use it for opacity; drive any
/// slide from it only when [`reduced_motion`] is off.
pub fn open_progress(
    id: impl Into<TransitionId>,
    open: bool,
    window: &mut Window,
    cx: &mut App,
) -> f32 {
    let motion = if open { Motion::ENTER } else { Motion::EXIT };
    animate(id, if open { 1f32 } else { 0. }, motion, window, cx)
}

/// Whether motion should be reduced now (system preference or override).
pub fn reduced_motion(cx: &App) -> bool {
    cx.reduce_motion()
}

/// The reduced-motion preference.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ReducedMotion {
    /// Follow the operating system (Base reads it at init and, on Linux, live).
    #[default]
    System,
    /// Always reduce.
    Reduce,
    /// Never reduce.
    Allow,
}

/// Set the reduced-motion preference and refresh windows. Event handlers only.
pub fn set_reduced_motion(cx: &mut App, preference: ReducedMotion) {
    base::set_reduce_motion_preference(
        match preference {
            ReducedMotion::System => ReduceMotionPreference::System,
            ReducedMotion::Reduce => ReduceMotionPreference::Reduce,
            ReducedMotion::Allow => ReduceMotionPreference::Allow,
        },
        cx,
    );
    cx.refresh_windows();
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::base::animation::Lerp as _;

    #[test]
    fn curves_hit_their_endpoints_and_are_monotonic() {
        for curve in Curve::ALL {
            assert_eq!(curve.ease(0.), 0.);
            assert_eq!(curve.ease(1.), 1.);
            let mut last = 0.;
            for i in 1..=100 {
                let v = curve.ease(i as f32 / 100.);
                assert!(v >= last - 1e-5, "{curve:?} not monotonic at {i}");
                last = v;
            }
        }
    }

    #[test]
    fn curves_match_css_reference_values() {
        // Reference values from an independent high-precision bisection of the
        // CSS cubic-bezier definition, to 1e-3.
        let close = |a: f32, b: f32| (a - b).abs() < 1e-3;
        assert!(close(Curve::ProductiveEntrance.ease(0.5), 0.7301));
        assert!(close(Curve::ProductiveStandard.ease(0.5), 0.6848));
        assert!(close(Curve::ProductiveExit.ease(0.5), 0.3848));
        assert!(close(Curve::ExpressiveEntrance.ease(0.5), 0.8055));
        // CSS `ease` (0.25, 0.1, 0.25, 1) at 0.5 is 0.8024.
        assert!(close(cubic_bezier_at(0.25, 0.1, 0.25, 1., 0.5), 0.8024));
        // Linear stays linear.
        assert!(close(cubic_bezier_at(0., 0., 1., 1., 0.3), 0.3));
    }

    #[test]
    fn our_bezier_agrees_with_base_easing() {
        for curve in Curve::ALL {
            let easing = curve.easing();
            for i in 0..=20 {
                let t = i as f32 / 20.;
                assert!(
                    (easing.sample(t) - curve.ease(t)).abs() < 2e-3,
                    "{curve:?} at {t}"
                );
            }
        }
    }

    #[test]
    fn duration_tokens() {
        let ms: Vec<u64> = Speed::ALL.iter().map(|s| s.millis()).collect();
        assert_eq!(ms, [70, 110, 150, 240, 400, 700]);
        assert_eq!(Motion::HOVER.speed.duration(), Duration::from_millis(70));
    }

    #[test]
    fn color_interpolation_is_premultiplied() {
        let clear = AnimatedColor(Rgba {
            r: 0.,
            g: 0.,
            b: 0.,
            a: 0.,
        });
        let gray = AnimatedColor(Rgba {
            r: 0.55,
            g: 0.55,
            b: 0.55,
            a: 0.12,
        });
        let mid = clear.lerp(&gray, 0.5);
        assert!((mid.0.r - 0.55).abs() < 1e-5, "no darkening through black");
        assert!((mid.0.a - 0.06).abs() < 1e-5);
    }
}
