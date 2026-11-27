//! Shared helpers of the T3 Code pattern library (`src/t3/base.jsx`).
//!
//! - [`fmt_elapsed`] (`fmtElapsed`) and [`use_ticker`] (`useTicker`): the
//!   human-friendly elapsed time and a counter that ticks once a second.
//! - [`Spin`]: the decorative 16px spinner (`.t3-spin`), safe inside buttons.
//! - [`Status`] / [`Tone`] (`STATUS`): the status vocabulary shared by
//!   threads, subagents, workflow steps and setup stages.
//! - [`use_flash`] (`useFlash`): a boolean that turns itself off after `ms`.
//! - [`visually_hidden`] (`VH`, `.cu-visually-hidden`) and [`radio_step`]
//!   (`radioKeys`).
//! - [`pulse_opacity`]: the `t3-pulse` keyframes (`50% { opacity: .35 }`)
//!   used by connecting dots and the working pill.
use crate::{
    motion::{self, cubic_bezier_at},
    theme::color,
    tokens::ColorRole,
};
use gpui_kit::{
    Animation, AnimationExt as _, App, Bounds, ElementId, Entity, Hsla, InteractiveElement as _,
    IntoElement, ParentElement, PathBuilder, Pixels, RenderOnce, SharedString, Styled, Task,
    Window, canvas, div, point, px,
};
use std::f32::consts::TAU;
use std::time::{Duration, Instant};

/// Human-friendly elapsed time: `74` → `"1m 14s"`, `3700` → `"1h 1m"`,
/// `9` → `"9s"` (`fmtElapsed`).
pub fn fmt_elapsed(secs: u64) -> String {
    let (h, m, s) = (secs / 3600, (secs % 3600) / 60, secs % 60);
    if h > 0 {
        format!("{h}h {m}m")
    } else if m > 0 {
        format!("{m}m {s}s")
    } else {
        format!("{s}s")
    }
}

/// [`fmt_elapsed`] for an optional value: `None` → `""` (`fmtElapsed(null)`).
pub fn fmt_elapsed_opt(secs: Option<u64>) -> String {
    secs.map(fmt_elapsed).unwrap_or_default()
}

struct TickerState {
    start: u64,
    /// When the counter last (re)started counting from `value`.
    since: Instant,
    value: u64,
    running: bool,
    _task: Option<Task<()>>,
}

impl TickerState {
    fn now(&self) -> u64 {
        if self.running {
            self.value + self.since.elapsed().as_secs()
        } else {
            self.value
        }
    }
}

/// A ticking elapsed counter (`useTicker(start, running)`): `start` seconds,
/// plus one every second while `running`. A new `start` resets it; pausing
/// keeps the value. Keyed by `id`; re-renders the window once a second.
pub fn use_ticker(
    id: impl Into<ElementId>,
    start: u64,
    running: bool,
    window: &mut Window,
    cx: &mut App,
) -> u64 {
    let id: ElementId = id.into();
    let state = window.use_keyed_state((id, "ticker"), cx, |_, _| TickerState {
        start,
        since: Instant::now(),
        value: start,
        running,
        _task: None,
    });
    let value = state.update(cx, |s, _| {
        if s.start != start {
            // `useEffect(() => setT(start), [start])`.
            s.start = start;
            s.value = start;
            s.since = Instant::now();
        }
        if s.running != running {
            s.value = s.now();
            s.since = Instant::now();
            s.running = running;
        }
        s.now()
    });
    if running && state.read(cx)._task.is_none() {
        let elapsed = state.read(cx).since.elapsed();
        let next = Duration::from_secs(elapsed.as_secs() + 1).saturating_sub(elapsed);
        let weak = state.downgrade();
        let task = window.spawn(cx, async move |cx| {
            cx.background_executor().timer(next).await;
            let _ = cx.update(|_, cx| {
                if let Some(state) = weak.upgrade() {
                    state.update(cx, |s, cx| {
                        s._task = None;
                        cx.notify();
                    });
                }
            });
        });
        state.update(cx, |s, _| s._task = Some(task));
    } else if !running {
        state.update(cx, |s, _| s._task = None);
    }
    value
}

