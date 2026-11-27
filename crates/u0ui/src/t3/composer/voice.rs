//! [`VoiceInputPill`] (`.t3-voice`, "Primary actions, voice" in `t3.css`
//! with the gemini-G and r2-gal fixes): the dictation status block.
//!
//! - `recording`: on `notification-background-error`, a 12-bar
//!   `support-error` waveform (each bar `t3-wave` 0.9s ease-in-out, offset
//!   `(i × 83) % 600` ms), "Recording", a ticking time, Discard (×) and a
//!   neutral square Done (✓) — never a second ■ beside the agent's Stop.
//! - `starting` / `downloading` (with %) / `transcribing` / `cleaning`: a
//!   spinner and the step on `layer-accent-01`.
//! - `error`: the warning glyph, the message and a dismiss ×.
//!
//! Square everywhere (gemini-G). Under reduced motion the waveform holds
//! still at mid height.
use crate::t3::base::{Spin, fmt_elapsed, use_ticker};
use crate::t3::press::{self, Handler, rem};
use crate::{
    icons::CarbonIcon,
    motion::{self, cubic_bezier_at},
    theme::color,
    tokens::ColorRole,
};
use gpui_kit::{
    Animation, AnimationExt as _, App, ElementId, Hsla, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _, px,
};
use std::rc::Rc;
use std::time::Duration;

/// `VoiceInputPill` `state`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum VoiceState {
    /// Recording.
    #[default]
    Recording,
    /// Starting…
    Starting,
    /// Downloading model n%.
    Downloading,
    /// Transcribing…
    Transcribing,
    /// Cleaning up…
    Cleaning,
    /// Failed.
    Error,
}

impl VoiceState {
    /// Every state.
    pub const ALL: [VoiceState; 6] = [
        Self::Recording,
        Self::Starting,
        Self::Downloading,
        Self::Transcribing,
        Self::Cleaning,
        Self::Error,
    ];

    /// The reference name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Recording => "recording",
            Self::Starting => "starting",
            Self::Downloading => "downloading",
            Self::Transcribing => "transcribing",
            Self::Cleaning => "cleaning",
            Self::Error => "error",
        }
    }
}

/// `VoiceInputPill`'s strings (the reference's English defaults).
#[derive(Clone)]
pub struct VoiceTexts {
    /// Recording.
    pub recording: SharedString,
    /// Starting…
    pub starting: SharedString,
    /// `"Downloading model {n}%"`.
    pub downloading: Rc<dyn Fn(u8) -> SharedString>,
    /// Transcribing…
    pub transcribing: SharedString,
    /// Cleaning up…
    pub cleaning: SharedString,
    /// The error without `error`.
    pub failed: SharedString,
    /// Dismiss (error ×).
    pub dismiss: SharedString,
    /// Discard recording (×).
    pub discard: SharedString,
    /// Stop recording (✓).
    pub stop: SharedString,
}

impl Default for VoiceTexts {
    fn default() -> Self {
        Self {
            recording: "Recording".into(),
            starting: "Starting…".into(),
            downloading: Rc::new(|n| format!("Downloading model {n}%").into()),
            transcribing: "Transcribing…".into(),
            cleaning: "Cleaning up…".into(),
            failed: "Voice input failed".into(),
            dismiss: "Dismiss".into(),
            discard: "Discard recording".into(),
            stop: "Stop recording".into(),
        }
    }
}

/// `@keyframes t3-wave { 0%, 100% { scaleY(.25) } 50% { scaleY(1) } }`,
/// `0.9s ease-in-out infinite`.
pub const WAVE_PERIOD: Duration = Duration::from_millis(900);

/// A wave bar's scale at cycle progress `t` (0…1), each half eased
/// `ease-in-out` (CSS applies the timing function per keyframe interval).
pub fn wave_scale(t: f32) -> f32 {
    let t = t.rem_euclid(1.);
    let (from, to, local) = if t < 0.5 {
        (0.25, 1., t * 2.)
    } else {
        (1., 0.25, (t - 0.5) * 2.)
    };
    from + (to - from) * cubic_bezier_at(0.42, 0., 0.58, 1., local)
}

