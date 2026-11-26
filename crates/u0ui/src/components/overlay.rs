//! Shared plumbing for floating and modal surfaces (Dialog, Sheet, Popover,
//! menus, tooltips): open state, presence (enter/exit motion that keeps a
//! closing surface mounted until it has faded), focus moves, the Carbon
//! floating shadow, and a translate wrapper.
//!
//! Behavior stays Kit's: modal hosts are [`gpui_kit::base::Dialog`] (focus
//! trap, Escape, backdrop press, deferred top layer) and anchored surfaces are
//! [`gpui_kit::base::Popup`] (trigger measurement, anchoring, window-edge
//! snapping). Base leaves open state, motion and focus moves to callers; they
//! live here.
use crate::motion::{self, Motion};
use crate::theme::color;
use crate::tokens::ColorRole;
use gpui_kit::{
    Anchor, AnyElement, App, Bounds, BoxShadow, Element, ElementId, Entity, FocusHandle,
    GlobalElementId, InspectorElementId, InteractiveElement, IntoElement, KeyDownEvent, LayoutId,
    MouseButton, ParentElement, Pixels, Point, Rems, Size, Styled, Window, canvas, div, point, px,
};
use std::{cell::Cell, rc::Rc};

/// `px` at 100% zoom as rems (Carbon's rem is 16px).
pub(crate) fn rem(px_value: f32) -> Rems {
    gpui_kit::rems(px_value / crate::tokens::REM_PX)
}

/// The one shadow floating layers carry: `box-shadow: 0 2px 6px var(--shadow)`
/// (`.cds--menu`, `.cu-panel`, `.cu-command`).
pub fn floating_shadow() -> Vec<BoxShadow> {
    vec![BoxShadow::new(px(0.), px(2.), color(ColorRole::Shadow)).blur_radius(px(6.))]
}

/// The popover's shadow, `.cds--popover--drop-shadow .cds--popover { filter:
/// drop-shadow(0 0.125rem 0.125rem rgba(0, 0, 0, 0.2)) }` (Popover, HoverCard).
/// A box shadow stands in for the filter, which GPUI lacks; the caret is
/// left without one.
pub fn popover_drop_shadow() -> Vec<BoxShadow> {
    // The CSS hard-codes black at 20%; there is no token for it.
    vec![BoxShadow::new(px(0.), px(2.), gpui_kit::hsla(0., 0., 0., 0.2)).blur_radius(px(2.))]
}

/// An open-change callback.
pub type OpenChange = Rc<dyn Fn(bool, &mut Window, &mut App)>;

/// A plain callback.
pub type Callback = Rc<dyn Fn(&mut Window, &mut App)>;

/// Resolved open state for one frame, with a setter that respects
/// controlled/uncontrolled ownership (`OpenState` in `index.d.ts`).
#[derive(Clone)]
pub(crate) struct Openness {
    /// Whether the surface is open this frame.
    pub open: bool,
    state: Entity<bool>,
    controlled: bool,
    on_change: Option<OpenChange>,
}

impl Openness {
    /// Request `open`: uncontrolled state updates itself; the callback is told
    /// either way.
    pub fn set(&self, open: bool, window: &mut Window, cx: &mut App) {
        if !self.controlled {
            self.state.update(cx, |state, cx| {
                if *state != open {
                    *state = open;
                    cx.notify();
                }
            });
        }
        if let Some(on_change) = &self.on_change {
            on_change(open, window, cx);
        }
        window.refresh();
    }

    /// A shareable setter.
    pub fn setter(&self) -> OpenChange {
        let this = self.clone();
        Rc::new(move |open, window, cx| this.set(open, window, cx))
    }
}

/// Resolve `open` (controlled) or the keyed internal state seeded by
/// `default_open` (uncontrolled).
pub(crate) fn openness(
    id: &ElementId,
    open: Option<bool>,
    default_open: bool,
    on_change: Option<OpenChange>,
    window: &mut Window,
    cx: &mut App,
) -> Openness {
    let state = window.use_keyed_state((id.clone(), "open"), cx, |_, _| default_open);
    Openness {
        open: open.unwrap_or_else(|| *state.read(cx)),
        state,
        controlled: open.is_some(),
        on_change,
    }
}

