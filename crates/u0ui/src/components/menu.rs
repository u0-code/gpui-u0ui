//! Menu: Carbon `Menu` (`.cds--menu`) with `MenuItem` (label, shortcut,
//! `kind="danger"`, nested submenus), `MenuItemDivider`, `MenuItemSelectable`
//! (checkbox), `MenuItemRadioGroup` and `MenuItemGroup`. The surface behind
//! DropdownMenu, ContextMenu and Menubar.
//!
//! - **Why not Kit's menu**: Kit's `PopupMenu` is an entity of `Action`s that
//!   paints its own paddings, radius, check column and hover, has no danger
//!   items or radio groups, and keeps its keyboard handling inside that
//!   entity, so its rendering cannot be made Carbon. The menu is drawn here;
//!   anchoring, window-edge snapping and the deferred top layer stay Base's
//!   (`Popup` / `Positioner`), and focus moves use [`super::overlay`].
//! - **Keyboard** (Carbon `Menu`): ↓/↑ move (wrapping, skipping disabled
//!   items), Home/End, Enter/Space activate, → or Enter opens a submenu and
//!   enters it, ←/Escape leaves a submenu, Escape closes the menu (focus
//!   returns to the trigger), Tab closes, typing jumps to the next item that
//!   starts with the typed text (type-ahead, 500ms buffer).
//! - **Appearance** ([`MenuLook`], [`MenuGeometry`]): `layer-01` with the
//!   floating shadow, 0.25rem block padding, 10rem–18rem wide (12rem with
//!   icons or selectable items); items `text-secondary` body-compact-01,
//!   1rem inline padding, height per size (24/32/40/48) or `density-row-md`
//!   in compact density; hover `layer-hover` + `text-primary`; danger items
//!   `text-error` at rest (as the live reference renders them); danger hover
//!   and focus `button-danger-primary` + `text-on-color`; disabled
//!   `text-disabled`; keyboard focus a 2px inset `focus` outline; dividers
//!   1px `border-subtle-00` with 0.25rem margins.
//! - **Motion**: item backgrounds `transition: background-color 70ms
//!   cubic-bezier(0.2, 0, 0.38, 0.9)`; the surface shows and hides at once
//!   (`.cds--menu { display: none }` → `.cds--menu--open`, no transition).
use super::overlay::{self, Callback, Translate, rem};
use crate::{
    density::{self, Density},
    icons::{CarbonIcon, Icon, IconSize},
    motion::{self, Curve, Motion, Speed},
    theme::color,
    tokens::{ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    Anchor, AnyElement, App, Bounds, CursorStyle, ElementId, Entity, FocusHandle, Hsla,
    InteractiveElement, IntoElement, KeyDownEvent, MouseButton, ParentElement, Pixels, RenderOnce,
    SharedString, StatefulInteractiveElement as _, Styled, Window, canvas, div, point,
    prelude::FluentBuilder as _, px,
};
use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    time::{Duration, Instant},
};

/// `.cds--menu-item { transition: background-color 70ms cubic-bezier(0.2, 0, 0.38, 0.9) }`.
pub const ITEM_MOTION: Motion = Motion::new(Curve::ProductiveStandard, Speed::Fast01);
/// How long typed characters accumulate for type-ahead.
pub const TYPEAHEAD: Duration = Duration::from_millis(500);

/// Menu sizes (`.cds--menu--xs/sm/md/lg`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MenuSize {
    /// `xs`: 1.5rem items.
    Xs,
    /// `sm`: 2rem items (the default).
    #[default]
    Sm,
    /// `md`: 2.5rem items.
    Md,
    /// `lg`: 3rem items.
    Lg,
}

impl MenuSize {
    /// Every size.
    pub const ALL: [MenuSize; 4] = [Self::Xs, Self::Sm, Self::Md, Self::Lg];

    /// The Carbon name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Xs => "xs",
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
        }
    }

    /// Item height in px under `density`: `.cds--menu--<size> .cds--menu-item
    /// { block-size }`, or `[data-density="compact"] .cds--menu-item { height:
    /// var(--density-row-md) }`.
    pub fn item_height(self, density: Density) -> f32 {
        match density {
            Density::Compact => crate::tokens::DENSITY_ROW_MD.px_value(),
            Density::Comfortable => match self {
                Self::Xs => 24.,
                Self::Sm => 32.,
                Self::Md => 40.,
                Self::Lg => 48.,
            },
        }
    }
}

/// Menu box geometry in px.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MenuGeometry {
    /// `padding: 0.25rem 0`.
    pub padding_block: f32,
    /// `.cds--menu { min-inline-size: 10rem }`; `.cds--menu--with-icons
    /// { min-inline-size: 12rem }` (selectable items alone keep 10rem).
    pub min_width: f32,
    /// `max-inline-size: 18rem`.
    pub max_width: f32,
    /// `.cds--menu-item { padding-inline: 1rem }`.
    pub item_padding: f32,
    /// `column-gap: 0.5rem`.
    pub column_gap: f32,
    /// `.cds--menu-item-divider { margin-block: 0.25rem }`.
    pub divider_margin: f32,
}

