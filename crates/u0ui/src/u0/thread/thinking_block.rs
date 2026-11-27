//! ThinkingBlock: the agent's reasoning (`ThinkingBlock`, `.g-think` in
//! `src/ghost.css`).
//!
//! While `live` the head reads "Thinking" under the `.t3-shimmer` sweep;
//! finished, it reads "Thought for 23s" and opens on click to show the
//! reasoning in italic secondary text. A 1px `border-subtle-01` rule on the
//! left lines it up with the tool calls around it.
//!
//! The sweep is `background-clip: text` in the reference, which GPUI lacks:
//! here each character takes the gradient's color at its share of the
//! line's width ([`shimmer_mix`]). Under reduced motion it pulses instead
//! (`ghost.css`: `.t3-shimmer { animation: g-pulse 1.4s ease-in-out infinite }`).
//!
//! ```ignore
//! ThinkingBlock::new("think-1").seconds(23).child("The G100 theme has no …")
//! ```
use super::parts::{self, rem};
use crate::{
    density,
    icons::{CarbonIcon, Icon},
    motion,
    t3::base::{endless_phase, fmt_elapsed_opt, reduced_pulse_opacity},
    theme::color,
    tokens::ColorRole,
};
use gpui_kit::{
    Animation, AnimationExt as _, AnyElement, App, ElementId, HighlightStyle, Hsla,
    InteractiveElement as _, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement as _, Styled, StyledText, Window, div, prelude::FluentBuilder as _,
    px, relative,
};
use std::time::Duration;

/// `.t3-shimmer { animation: t3-shim 2.4s linear infinite }`.
pub const SHIMMER_PERIOD: Duration = Duration::from_millis(2400);
/// `@keyframes g-pulse` under reduced motion: 1.4s ease-in-out.
pub const PULSE_PERIOD: Duration = Duration::from_millis(1400);

/// How much of `text-primary` (0..1, over `text-secondary`) the shimmer
/// paints at `u` (0 = the text's left edge, 1 = its right) at cycle
/// progress `t`.
///
/// `.t3-shimmer`: the gradient is secondary to 40%, primary at 50%,
/// secondary from 60%, over `background-size: 300%`, moving from
/// `background-position: 100%` to `-50%`. With the text W wide the image is
/// 3W, offset by −2W·p, so its 50% peak sits at u = 1.5 − 2p, sweeping from
/// −0.5 to 2.5, and each 10% ramp spans 0.3W.
pub fn shimmer_mix(u: f32, t: f32) -> f32 {
    let p = 1. - 1.5 * t.rem_euclid(1.);
    let peak = 1.5 - 2. * p;
    (1. - (u - peak).abs() / 0.3).clamp(0., 1.)
}

fn mix(a: Hsla, b: Hsla, f: f32) -> Hsla {
    let a = a.to_rgb();
    let b = b.to_rgb();
    gpui_kit::Rgba {
        r: a.r + (b.r - a.r) * f,
        g: a.g + (b.g - a.g) * f,
        b: a.b + (b.b - a.b) * f,
        a: a.a + (b.a - a.a) * f,
    }
    .into()
}

/// `text` under the `.t3-shimmer` sweep, or the reduced-motion pulse.
fn shimmer(id: ElementId, text: SharedString, window: &mut Window, cx: &App) -> AnyElement {
    let secondary = color(ColorRole::TextSecondary);
    if motion::reduced_motion(cx) {
        // GPUI freezes `with_animation` loops under reduced motion; the
        // pulse keeps running.
        let opacity = reduced_pulse_opacity(endless_phase(PULSE_PERIOD, window));
        // `@media (prefers-reduced-motion: reduce) { .t3-shimmer { color: text-secondary;
        // background: none } }`, pulsing (`ghost.css`).
        return div()
            .text_color(secondary)
            .child(text)
            .opacity(opacity)
            .into_any_element();
    }
    let primary = color(ColorRole::TextPrimary);
    let spans: Vec<(usize, usize)> = text
        .char_indices()
        .map(|(i, c)| (i, i + c.len_utf8()))
        .collect();
    let n = spans.len().max(1) as f32;
    div()
        .text_color(secondary)
        .child(StyledText::new(text.clone()).with_animation(
            id,
            Animation::new(SHIMMER_PERIOD).repeat(),
            move |styled, t| {
                styled.with_highlights(spans.iter().enumerate().map(|(k, &(a, b))| {
                    let f = shimmer_mix((k as f32 + 0.5) / n, t);
                    (
                        a..b,
                        HighlightStyle {
                            color: Some(mix(secondary, primary, f)),
                            ..Default::default()
                        },
                    )
                }))
            },
        ))
        .into_any_element()
}

/// The texts of a [`ThinkingBlock`].
#[derive(Clone, Debug)]
pub struct ThinkingBlockTexts {
    /// The live label (`Thinking`).
    pub thinking: SharedString,
    /// Before the elapsed time when done (`Thought for`).
    pub thought_for: SharedString,
}

impl Default for ThinkingBlockTexts {
    fn default() -> Self {
        Self {
            thinking: "Thinking".into(),
            thought_for: "Thought for".into(),
        }
    }
}

impl ThinkingBlockTexts {
    /// The finished label: `Thought for 23s`.
    pub fn thought(&self, seconds: Option<u64>) -> String {
        format!("{} {}", self.thought_for, fmt_elapsed_opt(seconds))
    }
}