/// Presence progress, 0 (absent) → 1 (present): `enter` while opening, `exit`
/// while closing, retargeting from the current value when interrupted (Base's
/// keyed transition shortens a reversal like a CSS transition does). The motion
/// is non-spatial, so under reduced motion the fade keeps playing; callers drop
/// any translation when [`motion::reduced_motion`] is on.
///
/// The surface stays mounted while `open || progress > 0`.
pub(crate) fn presence(
    id: impl Into<ElementId>,
    open: bool,
    enter: Motion,
    exit: Motion,
    window: &mut Window,
    cx: &mut App,
) -> f32 {
    let id: ElementId = id.into();
    let motion = if open { enter } else { exit };
    motion::animate(
        (id, "presence"),
        if open { 1f32 } else { 0. },
        motion.spatial(false),
        window,
        cx,
    )
}

/// Entrance-only presence, 0 → 1 with `enter` while open; closing cuts to 0
/// on the same frame (the reference unmounts the surface at once, as
/// `.cu-panel` does). Reopening animates the entrance again from 0.
pub(crate) fn entrance(
    id: impl Into<ElementId>,
    open: bool,
    enter: Motion,
    window: &mut Window,
    cx: &mut App,
) -> f32 {
    let id: ElementId = id.into();
    if open {
        // Non-spatial: callers drop any slide under reduced motion.
        motion::animate((id, "presence"), 1f32, enter.spatial(false), window, cx)
    } else {
        // A zero-length transition finishes at once and leaves the keyed
        // state at 0, so the next open starts from there.
        gpui_kit::base::transition(
            (id, "presence"),
            0f32,
            gpui_kit::base::Transition::new(std::time::Duration::ZERO),
            window,
            cx,
        )
    }
}

/// Focus that moves into a modal or floating surface when it opens and returns
/// to the previously focused element when it closes. Base's hosts trap focus
/// but leave both moves to the application.
struct SurfaceFocus {
    focus: FocusHandle,
    previous: Option<FocusHandle>,
    shown: bool,
}

/// The focus handle of the surface keyed by `id`, after applying this frame's
/// open/close focus moves. `initial` (if any) receives focus on open instead of
/// the surface itself (Carbon focuses a danger modal's secondary button).
pub(crate) fn surface_focus(
    id: &ElementId,
    open: bool,
    initial: Option<FocusHandle>,
    window: &mut Window,
    cx: &mut App,
) -> FocusHandle {
    let state = window.use_keyed_state((id.clone(), "surface-focus"), cx, |_, cx| SurfaceFocus {
        focus: cx.focus_handle(),
        previous: None,
        shown: false,
    });
    let shown = state.read(cx).shown;
    if open && !shown {
        let previous = window.focused(cx);
        state.update(cx, |state, _| {
            state.shown = true;
            state.previous = previous;
        });
        let target = initial.unwrap_or_else(|| state.read(cx).focus.clone());
        // The host attaches its focus handle while painting this frame.
        window.defer(cx, move |window, cx| target.focus(window, cx));
    } else if !open && shown {
        let previous = state.update(cx, |state, _| {
            state.shown = false;
            state.previous.take()
        });
        let surface = state.read(cx).focus.clone();
        // An action that closed the surface may also have moved focus on
        // purpose (a palette entry that navigates focuses the new page): keep
        // that instead of pulling focus back to the old trigger. The last
        // frame still holds the surface, so "inside it" is answerable here.
        let now = window.focused(cx);
        if moved_elsewhere(now.as_ref(), previous.as_ref(), || {
            surface.contains_focused(window, cx)
        }) {
            return state.read(cx).focus.clone();
        }
        window.defer(cx, move |window, cx| match previous {
            Some(previous) => previous.focus(window, cx),
            // Nothing had focus before: do not leave it on a surface that is
            // going away, where keys would hit its (closed) dialog context.
            None if surface.contains_focused(window, cx) => window.blur(cx),
            None => {}
        });
    }
    state.read(cx).focus.clone()
}

/// Whether focus left a closing surface for somewhere other than the element
/// it would return to: `now` is set, is not `previous`, and is not inside the
/// surface (`inside`, asked only when needed).
fn moved_elsewhere(
    now: Option<&FocusHandle>,
    previous: Option<&FocusHandle>,
    inside: impl FnOnce() -> bool,
) -> bool {
    match now {
        Some(now) => Some(now) != previous && !inside(),
        None => false,
    }
}

