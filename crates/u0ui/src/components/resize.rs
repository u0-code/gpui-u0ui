//! Vertical resizing for multi-line fields (`resize: vertical` on
//! `.cds--text-area` and `.cu-ig-textarea`).
//!
//! The reference leaves the resizer to the browser: a grip of two diagonal
//! strokes in the field's bottom-right corner that drags the field's height
//! (never below its `min-block-size`) with an `ns-resize` cursor. Kit's text
//! area has no resizer, so this module draws the grip and runs the drag with
//! GPUI's `on_drag` / `on_drag_move`, like [`super::resizable`] does. The
//! dragged height lives in keyed state under the field's id, in Carbon px
//! (rem-relative, so zoom keeps it in proportion).
use super::form::rem;
use crate::{theme::color, tokens::ColorRole};
use gpui_kit::{
    App, AppContext as _, CursorStyle, ElementId, Empty, Entity, EntityId, InteractiveElement as _,
    IntoElement, MouseButton, ParentElement, PathBuilder, Render, StatefulInteractiveElement as _,
    Styled, Window, canvas, div, point, px,
};

/// The grip's hit area: a 16px square in the bottom-right corner.
pub const GRIP: f32 = 16.;

/// A field's resize state, kept under its id.
#[derive(Default)]
pub struct Resize {
    /// The dragged field height in Carbon px, `None` until the user drags.
    pub height: Option<f32>,
    /// Where the current drag started (set on the grip's mouse-down; kept
    /// here, not in the drag payload, because the payload is rebuilt every
    /// frame and the drag starts from a later frame's).
    start: Start,
}

/// The keyed [`Resize`] state of a field.
pub type HeightStore = Entity<Resize>;

/// The keyed height store for the field `id`.
pub fn height_store(id: &ElementId, window: &mut Window, cx: &mut App) -> HeightStore {
    window.use_keyed_state((id.clone(), "resize-height"), cx, |_, _| Resize::default())
}

/// The field height while dragging: its height when the grip was pressed
/// plus how far the pointer has moved down since, never below `min`.
pub fn dragged_height(start_height: f32, pointer_delta: f32, min: f32) -> f32 {
    (start_height + pointer_delta).max(min)
}

/// The grip glyph's two strokes, in px from the grip box's top-left: the
/// long one from (6,14) to (14,6), the short one from (10,14) to (14,10)
/// (the browser's corner resizer).
pub const STROKES: [((f32, f32), (f32, f32)); 2] =
    [((6., 14.), (14., 6.)), ((10., 14.), (14., 10.))];

/// Where a drag started: the pointer's y at mouse-down and the field's
/// height then (filled on the first move), both in screen px.
#[derive(Clone, Copy, Default)]
struct Start {
    pointer_y: f32,
    height: Option<f32>,
}

/// The drag payload: which store to resize.
#[derive(Clone)]
pub struct ResizeDrag {
    store: EntityId,
}

struct DragGhost;

impl Render for DragGhost {
    fn render(&mut self, _: &mut Window, _: &mut gpui_kit::Context<Self>) -> impl IntoElement {
        Empty
    }
}

/// The grip: an absolutely positioned 16px box at the field's bottom-right
/// corner (the field must be `relative`) that starts a [`ResizeDrag`].
pub fn grip(id: &ElementId, store: &HeightStore) -> impl IntoElement {
    let payload = ResizeDrag {
        store: store.entity_id(),
    };
    let press = store.clone();
    let stroke = color(ColorRole::IconSecondary);
    div()
        .id((id.clone(), "resize-grip"))
        .absolute()
        .right_0()
        .bottom_0()
        .size(rem(GRIP))
        .cursor(CursorStyle::ResizeUpDown)
        // The browser's resizer neither focuses the field nor starts a selection.
        .on_mouse_down(MouseButton::Left, move |event, _, cx| {
            press.update(cx, |r, _| {
                r.start = Start {
                    pointer_y: f32::from(event.position.y),
                    height: None,
                }
            });
            cx.stop_propagation();
        })
        .on_drag(payload, |_, _, _, cx| cx.new(|_| DragGhost))
        .child(
            canvas(
                |_, _, _| (),
                move |bounds, _, window, _| {
                    let scale = f32::from(bounds.size.width) / GRIP;
                    let at = |(x, y): (f32, f32)| {
                        point(
                            bounds.origin.x + px(x * scale),
                            bounds.origin.y + px(y * scale),
                        )
                    };
                    for (from, to) in STROKES {
                        let mut path = PathBuilder::stroke(px(1.));
                        path.move_to(at(from));
                        path.line_to(at(to));
                        if let Ok(path) = path.build() {
                            window.paint_path(path, stroke);
                        }
                    }
                },
            )
            .size_full(),
        )
}

/// Resize `store` while its grip is dragged. Attach to the field element
/// whose height is dragged; `min` is its `min-block-size` in Carbon px.
pub fn on_drag_move(
    store: &HeightStore,
    min: f32,
) -> impl Fn(&gpui_kit::DragMoveEvent<ResizeDrag>, &mut Window, &mut App) + 'static {
    let store = store.clone();
    move |event, window, cx| {
        if event.drag(cx).store != store.entity_id() {
            return;
        }
        // Screen px per Carbon px (rem = 16 Carbon px).
        let unit = f32::from(window.rem_size()) / crate::tokens::REM_PX;
        let bounds_height = f32::from(event.bounds.size.height);
        let pointer_y = f32::from(event.event.position.y);
        store.update(cx, |r, cx| {
            let start_height = *r.start.height.get_or_insert(bounds_height);
            let delta = pointer_y - r.start.pointer_y;
            let height = dragged_height(start_height / unit, delta / unit, min);
            if r.height != Some(height) {
                r.height = Some(height);
                cx.notify();
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drag_follows_pointer_and_stops_at_min_block_size() {
        assert_eq!(dragged_height(100., 24., 40.), 124.);
        assert_eq!(dragged_height(100., -30., 40.), 70.);
        // `min-block-size: 2.5rem`.
        assert_eq!(dragged_height(100., -90., 40.), 40.);
    }

    #[test]
    fn grip_strokes_sit_in_the_corner_box() {
        for ((x0, y0), (x1, y1)) in STROKES {
            for v in [x0, y0, x1, y1] {
                assert!((0. ..=GRIP).contains(&v));
            }
            // Diagonal, rising to the right.
            assert_eq!(x1 - x0, y0 - y1);
        }
    }
}
