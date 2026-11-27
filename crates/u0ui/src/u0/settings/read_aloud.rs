//! ReadAloudControl: play / pause for text-to-speech with a voice menu and a
//! speed button (`ReadAloudControl` in `src/ghost/admin.jsx`, `.g-read` in
//! `src/ghost.css`).
//!
//! - **Behavior**: the blue square toggles playing; while playing a sound
//!   wave (`.t3-wave`, eight bars) replaces the "Read aloud" text. The voice
//!   menu lists `voices`; the speed button steps 0.25× up to 2×, then wraps
//!   to 0.75×. Playing and speed are the control's own state, seeded from
//!   `playing` / `rate`.
//! - **Motion**: each bar scales 0.25 → 1 → 0.25 every 0.9s (ease-in-out),
//!   staggered 90ms. Under reduced motion the bars stand still at full
//!   height (the reference cuts animations to 1ms).
//!
//! ```ignore
//! ReadAloudControl::new("read").voices(["Samantha", "Tingting", "Daniel"])
//! ```
use crate::{
    components::{DropdownMenu, DropdownMenuKind, DropdownMenuSize, MenuItem},
    icons::{CarbonIcon, Icon},
    motion,
    t3::{
        base::EASE_IN_OUT,
        craft::{rem, sans},
    },
    theme::color,
    tokens::{self, ColorRole},
    u0::{primitives::parts, rows as row},
};
use gpui_kit::{
    Animation, AnimationExt as _, App, CursorStyle, ElementId, Hsla, IntoElement, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _, px,
};
use std::{rc::Rc, time::Duration};

/// `.t3-wave i { animation: t3-wave 0.9s ease-in-out infinite }`.
pub const WAVE_PERIOD: Duration = Duration::from_millis(900);
/// `animationDelay: i * 90 + 'ms'`.
pub const WAVE_STAGGER_MS: f32 = 90.;
/// `@keyframes t3-wave { 0%, 100% { transform: scaleY(.25) } 50% { scaleY(1) } }`.
pub const WAVE_LOW: f32 = 0.25;

/// The `t3-wave` scale at cycle progress `t` (0..1), eased per keyframe
/// interval with `ease-in-out`.
pub fn wave_scale(t: f32) -> f32 {
    let [x1, y1, x2, y2] = EASE_IN_OUT;
    let t = t.rem_euclid(1.);
    let (from, to, local) = if t < 0.5 {
        (WAVE_LOW, 1., t * 2.)
    } else {
        (1., WAVE_LOW, (t - 0.5) * 2.)
    };
    from + (to - from) * motion::cubic_bezier_at(x1, y1, x2, y2, local)
}

/// The next speed: `r >= 2 ? 0.75 : +(r + 0.25).toFixed(2)`.
pub fn next_rate(rate: f32) -> f32 {
    if rate >= 2. {
        0.75
    } else {
        ((rate + 0.25) * 100.).round() / 100.
    }
}

/// A rate as JavaScript prints it (`1`, `1.25`, `0.75`).
pub fn rate_text(rate: f32) -> String {
    let s = format!("{rate:.2}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    s.to_string()
}

/// `.t3-wave`: `bars` 2px bars, 2px apart, 16px tall, in `ink`, scaling with
/// [`wave_scale`] (static at full height under reduced motion).
#[derive(IntoElement)]
pub struct Wave {
    id: ElementId,
    bars: usize,
    ink: Option<Hsla>,
    width: Option<f32>,
}

impl Wave {
    /// Eight bars (`.g-read-wave i { background: var(--interactive) }` by default).
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            bars: 8,
            ink: None,
            width: None,
        }
    }

    /// The number of bars.
    pub fn bars(mut self, bars: usize) -> Self {
        self.bars = bars;
        self
    }

    /// The bar color (`.t3-wave i { background: var(--support-error) }` in
    /// T3; `interactive` in `.g-read-wave`).
    pub fn ink(mut self, ink: Hsla) -> Self {
        self.ink = Some(ink);
        self
    }

    /// The box width in px (`.g-read-wave { width: 4rem }`).
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
}

