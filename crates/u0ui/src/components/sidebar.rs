//! Sidebar: application side navigation → Carbon UI shell `SideNav` /
//! `SideNavItems` / `SideNavLink` / `SideNavMenu` / `SideNavMenuItem` /
//! `SideNavDivider` (`.cds--side-nav`), plus `SidebarProvider` /
//! `SidebarTrigger` / `SidebarInset` (`.cu-sb-*`). Always laid out inline (the
//! reference's default `inline`).
//!
//! - **Behavior**: links and menu headings are Base `Button`s (focus,
//!   Enter/Space, pointer); a `SideNavMenu` toggles its links (open by
//!   `expanded`). [`SidebarProvider`] owns the open state (controlled or not)
//!   and puts a [`SidebarTrigger`] (a ghost `md` icon button) at the top of
//!   the inset.
//! - **Appearance**: 16rem on `background` (`expanded(false)`: a 3rem icon
//!   rail), items from 1rem down; links 2rem tall, `text-secondary` 14/20 with
//!   a 16px `icon-secondary` icon 1.5rem before the text, hover
//!   `background-hover` + `text-primary`, the current link
//!   `background-selected` + 600 + a 3px `border-interactive` bar; menu
//!   headings heading-compact-01 with a 16px chevron, their links indented
//!   2rem (4.5rem under an icon), an active closed menu marked like a current
//!   link; dividers 1px `border-subtle` with `0.5rem 1rem` margins; the
//!   optional header (600, 1rem padding, bottom line) and footer (top line).
//!   Focus: a 2px inset `focus` outline.
//! - **Motion**: colors 110ms; the menu chevron turns 180° in 110ms; a menu's
//!   links show and hide at once (`.cds--side-nav__menu` has no height
//!   transition); the nav's width animates over 110ms
//!   `cubic-bezier(0.2, 0, 1, 0.9)` (`.cds--side-nav { transition: inline-size
//!   0.11s }`) when it collapses to the rail or hides (reduced motion: a cut).
use super::button::{Button, ButtonSize, ButtonVariant};
use super::overlay::{self, Callback, OpenChange, rem};
use crate::{
    icons::{CarbonIcon, Icon, IconSize},
    motion::{self, Curve, Motion, Speed},
    theme::color,
    tokens::{ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    AnyElement, App, CursorStyle, ElementId, FontWeight, Hsla, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _, px,
};
use std::rc::Rc;

/// `transition: color 110ms, background-color 110ms` (CSS `ease` ≈ productive standard).
pub const ITEM_MOTION: Motion = Motion::new(Curve::ProductiveStandard, Speed::Fast02);
/// `.cds--side-nav { transition: inline-size 0.11s cubic-bezier(0.2, 0, 1, 0.9) }`.
pub const WIDTH_MOTION: Motion = Motion::new(Curve::ProductiveExit, Speed::Fast02).spatial(true);
/// `.cds--side-nav--expanded { inline-size: 16rem }`.
pub const EXPANDED_WIDTH: f32 = 256.;
/// `.cds--side-nav { inline-size: 3rem }` (the rail).
pub const RAIL_WIDTH: f32 = 48.;

/// One side navigation entry (`index.d.ts` `NavItem`).
#[derive(Clone)]
pub struct SideNavItem {
    label: SharedString,
    icon: Option<Icon>,
    active: bool,
    expanded: bool,
    divider: bool,
    items: Vec<SideNavItem>,
    on_click: Option<Callback>,
}

impl SideNavItem {
    /// A link (`SideNavLink`, or `SideNavMenuItem` inside a menu).
    pub fn link(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            active: false,
            expanded: false,
            divider: false,
            items: Vec::new(),
            on_click: None,
        }
    }

    /// A menu (`SideNavMenu`) of links.
    pub fn menu(
        label: impl Into<SharedString>,
        items: impl IntoIterator<Item = SideNavItem>,
    ) -> Self {
        Self {
            items: items.into_iter().collect(),
            ..Self::link(label)
        }
    }

    /// A divider (`SideNavDivider`).
    pub fn divider() -> Self {
        Self {
            divider: true,
            ..Self::link("")
        }
    }

    /// Leading icon (`renderIcon`).
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// The current page (`isActive`).
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// A menu that starts open (`defaultExpanded`).
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    /// Navigate when activated (`href`).
    pub fn on_click(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
}

/// `(background, text, icon)` of a link or menu heading.
pub fn item_colors(current: bool, hovered: bool) -> (Hsla, Hsla, Hsla) {
    use ColorRole::*;
    if current {
        // `.cds--side-nav__link--current { background-color: var(--background-selected) }`.
        let bg = if hovered {
            BackgroundSelectedHover
        } else {
            BackgroundSelected
        };
        (color(bg), color(TextPrimary), color(IconPrimary))
    } else if hovered {
        (
            color(BackgroundHover),
            color(TextPrimary),
            color(IconPrimary),
        )
    } else {
        (
            Hsla::transparent_black(),
            color(TextSecondary),
            color(IconSecondary),
        )
    }
}

/// A Carbon side navigation. See the module docs.
#[derive(IntoElement)]
pub struct Sidebar {
    id: ElementId,
    items: Vec<SideNavItem>,
    label: SharedString,
    expanded: bool,
    header: Option<AnyElement>,
    footer: Option<AnyElement>,
    visible: bool,
}

impl Sidebar {
    /// A side nav with a stable `id`.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            items: Vec::new(),
            label: SharedString::default(),
            expanded: true,
            header: None,
            footer: None,
            visible: true,
        }
    }

    /// Append an entry.
    pub fn item(mut self, item: SideNavItem) -> Self {
        self.items.push(item);
        self
    }

    /// Append entries.
    pub fn items(mut self, items: impl IntoIterator<Item = SideNavItem>) -> Self {
        self.items.extend(items);
        self
    }

    /// The accessible name (`label`).
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    /// Full width (default) or the 3rem icon rail.
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    /// Content above the items (`.cu-sidenav-header`).
    pub fn header(mut self, header: impl IntoElement) -> Self {
        self.header = Some(header.into_any_element());
        self
    }

    /// Content below the items (`.cu-sidenav-footer`).
    pub fn footer(mut self, footer: impl IntoElement) -> Self {
        self.footer = Some(footer.into_any_element());
        self
    }

    fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
}