/// Wrap a caller's trigger element so pressing it (pointer) or activating it
/// from the keyboard (Enter/Space while focus is inside) runs `activate`.
/// GPUI cannot inject a click handler into an arbitrary element, so this is
/// Base's `DialogTrigger` wrapper plus the keyboard path.
pub(crate) fn trigger_wrapper(
    id: impl Into<ElementId>,
    trigger: AnyElement,
    activate: Callback,
) -> gpui_kit::Stateful<gpui_kit::Div> {
    let on_key = activate.clone();
    div()
        .id(id)
        .flex_shrink_0()
        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
            activate(window, cx);
        })
        .on_key_down(move |event: &KeyDownEvent, window, cx| {
            if matches!(event.keystroke.key.as_str(), "enter" | "space") && !event.is_held {
                on_key(window, cx);
                cx.stop_propagation();
            }
        })
        .child(trigger)
}

/// The gap Base's `Popup` keeps from the window edges (`WINDOW_MARGIN`).
pub const WINDOW_MARGIN: f32 = 8.;

/// Which side of its trigger a dropdown surface (list-box menu, date-picker
/// calendar, dropdown menu) opens on.
///
/// Carbon's list boxes and menus open below the field (`direction="bottom"`,
/// the menu attached at the field's bottom edge); with `autoAlign` they flip
/// above when there is no room below but there is above. Base's `Popup` only
/// clamps a surface into the window, which slides it up over its own trigger
/// near the bottom edge; choosing the side first keeps the trigger visible.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DropSide {
    /// Hanging from the trigger's bottom edge.
    #[default]
    Below,
    /// Standing on the trigger's top edge.
    Above,
}

impl DropSide {
    /// The side for a surface `height` px tall under a trigger spanning
    /// `top..bottom` in a viewport `viewport` px tall: below when it fits,
    /// else above when it fits there, else the roomier side.
    pub fn choose(top: f32, bottom: f32, height: f32, viewport: f32) -> Self {
        let below = Self::Below.room(top, bottom, viewport);
        let above = Self::Above.room(top, bottom, viewport);
        if height <= below || (height > above && below >= above) {
            Self::Below
        } else {
            Self::Above
        }
    }

    /// The height available on this side, keeping [`WINDOW_MARGIN`] from the
    /// window edge.
    pub fn room(self, top: f32, bottom: f32, viewport: f32) -> f32 {
        match self {
            Self::Below => viewport - WINDOW_MARGIN - bottom,
            Self::Above => top - WINDOW_MARGIN,
        }
        .max(0.)
    }

    /// The Base anchor for a surface hanging from the trigger's start edge.
    pub fn anchor(self) -> Anchor {
        match self {
            Self::Below => Anchor::TopLeft,
            Self::Above => Anchor::BottomLeft,
        }
    }
}

/// Per-instance dropdown placement state: the trigger's window bounds and
/// the surface's size, both recorded while painting, and the side the
/// surface was last placed on.
#[derive(Clone, Default)]
pub(crate) struct DropAnchor {
    trigger: Rc<Cell<Bounds<Pixels>>>,
    surface: Rc<Cell<Size<Pixels>>>,
    side: Rc<Cell<DropSide>>,
    /// See [`Self::set_top_inset`].
    top_inset: Rc<Cell<f32>>,
}

/// The [`DropAnchor`] keyed by `id`.
pub(crate) fn drop_anchor(
    id: impl Into<ElementId>,
    window: &mut Window,
    cx: &mut App,
) -> DropAnchor {
    window
        .use_keyed_state(id.into(), cx, |_, _| DropAnchor::default())
        .read(cx)
        .clone()
}

impl DropAnchor {
    /// The trigger's last measured width in px.
    pub fn width(&self) -> f32 {
        f32::from(self.trigger.get().size.width)
    }

    /// The trigger's last measured height in px.
    pub fn height(&self) -> f32 {
        f32::from(self.trigger.get().size.height)
    }

    /// How far below the measured trigger's top edge a surface placed
    /// [`DropSide::Above`] stands (px): the measured element can hold more
    /// than the control the surface belongs to, such as a field's label over
    /// its input, and the room above is counted from that control's top.
    pub fn set_top_inset(&self, inset: f32) {
        self.top_inset.set(inset.max(0.));
    }

