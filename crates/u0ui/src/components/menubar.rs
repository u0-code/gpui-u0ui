//! Menubar: a desktop-style persistent menu bar (`.cu-menubar`). Each menu is
//! the Carbon `Menu` of [`super::menu`] (size `sm`) hanging under its
//! trigger.
//!
//! - **Behavior**: pressing a trigger (pointer, Enter, Space, ↓) toggles its
//!   menu; while a menu is open, pointing at another trigger switches to it
//!   and ←/→ inside a menu move to the neighbouring menu (wrapping); ←/→ on a
//!   focused trigger move focus along the bar. Escape closes and returns focus
//!   to the trigger. Menu keyboard handling is the menu's.
//! - **Appearance** (`custom.css`): a `size-md` (2.5rem) bar on `layer-01`
//!   with a 1px `border-subtle-01` bottom line; triggers fill the height with
//!   1rem inline padding, `text-primary`, `layer-hover-01` on hover and while
//!   their menu is open (70ms), and a 2px inset `focus` outline.
//!
//! ```ignore
//! Menubar::new("app")
//!     .menu("File", vec![MenuItem::item("New tab").shortcut("⌘T")])
//!     .menu("Help", vec![MenuItem::item("Documentation")])
//! ```
use super::menu::{self, MenuItem, MenuSize};
use super::overlay::{self, rem};
use crate::{
    motion::{self, Motion},
    theme::color,
    tokens::{ColorRole, SIZE_MD, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    App, CursorStyle, ElementId, Entity, FocusHandle, Hsla, InteractiveElement, IntoElement,
    KeyDownEvent, MouseButton, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement as _, Styled, TestSupportExt as _, Window, div,
    prelude::FluentBuilder as _,
};
use std::rc::Rc;

/// The bar's colors.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MenubarLook {
    /// `.cu-menubar { background: var(--layer-01) }`.
    pub background: Hsla,
    /// `border-bottom: 1px solid var(--border-subtle-01)`.
    pub border: Hsla,
    /// `.cu-menubar-trigger { color: var(--text-primary) }`.
    pub text: Hsla,
    /// `.cu-menubar-trigger:hover, --open { background: var(--layer-hover-01) }`.
    pub hover: Hsla,
}

impl MenubarLook {
    /// The look on the active theme.
    pub fn current() -> Self {
        Self {
            background: color(ColorRole::Layer01),
            border: color(ColorRole::BorderSubtle01),
            text: color(ColorRole::TextPrimary),
            hover: color(ColorRole::LayerHover01),
        }
    }
}

/// `.cu-menubar { height: var(--size-md) }`.
pub const BAR_HEIGHT: f32 = SIZE_MD.px_value();

/// The neighbour of `index` in `count` menus, `direction` ±1, wrapping.
pub fn neighbour(index: usize, count: usize, direction: isize) -> usize {
    (index as isize + direction).rem_euclid(count.max(1) as isize) as usize
}

/// A menu bar. See the module docs.
#[derive(IntoElement)]
pub struct Menubar {
    id: ElementId,
    menus: Vec<(SharedString, Vec<MenuItem>)>,
    default_open: Option<usize>,
}

struct BarState {
    open: Option<usize>,
    hovered: Option<usize>,
    focus: Vec<FocusHandle>,
}

impl Menubar {
    /// A menu bar with a stable `id`.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            menus: Vec::new(),
            default_open: None,
        }
    }

    /// Append a menu (`MenubarMenu label`) with its items.
    pub fn menu(mut self, label: impl Into<SharedString>, items: Vec<MenuItem>) -> Self {
        self.menus.push((label.into(), items));
        self
    }

    /// Open the menu at `index` on arrival (`defaultOpen`).
    pub fn default_open(mut self, index: usize) -> Self {
        self.default_open = Some(index);
        self
    }
}

fn set_open(state: &Entity<BarState>, open: Option<usize>, window: &mut Window, cx: &mut App) {
    state.update(cx, |s, cx| {
        s.open = open;
        cx.notify();
    });
    window.refresh();
}

/// Moving to another menu while one is open: focus that menu's trigger
/// first, so the closing menu sees focus already gone elsewhere and does not
/// pull it back to its own trigger (whichever of the two renders last would
/// win), and the opening menu returns to its own trigger when it closes.
fn switch_focus(trigger: &FocusHandle, window: &mut Window, cx: &mut App) {
    trigger.focus(window, cx);
}

