//! ScrollArea: a focusable scroll container with Carbon's thin scrollbar
//! (`.cu-scroll` in `custom.css`; Carbon has no core equivalent).
//!
//! - **Behavior** is Kit's: the scrollbar is Base's [`Scrollbar`] (thumb drag,
//!   track click, wheel, hit testing) over a GPUI `overflow_scroll` viewport
//!   tracked by a [`ScrollHandle`]. Kit's `Scrollable` wrapper is not used
//!   because it takes the scrollbar mode and styles from the global theme only;
//!   here every scrollbar carries its own Carbon styles and is always shown,
//!   like a styled `::-webkit-scrollbar`. The container is a tab stop
//!   (`tabIndex={0}` in `index.jsx`) and scrolls from the keyboard as a focused
//!   browser scroller does ([`keyboard_scroll`]); GPUI has no native keyboard
//!   scrolling, so that part is written here. Two more pieces are written here
//!   because Base lacks them: a press on the track pages one viewport toward
//!   the pointer, as macOS does by default ([`track_page`]; Base jumps the
//!   thumb there), and Shift turns a vertical wheel into a horizontal scroll
//!   ([`shift_wheel`]), as browsers do.
//! - **Horizontal axis**: the content takes its own width (at least the
//!   viewport's), so wide content overflows sideways and gets the same bar,
//!   wheel/trackpad, Shift+wheel, drag and arrow-key scrolling as the vertical
//!   axis. [`ScrollArea::restrict_wheel_to_axis`] keeps a vertical wheel from
//!   scrolling a horizontal-only area sideways (for a code block, where it
//!   should scroll the page).
//! - **Appearance**: `.cu-scroll::-webkit-scrollbar { width: 8px; height: 8px }`,
//!   `::-webkit-scrollbar-thumb { background: var(--border-strong-01) }` (square,
//!   no hover change), `::-webkit-scrollbar-track { background: transparent }`.
//!   A classic scrollbar takes layout space, so the content is inset by 8px on
//!   an axis that overflows. Keyboard focus draws `.cu-scroll:focus-visible
//!   { outline: 2px solid var(--focus); outline-offset: -2px }` (pointer focus
//!   draws nothing, as `:focus-visible`).
//! - **Motion**: none (`.cu-scroll` declares no transition; keyboard steps cut
//!   to the new offset).
//!
//! [`ScrollBar`] exists only so shadcn markup ports one to one: it renders
//! nothing. Other components that own their own scroller (a chat transcript, a
//! virtual list) can paint the same bar with [`carbon_scrollbar`], or wrap
//! their content in a [`ScrollArea`] with a caller-owned handle
//! ([`ScrollArea::track_scroll`]) and [`ScrollArea::focusable`] off.
//!
//! ```ignore
//! ScrollArea::new("releases")
//!     .h(px(180.))
//!     .w(px(280.))
//!     .child(div().p(tokens::SPACING_05).children(rows))
//! ```
use crate::{theme::color, tokens::ColorRole};
use gpui_kit::{
    AnyElement, App, Bounds, DispatchPhase, ElementId, FocusHandle, Hitbox, HitboxBehavior,
    InteractiveElement, IntoElement, KeyDownEvent, MouseButton, MouseDownEvent, ParentElement,
    Pixels, Point, RenderOnce, ScrollEvent, ScrollHandle, StatefulInteractiveElement,
    StyleRefinement, Styled, Window, canvas,
    component::scroll::{
        Scrollbar, ScrollbarAxis, ScrollbarHandle, ScrollbarMode, ScrollbarStyles,
    },
    div, point,
    prelude::FluentBuilder as _,
    px,
};

