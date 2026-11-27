//! Resizable: panels split by draggable, keyboard-resizable handles
//! (`.cu-rz*` in `custom.css`, `ResizablePanelGroup` in `index.jsx`; Carbon has
//! no core equivalent).
//!
//! - **Behavior** is written here, not Kit's. Base's `ResizablePanelGroup`
//!   sizes panels in pixels (100px minimum) and its handle is neither focusable
//!   nor keyboard-operable; the reference sizes panels in percentages of the
//!   group (`flex-basis: N%`, so they keep their proportions when the group
//!   resizes), clamps each pair to `minSize` (default 10%) and resizes the
//!   focused handle by ±5% with the arrow keys. Dragging uses GPUI's drag
//!   (`on_drag` / `on_drag_move`) like Base does; the arithmetic is the
//!   reference's ([`drag_sizes`], [`key_sizes`]).
//! - **Appearance**:
//!   - group: `.cu-rz { display: flex; width: 100%; height: 100%; min-height:
//!     10rem; outline: 1px solid var(--border-subtle-01); outline-offset: -1px }`,
//!     `.cu-rz--vertical { flex-direction: column }`;
//!   - panel: `.cu-rz-panel { min-width: 0; min-height: 0; overflow: auto;
//!     flex: 0 0 <size>% }`. Content is clipped rather than scrolled: a GPUI
//!     scroll container cannot hand its definite height to a nested group;
//!     put a [`super::scroll_area::ScrollArea`] in a panel that needs to scroll;
//!   - handle: `.cu-rz-handle { width: 1px; background: var(--border-subtle-01);
//!     cursor: col-resize }` with an invisible hit area `::after { inset: 0 -4px }`
//!     (`-4px 0` and `row-resize` when vertical); `:hover, :focus-visible
//!     { background: var(--border-interactive) }` (no transition);
//!   - grip (`withHandle`): `.cu-rz-grip { width: 1rem; height: 1.5rem;
//!     background: var(--layer-accent-01) }` holding the 16px `Draggable` icon in
//!     `icon-secondary`, `rotate(90deg)` in a vertical group (drawn as a 24×16
//!     box with the icon turned, which is what the rotation shows).
//!
//! ```ignore
//! ResizablePanelGroup::new("split")
//!     .panel(ResizablePanel::new().default_size(30.).child("Sidebar"))
//!     .handle(ResizableHandle::new().with_handle(true))
//!     .panel(ResizablePanel::new().default_size(70.).child("Main"))
//! ```
use crate::{
    icons::{CarbonIcon, Icon, IconSize},
    theme::color,
    tokens::ColorRole,
};
use gpui_kit::{
    AnyElement, App, AppContext as _, CursorStyle, ElementId, Empty, Hsla, InteractiveElement,
    IntoElement, KeyDownEvent, MouseButton, MouseDownEvent, ParentElement, Pixels, Render,
    RenderOnce, StatefulInteractiveElement, Styled, Window, deferred, div,
    prelude::FluentBuilder as _, px, relative, rems,
};

/// `index.d.ts` `ResizablePanelGroupProps.orientation`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ResizableOrientation {
    /// Panels side by side (default).
    #[default]
    Horizontal,
    /// Panels stacked.
    Vertical,
}

impl ResizableOrientation {
    /// Both orientations.
    pub const ALL: [ResizableOrientation; 2] = [Self::Horizontal, Self::Vertical];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }
}

/// `ResizablePanel.minSize` default: `panels[i].props.minSize || 10`.
pub const DEFAULT_MIN_SIZE: f32 = 10.;
/// The arrow-key step (`{ ArrowLeft: -5, ArrowUp: -5, ArrowRight: 5, ArrowDown: 5 }`).
pub const KEY_STEP: f32 = 5.;
/// `.cu-rz-handle { width: 1px }`.
pub const HANDLE_THICKNESS: f32 = 1.;
/// `.cu-rz-handle::after { inset: 0 -4px }`: the hit area overhangs 4px per side.
pub const HIT_OVERHANG: f32 = 4.;
/// `.cu-rz-grip { width: 1rem; height: 1.5rem }` in px (before rotation).
pub const GRIP: (f32, f32) = (16., 24.);
/// `.cu-rz { min-height: 10rem }` in px.
pub const MIN_HEIGHT: f32 = 160.;