impl MenuGeometry {
    /// Geometry for a menu with icons (or not).
    pub const fn of(with_icons: bool) -> Self {
        Self {
            padding_block: 4.,
            min_width: if with_icons { 192. } else { 160. },
            max_width: 288.,
            item_padding: 16.,
            column_gap: 8.,
            divider_margin: 4.,
        }
    }
}

/// The interaction state of one item row.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ItemState {
    /// Pointer over it (or its submenu is open).
    pub hovered: bool,
    /// Keyboard focus is on it.
    pub focused: bool,
    /// `kind="danger"`.
    pub danger: bool,
    /// Disabled.
    pub disabled: bool,
}

/// The colors of a menu and its items.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MenuLook {
    /// `.cds--menu { background-color: var(--layer) }`.
    pub background: Hsla,
    /// `.cds--menu-item-divider { background-color: var(--border-subtle) }`;
    /// menus portal to the document root, where `--border-subtle` is
    /// `border-subtle-00` (`:root { --border-subtle: var(--border-subtle-00) }`).
    pub divider: Hsla,
}

impl MenuLook {
    /// The look on the active theme.
    pub fn current() -> Self {
        Self {
            background: color(ColorRole::Layer01),
            divider: color(ColorRole::BorderSubtle00),
        }
    }

    /// `(background, text, focus outline)` of an item in `state`.
    pub fn item(state: ItemState) -> (Hsla, Hsla, Option<Hsla>) {
        use ColorRole::*;
        let clear = Hsla::transparent_black();
        if state.disabled {
            // `.cds--menu-item--disabled:hover { background-color: var(--layer); color: text-disabled }`.
            return (clear, color(TextDisabled), None);
        }
        let outline = state.focused.then(|| color(Focus));
        if state.danger && (state.hovered || state.focused) {
            // `.cds--menu-item--danger:hover, :focus`.
            return (color(ButtonDangerPrimary), color(TextOnColor), outline);
        }
        if state.hovered {
            // `.cds--menu-item:hover { background-color: var(--layer-hover); color: text-primary }`.
            return (color(LayerHover01), color(TextPrimary), outline);
        }
        if state.danger {
            // The live reference renders `kind="danger"` items in red at rest
            // (`#da1e28` on White), not `text-secondary`; red text is `text-error`.
            return (clear, color(TextError), outline);
        }
        // `.cds--menu-item { color: var(--text-secondary) }`.
        (clear, color(TextSecondary), outline)
    }
}

type Toggle = Rc<dyn Fn(bool, &mut Window, &mut App)>;
type Pick = Rc<dyn Fn(usize, &mut Window, &mut App)>;
/// A handler for keys a menu does not use; returns true when handled.
pub(crate) type KeyHook = Rc<dyn Fn(&str, &mut Window, &mut App) -> bool>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Item,
    Divider,
    Checkbox,
    Radio,
    Group,
}

/// One entry of a menu. Build with [`MenuItem::item`], [`MenuItem::divider`],
/// [`MenuItem::checkbox`], [`MenuItem::radio_group`] or [`MenuItem::group`].
#[derive(Clone)]
pub struct MenuItem {
    kind: Kind,
    label: SharedString,
    shortcut: Option<SharedString>,
    icon: Option<Icon>,
    disabled: bool,
    danger: bool,
    on_select: Option<Callback>,
    children: Vec<MenuItem>,
    selected: Option<bool>,
    default_selected: bool,
    on_toggle: Option<Toggle>,
    options: Vec<SharedString>,
    picked: Option<Option<usize>>,
    default_picked: Option<usize>,
    on_pick: Option<Pick>,
}

impl MenuItem {
    fn base(kind: Kind, label: SharedString) -> Self {
        Self {
            kind,
            label,
            shortcut: None,
            icon: None,
            disabled: false,
            danger: false,
            on_select: None,
            children: Vec::new(),
            selected: None,
            default_selected: false,
            on_toggle: None,
            options: Vec::new(),
            picked: None,
            default_picked: None,
            on_pick: None,
        }
    }

    /// `MenuItem`: an action (or, with [`Self::submenu`], a submenu).
    pub fn item(label: impl Into<SharedString>) -> Self {
        Self::base(Kind::Item, label.into())
    }

    /// `MenuItemDivider`.
    pub fn divider() -> Self {
        Self::base(Kind::Divider, SharedString::default())
    }

    /// `MenuItemSelectable`: a checkbox item.
    pub fn checkbox(label: impl Into<SharedString>) -> Self {
        Self::base(Kind::Checkbox, label.into())
    }