#[allow(clippy::too_many_arguments)]
fn row(
    id: ElementId,
    label: SharedString,
    icon: Option<Icon>,
    current: bool,
    heading: bool,
    indent: f32,
    show_text: bool,
    trailing: Option<AnyElement>,
    on_click: Callback,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let hover = window.use_keyed_state((id.clone(), "hover"), cx, |_, _| false);
    let focus = window
        .use_keyed_state((id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
        .read(cx)
        .clone();
    let focused = focus.is_focused(window);
    let (bg, text, icon_color) = item_colors(current, *hover.read(cx));
    let bg = motion::animate_color((id.clone(), "bg"), bg, ITEM_MOTION, window, cx);
    let text = motion::animate_color((id.clone(), "text"), text, ITEM_MOTION, window, cx);
    gpui_kit::base::Button::new(id)
        .track_focus(&focus)
        .accessibility_label(label.clone())
        .relative()
        .w_full()
        .h(rem(32.))
        .flex_shrink_0()
        .flex()
        .flex_row()
        .items_center()
        .justify_start()
        .pl(rem(16. + indent))
        .pr(rem(16.))
        .bg(bg)
        .text_color(text)
        .map(|this| {
            // `.cds--side-nav__link` is heading-compact-01 (600); only links in a
            // menu drop to 400 (`.cds--side-nav__menu .cds--side-nav__link`).
            if heading || current || indent == 0. {
                this.type_style(TypeStyle::HeadingCompact01)
            } else {
                this.type_style(TypeStyle::BodyCompact01)
                    .font_weight(FontWeight::NORMAL)
            }
        })
        .cursor(CursorStyle::PointingHand)
        .on_hover(move |h, _, cx| {
            hover.update(cx, |v, cx| {
                *v = *h;
                cx.notify();
            })
        })
        .on_click(move |_, window, cx| on_click(window, cx))
        .when_some(icon, |this, icon| {
            // `.cds--side-nav__icon { flex: 0 0 1rem; margin-inline-end: 1.5rem }`.
            this.child(
                div()
                    .flex_shrink_0()
                    .when(show_text, |this| this.mr(rem(24.)))
                    .child(icon.size(IconSize::S16).color(icon_color)),
            )
        })
        .when(show_text, |this| {
            this.child(
                div()
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .child(label),
            )
        })
        .children(trailing)
        .when(current, |this| {
            // `::before { inline-size: 3px; background-color: var(--border-interactive) }`.
            this.child(
                div()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .left_0()
                    .w(px(3.))
                    .bg(color(ColorRole::BorderInteractive)),
            )
        })
        .when(focused, |this| {
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
        .into_any_element()
}

impl RenderOnce for Sidebar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let target = if !self.visible {
            0.
        } else if self.expanded {
            EXPANDED_WIDTH
        } else {
            RAIL_WIDTH
        };
        let width = motion::animate((id.clone(), "width"), target, WIDTH_MOTION, window, cx);
        let show_text = self.expanded;
        let mut list = div()
            .id((id.clone(), "items"))
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .pt(rem(16.))
            .flex()
            .flex_col();
        for (index, item) in self.items.into_iter().enumerate() {
            let item_id = ElementId::NamedChild(
                std::sync::Arc::new(id.clone()),
                SharedString::from(format!("item-{index}")),
            );
            if item.divider {
                list = list.child(
                    div()
                        .flex_shrink_0()
                        .my(rem(8.))
                        .mx(rem(16.))
                        .h(px(1.))
                        .bg(color(ColorRole::BorderSubtle01)),
                );
                continue;
            }
            if item.items.is_empty() {
                let on_click = item.on_click.clone();
                list = list.child(row(
                    item_id,
                    item.label,
                    item.icon,
                    item.active,
                    false,
                    0.,
                    show_text,
                    None,
                    Rc::new(move |window, cx| {
                        if let Some(f) = &on_click {
                            f(window, cx)
                        }
                    }),
                    window,
                    cx,
                ));
                continue;
            }
            let default = item.expanded;
            let openness = overlay::openness(&item_id, None, default, None, window, cx);
            let open = openness.open && show_text;
            let set_open = openness.setter();
            let rotation = motion::chevron_rotation((item_id.clone(), "chevron"), open, window, cx);
            let child_active = item.items.iter().any(|i| i.active);
            let has_icon = item.icon.is_some();
            let chevron = show_text.then(|| {
                div()
                    .flex_1()
                    .flex()
                    .justify_end()
                    .child(
                        Icon::carbon(CarbonIcon::ChevronDown)
                            .size(IconSize::S16)
                            .color(color(ColorRole::IconSecondary))
                            .rotate(gpui_kit::radians(rotation)),
                    )
                    .into_any_element()
            });
            list = list.child(row(
                (item_id.clone(), "heading").into(),
                item.label.clone(),
                item.icon.clone(),
                // `--active .cds--side-nav__submenu[aria-expanded=false]`: marked while closed.
                (item.active || child_active) && !open,
                true,
                0.,
                show_text,
                chevron,
                Rc::new(move |window, cx| set_open(!open, window, cx)),
                window,
                cx,
            ));
            if open {
                let mut links = div().flex().flex_col();
                for (i, link) in item.items.into_iter().enumerate() {
                    let on_click = link.on_click.clone();
                    links = links.child(row(
                        ElementId::NamedChild(
                            std::sync::Arc::new(item_id.clone()),
                            SharedString::from(format!("link-{i}")),
                        ),
                        link.label,
                        None,
                        link.active,
                        false,
                        // `.cds--side-nav__menu .cds--side-nav__link { padding-inline-start: 2rem }`,
                        // 4.5rem under an icon.
                        if has_icon { 56. } else { 16. },
                        true,
                        None,
                        Rc::new(move |window, cx| {
                            if let Some(f) = &on_click {
                                f(window, cx)
                            }
                        }),
                        window,
                        cx,
                    ));
                }
                list = list.child(links);
            }
        }

        div()
            .id(id.clone())
            .flex_shrink_0()
            .h_full()
            .min_h(rem(320.))
            .w(rem(width))
            .overflow_hidden()
            .bg(color(ColorRole::Background))
            .border_r_1()
            .border_color(if width > 0.5 {
                color(ColorRole::BorderSubtle01)
            } else {
                Hsla::transparent_black()
            })
            .child(
                div()
                    .w(rem(target.max(width)))
                    .h_full()
                    .flex()
                    .flex_col()
                    .when_some(self.header.filter(|_| show_text), |this, header| {
                        // `.cu-sidenav-header { padding: spacing-05; font-weight: 600; border-bottom }`.
                        this.child(
                            div()
                                .flex_shrink_0()
                                .p(rem(16.))
                                .type_style(TypeStyle::HeadingCompact01)
                                .text_color(color(ColorRole::TextPrimary))
                                .border_b_1()
                                .border_color(color(ColorRole::BorderSubtle01))
                                .child(header),
                        )
                    })
                    .child(list)
                    .when_some(self.footer.filter(|_| show_text), |this, footer| {
                        this.child(
                            div()
                                .flex_shrink_0()
                                .p(rem(16.))
                                .border_t_1()
                                .border_color(color(ColorRole::BorderSubtle01))
                                .child(footer),
                        )
                    }),
            )
    }
}

/// `SidebarProvider` + `SidebarInset`: the side nav beside the page content,
/// with a [`SidebarTrigger`] at the top of the content. See the module docs.
#[derive(IntoElement)]
pub struct SidebarProvider {
    id: ElementId,
    sidebar: Option<Sidebar>,
    open: Option<bool>,
    default_open: bool,
    on_open_change: Option<OpenChange>,
    children: Vec<AnyElement>,
    open_label: SharedString,
    close_label: SharedString,
}

impl SidebarProvider {
    /// A provider with a stable `id`.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            sidebar: None,
            open: None,
            default_open: true,
            on_open_change: None,
            children: Vec::new(),
            // The reference trigger's labels; override per locale.
            open_label: "Open menu".into(),
            close_label: "Close menu".into(),
        }
    }

    /// The side nav.
    pub fn sidebar(mut self, sidebar: Sidebar) -> Self {
        self.sidebar = Some(sidebar);
        self
    }

    /// Controlled open state.
    pub fn open(mut self, open: bool) -> Self {
        self.open = Some(open);
        self
    }

    /// Initial open state when uncontrolled (default true).
    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }

    /// Called when the trigger toggles the side nav.
    pub fn on_open_change(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_open_change = Some(Rc::new(f));
        self
    }

    /// The trigger's accessible names when closed / open.
    pub fn trigger_labels(
        mut self,
        open: impl Into<SharedString>,
        close: impl Into<SharedString>,
    ) -> Self {
        self.open_label = open.into();
        self.close_label = close.into();
        self
    }
}

