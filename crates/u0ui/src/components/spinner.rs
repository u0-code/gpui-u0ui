//! Spinner: shadcn `Spinner` → Carbon `Loading` (`.cds--loading*`) and, with
//! a description, `InlineLoading` (`.cds--inline-loading*`).
//!
//! - **Geometry** (`Loading`'s SVG, `viewBox="0 0 100 100"`): large (`md`) is
//!   5.5rem with one `r=44` stroke circle, `stroke-width: 10`; small (`sm`) is
//!   1rem with an `r=42` `layer-accent` background ring under the stroke,
//!   `stroke-width: 16`. The stroke is `interactive`,
//!   `stroke-dasharray: 276.4608` with `stroke-dashoffset` 52.527552 (large,
//!   81% of the ring) or 143.759616 (small, half of it), butt caps, starting
//!   at 3 o'clock and running clockwise ([`arc`]).
//! - **Motion**: the whole SVG turns `cds--rotate` 690ms linear, forever. When
//!   it stops (`active = false`, `.cds--loading--stop`) it finishes one more
//!   clockwise turn (`cds--rotate-end-p1` 700ms `cubic-bezier(0, 0, 0.25, 1)`),
//!   then turns back while the stroke retracts (`cds--rotate-end-p2` +
//!   `cds--stroke-end`, 700ms after a 700ms delay) until nothing is left
//!   ([`stop_frame`]). Reduced motion: no rotation (a slow opacity pulse
//!   signals activity, per the u0ui motion rules); Carbon also drops the
//!   stroke animations (`.cds--loading svg circle { animation: none }`), so a
//!   stopped spinner keeps its arc.
//! - **Overlay** (`with_overlay`): a window-covering `overlay` scrim centring
//!   the large spinner (`.cds--loading-overlay`); it hides at once when the
//!   spinner stops (`.cds--loading-overlay--stop { display: none }`).
//! - **Inline** (`description` or `size = inline`): `.cds--inline-loading`
//!   is a 2rem-min row in `text-secondary`, `label-02` text 8px after the
//!   indicator: the small spinner (active), a stopping small spinner
//!   (inactive), `CheckmarkFilled` in `support-success` (finished) or
//!   `ErrorFilled` in `support-error` (error).
//! - **Behavior**: Kit's `Spinner` rotates an icon; Carbon's is a dash-offset
//!   arc with a stop sequence, so it is drawn here with a GPUI path.
use crate::{
    icons::{CarbonIcon, Icon, IconSize},
    motion::{self, cubic_bezier_at},
    theme::{LayerLevel, color},
    tokens::{ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    App, Bounds, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, PathBuilder,
    Pixels, RenderOnce, SharedString, Styled, Window, anchored, canvas, deferred, div, point,
    prelude::FluentBuilder as _, px, rems,
};
use std::f32::consts::{PI, TAU};
use std::time::{Duration, Instant};

/// `.cds--loading { animation-duration: 690ms; animation-name: cds--rotate }`.
pub const ROTATION_PERIOD: Duration = Duration::from_millis(690);
/// `.cds--loading--stop`: each stop phase lasts 700ms.
pub const STOP_PHASE: Duration = Duration::from_millis(700);
/// `stroke-dasharray: 276.4608` (2π·44).
pub const DASH: f32 = 276.4608;
/// `.cds--loading__stroke { stroke-dashoffset: 52.527552 }`.
pub const OFFSET_LARGE: f32 = 52.527552;
/// `.cds--loading--small .cds--loading__stroke { stroke-dashoffset: 143.759616 }`.
#[allow(clippy::excessive_precision)]
pub const OFFSET_SMALL: f32 = 143.759616;
/// `.cds--loading__background { stroke-dashoffset: -22 }`.
pub const OFFSET_BACKGROUND: f32 = -22.;

/// `index.d.ts` `SpinnerProps.size`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SpinnerSize {
    /// `sm`: Carbon `Loading small` (1rem).
    Sm,
    /// `md`: Carbon `Loading` (5.5rem), the default.
    #[default]
    Md,
    /// `inline`: Carbon `InlineLoading`.
    Inline,
}

impl SpinnerSize {
    /// Every size.
    pub const ALL: [SpinnerSize; 3] = [Self::Sm, Self::Md, Self::Inline];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Inline => "inline",
        }
    }
}