    /// `MenuItemRadioGroup`: one item per option, one selected (`label` is
    /// the group's accessible name).
    pub fn radio_group<S: Into<SharedString>>(
        label: impl Into<SharedString>,
        options: impl IntoIterator<Item = S>,
    ) -> Self {
        let mut item = Self::base(Kind::Radio, label.into());
        item.options = options.into_iter().map(Into::into).collect();
        item
    }

    /// `MenuItemGroup`: items under an accessible group name.
    pub fn group(
        label: impl Into<SharedString>,
        items: impl IntoIterator<Item = MenuItem>,
    ) -> Self {
        let mut item = Self::base(Kind::Group, label.into());
        item.children = items.into_iter().collect();
        item
    }

    /// Keyboard shortcut text (`.cds--menu-item__shortcut`).
    pub fn shortcut(mut self, shortcut: impl Into<SharedString>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// Leading icon (`.cds--menu-item__icon`).
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Disable the item.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// `kind="danger"`.
    pub fn danger(mut self, danger: bool) -> Self {
        self.danger = danger;
        self
    }

    /// Activation handler (the menu closes afterwards).
    pub fn on_select(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }

    /// Nested items: this item opens a submenu.
    pub fn submenu(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.children = items.into_iter().collect();
        self
    }

    /// Controlled checked state of a checkbox item.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = Some(selected);
        self
    }

    /// Initial checked state of an uncontrolled checkbox item (`defaultSelected`).
    pub fn default_selected(mut self, selected: bool) -> Self {
        self.default_selected = selected;
        self
    }

    /// Checkbox change handler.
    pub fn on_change(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Rc::new(f));
        self
    }

    /// Controlled selected option of a radio group.
    pub fn selected_item(mut self, index: Option<usize>) -> Self {
        self.picked = Some(index);
        self
    }

    /// Initial selected option of an uncontrolled radio group (`defaultSelectedItem`).
    pub fn default_selected_item(mut self, index: usize) -> Self {
        self.default_picked = Some(index);
        self
    }

    /// Radio group change handler.
    pub fn on_select_item(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_pick = Some(Rc::new(f));
        self
    }
}

/// What a row does when activated.
#[derive(Clone)]
enum Action {
    Select(Option<Callback>),
    Submenu(Vec<MenuItem>),
    Toggle {
        key: String,
        checked: bool,
        controlled: bool,
        on_change: Option<Toggle>,
    },
    Pick {
        key: String,
        index: usize,
        controlled: bool,
        on_change: Option<Pick>,
    },
}

/// One rendered line of a menu (built per frame; size is irrelevant).
#[derive(Clone)]
#[allow(clippy::large_enum_variant)]
enum Row {
    Divider,
    Entry {
        label: SharedString,
        shortcut: Option<SharedString>,
        icon: Option<Icon>,
        disabled: bool,
        danger: bool,
        checked: Option<bool>,
        action: Action,
    },
}

impl Row {
    fn enabled(&self) -> bool {
        matches!(
            self,
            Row::Entry {
                disabled: false,
                ..
            }
        )
    }

    fn label(&self) -> &str {
        match self {
            Row::Entry { label, .. } => label,
            Row::Divider => "",
        }
    }
}

/// Uncontrolled selections, keyed by item path.
pub(crate) type Selections = Entity<HashMap<String, usize>>;

fn flatten(
    items: &[MenuItem],
    prefix: &str,
    selections: &HashMap<String, usize>,
    out: &mut Vec<Row>,
) {
    for (i, item) in items.iter().enumerate() {
        let key = format!("{prefix}{i}");
        match item.kind {
            Kind::Divider => out.push(Row::Divider),
            Kind::Group => flatten(&item.children, &format!("{key}."), selections, out),
            Kind::Item => out.push(Row::Entry {
                label: item.label.clone(),
                shortcut: item.shortcut.clone(),
                icon: item.icon.clone(),
                disabled: item.disabled,
                danger: item.danger,
                checked: None,
                action: if item.children.is_empty() {
                    Action::Select(item.on_select.clone())
                } else {
                    Action::Submenu(item.children.clone())
                },
            }),
            Kind::Checkbox => {
                let checked = item.selected.unwrap_or_else(|| {
                    selections
                        .get(&key)
                        .map_or(item.default_selected, |v| *v == 1)
                });
                out.push(Row::Entry {
                    label: item.label.clone(),
                    shortcut: item.shortcut.clone(),
                    icon: item.icon.clone(),
                    disabled: item.disabled,
                    danger: false,
                    checked: Some(checked),
                    action: Action::Toggle {
                        key,
                        checked,
                        controlled: item.selected.is_some(),
                        on_change: item.on_toggle.clone(),
                    },
                })
            }
            Kind::Radio => {
                let picked = item
                    .picked
                    .unwrap_or_else(|| selections.get(&key).copied().or(item.default_picked));
                for (index, option) in item.options.iter().enumerate() {
                    out.push(Row::Entry {
                        label: option.clone(),
                        shortcut: None,
                        icon: None,
                        disabled: item.disabled,
                        danger: false,
                        checked: Some(picked == Some(index)),
                        action: Action::Pick {
                            key: key.clone(),
                            index,
                            controlled: item.picked.is_some(),
                            on_change: item.on_pick.clone(),
                        },
                    });
                }
            }
        }
    }
}