impl RenderOnce for Wave {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let ink = self.ink.unwrap_or_else(|| color(ColorRole::Interactive));
        let reduced = motion::reduced_motion(cx);
        let period = WAVE_PERIOD.as_millis() as f32;
        // `.t3-wave { display: inline-flex; align-items: center; gap: 2px; height: 16px }`.
        div()
            .flex()
            .flex_none()
            .items_center()
            .gap(px(2.))
            .h(px(16.))
            .when_some(self.width, |d, w| d.w(rem(w)))
            .children((0..self.bars).map(move |i| {
                let bar = div().w(px(2.)).h(px(16.)).bg(ink);
                if reduced {
                    bar.into_any_element()
                } else {
                    let delay = i as f32 * WAVE_STAGGER_MS / period;
                    bar.with_animation(
                        (self.id.clone(), SharedString::from(format!("bar-{i}"))),
                        Animation::new(WAVE_PERIOD).repeat(),
                        move |el, t| el.h(px(16. * wave_scale(t - delay))),
                    )
                    .into_any_element()
                }
            }))
    }
}

/// The control's user-facing strings (the reference's English defaults).
#[derive(Clone)]
pub struct ReadAloudTexts {
    /// The idle text and the play button's name: `"Read aloud"`.
    pub read_aloud: SharedString,
    /// The play button's name while playing: `"Pause"`.
    pub pause: SharedString,
    /// The speed button's name: `"Speed {r}×"`.
    pub speed: Rc<dyn Fn(&str) -> SharedString>,
}

impl Default for ReadAloudTexts {
    fn default() -> Self {
        Self {
            read_aloud: "Read aloud".into(),
            pause: "Pause".into(),
            speed: Rc::new(|r| format!("Speed {r}×").into()),
        }
    }
}

#[derive(Clone)]
struct State {
    playing: bool,
    rate: f32,
    voice: usize,
}

/// `.g-read-play` background: `button-primary`, hover `button-primary-hover`.
pub fn play_background(hovered: bool) -> Hsla {
    color(if hovered {
        ColorRole::ButtonPrimaryHover
    } else {
        ColorRole::ButtonPrimary
    })
}

/// A read-aloud control. See the module docs.
#[derive(IntoElement)]
pub struct ReadAloudControl {
    id: ElementId,
    voices: Vec<SharedString>,
    rate: f32,
    playing: bool,
    texts: ReadAloudTexts,
}

impl ReadAloudControl {
    /// A paused control at 1× with the voice "Default".
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            voices: vec!["Default".into()],
            rate: 1.,
            playing: false,
            texts: ReadAloudTexts::default(),
        }
    }

    /// The voices (`voices`).
    pub fn voices(mut self, voices: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.voices = voices.into_iter().map(Into::into).collect();
        self
    }

    /// The starting speed (`rate`, default 1).
    pub fn rate(mut self, rate: f32) -> Self {
        self.rate = rate;
        self
    }

    /// Start playing (`playing`).
    pub fn playing(mut self, playing: bool) -> Self {
        self.playing = playing;
        self
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: ReadAloudTexts) -> Self {
        self.texts = texts;
        self
    }
}

impl RenderOnce for ReadAloudControl {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let (playing, rate) = (self.playing, self.rate);
        let state = window.use_keyed_state((id.clone(), "read"), cx, |_, _| State {
            playing,
            rate,
            voice: 0,
        });
        let s = state.read(cx).clone();
        let field = rem(crate::density::current().field().px_value());
        let play_focus = parts::keyed_focus((id.clone(), "play-focus"), window, cx);
        let rate_focus = parts::keyed_focus((id.clone(), "rate-focus"), window, cx);
        let play_hover = window.use_keyed_state((id.clone(), "play-hover"), cx, |_, _| false);
        let rate_text_s = rate_text(s.rate);
        let voice = self
            .voices
            .get(s.voice)
            .or(self.voices.first())
            .cloned()
            .unwrap_or_default();
        let items = self.voices.iter().enumerate().map(|(i, v)| {
            let state = state.clone();
            MenuItem::item(v.clone()).on_select(move |_, cx| {
                state.update(cx, |s, cx| {
                    s.voice = i;
                    cx.notify();
                })
            })
        });
        let toggle = state.clone();
        let step = state.clone();

