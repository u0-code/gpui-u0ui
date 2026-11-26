//! Anchored floating surfaces (`.cds--popover`): placement for Carbon's 12
//! alignments, the caret and hover intent with delays.
//! Shared by Tooltip, Popover and HoverCard.
//!
//! Positioning is Kit's: [`gpui_kit::base::Popup`] measures the trigger,
//! places the surface on a side of it, snaps it inside the window and paints
//! it in a deferred top layer. Carbon then offsets the surface along the
//! trigger edge (`*-start`/`*-end`), which is a [`Translate`] here, like the
//! CSS `transform`.
//!
//! Motion: none. Carbon's popover CSS shows and hides at once
//! (`.cds--popover-content { display: none }` → `block` with
//! `.cds--popover--open`, no transition), so surfaces mount and unmount on
//! the frame their open state flips.
use super::overlay::{Translate, rem};
use gpui_kit::{
    Anchor, AnyElement, App, Bounds, DispatchPhase, Entity, Hsla, IntoElement, MouseExitEvent,
    MouseMoveEvent, ParentElement, PathBuilder, Pixels, Styled, Task, Window, canvas, div, point,
    px,
};
use std::time::Duration;

/// `--popover-caret-width` 0.75rem.
pub const CARET_WIDTH: f32 = 12.;
/// `--popover-caret-height` 0.375rem.
pub const CARET_HEIGHT: f32 = 6.;
/// `.cds--popover--caret { --popover-offset: 0.625rem }`.
pub const POPOVER_OFFSET: f32 = 10.;
/// `.cds--tooltip { --popover-offset: 12px }`.
pub const TOOLTIP_OFFSET: f32 = 12.;
/// Without a caret: `inset-block-end: -0.25rem` (a 4px gap).
pub const NO_CARET_GAP: f32 = 4.;
/// Without a caret, `*-start/-end` hang `1rem` past the trigger center
/// (`inset-inline-start: calc(50% - var(--popover-offset) - 1rem)`).
pub const NO_CARET_SHIFT: f32 = 16.;

/// Carbon's popover alignments (`index.d.ts` `Align`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Align {
    /// `top`.
    Top,
    /// `top-left` (`top-start`).
    TopLeft,
    /// `top-right` (`top-end`).
    TopRight,
    /// `bottom`.
    #[default]
    Bottom,
    /// `bottom-left` (`bottom-start`).
    BottomLeft,
    /// `bottom-right` (`bottom-end`).
    BottomRight,
    /// `left`.
    Left,
    /// `left-top`.
    LeftTop,
    /// `left-bottom`.
    LeftBottom,
    /// `right`.
    Right,
    /// `right-top`.
    RightTop,
    /// `right-bottom`.
    RightBottom,
}

/// The side of the trigger a surface sits on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    /// Above.
    Top,
    /// Below.
    Bottom,
    /// To the left.
    Left,
    /// To the right.
    Right,
}

impl Align {
    /// Every alignment, in `index.d.ts` order.
    pub const ALL: [Align; 12] = [
        Self::Top,
        Self::TopLeft,
        Self::TopRight,
        Self::Bottom,
        Self::BottomLeft,
        Self::BottomRight,
        Self::Left,
        Self::LeftTop,
        Self::LeftBottom,
        Self::Right,
        Self::RightTop,
        Self::RightBottom,
    ];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::TopLeft => "top-left",
            Self::TopRight => "top-right",
            Self::Bottom => "bottom",
            Self::BottomLeft => "bottom-left",
            Self::BottomRight => "bottom-right",
            Self::Left => "left",
            Self::LeftTop => "left-top",
            Self::LeftBottom => "left-bottom",
            Self::Right => "right",
            Self::RightTop => "right-top",
            Self::RightBottom => "right-bottom",
        }
    }

    /// Which side of the trigger the surface is on.
    pub const fn side(self) -> Side {
        match self {
            Self::Top | Self::TopLeft | Self::TopRight => Side::Top,
            Self::Bottom | Self::BottomLeft | Self::BottomRight => Side::Bottom,
            Self::Left | Self::LeftTop | Self::LeftBottom => Side::Left,
            Self::Right | Self::RightTop | Self::RightBottom => Side::Right,
        }
    }

    /// -1 start, 0 centered, +1 end along the trigger edge.
    pub const fn cross(self) -> i8 {
        match self {
            Self::TopLeft | Self::BottomLeft | Self::LeftTop | Self::RightTop => -1,
            Self::TopRight | Self::BottomRight | Self::LeftBottom | Self::RightBottom => 1,
            _ => 0,
        }
    }
}