/// Keyboard and pointer state of one menu level.
#[derive(Default)]
struct LevelState {
    /// Keyboard-focused row.
    active: Option<usize>,
    /// Pointer-hovered row.
    hovered: Option<usize>,
    /// Row whose submenu is open.
    submenu: Option<usize>,
    /// Whether the open submenu should take keyboard focus.
    submenu_focus: bool,
    /// Whether the open submenu should highlight its first item; the
    /// submenu clears it once it has (its popup may first paint a frame
    /// later than the request).
    submenu_first: bool,
    typed: String,
    typed_at: Option<Instant>,
}

/// Shared by every level of one open menu: closes the whole chain, holds
/// uncontrolled selections, and collects submenu bounds so a press inside a
/// submenu is not a press outside the root.
#[derive(Clone)]
pub(crate) struct Chain {
    pub close: Callback,
    pub selections: Selections,
    pub bounds: Rc<RefCell<Vec<Bounds<Pixels>>>>,
}

/// The next enabled row after `from` in `direction` (wrapping).
fn step(rows: &[Row], from: Option<usize>, direction: isize) -> Option<usize> {
    let n = rows.len() as isize;
    if n == 0 {
        return None;
    }
    let mut i = match from {
        Some(i) => i as isize,
        None if direction > 0 => -1,
        None => n,
    };
    for _ in 0..n {
        i = (i + direction).rem_euclid(n);
        if rows[i as usize].enabled() {
            return Some(i as usize);
        }
    }
    None
}

/// The next enabled row (after `from`) whose label starts with `typed`.
fn find_typed(rows: &[Row], from: Option<usize>, typed: &str) -> Option<usize> {
    let typed = typed.to_lowercase();
    let n = rows.len();
    let start = from.map_or(0, |i| if typed.chars().count() > 1 { i } else { i + 1 });
    (0..n)
        .map(|k| (start + k) % n)
        .find(|&i| rows[i].enabled() && rows[i].label().to_lowercase().starts_with(&typed))
}

/// A Carbon menu surface (one level). Usually created by DropdownMenu,
/// ContextMenu or Menubar, which own placement and open state.
#[derive(IntoElement)]
pub(crate) struct Menu {
    id: ElementId,
    items: Vec<MenuItem>,
    size: MenuSize,
    label: SharedString,
    focus: FocusHandle,
    chain: Chain,
    parent: Option<(FocusHandle, Callback)>,
    min_width: Option<f32>,
    path: String,
    on_key: Option<KeyHook>,
    /// A submenu opened from the keyboard highlights its first enabled item
    /// (Carbon focuses it), then clears the request on its parent level.
    focus_first: Option<Entity<LevelState>>,
}

impl Menu {
    pub fn new(
        id: impl Into<ElementId>,
        items: Vec<MenuItem>,
        focus: FocusHandle,
        chain: Chain,
    ) -> Self {
        Self {
            id: id.into(),
            items,
            size: MenuSize::Sm,
            label: SharedString::default(),
            focus,
            chain,
            parent: None,
            min_width: None,
            path: String::new(),
            on_key: None,
            focus_first: None,
        }
    }

    pub fn size(mut self, size: MenuSize) -> Self {
        self.size = size;
        self
    }

    pub fn label(mut self, label: SharedString) -> Self {
        self.label = label;
        self
    }

    pub fn min_width(mut self, width: Option<f32>) -> Self {
        self.min_width = width;
        self
    }
}

fn activate(
    row: &Row,
    index: usize,
    state: &Entity<LevelState>,
    chain: &Chain,
    from_keyboard: bool,
    window: &mut Window,
    cx: &mut App,
) {
    let Row::Entry {
        action, disabled, ..
    } = row
    else {
        return;
    };
    if *disabled {
        return;
    }
    match action {
        Action::Submenu(_) => state.update(cx, |s, cx| {
            s.submenu = Some(index);
            s.submenu_focus = from_keyboard;
            s.submenu_first = from_keyboard;
            cx.notify();
        }),
        Action::Select(on_select) => {
            if let Some(f) = on_select {
                f(window, cx);
            }
            (chain.close)(window, cx);
        }
        Action::Toggle {
            key,
            checked,
            controlled,
            on_change,
        } => {
            if !controlled {
                chain.selections.update(cx, |map, cx| {
                    map.insert(key.clone(), usize::from(!checked));
                    cx.notify();
                });
            }
            if let Some(f) = on_change {
                f(!checked, window, cx);
            }
            (chain.close)(window, cx);
        }
        Action::Pick {
            key,
            index,
            controlled,
            on_change,
        } => {
            if !controlled {
                chain.selections.update(cx, |map, cx| {
                    map.insert(key.clone(), *index);
                    cx.notify();
                });
            }
            if let Some(f) = on_change {
                f(*index, window, cx);
            }
            (chain.close)(window, cx);
        }
    }
}