        // `.g-read { display: inline-flex; align-items: center; gap:
        // var(--spacing-03); height: var(--density-field-md); padding-right:
        // var(--spacing-02); background: var(--layer-01); font-size: 0.75rem }`.
        sans(div(), 12., 400)
            .flex()
            .flex_none()
            .items_center()
            .gap(tokens::SPACING_03)
            .h(field)
            .pr(tokens::SPACING_02)
            .bg(color(ColorRole::Layer01))
            .text_color(color(ColorRole::TextPrimary))
            .child(
                // `.g-read-play { width/height: var(--density-field-md);
                // background: var(--button-primary); color: var(--text-on-color) }`.
                gpui_kit::base::Button::new((id.clone(), "play"))
                    .track_focus(&play_focus)
                    .accessibility_label(if s.playing {
                        self.texts.pause.clone()
                    } else {
                        self.texts.read_aloud.clone()
                    })
                    .relative()
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .size(field)
                    .bg(play_background(*play_hover.read(cx)))
                    .cursor(CursorStyle::PointingHand)
                    .on_hover(move |h, _, cx| {
                        play_hover.update(cx, |v, cx| {
                            *v = *h;
                            cx.notify();
                        })
                    })
                    .child(
                        Icon::carbon(if s.playing {
                            CarbonIcon::Pause
                        } else {
                            CarbonIcon::Play
                        })
                        .size_length(rem(16.))
                        .color(color(ColorRole::TextOnColor)),
                    )
                    .when(parts::focus_visible(&play_focus, window), |b| {
                        b.child(parts::focus_ring())
                    })
                    .on_click(move |_, _, cx| {
                        toggle.update(cx, |s, cx| {
                            s.playing = !s.playing;
                            cx.notify();
                        })
                    }),
            )
            .child(if s.playing {
                Wave::new((id.clone(), "wave"))
                    .width(64.)
                    .into_any_element()
            } else {
                row::dim(self.texts.read_aloud.clone(), 12.).into_any_element()
            })
            .child(
                DropdownMenu::new((id.clone(), "voice"))
                    .label(voice)
                    .kind(DropdownMenuKind::Ghost)
                    .size(DropdownMenuSize::Sm)
                    .items(items),
            )
            .child(
                // `.g-read-rate { min-width: 2.5rem; height: 1.5rem; padding: 0
                // var(--spacing-02); background: var(--layer-accent-01); color:
                // var(--text-primary); font-size: 0.75rem }`.
                gpui_kit::base::Button::new((id.clone(), "rate"))
                    .track_focus(&rate_focus)
                    .accessibility_label((self.texts.speed)(&rate_text_s))
                    .relative()
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .min_w(rem(40.))
                    .h(rem(24.))
                    .px(tokens::SPACING_02)
                    .bg(color(ColorRole::LayerAccent01))
                    .text_size(rem(12.))
                    .cursor(CursorStyle::PointingHand)
                    .child(format!("{rate_text_s}×"))
                    .when(parts::focus_visible(&rate_focus, window), |b| {
                        b.child(parts::focus_ring())
                    })
                    .on_click(move |_, _, cx| {
                        step.update(cx, |s, cx| {
                            s.rate = next_rate(s.rate);
                            cx.notify();
                        })
                    }),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CarbonTheme;

    #[test]
    fn rate_steps_and_wraps() {
        assert_eq!(next_rate(1.), 1.25);
        assert_eq!(next_rate(1.75), 2.);
        assert_eq!(next_rate(2.), 0.75);
        assert_eq!(rate_text(1.), "1");
        assert_eq!(rate_text(1.25), "1.25");
        assert_eq!(rate_text(0.75), "0.75");
        assert_eq!(rate_text(1.5), "1.5");
    }

    #[test]
    fn wave_follows_t3_wave_keyframes() {
        assert!((wave_scale(0.) - WAVE_LOW).abs() < 1e-4);
        assert!((wave_scale(0.5) - 1.).abs() < 1e-4);
        assert!((wave_scale(1.) - WAVE_LOW).abs() < 1e-4);
        assert!(wave_scale(0.25) > WAVE_LOW && wave_scale(0.25) < 1.);
        assert_eq!(WAVE_PERIOD.as_millis(), 900);
    }

    #[test]
    fn play_button_follows_button_primary() {
        crate::theme::tests_set(CarbonTheme::G100);
        assert_eq!(play_background(false), color(ColorRole::ButtonPrimary));
        assert_eq!(play_background(true), color(ColorRole::ButtonPrimaryHover));
    }
}