struct FlashState {
    on: bool,
    generation: u64,
    _task: Option<Task<()>>,
}

/// A boolean that turns itself off `ms` after [`Flash::arm`] (`useFlash`):
/// "Copied" confirmations and the like. The pending timer is cancelled on
/// re-arm and when the element stops rendering (its state is dropped).
#[derive(Clone)]
pub struct Flash {
    /// Whether the flash is on this frame.
    pub on: bool,
    state: Entity<FlashState>,
    duration: Duration,
}

impl Flash {
    /// Turn the flash on and restart its timer.
    pub fn arm(&self, cx: &mut App) {
        let duration = self.duration;
        self.state.update(cx, |s, cx| {
            s.on = true;
            s.generation += 1;
            let generation = s.generation;
            s._task = Some(cx.spawn(async move |this, cx| {
                cx.background_executor().timer(duration).await;
                let _ = this.update(cx, |s, cx| {
                    if s.generation == generation {
                        s.on = false;
                        cx.notify();
                    }
                });
            }));
            cx.notify();
        });
    }
}

/// The keyed [`Flash`] (`useFlash(ms = 1500)`).
pub fn use_flash(
    id: impl Into<ElementId>,
    duration: Duration,
    window: &mut Window,
    cx: &mut App,
) -> Flash {
    let id: ElementId = id.into();
    let state = window.use_keyed_state((id, "flash"), cx, |_, _| FlashState {
        on: false,
        generation: 0,
        _task: None,
    });
    Flash {
        on: state.read(cx).on,
        state,
        duration,
    }
}

/// `useFlash`'s default duration.
pub const FLASH_DURATION: Duration = Duration::from_millis(1500);

/// The next selected index of a radio group for `key` (`radioKeys`):
/// Right/Down +1, Left/Up −1 (both wrapping), Home first, End last; `None`
/// for any other key or an empty group. `current` not in the group counts
/// as −1, as `indexOf` does.
pub fn radio_step(key: &str, len: usize, current: Option<usize>) -> Option<usize> {
    if len == 0 {
        return None;
    }
    let i = current.map(|c| c as isize).unwrap_or(-1);
    let n = len as isize;
    match key {
        "right" | "down" => Some(((i + 1).rem_euclid(n)) as usize),
        "left" | "up" => Some(((i - 1 + n).rem_euclid(n)) as usize),
        "home" => Some(0),
        "end" => Some(len - 1),
        _ => None,
    }
}

/// The active tab index after tab `closed` is removed from a strip of
/// `len` tabs whose active index was `active`, the way a browser tab strip
/// does it: closing the active tab activates its neighbor (the next one,
/// else the previous one), closing a tab before it shifts it down by one,
/// and closing one after it leaves it alone. The result is always a valid
/// index of the `len - 1` remaining tabs (0 when none remain).
pub fn active_after_close(active: usize, closed: usize, len: usize) -> usize {
    let remaining = len.saturating_sub(1);
    let next = if closed < active {
        active - 1
    } else if closed == active && closed + 1 >= len {
        closed.saturating_sub(1)
    } else {
        active
    };
    next.min(remaining.saturating_sub(1))
}

/// A visually hidden label (`VH`, `.cu-visually-hidden`: absolute, 1×1,
/// clipped). GPUI has no screen-reader text yet, so this only keeps the
/// reference's structure: it takes no space and paints nothing visible.
pub fn visually_hidden(text: impl Into<SharedString>) -> gpui_kit::Div {
    div()
        .absolute()
        .size(px(1.))
        .overflow_hidden()
        .opacity(0.)
        .child(text.into())
}

