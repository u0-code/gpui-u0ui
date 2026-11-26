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
//!   scrolling, so that part is written here.
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
    AnyElement, App, ElementId, FocusHandle, InteractiveElement, IntoElement, KeyDownEvent,
    ParentElement, Pixels, Point, RenderOnce, ScrollHandle, StatefulInteractiveElement,
    StyleRefinement, Styled, Window,
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
        }
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
        // A classic scrollbar takes layout space on an axis that overflows
        // (read from the last layout; GPUI settles it on the next frame).
        let max = handle.max_offset();
        let v_bar = axes.vertical() && max.y > px(0.);
        let h_bar = axes.horizontal() && max.x > px(0.);
        let focus_visible =
            self.focusable && focus.is_focused(window) && window.last_input_was_keyboard();

        let content = div()
            .flex()
            .flex_col()
            .flex_none()
            .when(axes == ScrollAxes::Horizontal, |this| this.h_full())
            .when(v_bar, |this| this.mr(px(THICKNESS)))
            .when(h_bar, |this| this.mb(px(THICKNESS)))
            .children(self.children);

        let viewport = div()
            .id((id.clone(), "viewport"))
            .flex()
            .flex_col()
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
        .child(
            div()
                .absolute()
                .top_0()
                .left_0()
                .size_full()
                .child(carbon_scrollbar((id.clone(), "scrollbar"), &handle, axes)),
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
}
