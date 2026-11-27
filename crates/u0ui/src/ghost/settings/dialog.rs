//! SettingsDialog: the settings window's frame (`SettingsDialog` in
//! `src/ghost/admin.jsx`, `.g-sdlg` in `src/ghost.css`): a 14rem side column
//! on `background` with a search field, the server the settings belong to
//! and the section navigation; the chosen section's content on the right.
//!
//! - **Behavior**: typing in the search filters the navigation by item label
//!   (case-insensitive; empty groups disappear). Pressing an item makes it
//!   current (`state-current-*`: a 3px indicator, 600 weight). The current
//!   item and the server are controlled when [`SettingsDialog::value`] /
//!   [`SettingsDialog::server`] are set, otherwise the dialog keeps them,
//!   starting at the first item and the first server (`useControlled`).
//! - The caller renders the section in `children`.
//!
//! ```ignore
//! SettingsDialog::new("settings")
//!     .section(SettingsSection::new("").item("general", "General", CarbonIcon::Settings))
//!     .servers(["MacBook Pro", "build-box"])
//!     .child(div().child("General"))
//! ```
use crate::{
    components::{
        DropdownMenu, DropdownMenuKind, DropdownMenuSize, MenuItem, overlay::floating_shadow,
    },
    ghost::{fields, primitives::parts, rows as row},
    icons::{CarbonIcon, Icon, IconSize},
    t3::craft::{rem, sans},
    theme::color,
    tokens::{self, ColorRole},
};
use gpui_kit::{
    AnyElement, App, AppContext as _, CursorStyle, ElementId, Entity, Hsla,
    InteractiveElement as _, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement as _, Styled, Subscription, Window,
    component::input::{InputEvent, InputState},
    div,
    prelude::FluentBuilder as _,
    px,
};
use std::rc::Rc;

/// One navigation item (`items: {id, label, icon}[]`).
#[derive(Clone)]
pub struct SettingsNavItem {
    /// Its id (`value`).
    pub id: SharedString,
    /// Its label.
    pub label: SharedString,
    /// Its glyph (`it.icon || I.Settings`).
    pub icon: Icon,
}

/// A navigation group (`sections: {label, items}[]`); an empty label draws
/// no heading.
#[derive(Clone)]
pub struct SettingsSection {
    /// The heading.
    pub label: SharedString,
    /// The items.
    pub items: Vec<SettingsNavItem>,
}

impl SettingsSection {
    /// A group headed `label` ("" for none).
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            items: Vec::new(),
        }
    }

    /// Add an item.
    pub fn item(
        mut self,
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        icon: impl Into<Icon>,
    ) -> Self {
        self.items.push(SettingsNavItem {
            id: id.into(),
            label: label.into(),
            icon: icon.into(),
        });
        self
    }
}

/// The groups and items left after filtering by `query`
/// (`it.label.toLowerCase().includes(q.toLowerCase())`); groups with no
/// match are dropped.
pub fn filter_sections(sections: &[SettingsSection], query: &str) -> Vec<SettingsSection> {
    let q = query.to_lowercase();
    sections
        .iter()
        .filter_map(|s| {
            let items: Vec<_> = s
                .items
                .iter()
                .filter(|it| q.is_empty() || it.label.to_lowercase().contains(&q))
                .cloned()
                .collect();
            (!items.is_empty()).then(|| SettingsSection {
                label: s.label.clone(),
                items,
            })
        })
        .collect()
}

/// `.g-sdlg-item` colors (bg, text, indicator): `text-secondary` on none;
/// hover `layer-hover-01` / `text-primary`; current
/// `state-current-background` / `text-primary` with the 3px
/// `state-current-indicator`.
pub fn item_colors(current: bool, hovered: bool) -> (Hsla, Hsla, Option<Hsla>) {
    if current {
        (
            color(ColorRole::StateCurrentBackground),
            color(ColorRole::TextPrimary),
            Some(color(ColorRole::StateCurrentIndicator)),
        )
    } else if hovered {
        (
            color(ColorRole::LayerHover01),
            color(ColorRole::TextPrimary),
            None,
        )
    } else {
        (
            Hsla::transparent_black(),
            color(ColorRole::TextSecondary),
            None,
        )
    }
}