/// Bar `i`'s `animation-delay` in ms: `(i * 83) % 600`.
pub const fn wave_delay(i: usize) -> u64 {
    ((i * 83) % 600) as u64
}

/// The dictation status block. See the module docs.
#[derive(IntoElement)]
pub struct VoiceInputPill {
    id: ElementId,
    state: VoiceState,
    progress: u8,
    seconds: u64,
    error: Option<SharedString>,
    texts: VoiceTexts,
    on_stop: Option<Handler>,
    on_cancel: Option<Handler>,
}

impl VoiceInputPill {
    /// The pill in `state`.
    pub fn new(id: impl Into<ElementId>, state: VoiceState) -> Self {
        Self {
            id: id.into(),
            state,
            progress: 0,
            seconds: 0,
            error: None,
            texts: VoiceTexts::default(),
            on_stop: None,
            on_cancel: None,
        }
    }

    /// `progress` (downloading), 0–100.
    pub fn progress(mut self, progress: u8) -> Self {
        self.progress = progress;
        self
    }

    /// `seconds`: where the recording timer starts.
    pub fn seconds(mut self, seconds: u64) -> Self {
        self.seconds = seconds;
        self
    }

    /// `error`: the failure message.
    pub fn error(mut self, error: impl Into<SharedString>) -> Self {
        self.error = Some(error.into());
        self
    }

    /// Replace the strings.
    pub fn texts(mut self, texts: VoiceTexts) -> Self {
        self.texts = texts;
        self
    }

    /// `onStop` (✓ Done).
    pub fn on_stop(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_stop = Some(Rc::new(f));
        self
    }

    /// `onCancel` (× Discard, or dismiss the error).
    pub fn on_cancel(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_cancel = Some(Rc::new(f));
        self
    }
}

/// A square pill button: `size` px, `fill` at rest and `hover` under the
/// pointer, the 2px focus ring on keyboard focus.
#[allow(clippy::too_many_arguments)]
fn pill_button(
    id: ElementId,
    label: SharedString,
    size: f32,
    fill: Option<Hsla>,
    hover_fill: Option<Hsla>,
    icon: CarbonIcon,
    icon_size: f32,
    on_click: Option<Handler>,
    window: &mut Window,
    cx: &mut App,
) -> gpui_kit::base::Button {
    let (hovered, hover) = press::hover((id.clone(), "hover"), window, cx);
    let (focus, ring) = press::focus((id.clone(), "focus"), window, cx);
    let bg = if hovered { hover_fill.or(fill) } else { fill };
    gpui_kit::base::Button::new(id)
        .track_focus(&focus)
        .accessibility_label(label)
        .relative()
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .size(rem(size))
        .cursor_pointer()
        .when_some(bg, |this, bg| this.bg(bg))
        .on_hover(press::track_hover(hover))
        .when_some(on_click, |this, f| this.on_click(move |_, w, cx| f(w, cx)))
        .child(press::icon(icon, icon_size, color(ColorRole::IconPrimary)))
        .when(ring, |this| this.child(press::ring()))
}