/// `.cu-scroll::-webkit-scrollbar { width: 8px; height: 8px }`.
pub const THICKNESS: f32 = 8.;
/// A classic scrollbar's shortest thumb: Chromium sizes a custom
/// (`::-webkit-scrollbar`) thumb no shorter than the bar is thick.
pub const MIN_THUMB: f32 = THICKNESS;
/// Chromium's line step for an arrow key (`kPixelsPerLineStep`).
pub const LINE_STEP: f32 = 40.;
/// Chromium's page step: 87.5% of the viewport (`kMinFractionToStepWhenPaging`).
pub const PAGE_FRACTION: f32 = 0.875;
/// `.cu-scroll:focus-visible { outline: 2px solid var(--focus); outline-offset: -2px }`.
pub const FOCUS_WIDTH: f32 = 2.;

/// Which axes scroll. `.cu-scroll { overflow: auto }` scrolls both; the
/// single-axis forms lock the other axis (for a chat transcript, say).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ScrollAxes {
    /// `overflow: auto` on both axes (the reference).
    #[default]
    Both,
    /// Vertical only (`overflow-y: auto; overflow-x: hidden`).
    Vertical,
    /// Horizontal only.
    Horizontal,
}

impl ScrollAxes {
    /// Every option.
    pub const ALL: [ScrollAxes; 3] = [Self::Both, Self::Vertical, Self::Horizontal];

    /// A short name for controls.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Both => "both",
            Self::Vertical => "vertical",
            Self::Horizontal => "horizontal",
        }
    }

    fn scrollbar_axis(self) -> ScrollbarAxis {
        match self {
            Self::Both => ScrollbarAxis::Both,
            Self::Vertical => ScrollbarAxis::Vertical,
            Self::Horizontal => ScrollbarAxis::Horizontal,
        }
    }

    const fn vertical(self) -> bool {
        matches!(self, Self::Both | Self::Vertical)
    }

    const fn horizontal(self) -> bool {
        matches!(self, Self::Both | Self::Horizontal)
    }
}

/// The colors a Carbon scrollbar paints.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollbarLook {
    /// `::-webkit-scrollbar-thumb { background: var(--border-strong-01) }`.
    pub thumb: gpui_kit::Hsla,
    /// `::-webkit-scrollbar-track { background: transparent }`.
    pub track: gpui_kit::Hsla,
}

impl ScrollbarLook {
    /// The look on the active theme. The thumb does not change on hover or
    /// drag: `.cu-scroll` styles no `:hover`/`:active` thumb.
    pub fn current() -> Self {
        Self {
            thumb: color(ColorRole::BorderStrong01),
            track: gpui_kit::Hsla::transparent_black(),
        }
    }
}

/// Carbon's scrollbar styles for a Base/Kit [`Scrollbar`]: an 8px square
/// `border-strong-01` thumb filling an 8px transparent track, unchanged on
/// hover and drag.
pub fn carbon_scrollbar_styles(styles: ScrollbarStyles) -> ScrollbarStyles {
    let look = ScrollbarLook::current();
    styles
        .track(|t| {
            t.bg(look.track)
                .border_color(look.track)
                .width(px(THICKNESS))
        })
        .track_hover(|t| t.bg(look.track).border_color(look.track))
        .track_active(|t| t.bg(look.track).border_color(look.track))
        .thumb(|t| {
            t.bg(look.thumb)
                .width(px(THICKNESS))
                .inset(px(0.))
                .radius(px(0.))
                .min_length(px(MIN_THUMB))
        })
}

/// A Carbon-styled, always-visible Base [`Scrollbar`] for `handle` on `axes`,
/// sized to the element it is placed in (put it in an absolutely positioned
/// layer covering the viewport). For scrollers that are not a [`ScrollArea`].
pub fn carbon_scrollbar<H: ScrollbarHandle + Clone>(
    id: impl Into<ElementId>,
    handle: &H,
    axes: ScrollAxes,
) -> Scrollbar {
    Scrollbar::new(handle)
        .id(id)
        .axis(axes.scrollbar_axis())
        .mode(ScrollbarMode::Always)
        .viewport_from_layout()
        .styles(carbon_scrollbar_styles)
}

