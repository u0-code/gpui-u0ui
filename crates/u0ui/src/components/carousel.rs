//! Carousel: one slide at a time, Carbon secondary icon buttons and bar
//! indicators (`.cu-carousel*` in `custom.css`, `Carousel` in `index.jsx`;
//! Carbon has no core equivalent).
//!
//! - **Behavior** is written here. Kit's `carousel` scrolls a track with spring
//!   physics, swipe and its own buttons and dots; the reference moves the track
//!   by whole slides on a fixed 240ms productive curve, with Carbon icon
//!   buttons and clickable bars, which Kit's rendering cannot be restyled into.
//!   State is an index (controlled or kept under the id); Left/Right arrows
//!   move it while focus is inside the carousel (`onKeyDown` on the section).
//! - **Layout** (the second, overriding set of `.cu-carousel*` rules):
//!   `.cu-carousel { display: flex; flex-wrap: wrap; justify-content: flex-end;
//!   gap: 1px }`; the viewport takes a whole line (`flex: 0 0 100%;
//!   overflow: hidden`) and holds the track and, under it, the dots
//!   (`justify-content: flex-start; padding: spacing-04 0`, 8px apart); the
//!   previous/next buttons wrap to the next line, at the end, pulled up by
//!   `margin-top: calc(-1 * var(--size-md) - var(--spacing-02))` so they sit
//!   beside the dots.
//! - **Appearance**: dots `.cu-carousel-dot { width: spacing-06; height: 4px;
//!   background: layer-accent-01 }`, current `interactive`, focus-visible
//!   `outline: 2px solid focus; outline-offset: 2px`; buttons are
//!   [`Button`] `secondary`, size `md`, icon-only, disabled at the ends.
//! - **Motion**: `.cu-carousel-track { transition: transform 240ms
//!   cubic-bezier(0.2,0,0.38,0.9) }` ([`SLIDE`], moderate-02 productive
//!   standard). Under reduced motion the movement becomes a fade
//!   ([`reduced_fade`]): the current slide fades out, then the new one fades
//!   in, over the same 240ms, without moving.
//! - **Vertical**: the reference stacks the track (`flex-direction: column`)
//!   and translates it by `-i × 100%` of its own height, which in a browser
//!   moves past every slide; here the viewport is one slide tall (the first
//!   slide's measured height) and the track moves by one slide per step.
//!
//! ```ignore
//! Carousel::new("plans")
//!     .content(CarouselContent::new().items(slides).dot_labels(|i| format!("Slide {}", i + 1)))
//!     .previous(CarouselPrevious::new("Previous slide"))
//!     .next(CarouselNext::new("Next slide"))
//! ```
use super::button::{Button, ButtonSize, ButtonVariant};
use super::form::{controlled, rem};
use crate::{
    icons::CarbonIcon,
    motion::{self, Curve, Motion, Speed},
    theme::color,
    tokens::{ColorRole, SIZE_MD, SPACING_02, SPACING_03, SPACING_04, SPACING_06},
};
use gpui_kit::{
    AnyElement, App, CursorStyle, ElementId, Hsla, InteractiveElement, IntoElement, KeyDownEvent,
    ParentElement, Pixels, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled,
    Window, div, prelude::FluentBuilder as _, px, relative,
};
use std::{rc::Rc, time::Instant};

/// `.cu-carousel-track { transition: transform 240ms cubic-bezier(0.2,0,0.38,0.9) }`.
pub const SLIDE: Motion = Motion::new(Curve::ProductiveStandard, Speed::Moderate02).spatial(true);
/// Under reduced motion, which slide shows and at what opacity `elapsed`
/// seconds into a change: the old slide fades out over the first half of
/// [`SLIDE`]'s 240ms, the new one fades in over the second (README: "movement
/// becomes a fade"). `None` once the change is over.
pub fn reduced_fade(elapsed: f32) -> Option<(bool, f32)> {
    let duration = SLIDE.speed.duration().as_secs_f32();
    if elapsed >= duration {
        return None;
    }
    let half = duration / 2.;
    let ease = |t: f32| SLIDE.curve.ease(t.clamp(0., 1.));
    Some(if elapsed < half {
        (true, 1. - ease(elapsed / half))
    } else {
        (false, ease((elapsed - half) / half))
    })
}

/// A slide change being faded (reduced motion).
#[derive(Clone, Copy, Default)]
struct Fade {
    /// The slide shown when no fade runs.
    shown: usize,
    /// The slide faded out, and when the change began.
    from: Option<(usize, Instant)>,
}

