//! WindowChrome: the macOS window frame (`WindowChrome` in
//! `src/ghost/foundations.jsx`): a 38px title bar, the traffic-light inset
//! and the sidebar material.
//!
//! - `.g-window`: `background`, a 1px `border-subtle-01` ring and
//!   `0 12px 32px shadow`, 8px corners (a window, not a Carbon surface).
//! - Title bar (`.g-titlebar`): `titlebar-height` high, `titlebar-background`
//!   with a `titlebar-border` rule, padding `0 8px 0 16px`; without a sidebar
//!   the traffic lights sit in it and it starts at `traffic-light-inset`
//!   (76px). Title 13px 600, subtitle 12px `text-secondary`, both truncated;
//!   actions at the end, 2px apart.
//! - Sidebar (`.g-window-sidebar`): `sidebar-width`, `sidebar-material` with
//!   a `titlebar-border` end rule; the traffic lights sit on its own title
//!   bar. GPUI cannot blur what is behind a window region, so the material's
//!   `backdrop-filter: blur(material-blur)` is the translucent fill only (the
//!   host app draws the real NSVisualEffectView).
//! - Traffic lights (`.g-traffic`): three 12px circles 8px apart at 13px from
//!   the start, `traffic-light-offset-y` from the top. They stand in for the
//!   system buttons; unfocused they turn `layer-accent-01` and the title
//!   `text-disabled`.
use crate::{
    theme::color,
    tokens::{
        self, ColorRole, FontFamilyKey, SIDEBAR_WIDTH, TITLEBAR_HEIGHT, TRAFFIC_LIGHT_INSET,
        TRAFFIC_LIGHT_OFFSET_Y,
    },
    typography::font,
};
use gpui_kit::{
    AnyElement, App, BoxShadow, Hsla, IntoElement, ParentElement, Rems, RenderOnce, SharedString,
    Styled, Window, div, prelude::FluentBuilder as _, px, rems, rgb,
};

/// The macOS traffic-light fills (close, minimize, zoom). These are the
/// system's own colors, drawn only as a stand-in for the real buttons, so
/// they are not Carbon tokens (`.g-traffic i { background: #ff5f57 }`,
/// `:nth-child(2) { #febc2e }`, `:nth-child(3) { #28c840 }`).
pub fn traffic_colors(focused: bool) -> [Hsla; 3] {
    if focused {
        [
            rgb(0xff5f57).into(),
            rgb(0xfebc2e).into(),
            rgb(0x28c840).into(),
        ]
    } else {
        // `.g-window--blurred .g-traffic i { background: var(--layer-accent-01) }`.
        [color(ColorRole::LayerAccent01); 3]
    }
}

fn traffic(focused: bool) -> impl IntoElement {
    let ring = vec![BoxShadow {
        color: gpui_kit::hsla(0., 0., 0., 0.2),
        offset: gpui_kit::point(px(0.), px(0.)),
        blur_radius: px(0.),
        spread_radius: px(0.5),
        inset: true,
    }];
    div()
        .absolute()
        .left(rems(0.8125))
        .top(TRAFFIC_LIGHT_OFFSET_Y)
        .flex()
        .gap(px(8.))
        .children(traffic_colors(focused).map(|c| {
            div()
                .size(px(12.))
                .rounded_full()
                .bg(c)
                .shadow(ring.clone())
        }))
}

fn titlebar() -> gpui_kit::Div {
    div()
        .relative()
        .flex()
        .flex_none()
        .items_center()
        .gap(tokens::SPACING_03)
        .h(TITLEBAR_HEIGHT)
}

/// A macOS window frame. See the module docs.
#[derive(IntoElement)]
pub struct WindowChrome {
    title: SharedString,
    subtitle: Option<SharedString>,
    actions: Vec<AnyElement>,
    sidebar: Option<AnyElement>,
    children: Vec<AnyElement>,
    traffic_lights: bool,
    focused: bool,
    height: Option<Rems>,
}

