//! Anchored floating surfaces (`.cds--popover`): placement for Carbon's 12
//! alignments, the caret and hover intent with delays.
//! Shared by Tooltip, Popover and HoverCard.
//!
//! Positioning: [`gpui_kit::base::Popup`] measures the trigger and paints the
//! surface in a deferred top layer. The side is chosen like the dropdowns'
//! (`autoAlign`: the preferred side, else the opposite one, never over the
//! trigger; [`Side::choose`]) and Carbon's offset along the trigger edge
//! (`*-start`/`*-end`) is applied like the CSS `transform` ([`Placed`]).
//!
//! Motion: none. Carbon's popover CSS shows and hides at once
//! (`.cds--popover-content { display: none }` → `block` with
//! `.cds--popover--open`, no transition), so surfaces mount and unmount on
//! the frame their open state flips.
use super::overlay::{DropSide, cross_origin};
use gpui_kit::{
    Anchor, AnyElement, App, Bounds, DispatchPhase, Element, Entity, GlobalElementId, Hsla,
    InspectorElementId, IntoElement, LayoutId, MouseExitEvent, MouseMoveEvent, PathBuilder, Pixels,
    Point, Styled, Task, Window, canvas, point, px,
};
use std::{cell::Cell, rc::Rc, time::Duration};

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
        let anchor = align.side().anchor();
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