/// The scroll offset a key moves a focused scroller to, as a browser does for
/// a focused `overflow: auto` element: arrows by [`LINE_STEP`], Page Up/Down
/// and (Shift+)Space by [`PAGE_FRACTION`] of the viewport, Home/End to the
/// ends. `offset` is GPUI's (≤ 0, content moved up/left); `max` is the
/// positive scroll range. `None` when the key does not scroll (or the area is
/// already at that end), so the event can propagate.
pub fn keyboard_scroll(
    key: &str,
    shift: bool,
    offset: Point<f32>,
    max: Point<f32>,
    viewport: Point<f32>,
    axes: ScrollAxes,
) -> Option<Point<f32>> {
    let (mut x, mut y) = (offset.x, offset.y);
    let page = viewport.y * PAGE_FRACTION;
    match key {
        "up" if axes.vertical() => y += LINE_STEP,
        "down" if axes.vertical() => y -= LINE_STEP,
        "left" if axes.horizontal() => x += LINE_STEP,
        "right" if axes.horizontal() => x -= LINE_STEP,
        "pageup" if axes.vertical() => y += page,
        "pagedown" if axes.vertical() => y -= page,
        "space" if axes.vertical() => y += if shift { page } else { -page },
        "home" if axes.vertical() => y = 0.,
        "end" if axes.vertical() => y = -max.y,
        "home" => x = 0.,
        "end" => x = -max.x,
        _ => return None,
    }
    let next = point(x.clamp(-max.x.max(0.), 0.), y.clamp(-max.y.max(0.), 0.));
    (next != offset).then_some(next)
}

/// A thumb's `(start, length)` along a track of `track` px for a viewport of
/// `viewport` px over `content` px scrolled to `offset` (GPUI, ≤ 0): Base's
/// geometry with the Carbon styles (no inset, at least [`MIN_THUMB`] long).
pub fn thumb_span(track: f32, viewport: f32, content: f32, offset: f32) -> (f32, f32) {
    if content <= 0. {
        return (0., track);
    }
    let length = (viewport / content * track).max(MIN_THUMB).min(track);
    let extent = content - viewport;
    let start = if extent > 0. {
        (-offset / extent).clamp(0., 1.) * (track - length)
    } else {
        0.
    };
    (start, length)
}

/// The offset after a press at `at` px along a track whose thumb spans
/// `thumb` (`(start, length)`): one viewport toward the press, as a macOS
/// scroller pages by default ("Jump to the next page"), clamped to
/// `-max..=0`. `None` on the thumb itself (Base starts a drag there).
pub fn track_page(at: f32, thumb: (f32, f32), offset: f32, viewport: f32, max: f32) -> Option<f32> {
    let next = if at < thumb.0 {
        offset + viewport
    } else if at > thumb.0 + thumb.1 {
        offset - viewport
    } else {
        return None;
    };
    Some(next.clamp(-max.max(0.), 0.))
}

/// The horizontal offset a wheel step moves to when Shift turns a vertical
/// wheel into a horizontal one (as browsers do): `None` without Shift, for
/// a step that already has a horizontal part, when the area does not scroll
/// sideways, or when it is already at that end.
pub fn shift_wheel(shift: bool, delta: Point<f32>, offset_x: f32, max_x: f32) -> Option<f32> {
    if !shift || delta.x != 0. || delta.y == 0. || max_x <= 0. {
        return None;
    }
    let next = (offset_x + delta.y).clamp(-max_x, 0.);
    (next != offset_x).then_some(next)
}

/// Which bars a [`ScrollArea`] on `axes` shows for a scroll range `max`.
fn bars(axes: ScrollAxes, max: Point<Pixels>) -> (bool, bool) {
    (
        axes.vertical() && max.y > px(0.),
        axes.horizontal() && max.x > px(0.),
    )
}