impl ParentElement for SidebarProvider {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

/// `SidebarTrigger`: a ghost `md` icon button with the Menu icon.
pub struct SidebarTrigger;

impl SidebarTrigger {
    /// The trigger button; `label` is its accessible name and tooltip.
    pub fn button(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Button {
        Button::new(id, label)
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Md)
            .icon(CarbonIcon::Menu)
            .icon_only(true)
    }
}

impl RenderOnce for SidebarProvider {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let openness = overlay::openness(
            &id,
            self.open,
            self.default_open,
            self.on_open_change.clone(),
            window,
            cx,
        );
        let open = openness.open;
        let set_open = openness.setter();
        let trigger = SidebarTrigger::button(
            (id.clone(), "trigger"),
            if open {
                self.close_label.clone()
            } else {
                self.open_label.clone()
            },
        )
        .on_click(move |_, window, cx| set_open(!open, window, cx));
        div()
            .flex()
            .flex_row()
            .size_full()
            .children(self.sidebar.map(|s| s.visible(open)))
            .child(
                // `.cu-sb-inset { flex: 1; min-width: 0; padding: spacing-05 }`.
                div()
                    .flex_1()
                    .min_w_0()
                    .p(rem(16.))
                    .flex()
                    .flex_col()
                    .items_start()
                    .child(trigger)
                    .children(self.children),
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
    fn colors_follow_side_nav_css() {
        crate::theme::tests_set(CarbonTheme::White);
        let (bg, text, icon) = item_colors(false, false);
        assert_eq!(
            (bg.a, hex(text), hex(icon)),
            (0., "#525252".into(), "#525252".into())
        );
        let (_, text, _) = item_colors(false, true);
        assert_eq!(hex(text), "#161616");
        let (bg, text, _) = item_colors(true, false);
        assert_eq!(bg, color(ColorRole::BackgroundSelected));
        assert_eq!(hex(text), "#161616");
    }

    #[test]
    fn widths_and_motion() {
        assert_eq!((EXPANDED_WIDTH, RAIL_WIDTH), (256., 48.));
        assert_eq!(WIDTH_MOTION.speed.millis(), 110);
        assert_eq!(WIDTH_MOTION.curve.points(), (0.2, 0., 1., 0.9));
    }
}
