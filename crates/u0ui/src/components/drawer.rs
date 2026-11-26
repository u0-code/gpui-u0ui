//! Drawer: a bottom sheet with a drag handle (`.cu-panel--drawer`), for touch
//! and narrow layouts. Same props as [`Sheet`], `side = bottom` by default
//! (`index.jsx`: `Drawer = ({ side = 'bottom', ...p }) => <Panel kind="drawer" … />`).
//!
//! The handle is `.cu-drawer-handle`: 3rem × 4px, `border-strong-01`,
//! 0.5rem from the top edge, centered. Motion is the Sheet's: the panel slides
//! up from `translateY(100%)` over 240ms `cubic-bezier(0, 0, 0.3, 1)`
//! (`cu-in-b`) while the scrim fades in over 110ms.
//!
//! ```ignore
//! Drawer::new("filters")
//!     .title("Filter results")
//!     .footer(Button::new("reset", "Reset").variant(ButtonVariant::Secondary))
//!     .footer(Button::new("apply", "Apply"))
//! ```
use super::sheet::{Sheet, SheetSide};
use gpui_kit::ElementId;

/// The Drawer constructor; it returns a configured [`Sheet`], so every Sheet
/// builder applies.
pub struct Drawer;

impl Drawer {
    /// A drawer with a stable `id`: a bottom [`Sheet`] with the drag handle.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<ElementId>) -> Sheet {
        Sheet::new(id).side(SheetSide::Bottom).drag_handle(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drawer_is_a_bottom_sheet_with_a_handle() {
        // `Drawer = ({ side = 'bottom', ...p }) => <Panel kind="drawer" … />`.
        assert_eq!(
            Drawer::new("d").side_and_handle(),
            (SheetSide::Bottom, true)
        );
        assert!(!Sheet::new("s").side_and_handle().1);
    }
}