impl Side {
    /// The other side of the trigger on the same axis.
    pub const fn opposite(self) -> Self {
        match self {
            Self::Top => Self::Bottom,
            Self::Bottom => Self::Top,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }

    /// Whether the surface sits above or below (its cross axis is horizontal).
    pub const fn vertical(self) -> bool {
        matches!(self, Self::Top | Self::Bottom)
    }

    /// The Base anchor: the surface's edge facing the trigger.
    pub const fn anchor(self) -> Anchor {
        match self {
            Self::Top => Anchor::BottomCenter,
            Self::Bottom => Anchor::TopCenter,
            Self::Left => Anchor::RightCenter,
            Self::Right => Anchor::LeftCenter,
        }
    }

    /// The side for a surface `extent` px deep (its size along the main
    /// axis plus the gap the caret sits in) that prefers `self`, next to a
    /// `trigger` in a `viewport`: the preferred side when it fits, else the
    /// opposite side when it fits there, else the roomier side (Carbon
    /// `autoAlign`). This is the dropdowns' rule ([`DropSide::choose`]),
    /// mirrored for a top or left preference and turned for left/right.
    pub fn choose(
        self,
        trigger: Bounds<Pixels>,
        extent: f32,
        viewport: gpui_kit::Size<Pixels>,
    ) -> Self {
        let (top, bottom) = (f32::from(trigger.top()), f32::from(trigger.bottom()));
        let (left, right) = (f32::from(trigger.left()), f32::from(trigger.right()));
        let (vw, vh) = (f32::from(viewport.width), f32::from(viewport.height));
        let flipped = match self {
            Self::Bottom => DropSide::choose(top, bottom, extent, vh),
            Self::Top => DropSide::choose(vh - bottom, vh - top, extent, vh),
            Self::Right => DropSide::choose(left, right, extent, vw),
            Self::Left => DropSide::choose(vw - right, vw - left, extent, vw),
        } == DropSide::Above;
        if flipped { self.opposite() } else { self }
    }

    /// Where a surface `depth` px deep on this side of `trigger` starts on
    /// the main axis, `gap` px away. Never clamped into the window: a
    /// surface with room on neither side overflows the window edge instead
    /// of sliding over its own trigger.
    pub fn main_origin(self, trigger: Bounds<Pixels>, depth: f32, gap: f32) -> f32 {
        match self {
            Self::Bottom => f32::from(trigger.bottom()) + gap,
            Self::Top => f32::from(trigger.top()) - gap - depth,
            Self::Right => f32::from(trigger.right()) + gap,
            Self::Left => f32::from(trigger.left()) - gap - depth,
        }
    }

    /// The physical side in `rtl` layout: Carbon's `left`/`right`
    /// alignments are logical (`inline-start`/`inline-end`), and Base
    /// mirrors them the same way.
    fn logical(self, rtl: bool) -> Self {
        match self {
            Self::Left | Self::Right if rtl => self.opposite(),
            _ => self,
        }
    }
}

/// The caret triangle (12×6, `--popover-caret-width/height`) of a `surface`
/// placed on `side` of its trigger: it stands just outside the surface edge
/// facing the trigger and points at it, centered at `at` (fraction of the
/// surface size, px) along that edge. `clip-path: polygon(0% 100%, 50% 0%,
/// 100% 100%)` and its rotations.
pub fn caret_points(side: Side, surface: Bounds<Pixels>, at: (f32, f32)) -> [Point<Pixels>; 3] {
    let (o, s) = (surface.origin, surface.size);
    let (fraction, shift) = at;
    let (half, h) = (px(CARET_WIDTH / 2.), px(CARET_HEIGHT));
    match side {
        Side::Bottom | Side::Top => {
            let x = o.x + s.width * fraction + px(shift);
            let (edge, tip) = if side == Side::Bottom {
                (o.y, o.y - h)
            } else {
                (o.y + s.height, o.y + s.height + h)
            };
            [point(x - half, edge), point(x, tip), point(x + half, edge)]
        }
        Side::Right | Side::Left => {
            let y = o.y + s.height * fraction + px(shift);
            let (edge, tip) = if side == Side::Right {
                (o.x, o.x - h)
            } else {
                (o.x + s.width, o.x + s.width + h)
            };
            [point(edge, y - half), point(tip, y), point(edge, y + half)]
        }
    }
}

/// Position `surface` (already styled) for `align`, add the caret, and hand
/// it to Base's [`gpui_kit::base::Popup`] anchored to `trigger`.
///
/// The side is chosen like the dropdowns choose theirs ([`Side::choose`]):
/// `align`'s side when the surface fits there, else the opposite side with
/// the caret flipped to match. Base's `Popup` clamps a surface into the
/// window, which slid it up over its own trigger near the bottom edge and
/// hid the caret; [`Placed`] puts it back on the chosen side, overflowing
/// when neither side has room.
#[allow(clippy::too_many_arguments)]
pub(crate) fn anchored(
    id: impl Into<gpui_kit::ElementId>,
    trigger: AnyElement,
    surface: Option<gpui_kit::Stateful<gpui_kit::Div>>,
    align: Align,
    caret_color: Option<Hsla>,
    offset: f32,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id: gpui_kit::ElementId = id.into();
    let placement = Placement::of(align, caret_color.is_some(), offset);
    let rtl = gpui_kit::base::LayoutDirection::global(cx).is_rtl();
    let preferred = align.side().logical(rtl);
    // The side the surface was last placed on (physical), so the popup host
    // (and the hitbox it puts over the surface) starts on that side.
    let chosen = window
        .use_keyed_state((id.clone(), "side"), cx, move |_, _| {
            Rc::new(Cell::new(preferred))
        })
        .read(cx)
        .clone();
    if surface.is_none() {
        chosen.set(preferred);
    }
    let trigger_bounds = Rc::new(Cell::new(None));
    let popup = gpui_kit::base::Popup::new(id, trigger)
        // Base mirrors left/right anchors in RTL; hand it the logical one.
        .anchor(chosen.get().logical(rtl).anchor())
        .offset(px(placement.gap))
        .on_position({
            let trigger_bounds = trigger_bounds.clone();
            move |_, trigger| trigger_bounds.set(Some(trigger))
        });
    let Some(surface) = surface else {
        return popup.into_any_element();
    };
    popup
        .content(Placed {
            child: surface.into_any_element(),
            preferred,
            placement,
            caret_color,
            trigger: trigger_bounds,
            chosen,
        })
        .into_any_element()
}

/// The surface of an anchored popup, placed on the side [`Side::choose`]
/// picks (decided while prepainting, from the surface's real size, so even
/// its first frame is on the right side) and shifted along the trigger edge
/// for `*-start`/`*-end` like the CSS `transform` ([`cross_origin`]): layout
/// is the popup host's; paint, hit testing and the caret follow the offset.
struct Placed {
    child: AnyElement,
    /// `align`'s side (physical).
    preferred: Side,
    placement: Placement,
    caret_color: Option<Hsla>,
    /// The trigger's window bounds, recorded by the popup host this frame.
    trigger: Rc<Cell<Option<Bounds<Pixels>>>>,
    /// The side chosen last, for the host's anchor next frame.
    chosen: Rc<Cell<Side>>,
}

impl Element for Placed {
    type RequestLayoutState = ();
    /// The side and window bounds the surface was placed at.
    type PrepaintState = Option<(Side, Bounds<Pixels>)>;