/// Sizes after dragging handle `i` by `delta` percent from `start` (the sizes
/// when the drag began): `a = min(max(s0[i] + d, min_i), s0[i] + s0[i + 1] −
/// min_next)`; the pair keeps its sum.
pub fn drag_sizes(start: &[f32], i: usize, delta: f32, min_i: f32, min_next: f32) -> Vec<f32> {
    let mut next = start.to_vec();
    if i + 1 >= start.len() {
        return next;
    }
    let pair = start[i] + start[i + 1];
    let a = (start[i] + delta).max(min_i).min(pair - min_next);
    next[i] = a;
    next[i + 1] = pair - a;
    next
}

/// The step a key moves handle `i` by, if it is a resize key.
pub fn key_step(key: &str) -> Option<f32> {
    match key {
        "left" | "up" => Some(-KEY_STEP),
        "right" | "down" => Some(KEY_STEP),
        _ => None,
    }
}

/// Sizes after a resize key on handle `i` (`None` for other keys).
pub fn key_sizes(
    sizes: &[f32],
    i: usize,
    key: &str,
    min_i: f32,
    min_next: f32,
) -> Option<Vec<f32>> {
    key_step(key).map(|step| drag_sizes(sizes, i, step, min_i, min_next))
}

/// The handle line color: `.cu-rz-handle { background: border-subtle-01 }`,
/// `:hover, :focus-visible { background: border-interactive }`.
pub fn handle_color(hovered: bool, focus_visible: bool, dragging: bool) -> Hsla {
    color(if hovered || focus_visible || dragging {
        ColorRole::BorderInteractive
    } else {
        ColorRole::BorderSubtle01
    })
}

/// Initial sizes: each panel's `defaultSize`, else `100 / panels.length`.
pub fn initial_sizes(defaults: &[Option<f32>]) -> Vec<f32> {
    let even = 100. / defaults.len().max(1) as f32;
    defaults.iter().map(|d| d.unwrap_or(even)).collect()
}

/// One panel. See the module docs.
pub struct ResizablePanel {
    default_size: Option<f32>,
    min_size: Option<f32>,
    children: Vec<AnyElement>,
}

impl Default for ResizablePanel {
    fn default() -> Self {
        Self::new()
    }
}

impl ResizablePanel {
    /// A panel sharing the group evenly unless given a default size.
    pub fn new() -> Self {
        Self {
            default_size: None,
            min_size: None,
            children: Vec::new(),
        }
    }

    /// `defaultSize`: the initial size in percent of the group.
    pub fn default_size(mut self, percent: f32) -> Self {
        self.default_size = Some(percent);
        self
    }

    /// `minSize`: the smallest size in percent (default 10).
    pub fn min_size(mut self, percent: f32) -> Self {
        self.min_size = Some(percent);
        self
    }

    fn min(&self) -> f32 {
        self.min_size.unwrap_or(DEFAULT_MIN_SIZE)
    }
}

impl ParentElement for ResizablePanel {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

/// A handle between two panels. See the module docs.
#[derive(Clone, Copy, Default)]
pub struct ResizableHandle {
    with_handle: bool,
}

impl ResizableHandle {
    /// A plain 1px handle.
    pub fn new() -> Self {
        Self::default()
    }

    /// `withHandle`: show the grip.
    pub fn with_handle(mut self, with_handle: bool) -> Self {
        self.with_handle = with_handle;
        self
    }
}

enum Part {
    Panel(ResizablePanel),
    Handle(ResizableHandle),
}

/// The panel group (`ResizablePanelGroup`, also `Resizable`). Add panels and
/// handles in order with [`Self::panel`] and [`Self::handle`].
#[derive(IntoElement)]
pub struct ResizablePanelGroup {
    id: ElementId,
    orientation: ResizableOrientation,
    parts: Vec<Part>,
}

/// shadcn's alias.
pub type Resizable = ResizablePanelGroup;

impl ResizablePanelGroup {
    /// A horizontal group with a stable `id` (its sizes are kept under it).
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            orientation: ResizableOrientation::Horizontal,
            parts: Vec::new(),
        }
    }

    /// The orientation.
    pub fn orientation(mut self, orientation: ResizableOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Append a panel.
    pub fn panel(mut self, panel: ResizablePanel) -> Self {
        self.parts.push(Part::Panel(panel));
        self
    }

    /// Append a handle (between the previous and the next panel).
    pub fn handle(mut self, handle: ResizableHandle) -> Self {
        self.parts.push(Part::Handle(handle));
        self
    }
}