/// A scroll container with Carbon's thin scrollbar. See the module docs.
///
/// Size it like a div (`.h(..)`, `.w(..)`, `.max_h(..)`); children are laid
/// out in a column that stretches to the viewport width and scrolls when it
/// overflows.
#[derive(IntoElement)]
pub struct ScrollArea {
    id: ElementId,
    style: StyleRefinement,
    children: Vec<AnyElement>,
    axes: ScrollAxes,
    focusable: bool,
    focus_handle: Option<FocusHandle>,
    scroll_handle: Option<ScrollHandle>,
    restrict_wheel: bool,
}

impl ScrollArea {
    /// A scroll area with a stable `id` (its scroll position is kept under it).
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            style: StyleRefinement::default(),
            children: Vec::new(),
            axes: ScrollAxes::Both,
            focusable: true,
            focus_handle: None,
            scroll_handle: None,
            restrict_wheel: false,
        }
    }

    /// Let a wheel scroll only along its own axis: a vertical wheel over a
    /// horizontal-only area then scrolls the page instead of the area
    /// sideways (Shift+wheel and horizontal wheels still scroll it).
    pub fn restrict_wheel_to_axis(mut self) -> Self {
        self.restrict_wheel = true;
        self
    }

    /// Which axes scroll (default both, `overflow: auto`).
    pub fn axes(mut self, axes: ScrollAxes) -> Self {
        self.axes = axes;
        self
    }

    /// Whether the area is a tab stop that scrolls from the keyboard (default
    /// true, `tabIndex={0}`). Turn it off when the content has focusable items
    /// of its own that should take the keys.
    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    /// Use a caller-owned focus handle.
    pub fn track_focus(mut self, handle: &FocusHandle) -> Self {
        self.focus_handle = Some(handle.clone());
        self
    }

    /// Use a caller-owned scroll handle (to read or set the offset, e.g. to
    /// keep a transcript pinned to its end).
    pub fn track_scroll(mut self, handle: &ScrollHandle) -> Self {
        self.scroll_handle = Some(handle.clone());
        self
    }
}