/// Where a surface goes, in px, for an alignment with or without a caret.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placement {
    /// The Base anchor: the surface's corner placed at the trigger's
    /// opposite edge center.
    pub anchor: Anchor,
    /// Gap between trigger and surface (`--popover-offset`, or 4px).
    pub gap: f32,
    /// Shift along the trigger edge: a fraction of the surface's own size
    /// plus px. A `*-start` surface starts `shift` before the trigger center.
    pub cross_fraction: f32,
    /// The px part of the cross shift.
    pub cross_px: f32,
    /// The caret's center along the surface edge, from the surface start, as
    /// (fraction of the surface size, px), if there is a caret.
    pub caret_at: Option<(f32, f32)>,
}

impl Placement {
    /// Carbon's geometry (`.cds--popover--<align>` rules). `offset` is
    /// `--popover-offset` when `caret` is on.
    pub fn of(align: Align, caret: bool, offset: f32) -> Self {
        let anchor = match align.side() {
            Side::Top => Anchor::BottomCenter,
            Side::Bottom => Anchor::TopCenter,
            Side::Left => Anchor::RightCenter,
            Side::Right => Anchor::LeftCenter,
        };
        // Start edge at `center - 2 × offset` with a caret (`calc(50% -
        // offset)` then `translate(-offset)`), `center - 1rem` without.
        let shift = if caret { 2. * offset } else { NO_CARET_SHIFT };
        let (cross_fraction, cross_px) = match align.cross() {
            -1 => (0.5, -shift),
            1 => (-0.5, shift),
            _ => (0., 0.),
        };
        let caret_at = caret.then(|| match align.cross() {
            -1 => (0., shift),
            1 => (1., -shift),
            _ => (0.5, 0.),
        });
        Self {
            anchor,
            gap: if caret { offset } else { NO_CARET_GAP },
            cross_fraction,
            cross_px,
            caret_at,
        }
    }
}

/// A Carbon caret: a 12×6 triangle pointing at the trigger, painted in the
/// surface color just outside the surface edge on `side`.
pub(crate) fn caret(side: Side, at: (f32, f32), color: Hsla) -> AnyElement {
    let (w, h) = match side {
        Side::Top | Side::Bottom => (CARET_WIDTH, CARET_HEIGHT),
        Side::Left | Side::Right => (CARET_HEIGHT, CARET_WIDTH),
    };
    let base = div().absolute().w(rem(w)).h(rem(h));
    let (fraction, offset) = at;
    let placed = match side {
        // The surface is below the trigger: the caret sits on its top edge.
        Side::Bottom => base
            .top(rem(-h))
            .left(gpui_kit::relative(fraction))
            .ml(rem(offset - w / 2.)),
        Side::Top => base
            .bottom(rem(-h))
            .left(gpui_kit::relative(fraction))
            .ml(rem(offset - w / 2.)),
        Side::Right => base
            .left(rem(-w))
            .top(gpui_kit::relative(fraction))
            .mt(rem(offset - h / 2.)),
        Side::Left => base
            .right(rem(-w))
            .top(gpui_kit::relative(fraction))
            .mt(rem(offset - h / 2.)),
    };
    placed
        .child(
            canvas(
                |_, _, _| {},
                move |bounds: Bounds<Pixels>, _, window, _| {
                    let (o, s) = (bounds.origin, bounds.size);
                    // `clip-path: polygon(0% 100%, 50% 0%, 100% 100%)` and rotations.
                    let pts = match side {
                        Side::Bottom => [
                            point(o.x, o.y + s.height),
                            point(o.x + s.width / 2., o.y),
                            point(o.x + s.width, o.y + s.height),
                        ],
                        Side::Top => [
                            point(o.x, o.y),
                            point(o.x + s.width / 2., o.y + s.height),
                            point(o.x + s.width, o.y),
                        ],
                        Side::Right => [
                            point(o.x + s.width, o.y),
                            point(o.x, o.y + s.height / 2.),
                            point(o.x + s.width, o.y + s.height),
                        ],
                        Side::Left => [
                            point(o.x, o.y),
                            point(o.x + s.width, o.y + s.height / 2.),
                            point(o.x, o.y + s.height),
                        ],
                    };
                    let mut path = PathBuilder::fill();
                    path.move_to(pts[0]);
                    path.line_to(pts[1]);
                    path.line_to(pts[2]);
                    path.close();
                    if let Ok(path) = path.build() {
                        window.paint_path(path, color);
                    }
                },
            )
            .size_full(),
        )
        .into_any_element()
}

