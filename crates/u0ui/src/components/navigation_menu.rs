//! NavigationMenu: a site's top navigation → Carbon UI shell `Header` +
//! `HeaderName` + `HeaderNavigation` + `HeaderMenuItem` / `HeaderMenu`
//! (`.cds--header`). Always laid out inline (the reference's default
//! `inline`), so it can sit in a page or at the top of a window.
//!
//! - **Behavior**: links and menu titles are Base `Button`s (focus,
//!   Enter/Space, pointer). A `HeaderMenu` opens under its title on press or
//!   ↓; inside it ↑/↓ move between items, Enter runs one, Escape (or a press
//!   outside) closes and returns focus to the title. Placement is Base's
//!   `Popup`; open state and focus moves come from [`super::overlay`].
//! - **Appearance**: a 3rem bar on `background` with a 1px `border-subtle`
//!   bottom line; the name in body-compact-01 600 with a 400 prefix, padded
//!   `0 2rem 0 1rem`; the navigation after a 1.5rem `border-subtle` rule;
//!   items `text-secondary` on `background`, hover `background-hover` +
//!   `text-primary`, press `background-active`, focus a 2px `focus` border,
//!   the current item `text-primary` with a 3px `border-interactive` bottom
//!   bar. An open menu title turns `layer`, its chevron (`icon-secondary`)
//!   turns 180°; the menu is 12.5rem wide on `layer` with `0 4px 8px
//!   rgba(0,0,0,.5)`, 3rem items, hover `layer-hover`, the current one
//!   `layer-selected` with a 3px start bar. Global actions sit at the end.
//! - **Motion**: `transition: background-color 110ms, border-color 110ms,
//!   color 110ms`; chevron `transform 110ms`; the menu shows and hides at
//!   once (`.cds--header__menu { display: none }` → `flex`, no transition).
use super::overlay::{Callback, rem};
use crate::{
    icons::{CarbonIcon, Icon, IconSize},
    motion::{self, Curve, Motion, Speed},
    theme::color,
    tokens::{ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    Anchor, AnyElement, App, BoxShadow, CursorStyle, ElementId, Entity, FocusHandle, FontWeight,
    Hsla, InteractiveElement, IntoElement, KeyDownEvent, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement as _, Styled, Window, div, hsla, prelude::FluentBuilder as _, px,
};
use std::rc::Rc;

/// `a.cds--header__menu-item { transition: background-color 110ms, border-color 110ms, color 110ms }`
/// (CSS `ease`, the closest Carbon curve being productive standard).
pub const ITEM_MOTION: Motion = Motion::new(Curve::ProductiveStandard, Speed::Fast02);
/// `.cds--header { block-size: 3rem }`.
pub const HEADER_HEIGHT: f32 = 48.;
/// `.cds--header__menu { inline-size: 12.5rem }`.
pub const MENU_WIDTH: f32 = 200.;

/// One navigation entry (`index.d.ts` `NavItem`: `label`, `active`, `items`).
#[derive(Clone)]
pub struct NavItem {
    label: SharedString,
    active: bool,
    on_click: Option<Callback>,
    items: Vec<NavItem>,
}

impl NavItem {
    /// A link (`HeaderMenuItem`).
    pub fn link(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            active: false,
            on_click: None,
            items: Vec::new(),
        }
    }

    /// A dropdown (`HeaderMenu`) with links.
    pub fn menu(label: impl Into<SharedString>, items: impl IntoIterator<Item = NavItem>) -> Self {
        Self {
            items: items.into_iter().collect(),
            ..Self::link(label)
        }
    }

    /// The current page (`isActive`).
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// Navigate when activated (the reference's `href`).
    pub fn on_click(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
}

/// `(background, text)` of a header item.
pub fn item_colors(in_menu: bool, current: bool, hovered: bool, pressed: bool) -> (Hsla, Hsla) {
    use ColorRole::*;
    if in_menu {
        let bg = match (current, hovered, pressed) {
            (_, _, true) => LayerActive01,
            (true, true, _) => LayerSelectedHover01,
            (true, false, _) => LayerSelected01,
            (false, true, _) => LayerHover01,
            _ => Layer01,
        };
        let text = if hovered || pressed || current {
            TextPrimary
        } else {
            TextSecondary
        };
        return (color(bg), color(text));
    }
    let bg = if pressed {
        BackgroundActive
    } else if hovered {
        BackgroundHover
    } else {
        Background
    };
    let text = if hovered || pressed || current {
        TextPrimary
    } else {
        TextSecondary
    };
    (color(bg), color(text))
}

type KeyHandler = Rc<dyn Fn(&KeyDownEvent, &mut Window, &mut App)>;
/// Open or close a header menu, optionally focusing an item.
type SetMenu = Rc<dyn Fn(bool, Option<usize>, &mut Window, &mut App)>;

/// How far the menu shadow reaches past the menu's sides and bottom (4px
/// offset + 8px blur), so the clip that keeps it off the title spares it.
const MENU_SHADOW_REACH: f32 = 12.;

/// The header menu's shadow: `box-shadow: 0 4px 8px 0 rgba(0, 0, 0, 0.5)`.
pub fn menu_shadow() -> Vec<BoxShadow> {
    vec![BoxShadow::new(px(0.), px(4.), hsla(0., 0., 0., 0.5)).blur_radius(px(8.))]
}

/// The UI shell header. See the module docs.
#[derive(IntoElement)]
pub struct NavigationMenu {
    id: ElementId,
    brand_prefix: SharedString,
    brand: SharedString,
    on_brand_click: Option<Callback>,
    items: Vec<NavItem>,
    actions: Vec<AnyElement>,
}

impl NavigationMenu {
    /// A header with a stable `id`; pass the name with [`Self::brand`].
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            brand_prefix: SharedString::default(),
            brand: SharedString::default(),
            on_brand_click: None,
            items: Vec::new(),
            actions: Vec::new(),
        }
    }

    /// The name prefix (`brandPrefix`, regular weight).
    pub fn brand_prefix(mut self, prefix: impl Into<SharedString>) -> Self {
        self.brand_prefix = prefix.into();
        self
    }

    /// The product name (`brand`, semibold).
    pub fn brand(mut self, brand: impl Into<SharedString>) -> Self {
        self.brand = brand.into();
        self
    }

    /// Run when the name is activated (`href`).
    pub fn on_brand_click(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_brand_click = Some(Rc::new(f));
        self
    }

    /// Append a navigation item.
    pub fn item(mut self, item: NavItem) -> Self {
        self.items.push(item);
        self
    }

    /// Append a global action (`HeaderGlobalBar`), e.g. a 48px icon button.
    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.actions.push(action.into_any_element());
        self
    }
}