/// `index.d.ts` `SpinnerProps.status` (`InlineLoading`'s status).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SpinnerStatus {
    /// Spinning.
    #[default]
    Active,
    /// `CheckmarkFilled` in `support-success`.
    Finished,
    /// `ErrorFilled` in `support-error`.
    Error,
    /// The spinner stops (`active={false}`).
    Inactive,
}

impl SpinnerStatus {
    /// Every status.
    pub const ALL: [SpinnerStatus; 4] = [Self::Active, Self::Finished, Self::Error, Self::Inactive];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Finished => "finished",
            Self::Error => "error",
            Self::Inactive => "inactive",
        }
    }
}

/// The SVG of one Carbon spinner size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Arc {
    /// Box size in px (`.cds--loading` 5.5rem; `--small` 1rem).
    pub size: f32,
    /// Circle radius in viewBox units (`r = small ? 42 : 44`).
    pub radius: f32,
    /// `stroke-width` in viewBox units (10; small 16).
    pub stroke: f32,
    /// Resting `stroke-dashoffset`.
    pub offset: f32,
    /// Whether the `layer-accent` background ring is drawn (small only).
    pub background: bool,
}

/// The arc for `small` or large.
pub const fn arc(small: bool) -> Arc {
    if small {
        Arc {
            size: 16.,
            radius: 42.,
            stroke: 16.,
            offset: OFFSET_SMALL,
            background: true,
        }
    } else {
        Arc {
            size: 88.,
            radius: 44.,
            stroke: 10.,
            offset: OFFSET_LARGE,
            background: false,
        }
    }
}

/// The visible part of a dashed circle stroke, as (start, end) path lengths
/// from 3 o'clock, for `stroke-dasharray: DASH DASH` and `offset`.
pub fn dash_span(radius: f32, offset: f32) -> (f32, f32) {
    let circumference = TAU * radius;
    let period = DASH * 2.;
    // The pattern position at path length s is s + offset (mod period); the
    // dash covers pattern positions [0, DASH).
    let start = (period - offset.rem_euclid(period)) % period;
    if start == 0. || start >= DASH {
        // The path starts inside the dash.
        let first = (DASH - offset.rem_euclid(period)).max(0.);
        (0., first.min(circumference))
    } else {
        (start.min(circumference), (start + DASH).min(circumference))
    }
}

/// The rotation (degrees) and the stroke's dash offset `t` after the spinner
/// was told to stop: `cds--rotate-end-p1` 0 → 360° over 700ms, then
/// `cds--rotate-end-p2` 0 → −360° with `cds--stroke-end` (offset → `DASH`)
/// over the next 700ms, both `cubic-bezier(0, 0, 0.25, 1)`. `None` once over.
pub fn stop_frame(elapsed: Duration, resting_offset: f32) -> Option<(f32, f32)> {
    let phase = STOP_PHASE.as_secs_f32();
    let t = elapsed.as_secs_f32();
    let ease = |x: f32| cubic_bezier_at(0., 0., 0.25, 1., x);
    if t < phase {
        Some((360. * ease(t / phase), resting_offset))
    } else if t < 2. * phase {
        let k = ease((t - phase) / phase);
        Some((-360. * k, resting_offset + (DASH - resting_offset) * k))
    } else {
        None
    }
}

/// Rotation in degrees `elapsed` into the `cds--rotate` loop.
pub fn rotation_at(elapsed: Duration) -> f32 {
    let period = ROTATION_PERIOD.as_secs_f32();
    (elapsed.as_secs_f32() % period) / period * 360.
}

/// Paint a circle stroke segment of `arc` with dash `offset`, rotated by
/// `rotation` degrees, inside `bounds`.
fn paint_ring(
    bounds: Bounds<Pixels>,
    arc: Arc,
    offset: f32,
    rotation: f32,
    stroke: Hsla,
    background: Option<Hsla>,
    window: &mut Window,
) {
    let scale = f32::from(bounds.size.width) / 100.;
    let center = bounds.center();
    let r = arc.radius * scale;
    let width = px(arc.stroke * scale);
    let mut ring = |from: f32, to: f32, color: Hsla| {
        if to - from <= 0.01 {
            return;
        }
        let (a0, a1) = (from / arc.radius, to / arc.radius);
        let steps = ((a1 - a0) / (PI / 48.)).ceil().max(2.) as usize;
        let mut path = PathBuilder::stroke(width);
        for i in 0..=steps {
            let a = a0 + (a1 - a0) * i as f32 / steps as f32 + rotation.to_radians();
            let p = point(center.x + px(r * a.cos()), center.y + px(r * a.sin()));
            if i == 0 {
                path.move_to(p);
            } else {
                path.line_to(p);
            }
        }
        if let Ok(path) = path.build() {
            window.paint_path(path, color);
        }
    };
    if let Some(bg) = background {
        let (from, to) = dash_span(arc.radius, OFFSET_BACKGROUND);
        ring(from, to, bg);
    }
    let (from, to) = dash_span(arc.radius, offset);
    ring(from, to, stroke);
}