/// The `.t3-warnicon` glyph: `WarningAltFilled` in `support-warning` with
/// its inner "!" filled `#161616` in every theme (`.t3-warnicon
/// path[data-icon-path="inner-path"] { fill: #161616; opacity: 1 }`). The
/// Material glyph's "!" is a cut-out, so a bar of ink sits behind it.
pub fn warning_glyph(size: f32) -> gpui_kit::Div {
    use super::craft::rem;
    let k = size / 16.;
    div()
        .relative()
        .flex_none()
        .size(rem(size))
        .child(
            div()
                .absolute()
                .left(rem(7. * k))
                .top(rem(6. * k))
                .w(rem(2. * k))
                .h(rem(7.5 * k))
                .bg(ink()),
        )
        .child(
            crate::icons::Icon::carbon(crate::icons::CarbonIcon::WarningAltFilled)
                .size_length(rem(size))
                .color(color(ColorRole::SupportWarning))
                .absolute()
                .top_0()
                .left_0(),
        )
}

/// `#161616` in every theme (Gray 100, the White theme's `text-primary`):
/// the ink the reference puts on `support-warning` yellow.
pub fn ink() -> Hsla {
    crate::theme::color_in(crate::tokens::CarbonTheme::White, ColorRole::TextPrimary)
}

/// The semantic tone of a [`Status`] (`STATUS[k].tone`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tone {
    /// `interactive`: in progress (blue).
    Interactive,
    /// `warning`: needs attention (yellow).
    Warning,
    /// `success`.
    Success,
    /// `error`.
    Error,
    /// `neutral`.
    Neutral,
}

impl Tone {
    /// The reference name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Interactive => "interactive",
            Self::Warning => "warning",
            Self::Success => "success",
            Self::Error => "error",
            Self::Neutral => "neutral",
        }
    }
}

/// The status vocabulary (`STATUS`) shared by threads, subagents, workflow
/// steps and setup stages.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Status {
    /// `working`.
    Working,
    /// `needs-you`.
    NeedsYou,
    /// `done`.
    Done,
    /// `failed`.
    Failed,
    /// `idle` (the fallback for unknown names).
    #[default]
    Idle,
    /// `queued`.
    Queued,
    /// `waiting`.
    Waiting,
    /// `stopped`.
    Stopped,
}

impl Status {
    /// Every status, in `STATUS` order.
    pub const ALL: [Status; 8] = [
        Self::Working,
        Self::NeedsYou,
        Self::Done,
        Self::Failed,
        Self::Idle,
        Self::Queued,
        Self::Waiting,
        Self::Stopped,
    ];

    /// The reference key (`"needs-you"`).
    pub const fn name(self) -> &'static str {
        match self {
            Self::Working => "working",
            Self::NeedsYou => "needs-you",
            Self::Done => "done",
            Self::Failed => "failed",
            Self::Idle => "idle",
            Self::Queued => "queued",
            Self::Waiting => "waiting",
            Self::Stopped => "stopped",
        }
    }

    /// The status for a reference key; unknown keys are [`Status::Idle`]
    /// (`STATUS[status] || STATUS.idle`).
    pub fn from_name(name: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|s| s.name() == name)
            .unwrap_or_default()
    }

    /// The default English label (`STATUS[k].label`); callers may override it.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Working => "Working",
            Self::NeedsYou => "Needs you",
            Self::Done => "Done",
            Self::Failed => "Failed",
            Self::Idle => "Idle",
            Self::Queued => "Queued",
            Self::Waiting => "Waiting",
            Self::Stopped => "Stopped",
        }
    }

    /// The tone (`STATUS[k].tone`).
    pub const fn tone(self) -> Tone {
        match self {
            Self::Working => Tone::Interactive,
            Self::NeedsYou | Self::Waiting => Tone::Warning,
            Self::Done => Tone::Success,
            Self::Failed => Tone::Error,
            Self::Idle | Self::Queued | Self::Stopped => Tone::Neutral,
        }
    }
}

