//! The gallery header: Carbon's UI shell header in a G100 zone (always dark,
//! like IBM product headers): u0ui's [`NavigationMenu`] with the product
//! name and the three sections, then the global actions, u0ui ghost icon
//! buttons: search (the ⌘K palette), the theme and the display settings
//! (density, reduced motion), whose panels are u0ui [`Popover`]s.
use crate::registry::Section;
use crate::shell::{Gallery, Route};
use gpui_kit::{
    AnyElement, App, Context, Entity, IntoElement, ParentElement, Styled, Window, div,
    prelude::FluentBuilder as _, rems,
};
use u0ui::components::toggle::ToggleSize;
use u0ui::components::{Align, NavItem, NavigationMenu, Popover, ToggleGroup, ToggleGroupItem};
use u0ui::motion::ReducedMotion;
use u0ui::prelude::*;

/// The header's fixed theme.
pub const SHELL: CarbonTheme = CarbonTheme::G100;

/// A theme's display name.
pub fn theme_name(theme: CarbonTheme) -> &'static str {
    match theme {
        CarbonTheme::White => "White",
        CarbonTheme::G10 => "G10",
        CarbonTheme::G90 => "G90",
        CarbonTheme::G100 => "G100",
    }
}

/// A density's display name.
pub fn density_name(density: Density) -> &'static str {
    match density {
        Density::Compact => "Compact",
        Density::Comfortable => "Comfortable",
    }
}

/// A reduced-motion preference's display name.
fn motion_name(motion: ReducedMotion) -> &'static str {
    match motion {
        ReducedMotion::System => "System",
        ReducedMotion::Reduce => "Reduce",
        ReducedMotion::Allow => "Full",
    }
}

const MOTIONS: [ReducedMotion; 3] = [
    ReducedMotion::System,
    ReducedMotion::Reduce,
    ReducedMotion::Allow,
];

/// What a header action does when pressed.
type Action = Box<dyn Fn(&mut Window, &mut App)>;

/// A 48px global action (`.cds--header__action`): a ghost lg icon-only
/// button drawn in the header's theme. Its tooltip hangs below the header
/// (`HeaderGlobalAction` `tooltipAlignment`), `align`ed so it stays inside
/// the window at the header's ends.
fn action(
    id: &'static str,
    label: &'static str,
    icon: &'static str,
    align: Align,
    on_click: Option<Action>,
) -> impl IntoElement {
    theme::zone(SHELL, move |_, _| {
        let mut button = Button::new(id, label)
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Lg)
            .icon(Icon::named(icon))
            .icon_only(true)
            .tooltip_align(align);
        if let Some(on_click) = on_click {
            button = button.on_click(move |_, window, cx| on_click(window, cx));
        }
        button.into_any_element()
    })
}

