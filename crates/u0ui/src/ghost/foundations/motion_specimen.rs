//! MotionSpecimen: full motion beside its reduced-motion fallback
//! (`MotionSpecimen` in `src/ghost/foundations.jsx`, the ReducedMotion card).
//!
//! Three rows (`.g-motion-row`: a 6rem label, a 1.5rem `layer-01` track,
//! a 9rem caption, 12px apart, 12px text):
//!
//! - **Panel enter**: an `interactive` bar 30% of the track slides from
//!   −100% to 120% of its width over the first 40% of 2.4s
//!   (`cubic-bezier(0, 0, 0.38, 0.9)`) and holds (`g-slide`). Reduced: it
//!   stands at 35% and fades in and out instead (`g-fade`, linear).
//! - **Spinner**: a 16px ring (`layer-accent-01`, its top quarter
//!   `interactive`) turning every 690ms (`cu-spin`). Reduced: no rotation,
//!   an opacity pulse 1 → 0.35 → 1 over 1.4s ease-in-out (`g-pulse`).
//! - **Shimmer**: a `transparent → layer-hover-01 → transparent` band sweeping
//!   the track every 1.2s (`g-sweep`). Reduced: still, at 60% opacity.
//!
//! `reduced: None` follows [`motion::reduced_motion`] (the system setting or
//! the app's override), like `usePrefersReducedMotion()`.
use crate::{
    motion,
    t3::base::{spin_frame_now, stroke_arc},
    theme::color,
    tokens::ColorRole,
};
use gpui_kit::{
    Animation, AnimationExt as _, App, ElementId, Hsla, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, Window, canvas, div, linear_color_stop, linear_gradient, px, relative,
    rems,
};
use std::{f32::consts::TAU, time::Duration};

/// `g-slide`/`g-fade` period.
pub const PANEL_PERIOD: Duration = Duration::from_millis(2400);
/// `cu-spin .69s`.
pub const SPIN_PERIOD: Duration = Duration::from_millis(690);
/// `g-pulse 1.4s`.
pub const PULSE_PERIOD: Duration = Duration::from_millis(1400);
/// `g-sweep 1.2s`.
pub const SWEEP_PERIOD: Duration = Duration::from_millis(1200);

/// `g-slide` at progress `t` (0–1): the bar's offset in its own widths
/// (`0% { translateX(-100%) } 40%, 100% { translateX(120%) }`, eased by
/// `cubic-bezier(0, 0, 0.38, 0.9)` between 0% and 40%).
pub fn slide(t: f32) -> f32 {
    if t >= 0.4 {
        1.2
    } else {
        -1. + 2.2 * motion::cubic_bezier_at(0., 0., 0.38, 0.9, t / 0.4)
    }
}

/// `g-fade` at `t`: `0%, 100% { opacity: 0 } 30%, 70% { opacity: 1 }`, linear.
pub fn fade(t: f32) -> f32 {
    if t < 0.3 {
        t / 0.3
    } else if t <= 0.7 {
        1.
    } else {
        (1. - t) / 0.3
    }
}

/// `g-pulse` at `t`: `0%, 100% { opacity: 1 } 50% { opacity: .35 }`, each
/// half `ease-in-out`.
pub fn pulse(t: f32) -> f32 {
    let (from, to, local) = if t < 0.5 {
        (1., 0.35, t / 0.5)
    } else {
        (0.35, 1., (t - 0.5) / 0.5)
    };
    from + (to - from) * motion::cubic_bezier_at(0.42, 0., 0.58, 1., local)
}

/// `g-sweep` at `t`: the band's offset in track widths (−1 → 1, linear).
pub fn sweep(t: f32) -> f32 {
    -1. + 2. * t
}

/// The 16px ring: a 2px `layer-accent-01` track, its top quarter
/// `interactive` (`border-top-color`), turned by `turn` (0–1).
fn ring(turn: f32, track: Hsla, arc: Hsla) -> impl IntoElement {
    canvas(
        |_, _, _| {},
        move |bounds, _, window, _| {
            let c = bounds.center();
            // The border's centerline: radius 8 − 1.
            stroke_arc(window, c, 7., 2., 0., TAU, track);
            // `border-top` covers 90° centered on the top (−135° … −45°).
            stroke_arc(
                window,
                c,
                7.,
                2.,
                -0.75 * std::f32::consts::PI + turn * TAU,
                TAU / 4.,
                arc,
            );
        },
    )
    .size(px(16.))
}

/// A motion specimen. See the module docs.
#[derive(IntoElement)]
pub struct MotionSpecimen {
    id: ElementId,
    reduced: Option<bool>,
    labels: [SharedString; 3],
    captions: [SharedString; 6],
}

impl MotionSpecimen {
    /// A specimen following the reduced-motion preference.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            reduced: None,
            // The reference's texts; override per locale.
            labels: ["Panel enter".into(), "Spinner".into(), "Shimmer".into()],
            captions: [
                "slide 240ms".into(),
                "fade 70ms".into(),
                "rotate 690ms".into(),
                "pulse, no rotation".into(),
                "sweep 1.2s".into(),
                "static".into(),
            ],
        }
    }

    /// Force full (`false`) or reduced (`true`) motion.
    pub fn reduced(mut self, reduced: bool) -> Self {
        self.reduced = Some(reduced);
        self
    }

    /// Row labels and the (full, reduced) caption pairs.
    pub fn texts(mut self, labels: [SharedString; 3], captions: [SharedString; 6]) -> Self {
        self.labels = labels;
        self.captions = captions;
        self
    }
}