/// [`ThinkingBlock`] geometry in px.
pub mod geometry {
    /// `.g-think { margin-left: var(--spacing-03) }`.
    pub const INDENT: f32 = 8.;
    /// `.g-think-head { gap: var(--spacing-03); padding: 0 var(--spacing-03) }`.
    pub const HEAD_GAP: f32 = 8.;
    /// `font-size: 0.8125rem` (head and body).
    pub const FONT: f32 = 13.;
    /// `.g-think-body { padding: 0 spacing-05 spacing-03 spacing-07 }`.
    pub const BODY_PADDING: (f32, f32, f32) = (16., 8., 32.);
    /// `.g-think-body { line-height: 1.5 }`.
    pub const BODY_LINE: f32 = 1.5;
}

/// The agent's reasoning, folded. See the module docs.
#[derive(IntoElement)]
pub struct ThinkingBlock {
    id: ElementId,
    seconds: Option<u64>,
    live: bool,
    default_open: bool,
    children: Vec<AnyElement>,
    texts: ThinkingBlockTexts,
}

impl ThinkingBlock {
    /// A closed, finished block.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            seconds: None,
            live: false,
            default_open: false,
            children: Vec::new(),
            texts: ThinkingBlockTexts::default(),
        }
    }

    /// `seconds`: how long it thought.
    pub fn seconds(mut self, seconds: u64) -> Self {
        self.seconds = Some(seconds);
        self
    }

    /// `live`: still thinking (the shimmer label).
    pub fn live(mut self, live: bool) -> Self {
        self.live = live;
        self
    }

    /// `defaultOpen`.
    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }

    /// The labels.
    pub fn texts(mut self, texts: ThinkingBlockTexts) -> Self {
        self.texts = texts;
        self
    }
}

impl ParentElement for ThinkingBlock {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for ThinkingBlock {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id;
        let open_state = parts::open_state((id.clone(), "open"), self.default_open, window, cx);
        let open = *open_state.read(cx);
        let (focus, ring) = parts::focus((id.clone(), "focus"), window, cx);
        let (hovered, hover) = parts::hover((id.clone(), "hover"), window, cx);
        // `.g-think-head svg { fill: currentColor }`, `:hover { color: text-primary }`.
        let ink = parts::head_ink(hovered);
        let label = if self.live {
            shimmer(
                (id.clone(), "shimmer").into(),
                self.texts.thinking,
                window,
                cx,
            )
        } else {
            div()
                .child(self.texts.thought(self.seconds))
                .into_any_element()
        };
        let head = gpui_kit::base::Button::new((id.clone(), "head"))
            .track_focus(&focus)
            .relative()
            .flex()
            .flex_row()
            .flex_none()
            .items_center()
            .justify_start()
            // `.g-think-head { display: inline-flex; height: density-row-sm }`.
            .h(density::current().row_sm())
            .px(rem(geometry::HEAD_GAP))
            .gap(rem(geometry::HEAD_GAP))
            .text_color(ink)
            .text_size(rem(geometry::FONT))
            .line_height(relative(crate::t3::craft::BODY_LINE))
            .whitespace_nowrap()
            .cursor_pointer()
            .on_hover(parts::track_hover(hover))
            .on_click({
                let open_state = open_state.clone();
                move |_, _, cx| parts::toggle(&open_state, cx)
            })
            .child(
                Icon::carbon(CarbonIcon::Idea)
                    .size_length(rem(16.))
                    .color(ink),
            )
            .child(label)
            .child(parts::chevron_down(open, 14., ink))
            .when(ring, |b| b.child(parts::ring()));
        let (pr, pb, pl) = geometry::BODY_PADDING;
        let body = open.then(|| {
            // `.g-think-body { font-size: 0.8125rem; line-height: 1.5; color: text-secondary;
            // font-style: italic; white-space: pre-wrap }`.
            div()
                .pr(rem(pr))
                .pb(rem(pb))
                .pl(rem(pl))
                .text_size(rem(geometry::FONT))
                .line_height(relative(geometry::BODY_LINE))
                .text_color(color(ColorRole::TextSecondary))
                .italic()
                .children(self.children)
        });
        // `.g-think { border-left: 1px solid border-subtle-01; margin-left: spacing-03 }`.
        div()
            .id(id)
            .flex()
            .flex_col()
            .items_start()
            .ml(rem(geometry::INDENT))
            .border_l(px(1.))
            .border_color(color(ColorRole::BorderSubtle01))
            .child(head)
            .children(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sweep_crosses_from_left_to_right() {
        // At t = 0 the peak sits half a width before the text.
        assert_eq!(shimmer_mix(0., 0.), 0.);
        // A third of the way in, the peak is on the left edge.
        assert!((shimmer_mix(0., 1. / 6.) - 1.).abs() < 1e-5);
        // Halfway, it is at u = 1.0: the left half is plain secondary.
        assert!((shimmer_mix(1., 0.5) - 1.).abs() < 1e-5);
        assert_eq!(shimmer_mix(0.2, 0.5), 0.);
        // Ramps span 0.3 of the width.
        assert!((shimmer_mix(0.85, 0.5) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn labels_follow_the_reference() {
        let t = ThinkingBlockTexts::default();
        assert_eq!(t.thinking, "Thinking");
        assert_eq!(t.thought(Some(23)), "Thought for 23s");
        assert_eq!(t.thought(Some(74)), "Thought for 1m 14s");
        assert_eq!(geometry::BODY_PADDING.2, 32.);
    }
}