impl Styled for ScrollArea {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl ParentElement for ScrollArea {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

fn to_f32(p: Point<Pixels>) -> Point<f32> {
    point(f32::from(p.x), f32::from(p.y))
}

impl RenderOnce for ScrollArea {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let handle = self.scroll_handle.clone().unwrap_or_else(|| {
            window
                .use_keyed_state((id.clone(), "scroll"), cx, |_, _| ScrollHandle::default())
                .read(cx)
                .clone()
        });
        let focus = self.focus_handle.clone().unwrap_or_else(|| {
            window
                .use_keyed_state((id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
                .read(cx)
                .clone()
        });
        let axes = self.axes;
        // A classic scrollbar takes layout space on an axis that overflows,
        // read from the last layout; when this layout changes which bars
        // show, the next frame is drawn again with them.
        let (v_bar, h_bar) = bars(axes, handle.max_offset());
        let focus_visible =
            self.focusable && focus.is_focused(window) && window.last_input_was_keyboard();

        // The content column: as wide as the viewport, or wider when its
        // children are (horizontal scrolling), and inset by the bars.
        let content = div()
            .flex()
            .flex_col()
            .flex_none()
            .when(axes.horizontal(), |this| this.min_w_full())
            .when(axes == ScrollAxes::Horizontal, |this| this.min_h_full())
            .when(v_bar, |this| this.pr(px(THICKNESS)))
            .when(h_bar, |this| this.pb(px(THICKNESS)))
            .children(self.children);

        let viewport = div()
            .id((id.clone(), "viewport"))
            .flex()
            .flex_col()
            // Not stretched across: the content keeps its own width.
            .when(axes.horizontal(), |this| this.items_start())
            .flex_grow_1()
            .flex_shrink_1()
            .min_h_0()
            .w_full()
            .track_scroll(&handle)
            .map(|this| match axes {
                ScrollAxes::Both => this.overflow_scroll(),
                ScrollAxes::Vertical => this.overflow_y_scroll(),
                ScrollAxes::Horizontal => this.overflow_x_scroll(),
            })
            .when(self.restrict_wheel, |this| this.restrict_scroll_to_axis())
            .child(content);

        let on_key = {
            let handle = handle.clone();
            move |event: &KeyDownEvent, window: &mut Window, cx: &mut App| {
                let bounds = handle.bounds();
                let next = keyboard_scroll(
                    event.keystroke.key.as_str(),
                    event.keystroke.modifiers.shift,
                    to_f32(handle.offset()),
                    to_f32(handle.max_offset()),
                    point(f32::from(bounds.size.width), f32::from(bounds.size.height)),
                    axes,
                );
                if let Some(next) = next {
                    handle.set_offset(point(px(next.x), px(next.y)));
                    cx.stop_propagation();
                    window.refresh();
                }
            }
        };

        // Base draws only the bars that overflow, but reserves the vertical
        // bar's corner on a horizontal bar whenever it is given both axes.
        let bar_axes = match (v_bar, h_bar) {
            (true, true) => ScrollAxes::Both,
            (true, false) => ScrollAxes::Vertical,
            (false, true) => ScrollAxes::Horizontal,
            (false, false) => axes,
        };
        let behavior = {
            let measured = handle.clone();
            canvas(
                move |bounds, window, _| {
                    if bars(axes, measured.max_offset()) != (v_bar, h_bar) {
                        window.on_next_frame(|window, _| window.refresh());
                    }
                    window.insert_hitbox(bounds, HitboxBehavior::Normal)
                },
                {
                    let handle = handle.clone();
                    move |_, hitbox, window, _| {
                        install_pointer(&handle, hitbox, axes, (v_bar, h_bar), window)
                    }
                },
            )
            .absolute()
            .top_0()
            .left_0()
            .size_full()
        };

        let mut root = div()
            .id(id.clone())
            .relative()
            .flex()
            .flex_col()
            .overflow_hidden();
        gpui_kit::Refineable::refine(root.style(), &self.style);
        root.when(self.focusable, |this| {
            this.track_focus(&focus.clone().tab_stop(true))
                .on_key_down(on_key)
        })
        .child(viewport)
        .child(behavior)
        .child(
            div()
                .absolute()
                .top_0()
                .left_0()
                .size_full()
                .child(carbon_scrollbar(
                    (id.clone(), "scrollbar"),
                    &handle,
                    bar_axes,
                )),
        )
        .when(focus_visible, |this| {
            this.child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .size_full()
                    .border(px(FOCUS_WIDTH))
                    .border_color(color(ColorRole::Focus)),
            )
        })
    }
}

/// The pointer behavior Base's scrollbar lacks: track presses page
/// ([`track_page`]) and Shift turns a vertical wheel sideways
/// ([`shift_wheel`]). Registered after the viewport's own wheel listener, so
/// it runs first in the bubble phase; the track press is taken in the capture
/// phase, before Base's bar would jump the thumb there.
fn install_pointer(
    handle: &ScrollHandle,
    hitbox: Hitbox,
    axes: ScrollAxes,
    shown: (bool, bool),
    window: &mut Window,
) {
    let bounds = hitbox.bounds;
    window.on_mouse_event({
        let handle = handle.clone();
        let hitbox = hitbox.clone();
        move |event: &MouseDownEvent, phase, window, cx| {
            if phase != DispatchPhase::Capture
                || event.button != MouseButton::Left
                || !hitbox.is_hovered(window)
            {
                return;
            }
            if let Some(next) = track_press(&handle, bounds, shown, event.position) {
                handle.set_offset(next);
                cx.stop_propagation();
                window.refresh();
            }
        }
    });
    if !axes.horizontal() {
        return;
    }
    window.on_mouse_event({
        let handle = handle.clone();
        move |event: &ScrollEvent, phase, window, cx| {
            if phase != DispatchPhase::Bubble || !hitbox.should_handle_scroll(window) {
                return;
            }
            let delta = event.delta.pixel_delta(window.line_height());
            let offset = handle.offset();
            if let Some(x) = shift_wheel(
                event.modifiers.shift,
                to_f32(delta),
                f32::from(offset.x),
                f32::from(handle.max_offset().x),
            ) {
                handle.set_offset(point(px(x), offset.y));
                cx.stop_propagation();
                window.refresh();
            }
        }
    });
}

/// The offset a left press at `at` on a bar's track pages to, if it hit a
/// shown bar outside its thumb. The vertical bar runs the full height; the
/// horizontal one leaves the corner to it when both show (Base's layout).
fn track_press(
    handle: &ScrollHandle,
    bounds: Bounds<Pixels>,
    (v_bar, h_bar): (bool, bool),
    at: Point<Pixels>,
) -> Option<Point<Pixels>> {
    let offset = to_f32(handle.offset());
    let max = to_f32(handle.max_offset());
    let origin = to_f32(bounds.origin);
    let size = point(f32::from(bounds.size.width), f32::from(bounds.size.height));
    let local = point(f32::from(at.x) - origin.x, f32::from(at.y) - origin.y);
    if v_bar && local.x >= size.x - THICKNESS {
        let thumb = thumb_span(size.y, size.y, size.y + max.y, offset.y);
        let y = track_page(local.y, thumb, offset.y, size.y, max.y)?;
        return Some(point(px(offset.x), px(y)));
    }
    if h_bar && local.y >= size.y - THICKNESS {
        let track = size.x - if v_bar { THICKNESS } else { 0. };
        let thumb = thumb_span(track, size.x, size.x + max.x, offset.x);
        if local.x > track {
            return None;
        }
        let x = track_page(local.x, thumb, offset.x, size.x, max.x)?;
        return Some(point(px(x), px(offset.y)));
    }
    None
}

/// `index.d.ts` `ScrollBar.orientation`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ScrollBarOrientation {
    /// `vertical` (default).
    #[default]
    Vertical,
    /// `horizontal`.
    Horizontal,
}

impl ScrollBarOrientation {
    /// Both orientations.
    pub const ALL: [ScrollBarOrientation; 2] = [Self::Vertical, Self::Horizontal];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Vertical => "vertical",
            Self::Horizontal => "horizontal",
        }
    }
}