/// Pointer state of one header item.
#[derive(Default)]
struct Pointer {
    hovered: bool,
    pressed: bool,
}

#[allow(clippy::too_many_arguments)]
fn header_item(
    id: ElementId,
    label: SharedString,
    current: bool,
    in_menu: bool,
    expanded: Option<bool>,
    focus: FocusHandle,
    on_click: Callback,
    on_key: Option<KeyHandler>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let pointer: Entity<Pointer> =
        window.use_keyed_state((id.clone(), "pointer"), cx, |_, _| Pointer::default());
    let (hovered, pressed) = {
        let p = pointer.read(cx);
        (p.hovered, p.pressed)
    };
    let (mut bg, text) = item_colors(in_menu, current, hovered, pressed);
    if expanded == Some(true) {
        // `.cds--header__menu-title[aria-expanded=true] { background-color: var(--layer) }`.
        bg = color(ColorRole::Layer01);
    }
    let bg = motion::animate_color((id.clone(), "bg"), bg, ITEM_MOTION, window, cx);
    let text = motion::animate_color((id.clone(), "text"), text, ITEM_MOTION, window, cx);
    let focused = focus.is_focused(window);
    let rotation =
        expanded.map(|open| motion::chevron_rotation((id.clone(), "arrow"), open, window, cx));
    gpui_kit::base::Button::new(id.clone())
        .track_focus(&focus)
        .accessibility_label(label.clone())
        .relative()
        // `.cds--header__menu-item { block-size: 100% }` of the header's
        // content box (3rem minus its 1px bottom border), so the current bar
        // sits fully above that border; submenu items are 3rem.
        .map(|this| {
            if in_menu {
                this.h(rem(HEADER_HEIGHT)).w_full()
            } else {
                this.h_full()
            }
        })
        .flex()
        .flex_row()
        .items_center()
        .justify_start()
        .px(rem(16.))
        .bg(bg)
        .text_color(text)
        .type_style(TypeStyle::BodyCompact01)
        .whitespace_nowrap()
        // `border: 2px solid transparent`, `:focus { border-color: var(--focus) }`.
        .border_2()
        .border_color(if focused {
            color(ColorRole::Focus)
        } else {
            Hsla::transparent_black()
        })
        .cursor(CursorStyle::PointingHand)
        .on_hover({
            let pointer = pointer.clone();
            move |h, _, cx| {
                pointer.update(cx, |p, cx| {
                    p.hovered = *h;
                    if !*h {
                        p.pressed = false;
                    }
                    cx.notify();
                })
            }
        })
        .on_mouse_down(gpui_kit::MouseButton::Left, {
            let pointer = pointer.clone();
            move |_, _, cx| {
                pointer.update(cx, |p, cx| {
                    p.pressed = true;
                    cx.notify();
                })
            }
        })
        .on_mouse_up(gpui_kit::MouseButton::Left, move |_, _, cx| {
            pointer.update(cx, |p, cx| {
                p.pressed = false;
                cx.notify();
            })
        })
        .on_click(move |_, window, cx| on_click(window, cx))
        .when_some(on_key, |this, on_key| {
            this.on_key_down(move |event, window, cx| on_key(event, window, cx))
        })
        .child(label)
        .when_some(rotation, |this, rotation| {
            this.child(
                div().ml(rem(8.)).child(
                    Icon::carbon(CarbonIcon::ChevronDown)
                        .size(IconSize::S16)
                        .color(color(ColorRole::IconSecondary))
                        .rotate(gpui_kit::radians(rotation)),
                ),
            )
        })
        .when(current, |this| {
            this.child(if in_menu {
                // Submenu current: a 3px start bar.
                div()
                    .absolute()
                    .top(px(-2.))
                    .bottom(px(-2.))
                    .left(px(-2.))
                    .w(px(3.))
                    .bg(color(ColorRole::BorderInteractive))
            } else {
                // `--current::after { block-size: 3px; inset-block-end: -2px }`.
                div()
                    .absolute()
                    .left(px(-2.))
                    .right(px(-2.))
                    .bottom(px(-2.))
                    .h(px(3.))
                    .bg(color(ColorRole::BorderInteractive))
            })
        })
        .into_any_element()
}