impl RenderOnce for Menu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let rows = {
            let mut rows = Vec::new();
            flatten(
                &self.items,
                &self.path,
                self.chain.selections.read(cx),
                &mut rows,
            );
            Rc::new(rows)
        };
        let state = window.use_keyed_state((id.clone(), "level"), cx, |_, _| LevelState::default());
        if let Some(parent) = &self.focus_first {
            let first = step(&rows, None, 1);
            state.update(cx, |s, _| {
                s.active = first;
                s.hovered = None;
            });
            parent.update(cx, |s, _| s.submenu_first = false);
        }
        let sub_focus = window
            .use_keyed_state((id.clone(), "sub-focus"), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let density = density::current();
        let height = self.size.item_height(density);
        let with_icons = rows
            .iter()
            .any(|r| matches!(r, Row::Entry { icon: Some(_), .. }));
        let with_selection = rows.iter().any(|r| {
            matches!(
                r,
                Row::Entry {
                    checked: Some(_),
                    ..
                }
            )
        });
        let geometry = MenuGeometry::of(with_icons);
        let look = MenuLook::current();
        let (active, hovered, submenu, submenu_focus) = {
            let s = state.read(cx);
            (s.active, s.hovered, s.submenu, s.submenu_focus)
        };
        let keyboard_focus = self.focus.is_focused(window);

        let on_key = {
            let rows = rows.clone();
            let state = state.clone();
            let chain = self.chain.clone();
            let parent = self.parent.clone();
            let extra = self.on_key.clone();
            move |event: &KeyDownEvent, window: &mut Window, cx: &mut App| {
                let key = event.keystroke.key.as_str();
                let current = state.read(cx).active.or(state.read(cx).hovered);
                let set_active = |index: Option<usize>, cx: &mut App| {
                    state.update(cx, |s, cx| {
                        s.active = index;
                        s.hovered = None;
                        cx.notify();
                    })
                };
                match key {
                    "down" => set_active(step(&rows, current, 1), cx),
                    "up" => set_active(step(&rows, current, -1), cx),
                    "home" => set_active(step(&rows, None, 1), cx),
                    "end" => set_active(step(&rows, None, -1), cx),
                    "enter" | "space" => {
                        if let Some(i) = current {
                            activate(&rows[i], i, &state, &chain, true, window, cx);
                        }
                    }
                    "right" => {
                        if let Some(i) = current
                            && matches!(
                                &rows[i],
                                Row::Entry {
                                    action: Action::Submenu(_),
                                    disabled: false,
                                    ..
                                }
                            )
                        {
                            activate(&rows[i], i, &state, &chain, true, window, cx);
                        } else if let Some(extra) = &extra {
                            extra(key, window, cx);
                        }
                    }
                    "left" if parent.is_some() => {
                        if let Some((_, close)) = &parent {
                            close(window, cx);
                        }
                    }
                    "escape" => match &parent {
                        Some((_, close)) => close(window, cx),
                        None => (chain.close)(window, cx),
                    },
                    "tab" => (chain.close)(window, cx),
                    _ => {
                        if let Some(extra) = &extra
                            && extra(key, window, cx)
                        {
                            cx.stop_propagation();
                            return;
                        }
                        let Some(ch) = event.keystroke.key_char.as_deref() else {
                            return;
                        };
                        if ch.chars().any(|c| c.is_control()) || ch.trim().is_empty() {
                            return;
                        }
                        let now = Instant::now();
                        let typed = state.update(cx, |s, _| {
                            if s.typed_at.is_none_or(|t| now.duration_since(t) > TYPEAHEAD) {
                                s.typed.clear();
                            }
                            s.typed.push_str(ch);
                            s.typed_at = Some(now);
                            s.typed.clone()
                        });
                        if let Some(i) = find_typed(&rows, current, &typed) {
                            set_active(Some(i), cx);
                        }
                    }
                }
                cx.stop_propagation();
            }
        };

        let mut children: Vec<AnyElement> = Vec::new();
        for (index, row) in rows.iter().enumerate() {
            let Row::Entry {
                label,
                shortcut,
                icon,
                disabled,
                danger,
                checked,
                action,
            } = row
            else {
                children.push(
                    div()
                        .my(rem(geometry.divider_margin))
                        .h(px(1.))
                        .w_full()
                        .bg(look.divider)
                        .into_any_element(),
                );
                continue;
            };
            let is_submenu = matches!(action, Action::Submenu(_));
            let item_state = ItemState {
                hovered: hovered == Some(index) || submenu == Some(index),
                focused: keyboard_focus && active == Some(index),
                danger: *danger,
                disabled: *disabled,
            };
            let (bg, text, outline) = MenuLook::item(item_state);
            let row_id = ElementId::NamedChild(
                std::sync::Arc::new(id.clone()),
                SharedString::from(format!("row-{index}")),
            );
            let bg = motion::animate_color((row_id.clone(), "bg"), bg, ITEM_MOTION, window, cx);
            let icon_color = text;
            let mut line = div()
                .id(row_id.clone())
                .relative()
                .h(rem(height))
                .flex_shrink_0()
                .px(rem(geometry.item_padding))
                .flex()
                .flex_row()
                .items_center()
                .gap(rem(geometry.column_gap))
                .bg(bg)
                .text_color(text)
                .type_style(TypeStyle::BodyCompact01)
                .cursor(if *disabled {
                    CursorStyle::OperationNotAllowed
                } else {
                    CursorStyle::PointingHand
                });
            if with_selection {
                line = line.child(div().size(rem(16.)).flex_shrink_0().when(
                    checked == &Some(true),
                    |this| {
                        this.child(
                            Icon::carbon(CarbonIcon::Checkmark)
                                .size(IconSize::S16)
                                .color(icon_color),
                        )
                    },
                ));
            }
            if with_icons {
                line = line.child(
                    div()
                        .size(rem(16.))
                        .flex_shrink_0()
                        .when_some(icon.clone(), |this, icon| {
                            this.child(icon.size(IconSize::S16).color(icon_color))
                        }),
                );
            }
            line = line
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .text_ellipsis()
                        .child(label.clone()),
                )
                .when_some(shortcut.clone(), |this, shortcut| {
                    this.child(div().flex_shrink_0().child(shortcut))
                })
                .when(is_submenu, |this| {
                    this.child(
                        Icon::carbon(CarbonIcon::CaretRight)
                            .size(IconSize::S16)
                            .color(icon_color),
                    )
                })
                .when_some(outline, |this, outline| {
                    // `.cds--menu-item:focus { outline: 2px solid var(--focus); outline-offset: -2px }`;
                    // danger adds `box-shadow: inset 0 0 0 0.1875rem var(--background)`.
                    this.child(
                        div()
                            .absolute()
                            .top_0()
                            .left_0()
                            .size_full()
                            .border_2()
                            .border_color(outline)
                            .when(*danger, |this| {
                                this.child(
                                    div()
                                        .size_full()
                                        .border_1()
                                        .border_color(color(ColorRole::Background)),
                                )
                            }),
                    )
                });
            if !*disabled {
                let state_hover = state.clone();
                let state_click = state.clone();
                let chain = self.chain.clone();
                let row_owned = row.clone();
                line = line
                    .on_hover(move |hovering, _, cx| {
                        state_hover.update(cx, |s, cx| {
                            if *hovering {
                                s.hovered = Some(index);
                                s.active = None;
                                // Pointing at another item closes an open submenu;
                                // pointing at a submenu item opens it.
                                s.submenu = is_submenu.then_some(index);
                                s.submenu_focus = false;
                                s.submenu_first = false;
                            } else if s.hovered == Some(index) {
                                s.hovered = None;
                            }
                            cx.notify();
                        })
                    })
                    .on_click(move |_, window, cx| {
                        activate(&row_owned, index, &state_click, &chain, false, window, cx)
                    });
            }
            let line = line.into_any_element();
            // The submenu hangs off its item's right edge, top-aligned with it
            // (`Menu` positions submenus at `[left, right]` of the item).
            if submenu == Some(index)
                && let Action::Submenu(items) = action
            {
                let close_sub = {
                    let state = state.clone();
                    let focus = self.focus.clone();
                    Rc::new(move |window: &mut Window, cx: &mut App| {
                        state.update(cx, |s, cx| {
                            s.submenu = None;
                            s.active = Some(index);
                            cx.notify();
                        });
                        focus.focus(window, cx);
                    }) as Callback
                };
                if submenu_focus {
                    // Opened from the keyboard: the submenu takes focus once.
                    state.update(cx, |s, _| s.submenu_focus = false);
                    let sub_focus = sub_focus.clone();
                    window.defer(cx, move |window, cx| sub_focus.focus(window, cx));
                }
                let mut sub = Menu::new(
                    (row_id.clone(), "sub"),
                    items.clone(),
                    sub_focus.clone(),
                    self.chain.clone(),
                )
                .size(self.size)
                .label(label.clone());
                sub.parent = Some((self.focus.clone(), close_sub));
                sub.focus_first = state.read(cx).submenu_first.then(|| state.clone());
                sub.path = format!("{}{index}.", self.path);
                let bounds = self.chain.bounds.clone();
                let sub = div()
                    .child(sub)
                    .child(
                        canvas(move |b, _, _| bounds.borrow_mut().push(b), |_, _, _, _| {})
                            .absolute()
                            .top_0()
                            .left_0()
                            .size_full(),
                    )
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation());
                children.push(
                    gpui_kit::base::Popup::new((row_id, "popup"), line)
                        .anchor(Anchor::LeftCenter)
                        .content(Translate::new(
                            sub,
                            point(0., 0.5),
                            point(px(0.), px(-height / 2. - geometry.padding_block)),
                        ))
                        .into_any_element(),
                );
            } else {
                children.push(line);
            }
        }

        div()
            .id(id.clone())
            .track_focus(&self.focus)
            .on_key_down(on_key)
            .flex()
            .flex_col()
            .py(rem(geometry.padding_block))
            .min_w(rem(self.min_width.unwrap_or(0.).max(geometry.min_width)))
            .max_w(rem(geometry.max_width.max(self.min_width.unwrap_or(0.))))
            .bg(look.background)
            .shadow(overlay::floating_shadow())
            .occlude()
            .children(children)
    }
}

