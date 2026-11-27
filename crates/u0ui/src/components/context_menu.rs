//! ContextMenu: a menu opened with a secondary click → Carbon `Menu` +
//! `useContextMenu` (the menu's top-left corner at the pointer).
//!
//! - **Behavior**: a right press inside the area opens the [`super::menu`]
//!   Carbon menu at the pointer; Shift+F10 or the Menu key opens it at the
//!   area's top-left while focus is inside; a press outside, Escape, Tab or an
//!   activated item closes it. Placement is Base's `Positioner` in a corner
//!   mode (clamped to the window, like Carbon's menu) on the deferred popup
//!   layer. Kit's `context_menu` wraps its own `PopupMenu`, whose rendering
//!   cannot be made Carbon (see [`super::menu`]).
//! - `default_open_at` opens it on arrival at a point relative to the area's
//!   top-left (the reference preview's `defaultOpenAt`).
//!
//! ```ignore
//! ContextMenu::new("file", area)
//!     .item(MenuItem::item("Copy").shortcut("⌘C"))
//!     .item(MenuItem::divider())
//!     .item(MenuItem::item("Delete").danger(true))
//! ```
use super::menu::{self, MenuItem, MenuSize};
use super::overlay;
use gpui_kit::{
    Anchor, AnyElement, App, Bounds, ElementId, InteractiveElement, IntoElement, KeyDownEvent,
    MouseButton, ParentElement, Pixels, Point, RenderOnce, SharedString, Styled,
    TestSupportExt as _, Window, canvas, deferred, div, point, prelude::FluentBuilder as _, px,
};
use std::rc::Rc;

/// The keys that open a context menu from the keyboard.
pub fn is_menu_key(key: &str, shift: bool) -> bool {
    key == "menu" || (key == "f10" && shift)
}

/// A context menu around an area. See the module docs.
#[derive(IntoElement)]
pub struct ContextMenu {
    id: ElementId,
    area: AnyElement,
    items: Vec<MenuItem>,
    label: SharedString,
    size: MenuSize,
    default_open_at: Option<(f32, f32)>,
}

/// Where the menu is and whether it is open.
#[derive(Default)]
struct Placed {
    open: bool,
    at: Point<Pixels>,
    area: Bounds<Pixels>,
    measured: bool,
    defaulted: bool,
}

impl ContextMenu {
    /// A context menu with a stable `id` for `area` (the right-click target).
    pub fn new(id: impl Into<ElementId>, area: impl IntoElement) -> Self {
        Self {
            id: id.into(),
            area: area.into_any_element(),
            items: Vec::new(),
            label: SharedString::default(),
            size: MenuSize::Sm,
            default_open_at: None,
        }
    }

    /// Append a menu item.
    pub fn item(mut self, item: MenuItem) -> Self {
        self.items.push(item);
        self
    }

    /// Append menu items.
    pub fn items(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.items.extend(items);
        self
    }

    /// The menu's accessible name.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    /// Menu size (default `sm`, as the reference passes).
    pub fn size(mut self, size: MenuSize) -> Self {
        self.size = size;
        self
    }

    /// Open on arrival at `(x, y)` px from the area's top-left (demos).
    pub fn default_open_at(mut self, x: f32, y: f32) -> Self {
        self.default_open_at = Some((x, y));
        self
    }
}

impl RenderOnce for ContextMenu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let placed = window.use_keyed_state((id.clone(), "placed"), cx, |_, _| Placed::default());
        if let Some((x, y)) = self.default_open_at {
            let p = placed.read(cx);
            if p.measured && !p.defaulted {
                let at = p.area.origin + point(px(x), px(y));
                placed.update(cx, |p, _| {
                    p.defaulted = true;
                    p.open = true;
                    p.at = at;
                });
            }
        }
        let area_focus = window
            .use_keyed_state((id.clone(), "area-focus"), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        // The area is a tab stop of ours (the reference's `.cu-context-area`
        // is not focusable), so keyboard focus shows the 2px `focus` ring
        // every focusable element draws.
        let ring = area_focus.is_focused(window) && crate::focus::focus_visible_mode(window);
        let (open, at) = {
            let p = placed.read(cx);
            (p.open, p.at)
        };
        let set_open: overlay::OpenChange = {
            let placed = placed.clone();
            Rc::new(move |open, window, cx| {
                placed.update(cx, |p, cx| {
                    p.open = open;
                    cx.notify();
                });
                window.refresh();
            })
        };
        let surface = menu::menu_surface(
            &id,
            menu::MenuSpec {
                items: self.items,
                size: self.size,
                label: self.label,
                open,
                set_open,
                min_width: None,
                on_unhandled_key: None,
            },
            window,
            cx,
        );
        let measure = {
            let placed = placed.clone();
            canvas(
                move |bounds, _, cx| {
                    let p = placed.read(cx);
                    if !p.measured || p.area != bounds {
                        placed.update(cx, |p, _| {
                            p.area = bounds;
                            p.measured = true;
                        });
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
            .id((id.clone(), "area"))
            .test_support()
            // The area is a tab stop so Shift+F10 / the Menu key can reach it.
            .track_focus(&area_focus.tab_stop(true))
            .relative()
            .on_mouse_down(MouseButton::Right, {
                let placed = placed.clone();
                move |event, window, cx| {
                    placed.update(cx, |p, cx| {
                        p.open = true;
                        p.at = event.position;
                        cx.notify();
                    });
                    window.refresh();
                    cx.stop_propagation();
                }
            })
            .on_key_down({
                let placed = placed.clone();
                move |event: &KeyDownEvent, window, cx| {
                    if is_menu_key(&event.keystroke.key, event.keystroke.modifiers.shift) {
                        placed.update(cx, |p, cx| {
                            p.open = true;
                            p.at = p.area.origin;
                            cx.notify();
                        });
                        window.refresh();
                        cx.stop_propagation();
                    }
                }
            })
            .child(self.area)
            .child(measure)
            .when(ring, |d| {
                d.child(super::form::inset_outline(
                    2.,
                    crate::theme::color(crate::tokens::ColorRole::Focus),
                ))
            })
            .children(surface.map(|surface| {
                deferred(
                    gpui_kit::base::Positioner::corner(Anchor::TopLeft, at)
                        .margin(px(8.))
                        .occlude()
                        .child(surface),
                )
                .with_priority(gpui_kit::base::POPUP_PRIORITY)
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_keys() {
        assert!(is_menu_key("menu", false));
        assert!(is_menu_key("f10", true));
        assert!(!is_menu_key("f10", false));
        assert_eq!(ContextMenu::new("c", div()).size, MenuSize::Sm);
    }
}