/// `.g-sdlg-nav { width: 14rem }`.
pub const NAV_WIDTH: f32 = 224.;

/// The dialog's user-facing strings (the reference's English defaults).
#[derive(Clone)]
pub struct SettingsDialogTexts {
    /// The search placeholder and name: `"Search settings"`.
    pub search: SharedString,
    /// `"Server"`.
    pub server: SharedString,
}

impl Default for SettingsDialogTexts {
    fn default() -> Self {
        Self {
            search: "Search settings".into(),
            server: "Server".into(),
        }
    }
}

type ValueHandler = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;

struct Model {
    value: Option<SharedString>,
    server: Option<SharedString>,
    query: Entity<InputState>,
    _sub: Subscription,
}

/// The settings window frame. See the module docs.
#[derive(IntoElement)]
pub struct SettingsDialog {
    id: ElementId,
    sections: Vec<SettingsSection>,
    value: Option<SharedString>,
    on_value_change: Option<ValueHandler>,
    servers: Vec<SharedString>,
    server: Option<SharedString>,
    on_server_change: Option<ValueHandler>,
    texts: SettingsDialogTexts,
    children: Vec<AnyElement>,
}

impl SettingsDialog {
    /// An empty dialog.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            sections: Vec::new(),
            value: None,
            on_value_change: None,
            servers: Vec::new(),
            server: None,
            on_server_change: None,
            texts: SettingsDialogTexts::default(),
            children: Vec::new(),
        }
    }

    /// Add a navigation group (`sections`).
    pub fn section(mut self, section: SettingsSection) -> Self {
        self.sections.push(section);
        self
    }

    /// The current item's id (`value`; controlled).
    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// An item was pressed (`onValueChange`).
    pub fn on_value_change(
        mut self,
        f: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_value_change = Some(Rc::new(f));
        self
    }

    /// The servers the settings can belong to (`servers`); none hides the row.
    pub fn servers(mut self, servers: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.servers = servers.into_iter().map(Into::into).collect();
        self
    }

    /// The chosen server (`server`; controlled).
    pub fn server(mut self, server: impl Into<SharedString>) -> Self {
        self.server = Some(server.into());
        self
    }

    /// A server was picked (`onServerChange`).
    pub fn on_server_change(
        mut self,
        f: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_server_change = Some(Rc::new(f));
        self
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: SettingsDialogTexts) -> Self {
        self.texts = texts;
        self
    }
}

impl ParentElement for SettingsDialog {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for SettingsDialog {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let model = window.use_keyed_state((id.clone(), "sdlg"), cx, |window, cx| {
            let query = cx.new(|cx| InputState::new(window, cx));
            // Repaint the navigation as the query changes.
            let sub = cx.subscribe_in(&query, window, |_: &mut Model, _, _: &InputEvent, _, cx| {
                cx.notify()
            });
            Model {
                value: None,
                server: None,
                query,
                _sub: sub,
            }
        });
        let query_state = model.read(cx).query.clone();
        let q = query_state.read(cx).value().to_string();
        let first_item = self
            .sections
            .first()
            .and_then(|s| s.items.first())
            .map(|i| i.id.clone());
        let current = self
            .value
            .clone()
            .or_else(|| model.read(cx).value.clone())
            .or(first_item);
        let server = self
            .server
            .clone()
            .or_else(|| model.read(cx).server.clone())
            .or_else(|| self.servers.first().cloned());
        let groups = filter_sections(&self.sections, &q);
        let controlled_value = self.value.is_some();
        let controlled_server = self.server.is_some();