/// What every open menu needs, whatever places it.
pub(crate) struct MenuSpec {
    pub items: Vec<MenuItem>,
    pub size: MenuSize,
    pub label: SharedString,
    pub open: bool,
    pub set_open: overlay::OpenChange,
    pub min_width: Option<f32>,
    pub on_unhandled_key: Option<KeyHook>,
}

/// The menu surface for `spec` (closing on a press outside it and its
/// submenus), or `None` while closed. Focus moves into it on open
/// and back on close.
pub(crate) fn menu_surface(
    id: &ElementId,
    spec: MenuSpec,
    window: &mut Window,
    cx: &mut App,
) -> Option<AnyElement> {
    let open = spec.open;
    let focus = overlay::surface_focus(id, open, None, window, cx);
    let selections = window.use_keyed_state((id.clone(), "selections"), cx, |_, _| HashMap::new());
    let bounds = window
        .use_keyed_state((id.clone(), "sub-bounds"), cx, |_, _| {
            Rc::new(RefCell::new(Vec::<Bounds<Pixels>>::new()))
        })
        .read(cx)
        .clone();
    if !open {
        return None;
    }
    let set_open = spec.set_open;
    let chain = Chain {
        close: {
            let set_open = set_open.clone();
            Rc::new(move |window, cx| set_open(false, window, cx))
        },
        selections,
        bounds: bounds.clone(),
    };
    let outside = {
        let bounds = bounds.clone();
        move |event: &gpui_kit::MouseDownEvent, window: &mut Window, cx: &mut App| {
            if open && !bounds.borrow().iter().any(|b| b.contains(&event.position)) {
                set_open(false, window, cx);
            }
        }
    };
    // Submenus re-register their bounds while they paint.
    bounds.borrow_mut().clear();
    let mut menu = Menu::new((id.clone(), "menu"), spec.items, focus, chain)
        .size(spec.size)
        .label(spec.label)
        .min_width(spec.min_width);
    menu.on_key = spec.on_unhandled_key;
    Some(
        div()
            .id((id.clone(), "surface"))
            .on_mouse_down_out(outside)
            .child(menu)
            .into_any_element(),
    )
}