/// shadcn's `ScrollBar`, kept for API compatibility: [`ScrollArea`] always
/// draws its own bars, so this renders nothing (`export const ScrollBar = () => null`).
#[derive(IntoElement, Default)]
pub struct ScrollBar {
    orientation: ScrollBarOrientation,
}

impl ScrollBar {
    /// A no-op scrollbar.
    pub fn new() -> Self {
        Self::default()
    }

    /// Accepted and ignored.
    pub fn orientation(mut self, orientation: ScrollBarOrientation) -> Self {
        self.orientation = orientation;
        self
    }
}

impl RenderOnce for ScrollBar {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        gpui_kit::Empty
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    #[test]
    fn scrollbar_look_follows_custom_css() {
        crate::theme::tests_set(CarbonTheme::White);
        let look = ScrollbarLook::current();
        // `.cu-scroll::-webkit-scrollbar-thumb { background: var(--border-strong-01) }`.
        assert_eq!(look.thumb, color(ColorRole::BorderStrong01));
        // `::-webkit-scrollbar-track { background: transparent }`.
        assert_eq!(look.track.a, 0.);
        // `::-webkit-scrollbar { width: 8px; height: 8px }`.
        assert_eq!(THICKNESS, 8.);
        assert_eq!(FOCUS_WIDTH, 2.);
    }