/// Per-spinner animation clock.
struct Clock {
    started: Instant,
    stopped: Option<Instant>,
}

/// A Carbon spinner. See the module docs.
#[derive(IntoElement)]
pub struct Spinner {
    id: ElementId,
    size: SpinnerSize,
    status: SpinnerStatus,
    description: Option<SharedString>,
    label: SharedString,
    overlay: bool,
    level: LayerLevel,
}

impl Spinner {
    /// A large active spinner with a stable `id`.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            size: SpinnerSize::Md,
            status: SpinnerStatus::Active,
            description: None,
            label: SharedString::default(),
            overlay: false,
            level: LayerLevel::One,
        }
    }

    /// `sm`, `md` or `inline`.
    pub fn size(mut self, size: SpinnerSize) -> Self {
        self.size = size;
        self
    }

    /// The status; `Inactive` stops a `Loading`, the others need `inline`
    /// or a description to show their icon.
    pub fn status(mut self, status: SpinnerStatus) -> Self {
        self.status = status;
        self
    }

    /// Carbon `active` (`false` = [`SpinnerStatus::Inactive`]).
    pub fn active(self, active: bool) -> Self {
        self.status(if active {
            SpinnerStatus::Active
        } else {
            SpinnerStatus::Inactive
        })
    }

    /// The visible text: turns the spinner into an `InlineLoading`.
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// The accessible name (Carbon `iconDescription`, e.g. "Loading").
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    /// Cover the window with the `overlay` scrim (`withOverlay`).
    pub fn with_overlay(mut self, overlay: bool) -> Self {
        self.overlay = overlay;
        self
    }

    /// The layer the small spinner's background ring sits on (`layer-accent`).
    pub fn layer(mut self, level: LayerLevel) -> Self {
        self.level = level;
        self
    }

    /// Whether this renders as an `InlineLoading`.
    pub fn is_inline(&self) -> bool {
        self.description.is_some() || self.size == SpinnerSize::Inline
    }
}

/// The spinning ring alone: `small` or large, `active` or stopping.
fn ring(
    id: ElementId,
    small: bool,
    active: bool,
    level: LayerLevel,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement + use<> {
    let geometry = arc(small);
    let clock = window.use_keyed_state((id, "clock"), cx, |_, _| Clock {
        started: Instant::now(),
        stopped: None,
    });
    let now = Instant::now();
    clock.update(cx, |clock, _| match (active, clock.stopped) {
        (true, Some(_)) => {
            clock.stopped = None;
            clock.started = now;
        }
        (false, None) => clock.stopped = Some(now),
        _ => {}
    });
    let (started, stopped) = {
        let c = clock.read(cx);
        (c.started, c.stopped)
    };
    let reduced = motion::reduced_motion(cx);
    let (rotation, offset, opacity) = match stopped {
        None if reduced => {
            // A pulse instead of rotation: 1 → 0.4 → 1 per 2 periods.
            let t = (now - started).as_secs_f32() / (ROTATION_PERIOD.as_secs_f32() * 2.);
            let phase = t.fract();
            window.request_animation_frame();
            (
                0.,
                geometry.offset,
                0.4 + 0.6 * (1. - (phase * 2. - 1.).abs()),
            )
        }
        None => {
            window.request_animation_frame();
            (rotation_at(now - started), geometry.offset, 1.)
        }
        // Reduced motion: `.cds--loading--stop svg circle { animation: none }`
        // keeps the arc; nothing turns.
        Some(_) if reduced => (0., geometry.offset, 1.),
        Some(at) => match stop_frame(now - at, geometry.offset) {
            Some((rotation, offset)) => {
                window.request_animation_frame();
                (rotation, offset, 1.)
            }
            None => (0., DASH, 1.),
        },
    };
    // `.cds--loading--stop svg circle` also retracts the background ring.
    let background = (geometry.background && offset < DASH).then(|| color(level.layer_accent()));
    let stroke = color(ColorRole::Interactive);
    let size = rems(geometry.size / crate::tokens::REM_PX);
    div().flex_none().size(size).opacity(opacity).child(
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                paint_ring(
                    bounds, geometry, offset, rotation, stroke, background, window,
                )
            },
        )
        .size_full(),
    )
}