impl RenderOnce for MotionSpecimen {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let r = self.reduced.unwrap_or_else(|| motion::reduced_motion(cx));
        // ghost.css: `@media (prefers-reduced-motion: reduce) { .g-motion-spin
        // { animation: g-pulse 1.4s ease-in-out infinite !important } }`, so
        // the full row pulses too when the app reduces motion. Driven by
        // `spin_frame_now`, which (unlike `with_animation`) keeps running then.
        let spin = spin_frame_now(SPIN_PERIOD, r || motion::reduced_motion(cx), window);
        let id = self.id.clone();
        let interactive = color(ColorRole::Interactive);
        let track_fill = color(ColorRole::Layer01);
        let accent = color(ColorRole::LayerAccent01);
        let hover = color(ColorRole::LayerHover01);
        let clear = Hsla::transparent_black();
        let [l_panel, l_spin, l_shimmer] = self.labels.clone();
        let [c_slide, c_fade, c_rotate, c_pulse, c_sweep, c_static] = self.captions.clone();
        let row = |label: SharedString, track: gpui_kit::AnyElement, caption: SharedString| {
            div()
                .flex()
                .items_center()
                .gap(rems(0.75))
                .text_size(rems(0.75))
                .line_height(rems(1.))
                .child(
                    div()
                        .w(rems(6.))
                        .flex_none()
                        .text_color(color(ColorRole::TextPrimary))
                        .child(label),
                )
                .child(
                    div()
                        .relative()
                        .flex_1()
                        .min_w_0()
                        .h(rems(1.5))
                        .overflow_hidden()
                        .bg(track_fill)
                        .child(track),
                )
                .child(
                    div()
                        .w(rems(9.))
                        .flex_none()
                        .text_color(color(ColorRole::TextSecondary))
                        .child(caption),
                )
        };
        // `.g-motion-panel { position: absolute; top: 4px; bottom: 4px; left: 0; width: 30% }`.
        let bar = div()
            .absolute()
            .top(px(4.))
            .bottom(px(4.))
            .w(relative(0.3))
            .bg(interactive);
        let panel = if r {
            bar.left(relative(0.35))
                .with_animation(
                    (id.clone(), "fade"),
                    Animation::new(PANEL_PERIOD).repeat(),
                    |d, t| d.opacity(fade(t)),
                )
                .into_any_element()
        } else {
            bar.with_animation(
                (id.clone(), "slide"),
                Animation::new(PANEL_PERIOD).repeat(),
                |d, t| d.left(relative(0.3 * slide(t))),
            )
            .into_any_element()
        };
        let spin_box = div().absolute().left(px(8.)).top(px(4.)).size(px(16.));
        let spinner = spin_box
            .opacity(spin.opacity)
            .child(ring(spin.turn, accent, interactive))
            .into_any_element();
        let band = move |offset: f32| {
            div()
                .absolute()
                .top_0()
                .bottom_0()
                .w_full()
                .left(relative(offset))
                .flex()
                .child(div().flex_1().h_full().bg(linear_gradient(
                    90.,
                    linear_color_stop(clear, 0.),
                    linear_color_stop(hover, 1.),
                )))
                .child(div().flex_1().h_full().bg(linear_gradient(
                    90.,
                    linear_color_stop(hover, 0.),
                    linear_color_stop(clear, 1.),
                )))
        };
        let shimmer = if r {
            // `.g-motion--reduced .g-motion-shimmer { animation: none; opacity: .6 }`.
            band(0.).opacity(0.6).into_any_element()
        } else {
            div()
                .absolute()
                .inset_0()
                .with_animation(
                    (id.clone(), "sweep"),
                    Animation::new(SWEEP_PERIOD).repeat(),
                    move |d, t| d.child(band(sweep(t))),
                )
                .into_any_element()
        };
        div()
            .flex()
            .flex_col()
            .gap(rems(0.75))
            .child(row(l_panel, panel, if r { c_fade } else { c_slide }))
            .child(row(l_spin, spinner, if r { c_pulse } else { c_rotate }))
            .child(row(l_shimmer, shimmer, if r { c_static } else { c_sweep }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyframes_match_ghost_css() {
        // g-slide: -100% → 120% over the first 40%, then held.
        assert!((slide(0.) + 1.).abs() < 1e-4);
        assert!((slide(0.4) - 1.2).abs() < 1e-4);
        assert_eq!(slide(0.9), 1.2);
        // g-fade: 0 → 1 by 30%, 1 until 70%, back to 0.
        assert_eq!(fade(0.), 0.);
        assert!((fade(0.3) - 1.).abs() < 1e-6);
        assert_eq!(fade(0.5), 1.);
        assert!(fade(1.).abs() < 1e-6);
        // g-pulse: 1 → .35 at 50% → 1.
        assert!((pulse(0.) - 1.).abs() < 1e-4);
        assert!((pulse(0.5) - 0.35).abs() < 1e-4);
        assert!((pulse(1.) - 1.).abs() < 1e-4);
        assert_eq!(sweep(0.5), 0.);
        assert_eq!(SPIN_PERIOD.as_millis(), 690);
    }
}
