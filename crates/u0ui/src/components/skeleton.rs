//! Skeleton: Carbon `SkeletonText` (`variant = text`) and
//! `SkeletonPlaceholder` (`variant = block`), `.cds--skeleton__*`.
//!
//! - **Appearance**: a `skeleton-background` block; text lines are 1rem tall
//!   with 0.5rem below (`.cds--skeleton__text`), headings 1.5rem
//!   (`.cds--skeleton__heading`); the reference block defaults to 100% × 3rem.
//!   Paragraphs (`lines > 1`) shorten each line by Carbon's fixed pseudo-random
//!   amounts ([`paragraph_cut`]).
//! - **Motion**: a `skeleton-element` bar sweeps over the block
//!   (`::before { animation: 3000ms ease-in-out cds--skeleton infinite }`): it
//!   grows from the left, shrinks to the right, grows back from the right and
//!   shrinks to the left, its opacity going 0.3 → 1 → 0.3 ([`shimmer`]).
//!   Reduced motion: `animation: none` leaves the bar static and full, as in
//!   the CSS.
//! - **Behavior**: none. Kit's `Skeleton` pulses opacity; Carbon's sweep is
//!   drawn here.
use crate::{
    motion::{self, cubic_bezier_at},
    theme::color,
    tokens::ColorRole,
};
use gpui_kit::{
    App, DefiniteLength, IntoElement, ParentElement, RenderOnce, Styled, Window, div, px, relative,
    rems,
};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

/// `animation: 3000ms ease-in-out cds--skeleton infinite`.
pub const PERIOD: Duration = Duration::from_millis(3000);

/// One `cds--skeleton` keyframe: (offset, opacity if set, scaleX if set,
/// transform-origin is right).
type Keyframe = (f32, Option<f32>, Option<f32>, bool);

/// `@keyframes cds--skeleton`.
const KEYFRAMES: [Keyframe; 10] = [
    (0., Some(0.3), Some(0.), false),
    (0.20, Some(1.), Some(1.), false),
    (0.28, None, Some(1.), true),
    (0.51, None, Some(0.), true),
    (0.58, None, Some(0.), true),
    (0.82, None, Some(1.), true),
    (0.83, None, Some(1.), false),
    (0.96, None, Some(0.), false),
    (1.0, Some(0.3), Some(0.), false),
    (1.0, Some(0.3), Some(0.), false),
];

/// CSS `ease-in-out`, applied per keyframe segment.
fn ease_in_out(t: f32) -> f32 {
    cubic_bezier_at(0.42, 0., 0.58, 1., t)
}

/// Interpolate one property across the keyframes that set it.
fn property(t: f32, get: impl Fn(&Keyframe) -> Option<f32>) -> f32 {
    let frames: Vec<(f32, f32)> = KEYFRAMES
        .iter()
        .filter_map(|k| get(k).map(|v| (k.0, v)))
        .collect();
    for pair in frames.windows(2) {
        let ((t0, v0), (t1, v1)) = (pair[0], pair[1]);
        if t >= t0 && t <= t1 && t1 > t0 {
            return v0 + (v1 - v0) * ease_in_out((t - t0) / (t1 - t0));
        }
    }
    frames.last().map(|f| f.1).unwrap_or(1.)
}

/// The sweep bar at `t` (0..1 of the period): (opacity, left, width) as
/// fractions of the block. `transform-origin` flips between keyframes where
/// the scale is 0 or 1, so the side the bar is anchored to is that of the
/// segment's start keyframe.
pub fn shimmer(t: f32) -> (f32, f32, f32) {
    let t = t.rem_euclid(1.);
    let opacity = property(t, |k| k.1);
    let scale = property(t, |k| k.2).clamp(0., 1.);
    let right = KEYFRAMES
        .iter()
        .rev()
        .find(|k| k.0 <= t)
        .map(|k| k.3)
        .unwrap_or(false);
    let left = if right { 1. - scale } else { 0. };
    (opacity, left, scale)
}

/// Carbon `SkeletonText`'s fixed "random" numbers (`randoms = [0.973, 0.153, 0.567]`).
const RANDOMS: [f32; 3] = [0.973, 0.153, 0.567];

/// How many px a paragraph line `i` is shortened by for a percentage width:
/// `getRandomInt(0, 75, i)` = `floor(randoms[i % 3] * 76)`.
pub fn paragraph_cut(i: usize) -> f32 {
    (RANDOMS[i % 3] * 76.).floor()
}

/// `index.d.ts` `SkeletonProps.variant`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SkeletonVariant {
    /// `SkeletonPlaceholder`.
    #[default]
    Block,
    /// `SkeletonText`.
    Text,
}

impl SkeletonVariant {
    /// Both variants.
    pub const ALL: [SkeletonVariant; 2] = [Self::Block, Self::Text];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Block => "block",
            Self::Text => "text",
        }
    }
}

/// A skeleton length (`width` / `height` props).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SkeletonLength {
    /// A percentage of the container (`'40%'`).
    Percent(f32),
    /// Pixels (`'240px'`).
    Px(f32),
    /// Rems (`'3rem'`).
    Rem(f32),
}

impl SkeletonLength {
    fn definite(self) -> DefiniteLength {
        match self {
            Self::Percent(p) => relative(p / 100.),
            Self::Px(v) => px(v).into(),
            Self::Rem(v) => rems(v).into(),
        }
    }
}

fn epoch() -> Instant {
    static EPOCH: OnceLock<Instant> = OnceLock::new();
    *EPOCH.get_or_init(Instant::now)
}