/// Sizes and the drag in progress, kept per group.
#[derive(Default)]
struct GroupState {
    /// Panel sizes in percent.
    sizes: Vec<f32>,
    /// The defaults the sizes were built from (rebuilt when they change).
    defaults: Vec<Option<f32>>,
    /// Pointer position (along the axis) and sizes when a drag began.
    drag: Option<(Pixels, Vec<f32>)>,
    /// Handle being dragged.
    dragging: Option<usize>,
}

/// Pointer over one handle's hit area or grip.
#[derive(Default)]
struct HandlePointer {
    hit: bool,
    grip: bool,
}

/// The drag payload: which group and handle.
#[derive(Clone)]
struct HandleDrag {
    group: ElementId,
}

struct DragGhost;

impl Render for DragGhost {
    fn render(&mut self, _: &mut Window, _: &mut gpui_kit::Context<Self>) -> impl IntoElement {
        Empty
    }
}

impl RenderOnce for ResizablePanelGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let horizontal = self.orientation == ResizableOrientation::Horizontal;
        let defaults: Vec<Option<f32>> = self
            .parts
            .iter()
            .filter_map(|p| match p {
                Part::Panel(p) => Some(p.default_size),
                Part::Handle(_) => None,
            })
            .collect();
        let mins: Vec<f32> = self
            .parts
            .iter()
            .filter_map(|p| match p {
                Part::Panel(p) => Some(p.min()),
                Part::Handle(_) => None,
            })
            .collect();
        let state = window.use_keyed_state((id.clone(), "sizes"), cx, |_, _| GroupState::default());
        state.update(cx, |s, _| {
            if s.defaults != defaults || s.sizes.len() != defaults.len() {
                s.sizes = initial_sizes(&defaults);
                s.defaults = defaults.clone();
            }
        });
        let (sizes, dragging) = {
            let s = state.read(cx);
            (s.sizes.clone(), s.dragging)
        };
        let mins = std::rc::Rc::new(mins);
        let along = move |p: gpui_kit::Point<Pixels>| if horizontal { p.x } else { p.y };

        let mut panel_ix = 0usize;
        let mut handle_ix = 0usize;
        let mut children: Vec<AnyElement> = Vec::new();
        for part in self.parts {
            match part {
                Part::Panel(panel) => {
                    let size = sizes.get(panel_ix).copied().unwrap_or(0.);
                    panel_ix += 1;
                    children.push(
                        div()
                            .flex()
                            .flex_col()
                            .flex_grow_0()
                            .flex_shrink_0()
                            .flex_basis(relative(size / 100.))
                            .min_w_0()
                            .min_h_0()
                            .overflow_hidden()
                            .children(panel.children)
                            .into_any_element(),
                    );
                }
                Part::Handle(handle) => {
                    let i = handle_ix;
                    handle_ix += 1;
                    let hid = ElementId::NamedChild(
                        std::sync::Arc::new(id.clone()),
                        format!("handle-{i}").into(),
                    );
                    children.push(render_handle(
                        hid,
                        i,
                        handle,
                        horizontal,
                        dragging == Some(i),
                        &state,
                        &mins,
                        along,
                        &id,
                        window,
                        cx,
                    ));
                }
            }
        }

        let group = id.clone();
        div()
            .id(id.clone())
            .relative()
            .flex()
            .when(!horizontal, |this| this.flex_col())
            .size_full()
            .min_h(rems(MIN_HEIGHT / 16.))
            .on_drag_move::<HandleDrag>({
                let state = state.clone();
                let mins = mins.clone();
                move |event, _, cx| {
                    if event.drag(cx).group != group {
                        return;
                    }
                    let total = f32::from(if horizontal {
                        event.bounds.size.width
                    } else {
                        event.bounds.size.height
                    });
                    let position = along(event.event.position);
                    state.update(cx, |s, cx| {
                        let (Some(i), Some((start, s0))) = (s.dragging, s.drag.as_ref()) else {
                            return;
                        };
                        if total <= 0. {
                            return;
                        }
                        let delta = f32::from(position - *start) / total * 100.;
                        let next = drag_sizes(s0, i, delta, mins[i], mins[i + 1]);
                        if next != s.sizes {
                            s.sizes = next;
                            cx.notify();
                        }
                    });
                }
            })
            .children(children)
            // `outline: 1px solid border-subtle-01; outline-offset: -1px`.
            .child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .size_full()
                    .border_1()
                    .border_color(color(ColorRole::BorderSubtle01)),
            )
    }
}