        let search_focused = {
            use gpui_kit::Focusable as _;
            query_state.read(cx).focus_handle(cx).is_focused(window)
        };
        let empty = q.is_empty();
        // `.g-sdlg-search { display: flex; align-items: center; gap:
        // var(--spacing-03); height: var(--density-field-md); margin:
        // var(--spacing-04) var(--spacing-04) var(--spacing-03); padding: 0
        // var(--spacing-03); background: var(--field-01); border-bottom: 1px
        // solid var(--border-strong-01) }`, `:focus-within { outline: 2px focus }`.
        let search = div()
            .relative()
            .flex()
            .flex_none()
            .items_center()
            .gap(tokens::SPACING_03)
            .h(crate::density::current().field())
            .mt(tokens::SPACING_04)
            .mx(tokens::SPACING_04)
            .mb(tokens::SPACING_03)
            .px(tokens::SPACING_03)
            .bg(color(ColorRole::Field01))
            .border_b_1()
            .border_color(color(ColorRole::BorderStrong01))
            .child(
                Icon::carbon(CarbonIcon::Search)
                    .size(IconSize::S16)
                    .color(color(ColorRole::IconSecondary)),
            )
            .child(
                div()
                    .relative()
                    .flex()
                    .items_center()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .child(parts::editor(&query_state).text_color(color(ColorRole::TextPrimary)))
                    .when(empty, |d| {
                        d.child(
                            sans(div(), 14., 400)
                                .absolute()
                                .left_0()
                                .text_color(color(ColorRole::TextPlaceholder))
                                .whitespace_nowrap()
                                .child(self.texts.search.clone()),
                        )
                    }),
            )
            .when(search_focused, |d| {
                d.child(fields::outline(2., color(ColorRole::Focus)))
            });

        let server_row = (!self.servers.is_empty()).then(|| {
            let items = self.servers.iter().map(|s| {
                let model = model.clone();
                let f = self.on_server_change.clone();
                let name = s.clone();
                MenuItem::item(s.clone()).on_select(move |w, cx| {
                    if !controlled_server {
                        let name = name.clone();
                        model.update(cx, |m, cx| {
                            m.server = Some(name);
                            cx.notify();
                        });
                    }
                    if let Some(f) = &f {
                        f(name.clone(), w, cx);
                    }
                })
            });
            // `.g-sdlg-server { display: flex; align-items: center;
            // justify-content: space-between; margin: 0 var(--spacing-04)
            // var(--spacing-03); font-size: 0.75rem }`.
            div()
                .flex()
                .flex_none()
                .items_center()
                .justify_between()
                .mx(tokens::SPACING_04)
                .mb(tokens::SPACING_03)
                .child(row::dim(self.texts.server.clone(), 12.))
                .child(
                    DropdownMenu::new((id.clone(), "server"))
                        .label(server.clone().unwrap_or_default())
                        .kind(DropdownMenuKind::Ghost)
                        .size(DropdownMenuSize::Sm)
                        .items(items),
                )
        });

        let mut nav_groups = Vec::new();
        for (gi, group) in groups.into_iter().enumerate() {
            let mut g = div().flex().flex_col().py(tokens::SPACING_02);
            if !group.label.is_empty() {
                // `.g-sdlg-h { padding: var(--spacing-03) var(--spacing-05)
                // var(--spacing-02); font-size: 0.75rem; color: var(--text-secondary) }`.
                g = g.child(
                    sans(div(), 12., 400)
                        .pt(tokens::SPACING_03)
                        .px(tokens::SPACING_05)
                        .pb(tokens::SPACING_02)
                        .text_color(color(ColorRole::TextSecondary))
                        .child(group.label.clone()),
                );
            }
            for item in group.items {
                let key = SharedString::from(format!("item-{gi}-{}", item.id));
                let hover = window.use_keyed_state((id.clone(), key.clone()), cx, |_, _| false);
                let focus = parts::keyed_focus(
                    (id.clone(), SharedString::from(format!("{key}-focus"))),
                    window,
                    cx,
                );
                let on = current.as_ref() == Some(&item.id);
                let (bg, fg, indicator) = item_colors(on, *hover.read(cx));
                let ring = parts::focus_visible(&focus, window);
                let model = model.clone();
                let f = self.on_value_change.clone();
                let item_id = item.id.clone();
                // `.g-sdlg-item { display: flex; align-items: center; gap:
                // var(--spacing-03); width: 100%; height: var(--density-row-md);
                // padding: 0 var(--spacing-05); font-size: 0.875rem }`.
                g = g.child(
                    gpui_kit::base::Button::new((id.clone(), key))
                        .track_focus(&focus)
                        .relative()
                        .flex()
                        .items_center()
                        .justify_start()
                        .gap(tokens::SPACING_03)
                        .w_full()
                        .h(crate::density::current().row())
                        .px(tokens::SPACING_05)
                        .bg(bg)
                        .text_color(fg)
                        .font(crate::typography::font(
                            tokens::FontFamilyKey::Sans,
                            if on { 600 } else { 400 },
                        ))
                        .text_size(rem(14.))
                        .cursor(CursorStyle::PointingHand)
                        .on_hover(move |h, _, cx| {
                            hover.update(cx, |v, cx| {
                                *v = *h;
                                cx.notify();
                            })
                        })
                        .when_some(indicator, |b, c| {
                            b.child(div().absolute().left_0().top_0().bottom_0().w(px(3.)).bg(c))
                        })
                        .child(item.icon.clone().size(IconSize::S16).color(fg))
                        .child(item.label.clone())
                        .when(ring, |b| b.child(parts::focus_ring()))
                        .on_click(move |_, w, cx| {
                            if !controlled_value {
                                let item_id = item_id.clone();
                                model.update(cx, |m, cx| {
                                    m.value = Some(item_id);
                                    cx.notify();
                                });
                            }
                            if let Some(f) = &f {
                                f(item_id.clone(), w, cx);
                            }
                        }),
                );
            }
            nav_groups.push(g);
        }