/// `@keyframes t3-pulse { 50% { opacity: .35 } }`, 1.2s per cycle.
pub const PULSE_PERIOD: Duration = Duration::from_millis(1200);
/// The opacity at the middle of a `t3-pulse` cycle.
pub const PULSE_LOW: f32 = 0.35;
/// CSS `ease-in-out` (`.t3-dot--connecting { animation: t3-pulse 1.2s ease-in-out }`).
pub const EASE_IN_OUT: [f32; 4] = [0.42, 0., 0.58, 1.];
/// CSS `ease`, the default timing function (`.t3-pill--interactive .t3-pill-dot`).
pub const EASE: [f32; 4] = [0.25, 0.1, 0.25, 1.];

/// The `t3-pulse` opacity at cycle progress `t` (0..1) with keyframe timing
/// `curve`: 1 → 0.35 at 50% → 1, each half eased on its own (CSS applies the
/// timing function per keyframe interval).
pub fn pulse_opacity(t: f32, curve: [f32; 4]) -> f32 {
    let [x1, y1, x2, y2] = curve;
    let t = t.rem_euclid(1.);
    let (from, to, local) = if t < 0.5 {
        (1., PULSE_LOW, t * 2.)
    } else {
        (PULSE_LOW, 1., (t - 0.5) * 2.)
    };
    from + (to - from) * cubic_bezier_at(x1, y1, x2, y2, local)
}

/// Wrap `element` in the endless `t3-pulse` opacity animation.
pub fn pulse<E: IntoElement + Styled + 'static>(
    element: E,
    id: impl Into<ElementId>,
    curve: [f32; 4],
) -> impl IntoElement {
    element.with_animation(id, Animation::new(PULSE_PERIOD).repeat(), move |el, t| {
        el.opacity(pulse_opacity(t, curve))
    })
}

/// `.t3-spin svg { animation: cu-spin 0.69s linear infinite }`.
pub const SPIN_PERIOD: Duration = Duration::from_millis(690);
/// `t3.css`: `@media (prefers-reduced-motion: reduce) { .t3-spin svg {
/// animation-duration: 2s } }`. The bundle loads `ghost.css` after `t3.css`,
/// and its `!important` override (see [`REDUCED_PULSE_PERIOD`]) replaces the
/// whole animation, so this slower turn never shows; kept for API
/// compatibility.
#[deprecated(note = "ghost.css overrides it: `.t3-spin svg` pulses under reduced motion")]
pub const SPIN_PERIOD_REDUCED: Duration = Duration::from_millis(2000);

/// `ghost.css`, last in the bundle's cascade:
/// `@media (prefers-reduced-motion: reduce) { .t3-spin svg, .t3-provider-spin,
/// .g-motion-spin, .g-term-cursor, .t3-shimmer { animation: g-pulse 1.4s
/// ease-in-out infinite !important } }`. These keep animating under reduced
/// motion: rotation (and blink, and sweep) becomes this pulse.
pub const REDUCED_PULSE_PERIOD: Duration = Duration::from_millis(1400);

/// `@keyframes g-pulse { 0%, 100% { opacity: 1 } 50% { opacity: .35 } }`,
/// `ease-in-out`, at cycle progress `t`: the [`REDUCED_PULSE_PERIOD`] pulse.
pub fn reduced_pulse_opacity(t: f32) -> f32 {
    pulse_opacity(t, EASE_IN_OUT)
}

/// Progress (0..1) through a `period`-long cycle after `elapsed`.
pub fn loop_phase(elapsed: Duration, period: Duration) -> f32 {
    let period = period.as_nanos();
    if period == 0 {
        return 0.;
    }
    (elapsed.as_nanos() % period) as f32 / period as f32
}

fn loop_epoch() -> Instant {
    static EPOCH: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
    *EPOCH.get_or_init(Instant::now)
}

/// Progress (0..1) through an endless `period`-long cycle, and a request for
/// the next frame. Unlike GPUI's `with_animation`, which renders a repeating
/// animation frozen on its first frame whenever reduced motion is on, this
/// keeps running: for the loops the reference keeps alive under
/// `prefers-reduced-motion` (spinners and cursors that pulse instead). Call
/// while rendering.
pub fn endless_phase(period: Duration, window: &mut Window) -> f32 {
    window.request_animation_frame();
    loop_phase(loop_epoch().elapsed(), period)
}

