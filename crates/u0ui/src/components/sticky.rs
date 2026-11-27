//! StickyScroll: a vertical scroller whose sections keep their header pinned
//! at the top while the section scrolls by, as CSS `position: sticky; top: 0`
//! does (`.t3-dfh { position: sticky; top: 0; z-index: 1 }` on a diff's file
//! headers).
//!
//! GPUI has no sticky positioning, so this is written here on top of a
//! [`ScrollArea`] (Kit's scrolling, Carbon's scrollbar and keyboard
//! scrolling):
//!
//! - Each section is a header over a body. The header is drawn as an
//!   absolutely positioned layer painted after the body (`z-index: 1`), over
//!   an in-flow spacer of the header's height, so the body lays out exactly
//!   as it would below an in-flow header.
//! - Each section measures its box at prepaint (in content coordinates, so
//!   the measurement does not move with the scroll offset). On render, the
//!   header is pushed down by [`sticky_offset`] of the current scroll offset:
//!   it sticks to the viewport top while its section is in view and leaves
//!   with the section's bottom edge, the way a sticky box is held inside its
//!   containing block.
//! - Scrolling re-renders the owning view (GPUI notifies it on a scroll
//!   offset change), so the header follows the current offset. When the
//!   sections' layout changes (a file collapses), the next frame is drawn
//!   again with the new measurements.
//!
//! ```ignore
//! StickyScroll::new("files")
//!     .header_height(40.)
//!     .section(DiffFileHeader::new("h0", "src/a.rs"), DiffView::new("d0").hunks(a))
//!     .section(DiffFileHeader::new("h1", "src/b.rs"), DiffView::new("d1").hunks(b))
//!     .h(rems(24.))
//! ```
use super::scroll_area::ScrollArea;
use gpui_kit::{
    AnyElement, App, Bounds, ElementId, IntoElement, ParentElement, Pixels, RenderOnce,
    ScrollHandle, StyleRefinement, Styled, Window, canvas, div, px, rems,
};
use std::{cell::RefCell, rc::Rc};

/// How far a sticky header is pushed down from its section's top, in px:
/// `scroll − top`, held between 0 (the section has not reached the viewport
/// top) and `section − header` (the section's bottom edge takes it away).
/// `top` is the section's offset in the scrolled content, `scroll` the
/// content's scroll offset (positive down).
pub fn sticky_offset(scroll: f32, top: f32, section: f32, header: f32) -> f32 {
    (scroll - top).min(section - header).max(0.)
}

/// One measured section: its top and height in content coordinates.
type Measure = Option<(f32, f32)>;

#[derive(Default)]
struct State {
    scroll: ScrollHandle,
    sections: Rc<RefCell<Vec<Measure>>>,
}

/// A scroller of sections with sticky headers. See the module docs.
#[derive(IntoElement)]
pub struct StickyScroll {
    id: ElementId,
    header_height: f32,
    sections: Vec<(AnyElement, AnyElement)>,
    style: StyleRefinement,
}

impl StickyScroll {
    /// An empty scroller with a stable `id`. Give it a height (it scrolls
    /// inside it) with the `Styled` methods.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            header_height: 40.,
            sections: Vec::new(),
            style: StyleRefinement::default(),
        }
    }

    /// The headers' height in Carbon px (rem-relative; default 40, `size-md`).
    /// Every header is drawn at this height.
    pub fn header_height(mut self, px_value: f32) -> Self {
        self.header_height = px_value;
        self
    }

    /// Append a section: `header` stays pinned while `body` scrolls by.
    pub fn section(mut self, header: impl IntoElement, body: impl IntoElement) -> Self {
        self.sections
            .push((header.into_any_element(), body.into_any_element()));
        self
    }
}

impl Styled for StickyScroll {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for StickyScroll {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id;
        let state = window.use_keyed_state((id.clone(), "sticky"), cx, |_, _| State::default());
        let (handle, measured) = {
            let s = state.read(cx);
            (s.scroll.clone(), s.sections.clone())
        };
        let count = self.sections.len();
        measured.borrow_mut().resize(count, None);
        let scroll = -f32::from(handle.offset().y);
        let header_px = self.header_height / crate::tokens::REM_PX * f32::from(window.rem_size());
        let header_rems = rems(self.header_height / crate::tokens::REM_PX);

        let sections = self
            .sections
            .into_iter()
            .enumerate()
            .map(|(i, (header, body))| {
                let shift = measured.borrow()[i]
                    .map(|(top, height)| sticky_offset(scroll, top, height, header_px))
                    .unwrap_or(0.);
                let measure = {
                    let measured = measured.clone();
                    let handle = handle.clone();
                    canvas(
                        move |bounds: Bounds<Pixels>, window, _| {
                            // Content coordinates: undo the offset the bounds
                            // were laid out with.
                            let top = f32::from(bounds.origin.y)
                                - f32::from(handle.bounds().origin.y)
                                - f32::from(handle.offset().y);
                            let next = Some((top, f32::from(bounds.size.height)));
                            let mut all = measured.borrow_mut();
                            if let Some(slot) = all.get_mut(i)
                                && *slot != next
                            {
                                *slot = next;
                                window.on_next_frame(|window, _| window.refresh());
                            }
                        },
                        |_, _, _, _| {},
                    )
                    .absolute()
                    .top_0()
                    .left_0()
                    .size_full()
                };
                div()
                    .relative()
                    .flex()
                    .flex_col()
                    .flex_none()
                    .child(measure)
                    .child(div().flex_none().h(header_rems))
                    .child(body)
                    // Painted after the body: `z-index: 1`.
                    .child(
                        div()
                            .absolute()
                            .left_0()
                            .right_0()
                            .top(px(shift))
                            .h(header_rems)
                            .child(header),
                    )
            });

        let mut area = ScrollArea::new(id)
            .track_scroll(&handle)
            .focusable(false)
            .children(sections);
        gpui_kit::Refineable::refine(area.style(), &self.style);
        area
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headers_stick_inside_their_section() {
        // Not reached yet: stays at the section top.
        assert_eq!(sticky_offset(0., 200., 300., 40.), 0.);
        // Scrolled 50px into the section: pinned to the viewport top.
        assert_eq!(sticky_offset(250., 200., 300., 40.), 50.);
        // The section's bottom pushes it out (never past 300 − 40).
        assert_eq!(sticky_offset(480., 200., 300., 40.), 260.);
        // A section shorter than its header never moves.
        assert_eq!(sticky_offset(100., 0., 30., 40.), 0.);
    }
}