        // `.g-sdlg { display: flex; height: 100%; min-height: 0; background:
        // var(--layer-01); box-shadow: 0 2px 6px var(--shadow) }`.
        div()
            .flex()
            .size_full()
            .min_h_0()
            .bg(color(ColorRole::Layer01))
            .shadow(floating_shadow())
            .child(
                // `.g-sdlg-nav { display: flex; flex-direction: column; width:
                // 14rem; flex: none; background: var(--background); border-right:
                // 1px solid var(--border-subtle-01); overflow: auto }`.
                div()
                    .id((id.clone(), "nav"))
                    .flex()
                    .flex_col()
                    .flex_none()
                    .w(rem(NAV_WIDTH))
                    .h_full()
                    .overflow_y_scroll()
                    .bg(color(ColorRole::Background))
                    .border_r_1()
                    .border_color(color(ColorRole::BorderSubtle01))
                    .child(search)
                    .children(server_row)
                    .children(nav_groups),
            )
            .child(
                // `.g-sdlg-main { flex: 1; min-width: 0; overflow: auto; padding:
                // var(--spacing-06) }`.
                sans(div(), 14., 400)
                    .id((id.clone(), "main"))
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .overflow_y_scroll()
                    .p(tokens::SPACING_06)
                    .text_color(color(ColorRole::TextPrimary))
                    .children(self.children),
            )
    }
}

/// `.g-sdlg-main h2 { margin: 0 0 var(--spacing-05); font-size: 1.25rem;
/// font-weight: 400 }`: the section title callers put first in the content.
pub fn settings_heading(text: impl Into<SharedString>) -> impl IntoElement {
    sans(div(), 20., 400)
        .mb(tokens::SPACING_05)
        .text_color(color(ColorRole::TextPrimary))
        .child(text.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CarbonTheme;

    fn sections() -> Vec<SettingsSection> {
        vec![
            SettingsSection::new("")
                .item("general", "General", CarbonIcon::Settings)
                .item("agents", "Agents", CarbonIcon::Bot),
            SettingsSection::new("Workspace").item("scripts", "Scripts", CarbonIcon::Terminal),
        ]
    }

    #[test]
    fn search_filters_items_and_drops_empty_groups() {
        let f = filter_sections(&sections(), "AG");
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].items.len(), 1);
        assert_eq!(f[0].items[0].id, SharedString::from("agents"));
        assert_eq!(filter_sections(&sections(), "").len(), 2);
        assert!(filter_sections(&sections(), "zzz").is_empty());
    }

    #[test]
    fn current_item_uses_state_current() {
        crate::theme::tests_set(CarbonTheme::G100);
        let (bg, fg, ind) = item_colors(true, false);
        assert_eq!(bg, color(ColorRole::StateCurrentBackground));
        assert_eq!(fg, color(ColorRole::TextPrimary));
        assert_eq!(ind, Some(color(ColorRole::StateCurrentIndicator)));
        let (bg, fg, ind) = item_colors(false, true);
        assert_eq!(bg, color(ColorRole::LayerHover01));
        assert_eq!(fg, color(ColorRole::TextPrimary));
        assert_eq!(ind, None);
        assert_eq!(item_colors(false, false).1, color(ColorRole::TextSecondary));
        assert_eq!(NAV_WIDTH, 224.);
    }
}