#[allow(clippy::too_many_arguments)]
fn render_handle(
    hid: ElementId,
    i: usize,
    handle: ResizableHandle,
    horizontal: bool,
    dragging: bool,
    state: &gpui_kit::Entity<GroupState>,
    mins: &std::rc::Rc<Vec<f32>>,
    along: impl Fn(gpui_kit::Point<Pixels>) -> Pixels + Copy + 'static,
    group: &ElementId,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let pointer = window.use_keyed_state((hid.clone(), "pointer"), cx, |_, _| {
        HandlePointer::default()
    });
    let focus = window
        .use_keyed_state((hid.clone(), "focus"), cx, |_, cx| cx.focus_handle())
        .read(cx)
        .clone();
    let hovered = {
        let p = pointer.read(cx);
        p.hit || p.grip
    };
    let focus_visible = focus.is_focused(window) && crate::focus::focus_visible_mode(window);
    let line = handle_color(hovered, focus_visible, dragging);
    let cursor = if horizontal {
        CursorStyle::ResizeLeftRight
    } else {
        CursorStyle::ResizeUpDown
    };

    // Pointer down records where the drag starts; the group follows it.
    // It also focuses the handle, so the arrow keys work after a drag (the
    // hit area and grip are separate elements from the focusable line).
    let start_drag = {
        let state = state.clone();
        let focus = focus.clone();
        move |event: &MouseDownEvent, window: &mut Window, cx: &mut App| {
            window.focus(&focus, cx);
            let position = along(event.position);
            state.update(cx, |s, cx| {
                s.drag = Some((position, s.sizes.clone()));
                s.dragging = Some(i);
                cx.notify();
            });
        }
    };
    let end_drag = {
        let state = state.clone();
        move |_: &gpui_kit::MouseUpEvent, _: &mut Window, cx: &mut App| {
            state.update(cx, |s, cx| {
                if s.dragging.take().is_some() {
                    s.drag = None;
                    cx.notify();
                }
            });
        }
    };
    let set_pointer = |grip: bool| {
        let pointer = pointer.clone();
        move |over: &bool, _: &mut Window, cx: &mut App| {
            pointer.update(cx, |p, cx| {
                if grip {
                    p.grip = *over;
                } else {
                    p.hit = *over;
                }
                cx.notify();
            })
        }
    };
    let payload = HandleDrag {
        group: group.clone(),
    };
    let on_key = {
        let state = state.clone();
        let mins = mins.clone();
        move |event: &KeyDownEvent, _: &mut Window, cx: &mut App| {
            let key = event.keystroke.key.as_str();
            if key_step(key).is_none() {
                return;
            }
            cx.stop_propagation();
            state.update(cx, |s, cx| {
                if let Some(next) = key_sizes(&s.sizes, i, key, mins[i], mins[i + 1]) {
                    s.sizes = next;
                    cx.notify();
                }
            });
        }
    };

    // `::after { inset: 0 -4px }` (or `-4px 0`).
    let hit = div()
        .id((hid.clone(), "hit"))
        .absolute()
        .map(|this| {
            if horizontal {
                this.top_0()
                    .bottom_0()
                    .left(px(-HIT_OVERHANG))
                    .right(px(-HIT_OVERHANG))
            } else {
                this.left_0()
                    .right_0()
                    .top(px(-HIT_OVERHANG))
                    .bottom(px(-HIT_OVERHANG))
            }
        })
        .cursor(cursor)
        .on_hover(set_pointer(false))
        .on_mouse_down(MouseButton::Left, start_drag.clone())
        .on_mouse_up(MouseButton::Left, end_drag.clone())
        .on_mouse_up_out(MouseButton::Left, end_drag.clone())
        .on_drag(payload.clone(), |_, _, _, cx| cx.new(|_| DragGhost));

    let grip = handle.with_handle.then(|| {
        // Rotated 90° in a vertical group: a 24×16 box, icon turned.
        let (w, h) = if horizontal { GRIP } else { (GRIP.1, GRIP.0) };
        div()
            .id((hid.clone(), "grip"))
            .absolute()
            .map(|this| {
                if horizontal {
                    this.left(px((HANDLE_THICKNESS - w) / 2.))
                        .top(relative(0.5))
                        .mt(px(-h / 2.))
                } else {
                    this.top(px((HANDLE_THICKNESS - h) / 2.))
                        .left(relative(0.5))
                        .ml(px(-w / 2.))
                }
            })
            .w(rems(w / 16.))
            .h(rems(h / 16.))
            .flex()
            .items_center()
            .justify_center()
            .bg(color(ColorRole::LayerAccent01))
            .cursor(cursor)
            .on_hover(set_pointer(true))
            .on_mouse_down(MouseButton::Left, start_drag)
            .on_mouse_up(MouseButton::Left, end_drag.clone())
            .on_mouse_up_out(MouseButton::Left, end_drag)
            .on_drag(payload, |_, _, _, cx| cx.new(|_| DragGhost))
            .child(
                Icon::carbon(CarbonIcon::Draggable)
                    .size(IconSize::S16)
                    .color(color(ColorRole::IconSecondary))
                    .when(!horizontal, |icon| {
                        icon.rotate(gpui_kit::radians(std::f32::consts::FRAC_PI_2))
                    }),
            )
    });

    div()
        .id(hid.clone())
        .track_focus(&focus.tab_stop(true))
        .on_key_down(on_key)
        .relative()
        .flex_none()
        .map(|this| {
            if horizontal {
                this.w(px(HANDLE_THICKNESS)).self_stretch()
            } else {
                this.h(px(HANDLE_THICKNESS)).self_stretch()
            }
        })
        .bg(line)
        // The hit area and grip are positioned, so they paint (and take the
        // pointer) above the next panel, as positioned boxes do in CSS.
        .child(deferred(hit).with_priority(1))
        .when_some(grip, |this, grip| {
            this.child(deferred(grip).with_priority(1))
        })
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    #[test]
    fn drag_clamps_the_pair_to_min_sizes() {
        let s0 = [30., 70.];
        assert_eq!(drag_sizes(&s0, 0, 10., 10., 10.), vec![40., 60.]);
        // `Math.max(s0[i] + d, minSize || 10)`.
        assert_eq!(drag_sizes(&s0, 0, -50., 10., 10.), vec![10., 90.]);
        // `s0[i] + s0[i + 1] - (minSize || 10)`.
        assert_eq!(drag_sizes(&s0, 0, 80., 10., 10.), vec![90., 10.]);
        assert_eq!(drag_sizes(&s0, 0, 80., 10., 25.), vec![75., 25.]);
        // Other panels keep their sizes.
        assert_eq!(
            drag_sizes(&[20., 30., 50.], 1, 5., 10., 10.),
            vec![20., 35., 45.]
        );
    }

    #[test]
    fn arrow_keys_step_five_percent() {
        let s = [60., 40.];
        assert_eq!(key_sizes(&s, 0, "right", 10., 10.), Some(vec![65., 35.]));
        assert_eq!(key_sizes(&s, 0, "down", 10., 10.), Some(vec![65., 35.]));
        assert_eq!(key_sizes(&s, 0, "left", 10., 10.), Some(vec![55., 45.]));
        assert_eq!(key_sizes(&s, 0, "up", 10., 10.), Some(vec![55., 45.]));
        assert_eq!(
            key_sizes(&[12., 88.], 0, "left", 10., 10.),
            Some(vec![10., 90.])
        );
        assert_eq!(key_sizes(&s, 0, "enter", 10., 10.), None);
    }

    #[test]
    fn defaults_split_evenly() {
        assert_eq!(initial_sizes(&[Some(30.), Some(70.)]), vec![30., 70.]);
        assert_eq!(initial_sizes(&[None, None, None, None]), vec![25.; 4]);
    }

    #[test]
    fn handle_colors_and_geometry_follow_custom_css() {
        crate::theme::tests_set(CarbonTheme::White);
        assert_eq!(
            handle_color(false, false, false),
            color(ColorRole::BorderSubtle01)
        );
        assert_eq!(
            handle_color(true, false, false),
            color(ColorRole::BorderInteractive)
        );
        assert_eq!(
            handle_color(false, true, false),
            color(ColorRole::BorderInteractive)
        );
        assert_eq!((HANDLE_THICKNESS, HIT_OVERHANG), (1., 4.));
        assert_eq!(GRIP, (16., 24.));
        assert_eq!(MIN_HEIGHT, 160.);
    }
}