/// A menu that hangs from a trigger's start edge (Base's `Popup`): the
/// shared body of DropdownMenu and Menubar menus.
pub(crate) fn anchored_menu(
    id: &ElementId,
    trigger: AnyElement,
    spec: MenuSpec,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    // Below the trigger, or above it when the menu only fits there (Carbon
    // menus `autoAlign`), never over it ([`super::overlay::DropSide`]).
    let anchor = super::overlay::drop_anchor((id.clone(), "drop"), window, cx);
    let trigger = div()
        .relative()
        .child(trigger)
        .child(anchor.measure_trigger());
    let side = anchor.place_measured(window);
    let popup = gpui_kit::base::Popup::new((id.clone(), "popup"), trigger).anchor(side.anchor());
    match menu_surface(id, spec, window, cx) {
        Some(surface) => popup
            .content(
                div()
                    .relative()
                    .child(surface)
                    .child(anchor.measure_surface()),
            )
            .into_any_element(),
        None => popup.into_any_element(),
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

    fn rows(items: &[MenuItem]) -> Vec<Row> {
        let mut out = Vec::new();
        flatten(items, "", &HashMap::new(), &mut out);
        out
    }

    #[test]
    fn item_heights_follow_menu_size_and_density() {
        // `.cds--menu--xs/sm/md/lg .cds--menu-item { block-size }`.
        let c = Density::Comfortable;
        assert_eq!(
            MenuSize::ALL.map(|s| s.item_height(c)),
            [24., 32., 40., 48.]
        );
        // `[data-density="compact"] .cds--menu-item { height: var(--density-row-md) }`.
        assert_eq!(MenuSize::Lg.item_height(Density::Compact), 32.);
        let g = MenuGeometry::of(false);
        assert_eq!(
            (g.padding_block, g.min_width, g.max_width),
            (4., 160., 288.)
        );
        assert_eq!(MenuGeometry::of(true).min_width, 192.);
    }

    #[test]
    fn item_colors_follow_menu_item_rules() {
        crate::theme::tests_set(CarbonTheme::White);
        let (bg, text, outline) = MenuLook::item(ItemState::default());
        assert_eq!((bg.a, hex(text), outline), (0., "#525252".into(), None));
        let (bg, text, _) = MenuLook::item(ItemState {
            hovered: true,
            ..Default::default()
        });
        assert_eq!((hex(bg), hex(text)), ("#e8e8e8".into(), "#161616".into()));
        let (bg, text, _) = MenuLook::item(ItemState {
            hovered: true,
            danger: true,
            ..Default::default()
        });
        assert_eq!((hex(bg), hex(text)), ("#da1e28".into(), "#ffffff".into()));
        let (_, _, outline) = MenuLook::item(ItemState {
            focused: true,
            ..Default::default()
        });
        assert_eq!(outline.map(hex).as_deref(), Some("#0f62fe"));
        let (bg, _, _) = MenuLook::item(ItemState {
            hovered: true,
            disabled: true,
            ..Default::default()
        });
        assert_eq!(bg.a, 0.);
        assert_eq!(hex(MenuLook::current().background), "#f4f4f4");
    }

    #[test]
    fn danger_items_are_red_at_rest_and_dividers_use_border_subtle_00() {
        let danger = ItemState {
            danger: true,
            ..Default::default()
        };
        crate::theme::tests_set(CarbonTheme::White);
        let (bg, text, _) = MenuLook::item(danger);
        assert_eq!((bg.a, hex(text)), (0., "#da1e28".into()));
        assert_eq!(hex(MenuLook::current().divider), "#e0e0e0");
        // Disabled wins over danger.
        let (_, text, _) = MenuLook::item(ItemState {
            disabled: true,
            ..danger
        });
        assert_eq!(text, color(ColorRole::TextDisabled));
        crate::theme::tests_set(CarbonTheme::G90);
        let (_, text, _) = MenuLook::item(danger);
        assert_eq!(hex(text), "#ffb3b8");
        assert_eq!(hex(MenuLook::current().divider), "#525252");
        crate::theme::tests_set(CarbonTheme::White);
    }

    #[test]
    fn keyboard_skips_disabled_items_and_dividers_and_wraps() {
        let rows = rows(&[
            MenuItem::item("Restart"),
            MenuItem::item("Resize").disabled(true),
            MenuItem::divider(),
            MenuItem::item("Delete").danger(true),
        ]);
        assert_eq!(step(&rows, None, 1), Some(0));
        assert_eq!(step(&rows, Some(0), 1), Some(3));
        assert_eq!(step(&rows, Some(3), 1), Some(0));
        assert_eq!(step(&rows, None, -1), Some(3));
        assert_eq!(step(&rows, Some(0), -1), Some(3));
    }

    #[test]
    fn typeahead_finds_the_next_match() {
        let rows = rows(&[
            MenuItem::item("Restart"),
            MenuItem::item("Resize"),
            MenuItem::item("Move to"),
        ]);
        assert_eq!(find_typed(&rows, None, "r"), Some(0));
        assert_eq!(find_typed(&rows, Some(0), "r"), Some(1));
        assert_eq!(find_typed(&rows, Some(1), "r"), Some(0));
        assert_eq!(find_typed(&rows, Some(0), "res"), Some(0));
        assert_eq!(find_typed(&rows, Some(0), "resi"), Some(1));
        assert_eq!(find_typed(&rows, None, "m"), Some(2));
    }

    #[test]
    fn selectable_and_radio_rows_carry_their_state() {
        let rows = rows(&[
            MenuItem::checkbox("Show archived").default_selected(true),
            MenuItem::divider(),
            MenuItem::radio_group("Sort", ["Name", "Date"]).default_selected_item(1),
        ]);
        let checked: Vec<_> = rows
            .iter()
            .map(|r| match r {
                Row::Entry { checked, .. } => *checked,
                Row::Divider => None,
            })
            .collect();
        assert_eq!(checked, [Some(true), None, Some(false), Some(true)]);
    }
}