    fn id(&self) -> Option<gpui_kit::ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        (self.child.request_layout(window, cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let Some(trigger) = self.trigger.get() else {
            self.child.prepaint(window, cx);
            return None;
        };
        let placed = place(
            self.preferred,
            self.placement,
            trigger,
            bounds.size,
            window.viewport_size(),
        );
        let side = placed.0;
        if side != self.chosen.get() {
            self.chosen.set(side);
            // The host's anchor (and its hitbox) catch up next frame; this
            // frame already paints on `side`.
            window.request_animation_frame();
        }
        window.with_element_offset(placed.1.origin - bounds.origin, |window| {
            self.child.prepaint(window, cx)
        });
        Some(placed)
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        placed: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.child.paint(window, cx);
        let (Some((side, surface)), Some(at), Some(color)) =
            (*placed, self.placement.caret_at, self.caret_color)
        else {
            return;
        };
        let pts = caret_points(side, surface, at);
        let mut path = PathBuilder::fill();
        path.move_to(pts[0]);
        path.line_to(pts[1]);
        path.line_to(pts[2]);
        path.close();
        if let Ok(path) = path.build() {
            window.paint_path(path, color);
        }
    }
}

/// The side and window bounds of a `size` surface preferring `preferred`
/// with `placement`'s gap and cross shift, next to `trigger` in `viewport`.
pub fn place(
    preferred: Side,
    placement: Placement,
    trigger: Bounds<Pixels>,
    size: gpui_kit::Size<Pixels>,
    viewport: gpui_kit::Size<Pixels>,
) -> (Side, Bounds<Pixels>) {
    let (depth, along, center, room) = if preferred.vertical() {
        (size.height, size.width, trigger.center().x, viewport.width)
    } else {
        (size.width, size.height, trigger.center().y, viewport.height)
    };
    let (depth, along) = (f32::from(depth), f32::from(along));
    let side = preferred.choose(trigger, depth + placement.gap, viewport);
    let main = side.main_origin(trigger, depth, placement.gap);
    let cross = cross_origin(
        f32::from(center),
        along,
        placement.cross_fraction,
        placement.cross_px,
        f32::from(room),
    );
    let origin = if side.vertical() {
        point(px(cross), px(main))
    } else {
        point(px(main), px(cross))
    };
    (side, Bounds::new(origin, size))
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

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Bounds<Pixels> {
        Bounds::new(point(px(x), px(y)), gpui_kit::size(px(w), px(h)))
    }

    #[test]
    fn surfaces_flip_to_the_opposite_side_instead_of_covering_the_trigger() {
        let viewport = gpui_kit::size(px(1440.), px(900.));
        // The "Dimensions" popover's trigger at the window bottom
        // (popover-covers-trigger.png): a 32px button at 768..800 and a
        // 120px `bottom-left` surface with a caret (10px gap).
        let trigger = rect(296., 768., 154., 32.);
        let start = Placement::of(Align::BottomLeft, true, POPOVER_OFFSET);
        let size = gpui_kit::size(px(192.), px(120.));
        let (side, bounds) = place(Side::Bottom, start, trigger, size, viewport);
        // No room below (92px < 130px): it stands above, the caret flips to
        // its bottom edge, and it never overlaps the trigger.
        assert_eq!(side, Side::Top);
        assert_eq!(bounds.bottom(), px(758.));
        assert!(bounds.bottom() <= trigger.top());
        // `*-start` stays: the start edge 20px before the trigger center.
        assert_eq!(bounds.left(), px(296. + 77. - 20.));
        let caret = caret_points(side, bounds, start.caret_at.unwrap());
        assert_eq!(caret[1], point(px(373.), px(764.)));
        assert_eq!((caret[0].y, caret[2].y), (px(758.), px(758.)));
        assert_eq!(caret[2].x - caret[0].x, px(CARET_WIDTH));

        // With room it stays on the preferred side, caret on the top edge.
        let high = rect(296., 300., 154., 32.);
        let (side, bounds) = place(Side::Bottom, start, high, size, viewport);
        assert_eq!((side, bounds.top()), (Side::Bottom, px(342.)));
        let caret = caret_points(side, bounds, start.caret_at.unwrap());
        assert_eq!(caret[1], point(px(373.), px(336.)));

        // A `top` tooltip at the window top flips below; exactly fitting
        // (8px window margin) stays.
        let tip = Placement::of(Align::Top, true, TOOLTIP_OFFSET);
        let small = gpui_kit::size(px(100.), px(50.));
        assert_eq!(
            place(Side::Top, tip, rect(0., 60., 32., 32.), small, viewport).0,
            Side::Bottom
        );
        assert_eq!(
            place(Side::Top, tip, rect(0., 70., 32., 32.), small, viewport).0,
            Side::Top
        );
        assert_eq!(
            place(Side::Top, tip, rect(0., 69., 32., 32.), small, viewport).0,
            Side::Bottom
        );

        // Left/right flip on the horizontal axis.
        let wide = gpui_kit::size(px(200.), px(40.));
        let edge = rect(1380., 400., 32., 32.);
        let (side, bounds) = place(Side::Right, tip, edge, wide, viewport);
        assert_eq!((side, bounds.right()), (Side::Left, px(1368.)));
        assert_eq!(
            place(Side::Left, tip, rect(20., 400., 32., 32.), wide, viewport).0,
            Side::Right
        );
    }

    #[test]
    fn a_surface_with_room_on_neither_side_overflows_instead_of_covering() {
        // 600px tall surface beside a trigger in the middle of a 900px
        // window: neither side fits; the roomier side (above) wins and the
        // surface runs past the window top rather than over the trigger.
        let viewport = gpui_kit::size(px(1440.), px(900.));
        let plain = Placement::of(Align::Bottom, false, 0.);
        let size = gpui_kit::size(px(300.), px(600.));
        let trigger = rect(600., 500., 100., 32.);
        let (side, bounds) = place(Side::Bottom, plain, trigger, size, viewport);
        assert_eq!(side, Side::Top);
        assert_eq!(bounds.bottom(), px(496.));
        assert_eq!(bounds.top(), px(-104.));
        // Equal room keeps the preferred side.
        let middle = rect(600., 434., 100., 32.);
        assert_eq!(
            place(Side::Bottom, plain, middle, size, viewport).0,
            Side::Bottom
        );
        assert_eq!(place(Side::Top, plain, middle, size, viewport).0, Side::Top);
        // The cross axis is still kept inside the window.
        let corner = rect(1420., 100., 20., 20.);
        let (_, bounds) = place(Side::Bottom, plain, corner, size, viewport);
        assert_eq!(bounds.right(), px(1432.));
    }

    #[test]
    fn sides_flip_and_anchor() {
        for side in [Side::Top, Side::Bottom, Side::Left, Side::Right] {
            assert_eq!(side.opposite().opposite(), side);
            assert_eq!(side.logical(false), side);
        }
        assert_eq!(Side::Left.logical(true), Side::Right);
        assert_eq!(Side::Top.logical(true), Side::Top);
        assert_eq!(Side::Top.anchor(), Anchor::BottomCenter);
        assert_eq!(Side::Right.anchor(), Anchor::LeftCenter);
    }

    #[test]
    fn caret_size_follows_popover_caret() {
        assert_eq!((CARET_WIDTH, CARET_HEIGHT), (12., 6.));
    }
}