impl RenderOnce for Menubar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let count = self.menus.len();
        let default_open = self.default_open;
        let state = window.use_keyed_state((id.clone(), "bar"), cx, |_, _| BarState {
            open: default_open,
            hovered: None,
            focus: Vec::new(),
        });
        if state.read(cx).focus.len() < count {
            state.update(cx, |s, cx| {
                while s.focus.len() < count {
                    s.focus.push(cx.focus_handle());
                }
            });
        }
        let look = MenubarLook::current();
        let (open, hovered, focus) = {
            let s = state.read(cx);
            (s.open, s.hovered, s.focus.clone())
        };

        let mut bar = div()
            .id(id.clone())
            .flex()
            .flex_row()
            .h(rem(BAR_HEIGHT))
            .bg(look.background)
            .border_b_1()
            .border_color(look.border);
        for (index, (label, items)) in self.menus.into_iter().enumerate() {
            let menu_id = ElementId::NamedChild(
                std::sync::Arc::new(id.clone()),
                SharedString::from(format!("menu-{index}")),
            );
            let is_open = open == Some(index);
            let target = if is_open || hovered == Some(index) {
                look.hover
            } else {
                Hsla::transparent_black()
            };
            let bg =
                motion::animate_color((menu_id.clone(), "bg"), target, Motion::HOVER, window, cx);
            let focused = focus[index].is_focused(window);
            let trigger = div()
                .id((menu_id.clone(), "trigger"))
                .test_support()
                .relative()
                .h_full()
                .px(rem(16.))
                .flex()
                .items_center()
                .bg(bg)
                .text_color(look.text)
                .type_style(TypeStyle::BodyCompact01)
                .cursor(CursorStyle::PointingHand)
                .track_focus(&focus[index].clone().tab_stop(true))
                .on_hover({
                    let state = state.clone();
                    move |hovering, window, cx| {
                        let switched = state.update(cx, |s, cx| {
                            let mut switched = None;
                            if *hovering {
                                s.hovered = Some(index);
                                // A menubar follows the pointer once a menu is open.
                                if s.open.is_some() && s.open != Some(index) {
                                    s.open = Some(index);
                                    switched = s.focus.get(index).cloned();
                                }
                            } else if s.hovered == Some(index) {
                                s.hovered = None;
                            }
                            cx.notify();
                            switched
                        });
                        if let Some(trigger) = switched {
                            switch_focus(&trigger, window, cx);
                        }
                        window.refresh();
                    }
                })
                .on_mouse_down(MouseButton::Left, {
                    let state = state.clone();
                    move |_, window, cx| {
                        set_open(&state, (!is_open).then_some(index), window, cx);
                        cx.stop_propagation();
                    }
                })
                .on_key_down({
                    let state = state.clone();
                    let focus = focus.clone();
                    move |event: &KeyDownEvent, window, cx| match event.keystroke.key.as_str() {
                        "enter" | "space" | "down" => {
                            set_open(&state, Some(index), window, cx);
                            cx.stop_propagation();
                        }
                        "left" | "right" => {
                            let dir = if event.keystroke.key == "left" { -1 } else { 1 };
                            focus[neighbour(index, count, dir)].focus(window, cx);
                            cx.stop_propagation();
                        }
                        _ => {}
                    }
                })
                .when(focused, |this| {
                    // `.cu-menubar-trigger:focus-visible { outline: 2px solid var(--focus); outline-offset: -2px }`.
                    this.child(
                        div()
                            .absolute()
                            .top_0()
                            .left_0()
                            .size_full()
                            .border_2()
                            .border_color(color(ColorRole::Focus)),
                    )
                })
                .child(label.clone())
                .into_any_element();
            let set: overlay::OpenChange = {
                let state = state.clone();
                Rc::new(move |open, window, cx| {
                    let current = state.read(cx).open;
                    if open {
                        set_open(&state, Some(index), window, cx);
                    } else if current == Some(index) {
                        set_open(&state, None, window, cx);
                    }
                })
            };
            let hook: menu::KeyHook = {
                let state = state.clone();
                let focus = focus.clone();
                Rc::new(move |key, window, cx| match key {
                    "left" | "right" => {
                        let dir = if key == "left" { -1 } else { 1 };
                        let next = neighbour(index, count, dir);
                        switch_focus(&focus[next], window, cx);
                        set_open(&state, Some(next), window, cx);
                        true
                    }
                    _ => false,
                })
            };
            bar = bar.child(menu::anchored_menu_full_height(
                &menu_id,
                trigger,
                focus[index].clone(),
                menu::MenuSpec {
                    items,
                    size: MenuSize::Sm,
                    label,
                    open: is_open,
                    set_open: set,
                    min_width: None,
                    on_unhandled_key: Some(hook),
                },
                window,
                cx,
            ));
        }
        bar
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    fn hex(c: Hsla) -> String {
        let c = c.to_rgb();
        let b = |v: f32| (v * 255.).round() as u8;
        format!("#{:02x}{:02x}{:02x}", b(c.r), b(c.g), b(c.b))
    }

    #[test]
    fn bar_follows_cu_menubar() {
        crate::theme::tests_set(CarbonTheme::White);
        let look = MenubarLook::current();
        assert_eq!(hex(look.background), "#f4f4f4");
        assert_eq!(hex(look.hover), "#e8e8e8");
        assert_eq!(BAR_HEIGHT, 40.);
    }

    #[test]
    fn arrows_wrap_between_menus() {
        assert_eq!(neighbour(0, 3, -1), 2);
        assert_eq!(neighbour(2, 3, 1), 0);
        assert_eq!(neighbour(1, 3, 1), 2);
    }
}