/// A labeled content switcher inside a header panel.
fn setting(
    id: &'static str,
    label: &'static str,
    options: &[&'static str],
    selected: &'static str,
    on_select: impl Fn(usize, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let values: Vec<&'static str> = options.to_vec();
    let mut group = ToggleGroup::new(id)
        .size(ToggleSize::Sm)
        .value([selected])
        .on_value_change(move |value, window, cx| {
            if let Some(i) = value
                .first()
                .and_then(|v| values.iter().position(|o| *o == v.as_ref()))
            {
                on_select(i, window, cx);
            }
        });
    for option in options {
        group = group.item(ToggleGroupItem::new(*option).child(*option));
    }
    div()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_03)
        .child(
            div()
                .type_style(TypeStyle::Label01)
                .text_color(color(ColorRole::TextSecondary))
                .child(label),
        )
        .child(group)
}

/// A header panel's content width: the popover's 22rem
/// (`.cu-popover-content { max-width }`) less its 1rem padding on each side.
/// The popover shrink-wraps its content, and a content switcher's equal
/// segments would otherwise truncate their labels.
const PANEL_WIDTH: f32 = 20.;

fn panel_title(text: &'static str) -> impl IntoElement {
    div()
        .type_style(TypeStyle::HeadingCompact01)
        .text_color(color(ColorRole::TextPrimary))
        .child(text)
}

/// The theme panel: the four Carbon themes.
fn theme_panel() -> impl IntoElement {
    let names: Vec<&'static str> = CarbonTheme::ALL.iter().map(|t| theme_name(*t)).collect();
    Popover::new(
        "theme-panel",
        action("theme", "Theme", "contrast", Align::BottomRight, None),
    )
    .align(Align::BottomRight)
    .child(
        div()
            .w(rems(PANEL_WIDTH))
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(panel_title("Theme"))
            .child(setting(
                "theme-switch",
                "Carbon theme",
                &names,
                theme_name(theme::current()),
                |i, _, cx| theme::set_theme(cx, CarbonTheme::ALL[i]),
            )),
    )
}

/// The display panel: density and reduced motion.
fn settings_panel(gallery: Entity<Gallery>, motion: ReducedMotion) -> impl IntoElement {
    let densities: Vec<&'static str> = Density::ALL.iter().map(|d| density_name(*d)).collect();
    let motions: Vec<&'static str> = MOTIONS.iter().map(|m| motion_name(*m)).collect();
    Popover::new(
        "settings-panel",
        action(
            "settings",
            "Display settings",
            "tune",
            Align::BottomRight,
            None,
        ),
    )
    .align(Align::BottomRight)
    .child(
        div()
            .w(rems(PANEL_WIDTH))
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(panel_title("Display"))
            .child(setting(
                "density-switch",
                "Density",
                &densities,
                density_name(density::current()),
                |i, _, cx| density::set_density(cx, Density::ALL[i]),
            ))
            .child(setting(
                "motion-switch",
                "Motion",
                &motions,
                motion_name(motion),
                move |i, _, cx| {
                    motion::set_reduced_motion(cx, MOTIONS[i]);
                    gallery.update(cx, |this, cx| {
                        this.motion = MOTIONS[i];
                        cx.notify();
                    });
                },
            )),
    )
}

/// The header row.
pub fn render(gallery: &Gallery, narrow: bool, cx: &mut Context<Gallery>) -> AnyElement {
    let entity = cx.entity();
    let current = gallery.route.section();
    let go = |route: Route| {
        let entity = entity.clone();
        move |_: &mut Window, cx: &mut App| entity.update(cx, |this, cx| this.navigate(route, cx))
    };
    let mut menu = NavigationMenu::new("header")
        .brand_prefix("u0ui")
        .brand("Gallery")
        .on_brand_click(go(Route::Home));
    for section in Section::ALL {
        menu = menu.item(
            NavItem::link(section.title())
                .active(current == Some(section))
                .on_click(go(Route::Overview(Some(section)))),
        );
    }
    let search: Action = Box::new({
        let entity = entity.clone();
        move |_, cx| {
            entity.update(cx, |this, cx| {
                this.palette_open = true;
                cx.notify();
            })
        }
    });
    let nav_toggle: Action = Box::new({
        let entity = entity.clone();
        move |_, cx| {
            entity.update(cx, |this, cx| {
                this.nav_open = !this.nav_open;
                cx.notify();
            })
        }
    });
    div()
        // `.cds--header`: 3rem on `background`, 1px `border-subtle` under it.
        .h(rems(3.))
        .flex_shrink_0()
        .flex()
        .flex_row()
        .items_center()
        .bg(theme::color_in(SHELL, ColorRole::Background))
        .border_b_1()
        .border_color(theme::color_in(SHELL, ColorRole::BorderSubtle01))
        .when(narrow, |this| {
            this.child(action(
                "nav-toggle",
                if gallery.nav_open {
                    "Close menu"
                } else {
                    "Open menu"
                },
                if gallery.nav_open { "close" } else { "menu" },
                Align::BottomLeft,
                Some(nav_toggle),
            ))
        })
        .child(
            div()
                .flex_1()
                .min_w_0()
                .h_full()
                .overflow_hidden()
                .child(theme::zone(SHELL, move |_, _| menu.into_any_element())),
        )
        .child(action(
            "search",
            "Search (⌘K)",
            "search",
            Align::Bottom,
            Some(search),
        ))
        .child(theme_panel())
        .child(settings_panel(entity.clone(), gallery.motion))
        .into_any_element()
}