    /// The trigger's last measured `(top, bottom)`, with the top inset.
    fn edges(trigger: Bounds<Pixels>, inset: f32) -> (f32, f32) {
        let bottom = f32::from(trigger.bottom());
        ((f32::from(trigger.top()) + inset).min(bottom), bottom)
    }

    /// A zero-size canvas that records the bounds of its (`relative`) parent,
    /// the trigger.
    pub fn measure_trigger(&self) -> impl IntoElement + use<> {
        let trigger = self.trigger.clone();
        canvas(move |bounds, _, _| trigger.set(bounds), |_, _, _, _| {})
            .absolute()
            .top_0()
            .left_0()
            .size_full()
    }

    /// The side for a surface `height` px tall and the room there, from the
    /// trigger's last bounds (the side is remembered for
    /// [`Self::measure_surface`]).
    pub fn place(&self, height: f32, window: &Window) -> (DropSide, f32) {
        let (top, bottom) = Self::edges(self.trigger.get(), self.top_inset.get());
        let viewport = f32::from(window.viewport_size().height);
        let side = DropSide::choose(top, bottom, height, viewport);
        self.side.set(side);
        (side, side.room(top, bottom, viewport))
    }

    /// The side for the surface's last measured size (below until it has
    /// been measured once).
    pub fn place_measured(&self, window: &Window) -> DropSide {
        self.place(f32::from(self.surface.get().height), window).0
    }

    /// A canvas filling its (`relative`) parent, the surface, that records
    /// the surface's size and asks for another frame when the side chosen
    /// for it no longer fits (its first open, or its size changed).
    pub fn measure_surface(&self) -> impl IntoElement + use<> {
        let (trigger, surface, side, inset) = (
            self.trigger.clone(),
            self.surface.clone(),
            self.side.clone(),
            self.top_inset.clone(),
        );
        canvas(
            move |bounds, window, _| {
                surface.set(bounds.size);
                let (top, bottom) = Self::edges(trigger.get(), inset.get());
                let want = DropSide::choose(
                    top,
                    bottom,
                    f32::from(bounds.size.height),
                    f32::from(window.viewport_size().height),
                );
                if want != side.get() {
                    window.request_animation_frame();
                }
            },
            |_, _, _, _| {},
        )
        .absolute()
        .top_0()
        .left_0()
        .size_full()
    }
}

/// Translates its child by a fraction of the child's own size plus a fixed
/// offset, like CSS `transform: translate(…)`: layout is unchanged; paint, hit
/// testing and accessibility follow the offset (`window.with_element_offset`,
/// as Kit's dialog entrance surface does).
pub(crate) struct Translate {
    child: AnyElement,
    fraction: Point<f32>,
    offset: Point<Pixels>,
    /// See [`Translate::along_trigger`].
    cross: Option<(bool, Rc<Cell<Option<f32>>>)>,
}

/// Where a surface centered on `center` along one axis starts once shifted by
/// `fraction` of its `size` plus `px` (a Carbon `*-start`/`*-end` alignment),
/// kept [`WINDOW_MARGIN`] inside a `viewport` that long.
pub fn cross_origin(center: f32, size: f32, fraction: f32, px: f32, viewport: f32) -> f32 {
    let desired = center - size / 2. + fraction * size + px;
    desired
        .min(viewport - WINDOW_MARGIN - size)
        .max(WINDOW_MARGIN)
}

impl Translate {
    /// `translate(fraction.x * 100%, fraction.y * 100%)` + `offset`.
    pub fn new(child: impl IntoElement, fraction: Point<f32>, offset: Point<Pixels>) -> Self {
        Self {
            child: child.into_any_element(),
            fraction,
            offset,
            cross: None,
        }
    }

    /// Place the child along one axis (`horizontal`) from the trigger's
    /// `center` on it (recorded by the popup before this prepaints) rather
    /// than from where the popup host put it. The host clamps the surface
    /// into the window *before* the translation, so a shifted `*-end`
    /// surface near the window edge was moved twice and its caret missed the
    /// trigger; here the shift is applied first and the clamp after
    /// ([`cross_origin`]).
    pub fn along_trigger(mut self, horizontal: bool, center: Rc<Cell<Option<f32>>>) -> Self {
        self.cross = Some((horizontal, center));
        self
    }