/// One shimmering block (`.cds--skeleton__text` / `__placeholder`).
fn block(width: DefiniteLength, height: f32, reduced: bool, window: &mut Window) -> gpui_kit::Div {
    let (opacity, left, scale) = if reduced {
        // `@media (prefers-reduced-motion: reduce) { ::before { animation: none } }`.
        (1., 0., 1.)
    } else {
        window.request_animation_frame();
        let t = epoch().elapsed().as_secs_f32() / PERIOD.as_secs_f32();
        shimmer(t)
    };
    div()
        .relative()
        .flex_none()
        .w(width)
        .h(rems(height / crate::tokens::REM_PX))
        .bg(color(ColorRole::SkeletonBackground))
        .overflow_hidden()
        .child(
            div()
                .absolute()
                .top_0()
                .h_full()
                .left(relative(left))
                .w(relative(scale))
                .opacity(opacity)
                .bg(color(ColorRole::SkeletonElement)),
        )
}

/// A skeleton. See the module docs.
#[derive(IntoElement, Clone)]
pub struct Skeleton {
    variant: SkeletonVariant,
    lines: usize,
    heading: bool,
    width: Option<SkeletonLength>,
    height: Option<SkeletonLength>,
}

impl Skeleton {
    /// A `SkeletonPlaceholder` (`variant = block`): 100% × 3rem by default.
    pub fn block() -> Self {
        Self {
            variant: SkeletonVariant::Block,
            lines: 3,
            heading: false,
            width: None,
            height: None,
        }
    }

    /// A `SkeletonText` (`variant = text`), 3 lines by default.
    pub fn text() -> Self {
        Self {
            variant: SkeletonVariant::Text,
            ..Self::block()
        }
    }

    /// The variant.
    pub fn variant(mut self, variant: SkeletonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Number of text lines (`lines`, default 3; more than one is a paragraph).
    pub fn lines(mut self, lines: usize) -> Self {
        self.lines = lines.max(1);
        self
    }

    /// Heading-height lines (1.5rem).
    pub fn heading(mut self, heading: bool) -> Self {
        self.heading = heading;
        self
    }

    /// Width (text default 100%; block default 100%).
    pub fn width(mut self, width: SkeletonLength) -> Self {
        self.width = Some(width);
        self
    }

    /// Block height (default 3rem).
    pub fn height(mut self, height: SkeletonLength) -> Self {
        self.height = Some(height);
        self
    }
}

impl RenderOnce for Skeleton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let reduced = motion::reduced_motion(cx);
        match self.variant {
            SkeletonVariant::Block => {
                // `index.jsx`: `style={{ width: width || '100%', height: height || '3rem' }}`.
                let width = self.width.unwrap_or(SkeletonLength::Percent(100.));
                let height = match self.height.unwrap_or(SkeletonLength::Rem(3.)) {
                    SkeletonLength::Px(v) => v,
                    SkeletonLength::Rem(v) => v * crate::tokens::REM_PX,
                    // A percentage height needs a sized parent; fall back to 3rem.
                    SkeletonLength::Percent(_) => 48.,
                };
                div()
                    .w_full()
                    .child(block(width.definite(), height, reduced, window))
            }
            SkeletonVariant::Text => {
                let width = self.width.unwrap_or(SkeletonLength::Percent(100.));
                // `.cds--skeleton__text { block-size: 1rem }`, `.cds--skeleton__heading { 1.5rem }`.
                let height = if self.heading { 24. } else { 16. };
                let paragraph = self.lines > 1;
                div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .children((0..self.lines).map(|i| {
                        let (width, cut) = match (paragraph, width) {
                            // `calc(${width} - ${getRandomInt(0, 75, i)}px)`: the
                            // px cut is a right padding inside the percent box.
                            (true, SkeletonLength::Percent(_)) => {
                                (width.definite(), paragraph_cut(i))
                            }
                            // `getRandomInt(widthNum - 75, widthNum, i)` px.
                            (true, SkeletonLength::Px(v)) => {
                                (px(v - 75. + paragraph_cut(i)).into(), 0.)
                            }
                            _ => (width.definite(), 0.),
                        };
                        // `.cds--skeleton__text { margin-block-end: 0.5rem }`.
                        div().w(width).pr(px(cut)).mb(rems(0.5)).child(block(
                            relative(1.),
                            height,
                            reduced,
                            window,
                        ))
                    }))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    #[test]
    fn shimmer_follows_the_keyframes() {
        // 0%: opacity 0.3, scaleX 0 from the left.
        assert_eq!(shimmer(0.), (0.3, 0., 0.));
        // 20%: full bar, opacity 1.
        let (o, l, w) = shimmer(0.2);
        assert!(close(o, 1.) && close(l, 0.) && close(w, 1.));
        // 40%: shrinking toward the right (origin right).
        let (_, l, w) = shimmer(0.4);
        assert!(w > 0. && w < 1. && close(l + w, 1.));
        // 55%: hidden.
        assert!(close(shimmer(0.55).2, 0.));
        // 90%: shrinking toward the left again.
        let (_, l, w) = shimmer(0.9);
        assert!(close(l, 0.) && w > 0. && w < 1.);
        assert_eq!(PERIOD.as_millis(), 3000);
    }

    #[test]
    fn paragraph_cuts_are_carbons() {
        // `floor(randoms[n % 3] * (75 - 0 + 1))`.
        assert_eq!([0, 1, 2, 3].map(paragraph_cut), [73., 11., 43., 73.]);
    }
}