impl RenderOnce for Spinner {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        if self.is_inline() {
            // `.cds--inline-loading__animation { margin-inline-end: 0.5rem }`.
            let indicator = match self.status {
                SpinnerStatus::Active | SpinnerStatus::Inactive => ring(
                    id.clone(),
                    true,
                    self.status == SpinnerStatus::Active,
                    self.level,
                    window,
                    cx,
                )
                .into_any_element(),
                // `.cds--inline-loading__checkmark-container { fill: var(--support-success) }`.
                SpinnerStatus::Finished => Icon::carbon(CarbonIcon::CheckmarkFilled)
                    .size(IconSize::S16)
                    .color(color(ColorRole::SupportSuccess))
                    .into_any_element(),
                // `.cds--inline-loading--error { fill: var(--support-error) }`.
                SpinnerStatus::Error => Icon::carbon(CarbonIcon::ErrorFilled)
                    .size(IconSize::S16)
                    .color(color(ColorRole::SupportError))
                    .into_any_element(),
            };
            return div()
                .flex()
                .flex_row()
                .items_center()
                .min_h(rems(2.))
                .text_color(color(ColorRole::TextSecondary))
                .child(div().flex().flex_none().mr(rems(0.5)).child(indicator))
                .when_some(self.description, |this, text| {
                    this.child(div().type_style(TypeStyle::Label02).child(text))
                })
                .into_any_element();
        }
        let small = self.size == SpinnerSize::Sm;
        let active = self.status != SpinnerStatus::Inactive;
        let spinner = ring(id, small, active, self.level, window, cx);
        if self.overlay && active {
            // `.cds--loading-overlay { position: fixed; inset: 0; display: flex;
            // align-items: center; justify-content: center; background: var(--overlay) }`.
            let viewport = window.viewport_size();
            return deferred(
                anchored().position(point(px(0.), px(0.))).child(
                    div()
                        .w(viewport.width)
                        .h(viewport.height)
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(color(ColorRole::Overlay))
                        .occlude()
                        .child(spinner),
                ),
            )
            .with_priority(6000)
            .into_any_element();
        }
        spinner.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arcs_follow_bundle_css() {
        let large = arc(false);
        let small = arc(true);
        assert_eq!((large.size, large.radius, large.stroke), (88., 44., 10.));
        assert_eq!((small.size, small.radius, small.stroke), (16., 42., 16.));
        assert!(
            (DASH - TAU * 44.).abs() < 1e-3,
            "dasharray is the r=44 circumference"
        );
        // Large: 81% of the ring is drawn from 3 o'clock.
        let (from, to) = dash_span(44., OFFSET_LARGE);
        assert_eq!(from, 0.);
        assert!(((to - from) - (DASH - OFFSET_LARGE)).abs() < 1e-3);
        // Small: 132.7 units of the r=42 ring (about half).
        let (from, to) = dash_span(42., OFFSET_SMALL);
        assert!((to - from - 132.70118).abs() < 1e-3);
        // Background: a −22 offset starts the dash 22 units in and runs to the end.
        let (from, to) = dash_span(42., OFFSET_BACKGROUND);
        assert!((from - 22.).abs() < 1e-3 && (to - TAU * 42.).abs() < 1e-3);
    }

    #[test]
    fn rotation_and_stop_sequence() {
        assert_eq!(ROTATION_PERIOD.as_millis(), 690);
        assert!((rotation_at(Duration::from_millis(345)) - 180.).abs() < 1e-3);
        // p1: one more clockwise turn, eased, stroke unchanged.
        let (rot, off) = stop_frame(Duration::from_millis(350), OFFSET_LARGE).unwrap();
        assert!(rot > 180. && off == OFFSET_LARGE);
        // p2: turning back while the stroke retracts toward DASH.
        let (rot, off) = stop_frame(Duration::from_millis(1050), OFFSET_LARGE).unwrap();
        assert!(rot < 0. && off > OFFSET_LARGE && off < DASH);
        assert!(stop_frame(Duration::from_millis(1400), OFFSET_LARGE).is_none());
        assert!(Spinner::new("s").description("Loading").is_inline());
        assert!(!Spinner::new("s").size(SpinnerSize::Sm).is_inline());
    }
}