/// One frame of a spinner: its rotation in turns and its opacity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpinFrame {
    /// Clockwise rotation, in turns (0..1).
    pub turn: f32,
    /// Opacity of the whole spinner.
    pub opacity: f32,
}

/// A spinner `elapsed` into its life: one turn per `period` (`cu-spin
/// linear`), or under reduced motion no rotation and the `g-pulse` opacity
/// ([`REDUCED_PULSE_PERIOD`]).
pub fn spin_frame(elapsed: Duration, period: Duration, reduced: bool) -> SpinFrame {
    if reduced {
        SpinFrame {
            turn: 0.,
            opacity: reduced_pulse_opacity(loop_phase(elapsed, REDUCED_PULSE_PERIOD)),
        }
    } else {
        SpinFrame {
            turn: loop_phase(elapsed, period),
            opacity: 1.,
        }
    }
}

/// [`spin_frame`] now, requesting the next frame (see [`endless_phase`]).
pub fn spin_frame_now(period: Duration, reduced: bool, window: &mut Window) -> SpinFrame {
    window.request_animation_frame();
    spin_frame(loop_epoch().elapsed(), period, reduced)
}

/// The `.t3-spin` SVG geometry in its 16×16 viewBox.
pub mod spin_geometry {
    /// `<circle cx="8" cy="8" r="6.5">`.
    pub const RADIUS: f32 = 6.5;
    /// `.t3-spin circle { stroke-width: 2.25 }`.
    pub const STROKE: f32 = 2.25;
    /// `.t3-spin-arc { stroke-dasharray: 40.8 }`.
    pub const DASH: f32 = 40.8;
    /// `.t3-spin-arc { stroke-dashoffset: 28 }`.
    pub const OFFSET: f32 = 28.;
    /// The arc's visible length: the dash (40.8, ≈ the 40.84 circumference)
    /// shifted back by the offset leaves 12.8 from 3 o'clock, clockwise.
    pub const ARC_LENGTH: f32 = DASH - OFFSET;
    /// The arc's sweep in radians (≈ 113°).
    pub const SWEEP: f32 = ARC_LENGTH / RADIUS;
}

/// Stroke an arc of `sweep` radians starting at `start` (0 = 3 o'clock,
/// clockwise) on a circle of `radius` px around `center`.
pub(crate) fn stroke_arc(
    window: &mut Window,
    center: gpui_kit::Point<Pixels>,
    radius: f32,
    width: f32,
    start: f32,
    sweep: f32,
    color: Hsla,
) {
    if sweep <= 0.001 || color.a == 0. {
        return;
    }
    let steps = ((sweep / (TAU / 96.)).ceil() as usize).max(2);
    let mut path = PathBuilder::stroke(px(width));
    for i in 0..=steps {
        let a = start + sweep * i as f32 / steps as f32;
        let p = point(
            center.x + px(radius * a.cos()),
            center.y + px(radius * a.sin()),
        );
        if i == 0 {
            path.move_to(p);
        } else {
            path.line_to(p);
        }
    }
    if let Ok(path) = path.build() {
        window.paint_path(path, color);
    }
}

fn paint_spin(bounds: Bounds<Pixels>, turn: f32, arc: Hsla, track: Hsla, window: &mut Window) {
    use spin_geometry::*;
    let scale = f32::from(bounds.size.width) / 16.;
    let center = bounds.center();
    let (r, w) = (RADIUS * scale, STROKE * scale);
    stroke_arc(window, center, r, w, 0., TAU, track);
    stroke_arc(window, center, r, w, turn * TAU, SWEEP, arc);
}