/// Open state and focus handles of one header menu.
struct MenuState {
    open: bool,
    title: FocusHandle,
    items: Vec<FocusHandle>,
}

impl RenderOnce for NavigationMenu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let mut nav = div().flex().flex_row().h_full();
        for (index, item) in self.items.into_iter().enumerate() {
            let item_id = ElementId::NamedChild(
                std::sync::Arc::new(id.clone()),
                SharedString::from(format!("item-{index}")),
            );
            if item.items.is_empty() {
                let focus = window
                    .use_keyed_state((item_id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
                    .read(cx)
                    .clone();
                let on_click = item.on_click.clone();
                nav = nav.child(header_item(
                    item_id,
                    item.label,
                    item.active,
                    false,
                    None,
                    focus,
                    Rc::new(move |window, cx| {
                        if let Some(f) = &on_click {
                            f(window, cx)
                        }
                    }),
                    None,
                    window,
                    cx,
                ));
                continue;
            }
            let count = item.items.len();
            let state = window.use_keyed_state((item_id.clone(), "menu"), cx, |_, cx| MenuState {
                open: false,
                title: cx.focus_handle(),
                items: Vec::new(),
            });
            if state.read(cx).items.len() < count {
                state.update(cx, |s, cx| {
                    while s.items.len() < count {
                        s.items.push(cx.focus_handle());
                    }
                });
            }
            let (open, title_focus, item_focus) = {
                let s = state.read(cx);
                (s.open, s.title.clone(), s.items.clone())
            };
            let set_open: SetMenu = {
                let state = state.clone();
                Rc::new(move |open, focus_item, window, cx| {
                    let (title, items) = state.update(cx, |s, cx| {
                        s.open = open;
                        cx.notify();
                        (s.title.clone(), s.items.clone())
                    });
                    match (open, focus_item) {
                        (true, Some(i)) => {
                            let handle = items[i].clone();
                            window.defer(cx, move |window, cx| handle.focus(window, cx));
                        }
                        (false, _) if items.iter().any(|h| h.is_focused(window)) => {
                            title.focus(window, cx)
                        }
                        _ => {}
                    }
                    window.refresh();
                })
            };
            let title = header_item(
                (item_id.clone(), "title").into(),
                item.label.clone(),
                item.active || item.items.iter().any(|i| i.active),
                false,
                Some(open),
                title_focus,
                {
                    let set_open = set_open.clone();
                    Rc::new(move |window, cx| set_open(!open, None, window, cx))
                },
                Some({
                    let set_open = set_open.clone();
                    Rc::new(
                        move |event: &KeyDownEvent, window: &mut Window, cx: &mut App| {
                            if event.keystroke.key == "down" {
                                set_open(true, Some(0), window, cx);
                                cx.stop_propagation();
                            }
                        },
                    )
                }),
                window,
                cx,
            );
            let popup = gpui_kit::base::Popup::new((item_id.clone(), "popup"), title)
                .anchor(Anchor::TopLeft)
                .h_full();
            if !open {
                nav = nav.child(popup);
                continue;
            }
            let mut menu = div()
                .id((item_id.clone(), "list"))
                .w(rem(MENU_WIDTH))
                .flex()
                .flex_col()
                .bg(color(ColorRole::Layer01))
                .shadow(menu_shadow())
                .on_mouse_down_out({
                    let set_open = set_open.clone();
                    move |_, window, cx| {
                        if open {
                            set_open(false, None, window, cx)
                        }
                    }
                });
            for (i, link) in item.items.into_iter().enumerate() {
                let on_click = link.on_click.clone();
                let set_close = set_open.clone();
                let set_key = set_open.clone();
                menu = menu.child(header_item(
                    ElementId::NamedChild(
                        std::sync::Arc::new(item_id.clone()),
                        SharedString::from(format!("link-{i}")),
                    ),
                    link.label,
                    link.active,
                    true,
                    None,
                    item_focus[i].clone(),
                    Rc::new(move |window, cx| {
                        if let Some(f) = &on_click {
                            f(window, cx)
                        }
                        set_close(false, None, window, cx);
                    }),
                    Some(Rc::new(
                        move |event: &KeyDownEvent, window: &mut Window, cx: &mut App| {
                            match event.keystroke.key.as_str() {
                                "down" => set_key(true, Some((i + 1) % count), window, cx),
                                "up" => set_key(true, Some((i + count - 1) % count), window, cx),
                                "escape" | "tab" => set_key(false, None, window, cx),
                                _ => return,
                            }
                            cx.stop_propagation();
                        },
                    )),
                    window,
                    cx,
                ));
            }
            // The open title sits above its menu (`[aria-expanded=true]`
            // z-index 8002 over the menu's 8001), so the menu's shadow never
            // falls on the title: clip the part above the menu's top edge.
            let pad = rem(MENU_SHADOW_REACH);
            let menu = div()
                .overflow_hidden()
                .ml(rem(-MENU_SHADOW_REACH))
                .pl(pad)
                .pr(pad)
                .pb(pad)
                .child(menu);
            nav = nav.child(popup.content(menu));
        }

        let brand_focus = window
            .use_keyed_state((id.clone(), "brand-focus"), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let brand_focused = brand_focus.is_focused(window);
        let on_brand = self.on_brand_click.clone();
        div()
            .id(id.clone())
            .w_full()
            .h(rem(HEADER_HEIGHT))
            .flex()
            .flex_row()
            .items_center()
            .bg(color(ColorRole::Background))
            .border_b_1()
            .border_color(color(ColorRole::BorderSubtle01))
            .child(
                // `a.cds--header__name`: padding 0 2rem 0 1rem, 2px focus border.
                gpui_kit::base::Button::new((id.clone(), "name"))
                    .track_focus(&brand_focus)
                    .accessibility_label(self.brand.clone())
                    .h_full()
                    .flex()
                    .flex_row()
                    .items_center()
                    .pl(rem(16.))
                    .pr(rem(32.))
                    .border_2()
                    .border_color(if brand_focused {
                        color(ColorRole::Focus)
                    } else {
                        Hsla::transparent_black()
                    })
                    .type_style(TypeStyle::BodyCompact01)
                    .text_color(color(ColorRole::TextPrimary))
                    .whitespace_nowrap()
                    .cursor(CursorStyle::PointingHand)
                    .on_click(move |_, window, cx| {
                        if let Some(f) = &on_brand {
                            f(window, cx)
                        }
                    })
                    .when(!self.brand_prefix.is_empty(), |this| {
                        this.child(div().mr(rem(4.)).child(self.brand_prefix.clone()))
                    })
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(self.brand.clone()),
                    ),
            )
            .child(
                // `.cds--header__nav { padding-inline-start: 1rem }` with a 1.5rem rule.
                div()
                    .relative()
                    .h_full()
                    .pl(rem(16.))
                    .child(
                        div()
                            .absolute()
                            .left_0()
                            .top(rem(12.))
                            .h(rem(24.))
                            .w(px(1.))
                            .bg(color(ColorRole::BorderSubtle01)),
                    )
                    .child(nav),
            )
            .child(
                // `.cds--header__global { flex: 1 1; justify-content: flex-end }`.
                div()
                    .flex_1()
                    .h_full()
                    .flex()
                    .flex_row()
                    .justify_end()
                    .items_center()
                    .children(self.actions),
            )
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
    fn item_colors_follow_header_css() {
        crate::theme::tests_set(CarbonTheme::White);
        let (bg, text) = item_colors(false, false, false, false);
        assert_eq!((hex(bg), hex(text)), ("#ffffff".into(), "#525252".into()));
        let (_, text) = item_colors(false, true, false, false);
        assert_eq!(hex(text), "#161616");
        let (bg, _) = item_colors(true, true, false, false);
        assert_eq!(hex(bg), "#e0e0e0");
        let (bg, _) = item_colors(true, false, true, false);
        assert_eq!(hex(bg), "#e8e8e8");
        assert_eq!((HEADER_HEIGHT, MENU_WIDTH), (48., 200.));
        let shadow = &menu_shadow()[0];
        assert_eq!((shadow.offset.y, shadow.blur_radius), (px(4.), px(8.)));
    }
}