/// Position `surface` (already styled) for `align`, add the caret, and hand
/// it to Base's [`gpui_kit::base::Popup`] anchored to `trigger`.
pub(crate) fn anchored(
    id: impl Into<gpui_kit::ElementId>,
    trigger: AnyElement,
    surface: Option<gpui_kit::Stateful<gpui_kit::Div>>,
    align: Align,
    caret_color: Option<Hsla>,
    offset: f32,
) -> AnyElement {
    let placement = Placement::of(align, caret_color.is_some(), offset);
    let popup = gpui_kit::base::Popup::new(id, trigger)
        .anchor(placement.anchor)
        .offset(px(placement.gap));
    let Some(surface) = surface else {
        return popup.into_any_element();
    };
    let side = align.side();
    let surface = surface
        .relative()
        .when_caret(placement.caret_at.zip(caret_color), side);
    let (fraction, shift) = match side {
        Side::Top | Side::Bottom => (
            point(placement.cross_fraction, 0.),
            point(px(placement.cross_px), px(0.)),
        ),
        Side::Left | Side::Right => (
            point(0., placement.cross_fraction),
            point(px(0.), px(placement.cross_px)),
        ),
    };
    popup
        .content(Translate::new(surface, fraction, shift))
        .into_any_element()
}

trait WhenCaret {
    fn when_caret(self, caret: Option<((f32, f32), Hsla)>, side: Side) -> Self;
}

impl WhenCaret for gpui_kit::Stateful<gpui_kit::Div> {
    fn when_caret(self, caret: Option<((f32, f32), Hsla)>, side: Side) -> Self {
        match caret {
            Some((at, color)) => self.child(self::caret(side, at, color)),
            None => self,
        }
    }
}

/// Hover/focus intent with open and close delays (Carbon Tooltip
/// `enterDelayMs`/`leaveDelayMs`, the HoverCard's 300ms/150ms).
pub(crate) struct HoverIntent {
    /// Open now.
    pub open: bool,
    /// The pointer left the window and has not moved inside it since.
    outside: bool,
    /// The pointer is on the trigger (or the surface).
    pointer_in: bool,
    /// Where the state returns to once the pointer leaves.
    rest: bool,
    generation: u64,
    _task: Option<Task<()>>,
}

/// The keyed hover intent for `id`, initially `open`; leaving returns it to
/// `rest` (HoverCard: `setTimeout(() => set(defaultOpen), 150)`).
pub(crate) fn hover_intent(
    id: impl Into<gpui_kit::ElementId>,
    open: bool,
    rest: bool,
    window: &mut Window,
    cx: &mut App,
) -> Entity<HoverIntent> {
    window.use_keyed_state(id, cx, move |_, _| HoverIntent {
        open,
        outside: false,
        pointer_in: false,
        rest,
        generation: 0,
        _task: None,
    })
}