/// `.cu-carousel-dot { height: 4px }`.
pub const DOT_HEIGHT: f32 = 4.;
/// `.cu-carousel-dot:focus-visible { outline: 2px solid var(--focus); outline-offset: 2px }`.
pub const DOT_FOCUS: (f32, f32) = (2., 2.);
/// `.cu-carousel { gap: 1px }`.
pub const GAP: f32 = 1.;

/// `index.d.ts` `Carousel.orientation`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CarouselOrientation {
    /// Slides side by side (default).
    #[default]
    Horizontal,
    /// Slides stacked.
    Vertical,
}

impl CarouselOrientation {
    /// Both orientations.
    pub const ALL: [CarouselOrientation; 2] = [Self::Horizontal, Self::Vertical];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }
}

/// `.cu-carousel-prev, .cu-carousel-next { margin-top: calc(-1 * var(--size-md)
/// - var(--spacing-02)) }` in px: −44.
pub const fn button_margin_top() -> f32 {
    -(SIZE_MD.px_value() + SPACING_02.px_value())
}

/// The dot fill: `.cu-carousel-dot { background: layer-accent-01 }`,
/// `.cu-carousel-dot--on { background: interactive }`.
pub fn dot_color(current: bool) -> Hsla {
    color(if current {
        ColorRole::Interactive
    } else {
        ColorRole::LayerAccent01
    })
}

/// The index after a key (`ArrowRight`: `min(i + 1, n − 1)`, `ArrowLeft`:
/// `max(i − 1, 0)`), or `None` for other keys.
pub fn key_index(key: &str, index: usize, count: usize) -> Option<usize> {
    match key {
        "right" => Some((index + 1).min(count.saturating_sub(1))),
        "left" => Some(index.saturating_sub(1)),
        _ => None,
    }
}

/// Whether previous/next are disabled at `index` of `count`
/// (`disabled={c.i === 0}`, `disabled={c.i >= c.n.current - 1}`).
pub fn ends(index: usize, count: usize) -> (bool, bool) {
    (index == 0, index + 1 >= count)
}

/// One slide (`.cu-carousel-item { flex: 0 0 100%; min-width: 0 }`).
#[derive(Default)]
pub struct CarouselItem {
    children: Vec<AnyElement>,
}

impl CarouselItem {
    /// An empty slide.
    pub fn new() -> Self {
        Self::default()
    }
}

impl ParentElement for CarouselItem {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

type DotLabel = Rc<dyn Fn(usize) -> SharedString>;

/// The slides (`CarouselContent`): the track and its indicators.
#[derive(Default)]
pub struct CarouselContent {
    items: Vec<CarouselItem>,
    dot_label: Option<DotLabel>,
}

impl CarouselContent {
    /// No slides yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a slide.
    pub fn item(mut self, item: CarouselItem) -> Self {
        self.items.push(item);
        self
    }

    /// Append slides.
    pub fn items(mut self, items: impl IntoIterator<Item = CarouselItem>) -> Self {
        self.items.extend(items);
        self
    }

    /// The accessible name of indicator `i` (the reference's `Slide ${k + 1}`).
    pub fn dot_labels(mut self, label: impl Fn(usize) -> SharedString + 'static) -> Self {
        self.dot_label = Some(Rc::new(label));
        self
    }
}

/// The previous-slide button; `label` is its tooltip and accessible name.
pub struct CarouselPrevious {
    label: SharedString,
}

impl CarouselPrevious {
    /// A previous button labelled `label` (e.g. "Previous slide").
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
        }
    }
}

/// The next-slide button; `label` is its tooltip and accessible name.
pub struct CarouselNext {
    label: SharedString,
}

impl CarouselNext {
    /// A next button labelled `label` (e.g. "Next slide").
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
        }
    }
}

type IndexHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// The carousel. See the module docs.
#[derive(IntoElement)]
pub struct Carousel {
    id: ElementId,
    orientation: CarouselOrientation,
    label: Option<SharedString>,
    content: CarouselContent,
    previous: Option<CarouselPrevious>,
    next: Option<CarouselNext>,
    index: Option<usize>,
    default_index: usize,
    on_index_change: Option<IndexHandler>,
}