/// The decorative T3 spinner (`Spin`, `.t3-spin`): a `layer-accent-01`
/// track ring under a 113° `interactive` arc (r 6.5, stroke 2.25 in a 16
/// viewBox), turning once per 690ms; under reduced motion it holds still and
/// pulses ([`REDUCED_PULSE_PERIOD`]). It has no
/// live region; pair it with visible text, or give it a [`Spin::label`].
#[derive(IntoElement)]
pub struct Spin {
    id: ElementId,
    size: f32,
    label: Option<SharedString>,
    color: Option<Hsla>,
    track: Option<Hsla>,
}

impl Spin {
    /// A 16px spinner.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            size: 16.,
            label: None,
            color: None,
            track: None,
        }
    }

    /// Size in px (`size`, default 16).
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// A visually hidden label (`label`).
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// The arc color (`.t3-spin { color }`, default `interactive`; e.g.
    /// white over a thumbnail, `.t3-thumb-over .t3-spin`).
    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }

    /// The track color (`.t3-spin-track { stroke }`, default `layer-accent-01`).
    pub fn track(mut self, track: Hsla) -> Self {
        self.track = Some(track);
        self
    }
}

impl RenderOnce for Spin {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `.t3-spin { color: var(--interactive) }`, `.t3-spin-track { stroke: var(--layer-accent-01) }`.
        let arc = self.color.unwrap_or_else(|| color(ColorRole::Interactive));
        let track = self
            .track
            .unwrap_or_else(|| color(ColorRole::LayerAccent01));
        let frame = spin_frame_now(SPIN_PERIOD, motion::reduced_motion(cx), window);
        // A rem length, so it scales with the text it sits in (zoom,
        // `RemScale`); `paint_spin` scales the geometry to the box.
        let size = super::craft::rem(self.size);
        div()
            .id(self.id)
            .relative()
            .flex_none()
            .size(size)
            .children(self.label.map(visually_hidden))
            .child(
                div().size_full().opacity(frame.opacity).child(
                    canvas(
                        |_, _, _| {},
                        move |bounds, _, window, _| {
                            paint_spin(bounds, frame.turn, arc, track, window)
                        },
                    )
                    .size_full(),
                ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elapsed_follows_fmt_elapsed() {
        assert_eq!(fmt_elapsed(74), "1m 14s");
        assert_eq!(fmt_elapsed(9), "9s");
        assert_eq!(fmt_elapsed(0), "0s");
        assert_eq!(fmt_elapsed(3600), "1h 0m");
        assert_eq!(fmt_elapsed(3725), "1h 2m");
        assert_eq!(fmt_elapsed_opt(None), "");
    }

    #[test]
    fn status_vocabulary_follows_base_jsx() {
        assert_eq!(Status::from_name("needs-you"), Status::NeedsYou);
        assert_eq!(Status::from_name("bogus"), Status::Idle);
        assert_eq!(Status::NeedsYou.label(), "Needs you");
        assert_eq!(Status::Waiting.tone(), Tone::Warning);
        assert_eq!(Status::Working.tone(), Tone::Interactive);
        assert_eq!(Status::Queued.tone(), Tone::Neutral);
        assert_eq!(Status::Failed.tone().name(), "error");
    }

    #[test]
    fn radio_keys_wrap_and_jump() {
        assert_eq!(radio_step("right", 3, Some(2)), Some(0));
        assert_eq!(radio_step("up", 3, Some(0)), Some(2));
        assert_eq!(radio_step("home", 3, Some(1)), Some(0));
        assert_eq!(radio_step("end", 3, Some(0)), Some(2));
        assert_eq!(radio_step("down", 3, None), Some(0));
        assert_eq!(radio_step("a", 3, Some(0)), None);
        assert_eq!(radio_step("down", 0, None), None);
    }

    #[test]
    fn closing_a_tab_moves_the_active_index_like_a_browser() {
        // Close the active last tab: the previous one becomes active.
        assert_eq!(active_after_close(1, 1, 2), 0);
        // Close the active middle tab: the next one (shifted into place) wins.
        assert_eq!(active_after_close(1, 1, 3), 1);
        // Close the active first tab: the next one becomes index 0.
        assert_eq!(active_after_close(0, 0, 3), 0);
        // Close a tab before the active one: the index shifts down.
        assert_eq!(active_after_close(2, 0, 3), 1);
        assert_eq!(active_after_close(1, 0, 2), 0);
        // Close a tab after the active one: unchanged.
        assert_eq!(active_after_close(0, 1, 2), 0);
        assert_eq!(active_after_close(1, 2, 4), 1);
        // The last tab closed, or a stale index: clamped.
        assert_eq!(active_after_close(0, 0, 1), 0);
        assert_eq!(active_after_close(5, 0, 3), 1);
    }

    #[test]
    fn pulse_dips_to_35_percent_halfway() {
        // `@keyframes t3-pulse { 50% { opacity: .35 } }`.
        assert!((pulse_opacity(0., EASE_IN_OUT) - 1.).abs() < 1e-4);
        assert!((pulse_opacity(0.5, EASE_IN_OUT) - 0.35).abs() < 1e-4);
        assert!((pulse_opacity(0.999, EASE) - 1.).abs() < 1e-2);
        assert!(pulse_opacity(0.25, EASE_IN_OUT) < 1.);
    }

    #[test]
    fn spin_arc_follows_t3_css() {
        // `.t3-spin-arc { stroke-dasharray: 40.8; stroke-dashoffset: 28 }` on r=6.5.
        assert!((spin_geometry::ARC_LENGTH - 12.8).abs() < 1e-4);
        assert!((spin_geometry::SWEEP.to_degrees() - 112.8).abs() < 0.1);
        assert_eq!(SPIN_PERIOD.as_millis(), 690);
    }

    #[test]
    fn loop_phase_wraps_each_period() {
        let ms = Duration::from_millis;
        assert_eq!(loop_phase(ms(0), SPIN_PERIOD), 0.);
        assert!((loop_phase(ms(345), SPIN_PERIOD) - 0.5).abs() < 1e-4);
        assert!(loop_phase(ms(690), SPIN_PERIOD).abs() < 1e-4);
        assert!((loop_phase(ms(690 * 1000 + 69), SPIN_PERIOD) - 0.1).abs() < 1e-3);
        assert_eq!(loop_phase(ms(5), Duration::ZERO), 0.);
    }

    #[test]
    fn spinner_turns_at_full_motion() {
        // `.t3-spin svg { animation: cu-spin 0.69s linear infinite }`.
        let ms = Duration::from_millis;
        let quarter = spin_frame(ms(172) + Duration::from_micros(500), SPIN_PERIOD, false);
        assert!((quarter.turn - 0.25).abs() < 1e-3);
        assert_eq!(quarter.opacity, 1.);
        // `.t3-provider-spin { animation: cu-spin 0.8s linear infinite }`.
        let half = spin_frame(ms(400), Duration::from_millis(800), false);
        assert!((half.turn - 0.5).abs() < 1e-4);
    }

    #[test]
    fn spinner_pulses_under_reduced_motion() {
        // ghost.css: `.t3-spin svg, .t3-provider-spin { animation: g-pulse
        // 1.4s ease-in-out infinite !important }`, `g-pulse { 50% { opacity: .35 } }`.
        let ms = Duration::from_millis;
        assert_eq!(REDUCED_PULSE_PERIOD.as_millis(), 1400);
        let start = spin_frame(ms(0), SPIN_PERIOD, true);
        assert_eq!(start.turn, 0.);
        assert!((start.opacity - 1.).abs() < 1e-4);
        let low = spin_frame(ms(700), SPIN_PERIOD, true);
        assert_eq!(low.turn, 0.);
        assert!((low.opacity - 0.35).abs() < 1e-4);
        // It keeps going: the second cycle repeats the first.
        let again = spin_frame(ms(1400 + 700), SPIN_PERIOD, true);
        assert!((again.opacity - 0.35).abs() < 1e-3);
        let between = spin_frame(ms(350), SPIN_PERIOD, true).opacity;
        assert!(between > 0.35 && between < 1.);
        assert!((reduced_pulse_opacity(0.25) - between).abs() < 1e-4);
    }
}