/// Ask `intent` to become `open` after `delay` (cancelling any pending
/// change; a zero delay applies at once). A request to open while the
/// pointer is outside the window is ignored (see [`window_exit_guard`]).
pub(crate) fn schedule(intent: &Entity<HoverIntent>, open: bool, delay: Duration, cx: &mut App) {
    intent.update(cx, |state, cx| {
        if open && state.outside {
            return;
        }
        state.generation += 1;
        let generation = state.generation;
        if delay.is_zero() || state.open == open {
            state._task = None;
            if state.open != open {
                state.open = open;
                cx.notify();
            }
            return;
        }
        state._task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(delay).await;
            let _ = this.update(cx, |state, cx| {
                if state.generation == generation {
                    state.open = open;
                    cx.notify();
                }
            });
        }));
    });
}

/// The pointer entered (`inside`) or left the trigger: open after `delay`,
/// or return to the rest state after `delay`.
pub(crate) fn pointer(intent: &Entity<HoverIntent>, inside: bool, delay: Duration, cx: &mut App) {
    let target = {
        let state = intent.read(cx);
        if inside { true } else { state.rest }
    };
    intent.update(cx, |state, _| state.pointer_in = inside);
    schedule(intent, target, delay, cx);
}

/// Returns `intent` to rest `delay` after the pointer leaves the window from
/// the trigger, and keeps it
/// from reopening until the pointer moves inside again.
///
/// GPUI reports the exit (`MouseExitEvent`) but keeps the last in-window
/// pointer position, so the next frame's hit test still finds the trigger
/// hovered and `on_hover(true)` fires again: a hover surface left the window
/// open (and its trigger hovered) when the pointer went straight out.
pub(crate) fn window_exit_guard(intent: Entity<HoverIntent>, delay: Duration) -> impl IntoElement {
    canvas(
        |_, _, _| {},
        move |_, _, window, _| {
            let exit = intent.clone();
            window.on_mouse_event(move |_: &MouseExitEvent, phase, _, cx| {
                if phase == DispatchPhase::Bubble {
                    let was_in = exit.read(cx).pointer_in;
                    exit.update(cx, |state, _| state.outside = true);
                    if was_in {
                        pointer(&exit, false, delay, cx);
                    }
                }
            });
            let moved = intent.clone();
            window.on_mouse_event(move |_: &MouseMoveEvent, phase, _, cx| {
                if phase == DispatchPhase::Capture && moved.read(cx).outside {
                    moved.update(cx, |state, _| state.outside = false);
                }
            });
        },
    )
    .absolute()
    .size_0()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alignments_follow_popover_rules() {
        assert_eq!(Align::ALL.len(), 12);
        assert_eq!(Align::BottomLeft.name(), "bottom-left");
        // `.cds--popover--bottom`: centered, `--popover-offset` gap, caret centered.
        let bottom = Placement::of(Align::Bottom, true, POPOVER_OFFSET);
        assert_eq!(bottom.anchor, Anchor::TopCenter);
        assert_eq!((bottom.gap, bottom.cross_fraction), (10., 0.));
        assert_eq!(bottom.caret_at, Some((0.5, 0.)));
        // `.cds--popover--bottom-start`: starts 2 × offset before the trigger center,
        // the caret stays on the trigger center.
        let start = Placement::of(Align::BottomLeft, true, POPOVER_OFFSET);
        assert_eq!((start.cross_fraction, start.cross_px), (0.5, -20.));
        assert_eq!(start.caret_at, Some((0., 20.)));
        // No caret: 4px gap, start 1rem before the center.
        let plain = Placement::of(Align::TopRight, false, 0.);
        assert_eq!(plain.anchor, Anchor::BottomCenter);
        assert_eq!((plain.gap, plain.cross_px, plain.caret_at), (4., 16., None));
        // Tooltip offset.
        assert_eq!(Placement::of(Align::Top, true, TOOLTIP_OFFSET).gap, 12.);
        assert_eq!(
            Placement::of(Align::RightTop, true, 10.).anchor,
            Anchor::LeftCenter
        );
    }

    #[test]
    fn caret_size_follows_popover_caret() {
        assert_eq!((CARET_WIDTH, CARET_HEIGHT), (12., 6.));
    }
}