impl WindowChrome {
    /// A window titled `title`.
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            actions: Vec::new(),
            sidebar: None,
            children: Vec::new(),
            traffic_lights: true,
            focused: true,
            height: None,
        }
    }

    /// The secondary title (project · branch).
    pub fn subtitle(mut self, subtitle: impl Into<SharedString>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// A title-bar action (icon buttons).
    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.actions.push(action.into_any_element());
        self
    }

    /// The sidebar column (its title bar holds the traffic lights).
    pub fn sidebar(mut self, sidebar: impl IntoElement) -> Self {
        self.sidebar = Some(sidebar.into_any_element());
        self
    }

    /// Show the traffic lights (default true).
    pub fn traffic_lights(mut self, on: bool) -> Self {
        self.traffic_lights = on;
        self
    }

    /// Whether the window is focused (default true).
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// A fixed height.
    pub fn height(mut self, height: Rems) -> Self {
        self.height = Some(height);
        self
    }
}

impl ParentElement for WindowChrome {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for WindowChrome {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let has_sidebar = self.sidebar.is_some();
        let lights = self.traffic_lights;
        let focused = self.focused;
        let sidebar = self.sidebar.map(|content| {
            div()
                .flex()
                .flex_col()
                .flex_none()
                .w(SIDEBAR_WIDTH)
                .bg(color(ColorRole::SidebarMaterial))
                .border_r(px(1.))
                .border_color(color(ColorRole::TitlebarBorder))
                // `.g-titlebar--sidebar { background: transparent; border-bottom: 0 }`.
                .child(titlebar().when(lights, |t| t.child(traffic(focused))))
                .child(content)
        });
        let title = div()
            .flex()
            .flex_1()
            .min_w_0()
            .items_baseline()
            .gap(tokens::SPACING_03)
            .text_size(rems(0.8125))
            .text_color(color(if focused {
                ColorRole::TextPrimary
            } else {
                ColorRole::TextDisabled
            }))
            .child(
                div()
                    .min_w_0()
                    .font(font(FontFamilyKey::Sans, 600))
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .child(self.title.clone()),
            )
            .children(self.subtitle.clone().map(|s| {
                div()
                    .min_w_0()
                    .text_size(rems(0.75))
                    .text_color(color(ColorRole::TextSecondary))
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .child(s)
            }));
        let inset = !has_sidebar && lights;
        let bar = titlebar()
            .pr(tokens::SPACING_03)
            .pl(if inset {
                TRAFFIC_LIGHT_INSET
            } else {
                tokens::SPACING_05
            })
            .bg(color(ColorRole::TitlebarBackground))
            .border_b(px(1.))
            .border_color(color(ColorRole::TitlebarBorder))
            .when(inset, |t| t.child(traffic(focused)))
            .child(title)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(2.))
                    .children(self.actions),
            );
        let main = div().flex().flex_col().flex_1().min_w_0().child(bar).child(
            div()
                .flex_1()
                .min_h_0()
                .overflow_hidden()
                .children(self.children),
        );
        div()
            .flex()
            .min_h_0()
            .overflow_hidden()
            .rounded(tokens::BORDER_RADIUS_08)
            .bg(color(ColorRole::Background))
            // `box-shadow: 0 0 0 1px var(--border-subtle-01), 0 12px 32px var(--shadow)`.
            .shadow(vec![
                BoxShadow {
                    color: color(ColorRole::BorderSubtle01),
                    offset: gpui_kit::point(px(0.), px(0.)),
                    blur_radius: px(0.),
                    spread_radius: px(1.),
                    inset: false,
                },
                BoxShadow {
                    color: color(ColorRole::Shadow),
                    offset: gpui_kit::point(px(0.), px(12.)),
                    blur_radius: px(32.),
                    spread_radius: px(0.),
                    inset: false,
                },
            ])
            .when_some(self.height, |d, h| d.h(h))
            .children(sidebar)
            .child(main)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CarbonTheme, theme::color_in};

    #[test]
    fn chrome_metrics_follow_the_tokens() {
        // `titlebar-height` 38px, `traffic-light-inset` 76px, offset 13px.
        assert_eq!(TITLEBAR_HEIGHT.px_value(), 38.);
        assert_eq!(TRAFFIC_LIGHT_INSET.px_value(), 76.);
        assert_eq!(TRAFFIC_LIGHT_OFFSET_Y.px_value(), 13.);
        assert_eq!(SIDEBAR_WIDTH.px_value(), 240.);
    }

    #[test]
    fn blurred_lights_go_gray() {
        crate::theme::tests_set(CarbonTheme::G100);
        assert_eq!(
            traffic_colors(false),
            [color_in(CarbonTheme::G100, ColorRole::LayerAccent01); 3]
        );
        assert_ne!(traffic_colors(true)[0], traffic_colors(true)[1]);
    }
}