impl Carousel {
    /// A horizontal carousel with a stable `id`.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            orientation: CarouselOrientation::Horizontal,
            label: None,
            content: CarouselContent::new(),
            previous: None,
            next: None,
            index: None,
            default_index: 0,
            on_index_change: None,
        }
    }

    /// The orientation.
    pub fn orientation(mut self, orientation: CarouselOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// The accessible name of the region (`aria-label`).
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// The slides.
    pub fn content(mut self, content: CarouselContent) -> Self {
        self.content = content;
        self
    }

    /// The previous button.
    pub fn previous(mut self, previous: CarouselPrevious) -> Self {
        self.previous = Some(previous);
        self
    }

    /// The next button.
    pub fn next(mut self, next: CarouselNext) -> Self {
        self.next = Some(next);
        self
    }

    /// Control the current slide.
    pub fn index(mut self, index: usize) -> Self {
        self.index = Some(index);
        self
    }

    /// The first slide shown when uncontrolled (default 0).
    pub fn default_index(mut self, index: usize) -> Self {
        self.default_index = index;
        self
    }

    /// Called with the new index on every change.
    pub fn on_index_change(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_index_change = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for Carousel {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let count = self.content.items.len();
        let (index, store) = controlled(
            (id.clone(), "index"),
            self.index,
            self.default_index,
            window,
            cx,
        );
        let index = index.min(count.saturating_sub(1));
        let on_change = self.on_index_change.clone();
        let set: IndexHandler = Rc::new(move |i, window, cx| {
            store.update(cx, |v, cx| {
                *v = i;
                cx.notify();
            });
            if let Some(f) = on_change.as_ref() {
                f(i, window, cx);
            }
        });
        let vertical = self.orientation == CarouselOrientation::Vertical;
        let position = motion::animate((id.clone(), "slide"), index as f32, SLIDE, window, cx);
        // Reduced motion: the slide does not move (the position cuts), it fades.
        let fade = window.use_keyed_state((id.clone(), "fade"), cx, move |_, _| Fade {
            shown: index,
            from: None,
        });
        let reduced = motion::reduced_motion(cx);
        let (position, track_opacity) = {
            let now = Instant::now();
            let f = *fade.read(cx);
            let next = if f.shown != index {
                Fade {
                    shown: index,
                    from: reduced.then_some((f.shown, now)),
                }
            } else {
                f
            };
            if next.shown != f.shown || next.from.is_some() != f.from.is_some() {
                fade.update(cx, |v, _| *v = next);
            }
            match next.from.and_then(|(from, at)| {
                reduced_fade((now - at).as_secs_f32()).map(|step| (from, step))
            }) {
                Some((from, (show_from, opacity))) if reduced => {
                    window.request_animation_frame();
                    (if show_from { from as f32 } else { index as f32 }, opacity)
                }
                _ => (position, 1.),
            }
        };
        let slide_height =
            window.use_keyed_state((id.clone(), "slide-height"), cx, |_, _| None::<Pixels>);
        let measured = *slide_height.read(cx);

        // `.cu-carousel-track`: the slides in a row (column), moved by whole slides.
        let track = div()
            .relative()
            .opacity(track_opacity)
            .flex()
            .when(vertical, |this| this.flex_col())
            .map(|this| {
                if vertical {
                    this.top(measured.unwrap_or(px(0.)) * -position)
                } else {
                    this.left(relative(-position))
                }
            })
            .on_children_prepainted({
                let slide_height = slide_height.clone();
                move |bounds, _, cx| {
                    let h = bounds.first().map(|b| b.size.height);
                    if h.is_some() && h != *slide_height.read(cx) {
                        slide_height.update(cx, |v, cx| {
                            *v = h;
                            cx.notify();
                        });
                    }
                }
            })
            .children(self.content.items.into_iter().map(|item| {
                div()
                    .flex_none()
                    .flex()
                    .flex_col()
                    .min_w_0()
                    .when(!vertical, |this| this.w_full())
                    .children(item.children)
            }));

        let dot_label = self.content.dot_label.clone();
        let dots = div()
            .flex()
            .flex_row()
            .justify_start()
            .gap(SPACING_03)
            .py(SPACING_04)
            .children((0..count).map(|k| {
                let set = set.clone();
                let focus = window
                    .use_keyed_state((id.clone(), format!("dot-{k}")), cx, |_, cx| {
                        cx.focus_handle()
                    })
                    .read(cx)
                    .clone();
                let focus_visible =
                    focus.is_focused(window) && crate::focus::focus_visible_mode(window);
                let (width, offset) = (DOT_FOCUS.0, DOT_FOCUS.1);
                gpui_kit::base::Button::new((id.clone(), format!("dot-{k}")))
                    .track_focus(&focus)
                    .when_some(dot_label.as_ref(), |this, label| {
                        this.accessibility_label(label(k))
                    })
                    .selected(k == index)
                    .relative()
                    .flex_none()
                    .w(SPACING_06)
                    .h(rem(DOT_HEIGHT))
                    .bg(dot_color(k == index))
                    .cursor(CursorStyle::PointingHand)
                    .on_click(move |_, window, cx| set(k, window, cx))
                    .when(focus_visible, |this| {
                        this.child(
                            div()
                                .absolute()
                                .top(px(-(width + offset)))
                                .left(px(-(width + offset)))
                                .right(px(-(width + offset)))
                                .bottom(px(-(width + offset)))
                                .border(px(width))
                                .border_color(color(ColorRole::Focus)),
                        )
                    })
            }));

        // `.cu-carousel-viewport { flex: 0 0 100%; overflow: hidden }`.
        let viewport = div()
            .flex_none()
            .w_full()
            .flex()
            .flex_col()
            .child(
                div()
                    .w_full()
                    .overflow_hidden()
                    .when_some(measured.filter(|_| vertical), |this, h| this.h(h))
                    .child(track),
            )
            .child(dots);

        let (at_start, at_end) = ends(index, count);
        let nav = |icon: CarbonIcon, label: SharedString, disabled: bool, to: usize, key: &str| {
            let set = set.clone();
            // `.cu-carousel-prev, .cu-carousel-next { margin-top: −(size-md + spacing-02) }`.
            div().mt(rem(button_margin_top())).child(
                Button::new((id.clone(), key.to_string()), label)
                    .variant(ButtonVariant::Secondary)
                    .size(ButtonSize::Md)
                    .icon(icon)
                    .icon_only(true)
                    .disabled(disabled)
                    .on_click(move |_, window, cx| set(to, window, cx)),
            )
        };
        let previous = self.previous.map(|p| {
            nav(
                CarbonIcon::ChevronLeft,
                p.label,
                at_start,
                index.saturating_sub(1),
                "previous",
            )
        });
        let next = self.next.map(|n| {
            nav(
                CarbonIcon::ChevronRight,
                n.label,
                at_end,
                (index + 1).min(count.saturating_sub(1)),
                "next",
            )
        });

        let on_key = {
            let set = set.clone();
            move |event: &KeyDownEvent, window: &mut Window, cx: &mut App| {
                if let Some(next) = key_index(event.keystroke.key.as_str(), index, count) {
                    cx.stop_propagation();
                    if next != index {
                        set(next, window, cx);
                    }
                }
            }
        };

        div()
            .id(id.clone())
            .relative()
            .w_full()
            .flex()
            .flex_row()
            .flex_wrap()
            .justify_end()
            .gap(px(GAP))
            .when_some(self.label, |this, label| this.aria_label(label))
            .on_key_down(on_key)
            .child(viewport)
            .when_some(previous, |this, b| this.child(b))
            .when_some(next, |this, b| this.child(b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    #[test]
    fn dots_follow_custom_css() {
        crate::theme::tests_set(CarbonTheme::White);
        assert_eq!(dot_color(false), color(ColorRole::LayerAccent01));
        assert_eq!(dot_color(true), color(ColorRole::Interactive));
        // `.cu-carousel-dot { width: var(--spacing-06); height: 4px }`.
        assert_eq!((SPACING_06.px_value(), DOT_HEIGHT), (24., 4.));
        assert_eq!(DOT_FOCUS, (2., 2.));
        // `.cu-carousel-dots { gap: spacing-03; padding: spacing-04 0 }`.
        assert_eq!((SPACING_03.px_value(), SPACING_04.px_value()), (8., 12.));
    }

    #[test]
    fn layout_and_motion_follow_custom_css() {
        // `margin-top: calc(-1 * var(--size-md) - var(--spacing-02))`.
        assert_eq!(button_margin_top(), -44.);
        assert_eq!(GAP, 1.);
        assert_eq!(SLIDE.speed.millis(), 240);
        assert_eq!(SLIDE.curve, Curve::ProductiveStandard);
        const { assert!(SLIDE.spatial) };
    }

    #[test]
    fn keys_and_ends_clamp() {
        assert_eq!(key_index("right", 0, 3), Some(1));
        assert_eq!(key_index("right", 2, 3), Some(2));
        assert_eq!(key_index("left", 0, 3), Some(0));
        assert_eq!(key_index("left", 2, 3), Some(1));
        assert_eq!(key_index("up", 1, 3), None);
        assert_eq!(ends(0, 3), (true, false));
        assert_eq!(ends(2, 3), (false, true));
        assert_eq!(ends(1, 3), (false, false));
    }

    #[test]
    fn reduced_motion_fades_instead_of_cutting() {
        // Out, then in, over the same 240ms; no movement in between.
        assert_eq!(reduced_fade(0.), Some((true, 1.)));
        let (old, o) = reduced_fade(0.06).unwrap();
        assert!(old && o > 0. && o < 1.);
        let (old, o) = reduced_fade(0.18).unwrap();
        assert!(!old && o > 0. && o < 1.);
        assert_eq!(reduced_fade(0.24), None);
    }
}