impl RenderOnce for VoiceInputPill {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let recording = self.state == VoiceState::Recording;
        let t = use_ticker((id.clone(), "ticker"), self.seconds, recording, window, cx);
        let texts = self.texts.clone();
        // `.t3-voice { display: inline-flex; align-items: center; gap: spacing-03; height: size-sm;
        // padding: 0 spacing-02 0 spacing-04; border-radius: 0; background: layer-accent-01;
        // color: text-primary; font-size: 0.75rem; white-space: nowrap }`.
        let pill = div()
            .flex()
            .flex_row()
            .flex_none()
            .items_center()
            .gap(rem(8.))
            .h(rem(32.))
            .pl(rem(12.))
            .pr(rem(4.))
            .text_color(color(ColorRole::TextPrimary))
            .text_size(rem(12.))
            .line_height(rem(16.))
            .whitespace_nowrap();
        if self.state == VoiceState::Error {
            // `.t3-voice--error { background: notification-background-error; padding-right: spacing-03 }`,
            // `> svg { fill: support-error }`.
            return pill
                .pr(rem(8.))
                .bg(color(ColorRole::NotificationBackgroundError))
                .child(press::icon(
                    CarbonIcon::WarningAltFilled,
                    16.,
                    color(ColorRole::SupportError),
                ))
                .child(self.error.clone().unwrap_or(texts.failed.clone()))
                // `.t3-voice-x { width: 1.5rem; height: 1.5rem; background: transparent; color: icon-primary }`.
                .child(pill_button(
                    (id.clone(), "dismiss").into(),
                    texts.dismiss.clone(),
                    24.,
                    None,
                    None,
                    CarbonIcon::Close,
                    12.,
                    self.on_cancel.clone(),
                    window,
                    cx,
                ))
                .into_any_element();
        }
        if !recording {
            let text = match self.state {
                VoiceState::Starting => texts.starting.clone(),
                VoiceState::Downloading => (texts.downloading)(self.progress),
                VoiceState::Transcribing => texts.transcribing.clone(),
                _ => texts.cleaning.clone(),
            };
            return pill
                .bg(color(ColorRole::LayerAccent01))
                .child(Spin::new((id.clone(), "spin")))
                .child(text)
                .into_any_element();
        }
        let reduced = motion::reduced_motion(cx);
        let red = color(ColorRole::SupportError);
        // `.t3-wave { display: inline-flex; align-items: center; gap: 2px; height: 16px }`,
        // `i { width: 2px; height: 100%; background: support-error; transform-origin: center }`.
        let wave = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(2.))
            .h(px(16.));
        let wave = if reduced {
            wave.children((0..12).map(|_| div().w(px(2.)).h(px(16. * 0.6)).bg(red)))
                .into_any_element()
        } else {
            wave.with_animation(
                (id.clone(), "wave"),
                Animation::new(WAVE_PERIOD).repeat(),
                move |this, t| {
                    this.children((0..12).map(|i| {
                        let delay = wave_delay(i) as f32 / WAVE_PERIOD.as_millis() as f32;
                        div().w(px(2.)).h(px(16. * wave_scale(t - delay))).bg(red)
                    }))
                },
            )
            .into_any_element()
        };
        // `.t3-voice--recording { background: notification-background-error }`.
        pill.bg(color(ColorRole::NotificationBackgroundError))
            .child(wave)
            .child(texts.recording.clone())
            // `.t3-voice-time { tabular-nums; color: text-secondary }`.
            .child(
                div()
                    .text_color(color(ColorRole::TextSecondary))
                    .child(SharedString::from(fmt_elapsed(t))),
            )
            // r2-gal: `.t3-voice .t3-voice-cancel { width: 2rem; height: 2rem; border-radius: 0 }`,
            // `:hover { background: layer-hover-01 }`.
            .child(pill_button(
                (id.clone(), "discard").into(),
                texts.discard.clone(),
                32.,
                None,
                Some(color(ColorRole::LayerHover01)),
                CarbonIcon::Close,
                16.,
                self.on_cancel.clone(),
                window,
                cx,
            ))
            // `.t3-voice .t3-voice-stop { width: 2rem; height: 2rem; background: layer-accent-01;
            // color: icon-primary }` showing a check (gemini-G), `:hover { layer-accent-hover-01 }`.
            .child(pill_button(
                (id.clone(), "stop").into(),
                texts.stop.clone(),
                32.,
                Some(color(ColorRole::LayerAccent01)),
                Some(color(ColorRole::LayerAccentHover01)),
                CarbonIcon::Checkmark,
                12.,
                self.on_stop.clone(),
                window,
                cx,
            ))
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_wave_breathes_between_a_quarter_and_full() {
        assert!((wave_scale(0.) - 0.25).abs() < 1e-4);
        assert!((wave_scale(0.5) - 1.).abs() < 1e-4);
        assert!((wave_scale(1.) - 0.25).abs() < 1e-4);
        assert_eq!(wave_delay(1), 83);
        assert_eq!(wave_delay(8), 64);
    }
}