    #[test]
    fn keyboard_scrolls_like_a_focused_browser_scroller() {
        let max = point(100., 800.);
        let view = point(280., 180.);
        let at = |x, y| point(x, y);
        let step =
            |key, shift, offset| keyboard_scroll(key, shift, offset, max, view, ScrollAxes::Both);
        assert_eq!(step("down", false, at(0., 0.)), Some(at(0., -40.)));
        assert_eq!(step("up", false, at(0., 0.)), None);
        assert_eq!(step("pagedown", false, at(0., 0.)), Some(at(0., -157.5)));
        assert_eq!(step("space", false, at(0., 0.)), Some(at(0., -157.5)));
        assert_eq!(step("space", true, at(0., -157.5)), Some(at(0., 0.)));
        assert_eq!(step("end", false, at(0., 0.)), Some(at(0., -800.)));
        assert_eq!(step("down", false, at(0., -790.)), Some(at(0., -800.)));
        assert_eq!(step("home", false, at(0., -300.)), Some(at(0., 0.)));
        assert_eq!(step("right", false, at(0., 0.)), Some(at(-40., 0.)));
        assert_eq!(step("a", false, at(0., 0.)), None);
        // A vertical-only area ignores horizontal keys.
        assert_eq!(
            keyboard_scroll("right", false, at(0., 0.), max, view, ScrollAxes::Vertical),
            None
        );
    }

    #[test]
    fn horizontal_keys_scroll_a_wide_area() {
        let max = point(360., 0.);
        let view = point(280., 180.);
        let step = |key, offset, axes| keyboard_scroll(key, false, offset, max, view, axes);
        assert_eq!(
            step("right", point(0., 0.), ScrollAxes::Horizontal),
            Some(point(-40., 0.))
        );
        assert_eq!(
            step("left", point(-40., 0.), ScrollAxes::Horizontal),
            Some(point(0., 0.))
        );
        // Home/End move along the only axis.
        assert_eq!(
            step("end", point(0., 0.), ScrollAxes::Horizontal),
            Some(point(-360., 0.))
        );
        assert_eq!(step("right", point(-360., 0.), ScrollAxes::Both), None);
    }

    #[test]
    fn thumb_follows_base_geometry() {
        // 180px viewport over 720px: a quarter-length thumb.
        assert_eq!(thumb_span(180., 180., 720., 0.), (0., 45.));
        assert_eq!(thumb_span(180., 180., 720., -540.), (135., 45.));
        // Never shorter than the bar is thick.
        assert_eq!(thumb_span(100., 100., 100_000., 0.).1, MIN_THUMB);
    }

    #[test]
    fn a_track_press_pages_one_viewport_toward_the_pointer() {
        let thumb = thumb_span(180., 180., 720., -270.);
        // Below the thumb: one viewport down.
        assert_eq!(track_page(170., thumb, -270., 180., 540.), Some(-450.));
        // Above it: one viewport up.
        assert_eq!(track_page(5., thumb, -270., 180., 540.), Some(-90.));
        // On the thumb: Base drags.
        assert_eq!(track_page(thumb.0 + 1., thumb, -270., 180., 540.), None);
        // Clamped at the ends.
        assert_eq!(track_page(170., thumb, -500., 180., 540.), Some(-540.));
        assert_eq!(track_page(0., (50., 45.), -20., 180., 540.), Some(0.));
    }

    #[test]
    fn shift_turns_a_vertical_wheel_sideways() {
        let wheel = point(0., -30.);
        assert_eq!(shift_wheel(true, wheel, 0., 360.), Some(-30.));
        assert_eq!(shift_wheel(false, wheel, 0., 360.), None);
        // A horizontal step is already sideways.
        assert_eq!(shift_wheel(true, point(-30., 0.), 0., 360.), None);
        // Nothing to scroll, or already at the end.
        assert_eq!(shift_wheel(true, wheel, 0., 0.), None);
        assert_eq!(shift_wheel(true, wheel, -360., 360.), None);
        assert_eq!(shift_wheel(true, point(0., 50.), -20., 360.), Some(0.));
    }

    #[test]
    fn bars_show_only_on_overflowing_scroll_axes() {
        let max = point(px(360.), px(0.));
        assert_eq!(bars(ScrollAxes::Both, max), (false, true));
        assert_eq!(bars(ScrollAxes::Horizontal, max), (false, true));
        assert_eq!(bars(ScrollAxes::Vertical, max), (false, false));
        let both = point(px(10.), px(10.));
        assert_eq!(bars(ScrollAxes::Both, both), (true, true));
    }
}