    /// `translateY(y)` in px.
    pub fn y(child: impl IntoElement, y: f32) -> Self {
        Self::new(child, point(0., 0.), point(px(0.), px(y)))
    }
}

impl Element for Translate {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
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
    ) {
        let mut offset = point(
            bounds.size.width * self.fraction.x + self.offset.x,
            bounds.size.height * self.fraction.y + self.offset.y,
        );
        if let Some((horizontal, center)) = &self.cross
            && let Some(center) = center.get()
        {
            let viewport = window.viewport_size();
            if *horizontal {
                let origin = cross_origin(
                    center,
                    f32::from(bounds.size.width),
                    self.fraction.x,
                    f32::from(self.offset.x),
                    f32::from(viewport.width),
                );
                offset.x = px(origin) - bounds.origin.x;
            } else {
                let origin = cross_origin(
                    center,
                    f32::from(bounds.size.height),
                    self.fraction.y,
                    f32::from(self.offset.y),
                    f32::from(viewport.height),
                );
                offset.y = px(origin) - bounds.origin.y;
            }
        }
        window.with_element_offset(offset, |window| self.child.prepaint(window, cx));
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        self.child.paint(window, cx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floating_shadow_is_carbons() {
        crate::theme::tests_set(crate::tokens::CarbonTheme::White);
        let shadow = &floating_shadow()[0];
        // `.cds--menu { box-shadow: 0 2px 6px var(--shadow) }`.
        assert_eq!(
            (shadow.offset.x, shadow.offset.y, shadow.blur_radius),
            (px(0.), px(2.), px(6.))
        );
        assert!(!shadow.inset);
        let drop = &popover_drop_shadow()[0];
        assert_eq!(
            (drop.offset.y, drop.blur_radius, drop.color.a),
            (px(2.), px(2.), 0.2)
        );
    }

    #[test]
    fn end_aligned_surfaces_shift_before_clamping() {
        // The header's Theme button centered at 1390 in a 1440 window: a
        // 336px `bottom-right` popover ends 20px past the center (caret over
        // the button) until that would cross the window margin.
        assert_eq!(cross_origin(1390., 336., -0.5, 20., 1440.), 1074.);
        assert_eq!(cross_origin(1430., 336., -0.5, 20., 1440.), 1096.);
        assert_eq!(cross_origin(1300., 336., -0.5, 20., 1440.), 984.);
        // Centered surfaces are only clamped.
        assert_eq!(cross_origin(100., 336., 0., 0., 1440.), 8.);
        assert_eq!(cross_origin(720., 336., 0., 0., 1440.), 552.);
        // `*-start`: starts 20px before the center.
        assert_eq!(cross_origin(400., 200., 0.5, -20., 1440.), 380.);
    }

    #[test]
    fn drop_side_opens_below_and_flips_above_without_covering_the_trigger() {
        // A 32px field at 400..432 in a 900px window: 160px fits below.
        assert_eq!(DropSide::choose(400., 432., 160., 900.), DropSide::Below);
        assert_eq!(DropSide::Below.room(400., 432., 900.), 460.);
        // Near the bottom (the Tags combobox at 804..836): flip above.
        assert_eq!(DropSide::choose(804., 836., 160., 900.), DropSide::Above);
        assert_eq!(DropSide::Above.room(804., 836., 900.), 796.);
        // Exactly fitting below stays below (8px window margin).
        assert_eq!(DropSide::choose(700., 732., 160., 900.), DropSide::Below);
        assert_eq!(DropSide::choose(700., 733., 160., 900.), DropSide::Above);
        // Fits on neither side: the roomier one; the caller caps the height to its room.
        assert_eq!(DropSide::choose(100., 132., 800., 900.), DropSide::Below);
        assert_eq!(DropSide::choose(700., 732., 800., 900.), DropSide::Above);
        assert_eq!(DropSide::Below.room(100., 900., 900.), 0.);
        assert_eq!(DropSide::Below.anchor(), Anchor::TopLeft);
        assert_eq!(DropSide::Above.anchor(), Anchor::BottomLeft);
    }
}
