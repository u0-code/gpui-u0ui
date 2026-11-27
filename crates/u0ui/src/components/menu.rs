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
//! - **Keyboard** (Carbon `Menu`): opening (pointer or keyboard) focuses the
//!   first enabled item, with its focus ring; ↓/↑ move (wrapping, skipping disabled
//!   items), Home/End, Enter/Space activate, → or Enter opens a submenu and
//!   enters it, ←/Escape leaves a submenu, Escape closes the menu (focus
//!   returns to the trigger), typing jumps to the next item that starts
//!   with the typed text (type-ahead, 500ms buffer).
//! - **Leaving** (Carbon `Menu`'s `onKeyDown` / `onBlur`): Tab and Shift+Tab
//!   close the menu and return focus to the element that had it when the
//!   menu opened (Carbon prevents the Tab, focuses that element and calls
//!   `onClose`). They need [`init`]'s bindings, which beat Base's
//!   window-level Tab. A press outside closes it too (focus stays where the
//!   press put it), and so does focus leaving the menu any other way.
//! - **Appearance** ([`MenuLook`], [`MenuGeometry`]): `layer-01` with the
//!   floating shadow, 0.25rem block padding, 10rem–18rem wide (12rem with
//!   icons or selectable items); items `text-secondary` body-compact-01,
//!   1rem inline padding, height per size (24/32/40/48) or `density-row-md`
//!   in compact density; hover `layer-hover` + `text-primary`; danger labels
//!   `text-error` at rest (icon and shortcut stay `text-secondary`); danger hover
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
use gpui_kit::{KeyBinding, TestSupportExt as _, actions};
use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    time::{Duration, Instant},
};

actions!(
    u0ui_menu,
    [
        /// Tab or Shift+Tab inside an open menu: close it (Carbon `Menu`).
        TabOut
    ]
);

/// The key context of an open menu's surface.
pub(crate) const KEY_CONTEXT: &str = "U0Menu";

/// Bind Tab and Shift+Tab inside menus to [`TabOut`]. A binding in the
/// menu's own key context is deeper than Base's window-level `root::Tab`,
/// so it wins: without it the root moves focus and the menu stays open.
pub(crate) fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("tab", TabOut, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-tab", TabOut, Some(KEY_CONTEXT)),
    ]);
}

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
        // `.cds--menu-item { color: var(--text-secondary) }`; a danger item's
        // label is red at rest ([`Self::label`]).
        (clear, color(TextSecondary), outline)
    }

    /// The label color of an item in `state`: the item color, except a
    /// danger item at rest, whose label is `text-error` (`t3.css` app-r3 "the
    /// dangerous git menu item reads as dangerous before hover", also in
    /// `bundle.css`: `.cds--menu-item--danger:not(.cds--menu-item--disabled)
    /// .cds--menu-item__label { color: var(--text-error) }`, and `:hover`,
    /// `:focus` labels `text-on-color`). The rule names `text-error`, not
    /// `support-error`, in every theme (G90/G100 `#ffb3b8`/`#ff8389`); no later
    /// u0/t3 rule overrides it. Only the label: the icon and shortcut keep the
    /// item color.
    pub fn label(state: ItemState) -> Hsla {
        let (_, text, _) = Self::item(state);
        if state.danger && !state.disabled && !state.hovered && !state.focused {
            color(ColorRole::TextError)
        } else {
            text
        }
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

/// The row a menu focuses when it opens. Carbon `Menu` focuses its first
/// enabled item (`focusItem()` on open, from the pointer or the keyboard).
/// A menu that is only one radio group is a select (the Pagination pickers):
/// it starts on the picked option, as a native `<select>` does.
fn initial_row(rows: &[Row]) -> Option<usize> {
    let only_picks = rows.iter().all(|r| {
        matches!(
            r,
            Row::Entry {
                action: Action::Pick { .. },
                ..
            }
        )
    });
    let picked = only_picks
        .then(|| {
            rows.iter().position(|r| {
                r.enabled()
                    && matches!(
                        r,
                        Row::Entry {
                            checked: Some(true),
                            ..
                        }
                    )
            })
        })
        .flatten();
    picked.or_else(|| step(rows, None, 1))
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
    /// Set when the root menu opens and cleared by the first render of the
    /// menu itself (which may come frames later: an anchored menu waits to be
    /// measured): start over on [`initial_row`].
    opening: Option<Entity<bool>>,
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
            opening: None,
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
        let opening = self.opening.as_ref().is_some_and(|o| *o.read(cx));
        if let Some(flag) = &self.opening
            && opening
        {
            flag.update(cx, |v, _| *v = false);
        }
        if opening {
            // A fresh open forgets the last one (highlight, open submenu,
            // type-ahead) and focuses the first item, keyboard ring included.
            let first = initial_row(&rows);
            state.update(cx, |s, _| {
                *s = LevelState {
                    active: first,
                    ..LevelState::default()
                }
            });
        }
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
                    // Reached only without [`init`]'s bindings.
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
            let label_color = MenuLook::label(item_state);
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
                        .text_color(label_color)
                        .child(label.clone()),
                )
                // The trailing `max-content` column (`grid-template-columns:
                // … 1fr max-content; column-gap: 0.5rem`) is there even when
                // empty, so its 8px gap always counts toward the menu's
                // width: the shortcut, or a submenu's caret.
                .child(
                    div()
                        .flex_shrink_0()
                        .flex()
                        .items_center()
                        .when(!is_submenu, |this| this.children(shortcut.clone()))
                        .when(is_submenu, |this| {
                            this.child(
                                Icon::carbon(CarbonIcon::CaretRight)
                                    .size(IconSize::S16)
                                    .color(icon_color),
                            )
                        }),
                )
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

        let tab_out = {
            let chain = self.chain.clone();
            move |_: &TabOut, window: &mut Window, cx: &mut App| {
                (chain.close)(window, cx);
                cx.stop_propagation();
            }
        };
        div()
            .id(id.clone())
            .test_support()
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus)
            .on_key_down(on_key)
            .on_action(tab_out)
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
    menu_surface_returning(id, spec, None, window, cx)
}

/// [`menu_surface`] that returns focus to `return_to` on close (when given)
/// instead of to whatever had focus when it opened.
pub(crate) fn menu_surface_returning(
    id: &ElementId,
    spec: MenuSpec,
    return_to: Option<FocusHandle>,
    window: &mut Window,
    cx: &mut App,
) -> Option<AnyElement> {
    let open = spec.open;
    let focus = overlay::surface_focus_returning(id, open, None, return_to, window, cx);
    let selections = window.use_keyed_state((id.clone(), "selections"), cx, |_, _| HashMap::new());
    let bounds = window
        .use_keyed_state((id.clone(), "sub-bounds"), cx, |_, _| {
            Rc::new(RefCell::new(Vec::<Bounds<Pixels>>::new()))
        })
        .read(cx)
        .clone();
    let was_open = window.use_keyed_state((id.clone(), "was-open"), cx, |_, _| false);
    let opening = open && !*was_open.read(cx);
    if open != *was_open.read(cx) {
        was_open.update(cx, |v, _| *v = open);
    }
    // Whether focus has been inside the open menu (it moves in a frame
    // after the open); once it has, focus leaving closes the menu (Carbon
    // `Menu` `onBlur`). Submenus count as inside: they are its children.
    let had_focus = window.use_keyed_state((id.clone(), "had-focus"), cx, |_, _| false);
    if !open {
        if *had_focus.read(cx) {
            had_focus.update(cx, |v, _| *v = false);
        }
        return None;
    }
    let set_open = spec.set_open;
    // Focus moves in once the surface has painted. Overlay's open-time
    // focus can land before that (a menu open on arrival, or one waiting a
    // frame to be measured), when the handle is in no rendered frame yet and
    // the focus is lost: the first paint takes it again.
    let pending = window.use_keyed_state((id.clone(), "focus-pending"), cx, |_, _| false);
    let fresh = window.use_keyed_state((id.clone(), "fresh"), cx, |_, _| false);
    if opening {
        had_focus.update(cx, |v, _| *v = false);
        pending.update(cx, |v, _| *v = true);
        fresh.update(cx, |v, _| *v = true);
    } else if focus.contains_focused(window, cx) {
        if !*had_focus.read(cx) {
            had_focus.update(cx, |v, _| *v = true);
        }
    } else if *had_focus.read(cx)
        // "Inside" is answered from the last painted frame. A submenu opened
        // from the keyboard takes focus before its popup has painted (the
        // popup waits a frame to measure its item), so for one frame the
        // focused handle is in no frame at all: decide once it is painted.
        && window.focused(cx).is_none_or(|f| f.is_rendered(window))
    {
        had_focus.update(cx, |v, _| *v = false);
        let set_open = set_open.clone();
        window.defer(cx, move |window, cx| set_open(false, window, cx));
    }
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
    menu.opening = Some(fresh);
    let take_focus = {
        let focus = menu.focus.clone();
        canvas(
            |_, _, _| {},
            move |_, _, window, cx| {
                if !*pending.read(cx) {
                    return;
                }
                pending.update(cx, |v, _| *v = false);
                let focus = focus.clone();
                window.defer(cx, move |window, cx| {
                    if !focus.contains_focused(window, cx) {
                        focus.focus(window, cx);
                    }
                });
            },
        )
        .absolute()
        .size_0()
    };
    Some(
        div()
            .id((id.clone(), "surface"))
            .on_mouse_down_out(outside)
            .child(menu)
            .child(take_focus)
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
    anchored_menu_aligned(id, trigger, spec, false, window, cx)
}

/// [`anchored_menu`], hanging from the trigger's end edge when `end`
/// (`menuAlignment="bottom-end"`).
pub(crate) fn anchored_menu_aligned(
    id: &ElementId,
    trigger: AnyElement,
    spec: MenuSpec,
    end: bool,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    anchored_menu_impl(
        id,
        trigger,
        spec,
        end,
        false,
        None,
        None,
        Shift::NONE,
        window,
        cx,
    )
}

/// [`anchored_menu`] moved `dx` px along the trigger's start edge (only a
/// negative shift, toward the start) and `gap` px away from it: a
/// breadcrumb's overflow menu (`.cds--breadcrumb-menu-options`, placed 13px
/// before its trigger and 10px below it to leave room for its caret).
pub(crate) fn anchored_menu_offset(
    id: &ElementId,
    trigger: AnyElement,
    spec: MenuSpec,
    dx: f32,
    gap: f32,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    anchored_menu_impl(
        id,
        trigger,
        spec,
        false,
        false,
        None,
        None,
        Shift {
            dx: dx.min(0.),
            gap,
            caret: true,
        },
        window,
        cx,
    )
}

/// [`anchored_menu`] whose trigger fills its parent's height (a menubar
/// trigger: `.cu-menubar-trigger { height: 100% }`). Closing returns focus
/// to `trigger_focus`, the trigger whose menu was open.
pub(crate) fn anchored_menu_full_height(
    id: &ElementId,
    trigger: AnyElement,
    trigger_focus: FocusHandle,
    spec: MenuSpec,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    anchored_menu_impl(
        id,
        trigger,
        spec,
        false,
        true,
        Some(trigger_focus),
        None,
        Shift::NONE,
        window,
        cx,
    )
}

/// Carbon `MenuButton`'s menu (`MenuButton` in `bundle.js`): centred under
/// its trigger (`menuAlignment="bottom"`), and as wide as the trigger
/// (floating-ui's `size` middleware sets the menu's `width` to the
/// trigger's) held to `.cds--menu`'s `min-inline-size` / `max-inline-size`
/// ([`button_menu_width`]), so a long label ellipsizes instead of widening
/// the menu. `trigger_width` is in px at the 16px root size.
pub(crate) fn anchored_button_menu(
    id: &ElementId,
    trigger: AnyElement,
    spec: MenuSpec,
    trigger_width: f32,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let width = button_menu_width(&spec.items, trigger_width);
    anchored_menu_impl(
        id,
        trigger,
        spec,
        false,
        false,
        None,
        Some(width),
        Shift::NONE,
        window,
        cx,
    )
}

/// The width (px at the 16px root size) of a `MenuButton` menu of `items`
/// under a `trigger_width` trigger: the trigger's width between `10rem` (or
/// `12rem` with icons) and `18rem`.
pub fn button_menu_width(items: &[MenuItem], trigger_width: f32) -> f32 {
    let mut rows = Vec::new();
    flatten(items, "", &HashMap::new(), &mut rows);
    let with_icons = rows
        .iter()
        .any(|r| matches!(r, Row::Entry { icon: Some(_), .. }));
    let geometry = MenuGeometry::of(with_icons);
    trigger_width
        .min(geometry.max_width)
        .max(geometry.min_width)
}

/// Where an anchored menu sits against its trigger, beyond the anchor.
#[derive(Clone, Copy)]
struct Shift {
    /// Toward the start edge (px, zero or negative).
    dx: f32,
    /// Away from the trigger (px).
    gap: f32,
    /// Draw the breadcrumb caret.
    caret: bool,
}

impl Shift {
    const NONE: Self = Self {
        dx: 0.,
        gap: 0.,
        caret: false,
    };
}

/// `.cds--breadcrumb-menu-options.cds--overflow-menu-options::after`: a
/// 0.4375rem-tall triangle in `field` standing on the menu's top edge,
/// 0.875rem from its start (`border-block-end: 0.4375rem solid var(--field);
/// border-inline: 0.4375rem solid transparent; inset-block-start: -0.4375rem;
/// inset-inline-start: 0.875rem`).
fn breadcrumb_caret() -> impl IntoElement {
    let fill = color(ColorRole::Field01);
    canvas(
        |_, _, _| {},
        move |bounds, _, window, _| {
            let o = bounds.origin;
            let w = bounds.size.width;
            let h = bounds.size.height;
            let mut path = gpui_kit::PathBuilder::fill();
            path.move_to(gpui_kit::point(o.x, o.y + h));
            path.line_to(gpui_kit::point(o.x + w / 2., o.y));
            path.line_to(gpui_kit::point(o.x + w, o.y + h));
            path.close();
            if let Ok(path) = path.build() {
                window.paint_path(path, fill);
            }
        },
    )
    .absolute()
    .top(rem(-7.))
    .left(rem(14.))
    .w(rem(14.))
    .h(rem(7.))
}

#[allow(clippy::too_many_arguments)]
fn anchored_menu_impl(
    id: &ElementId,
    trigger: AnyElement,
    spec: MenuSpec,
    end: bool,
    full_height: bool,
    return_to: Option<FocusHandle>,
    centered_width: Option<f32>,
    offset: Shift,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    // Below the trigger, or above it when the menu only fits there (Carbon
    // menus `autoAlign`), never over it ([`super::overlay::DropSide`]).
    let anchor = super::overlay::drop_anchor((id.clone(), "drop"), window, cx);
    let trigger = div()
        .relative()
        .when(full_height, |this| this.h_full())
        .child(trigger)
        .child(anchor.measure_trigger());
    let side = anchor.place_measured(window);
    let popup = gpui_kit::base::Popup::new((id.clone(), "popup"), trigger)
        .anchor(if centered_width.is_some() {
            side.anchor_center()
        } else if end {
            side.anchor_end()
        } else {
            side.anchor()
        })
        .when(full_height, |this| this.h_full())
        // A start-edge shift: the measured box reaches `-dx` px before the
        // trigger and pads it back, so the trigger stays put and the surface
        // hangs from the box's start edge.
        .when(offset.dx < 0., |this| {
            this.ml(gpui_kit::px(offset.dx))
                .pl(gpui_kit::px(-offset.dx))
        })
        .offset(gpui_kit::px(offset.gap));
    let surface =
        menu_surface_returning(id, spec, return_to, window, cx).map(
            |surface| match centered_width {
                // The menu stretches to the fixed width (its own min/max agree).
                Some(width) => div().w(rem(width)).child(surface).into_any_element(),
                None => surface,
            },
        );
    // The caret points at the trigger from the menu's top edge; a menu
    // flipped above its trigger has none.
    let surface = surface.map(|surface| {
        if offset.caret && side == super::overlay::DropSide::Below {
            div()
                .relative()
                .child(surface)
                .child(breadcrumb_caret())
                .into_any_element()
        } else {
            surface
        }
    });
    match surface {
        // Not painted until measured on the side it fits (no first-open
        // frame below the trigger before the flip).
        Some(surface) => popup.content(anchor.surface(surface)).into_any_element(),
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
        assert_eq!((bg.a, hex(text)), (0., "#525252".into()));
        assert_eq!(hex(MenuLook::label(danger)), "#da1e28");
        let hovered = ItemState {
            hovered: true,
            ..danger
        };
        assert_eq!(hex(MenuLook::label(hovered)), "#ffffff");
        assert_eq!(hex(MenuLook::current().divider), "#e0e0e0");
        // Disabled wins over danger.
        let (_, text, _) = MenuLook::item(ItemState {
            disabled: true,
            ..danger
        });
        assert_eq!(text, color(ColorRole::TextDisabled));
        crate::theme::tests_set(CarbonTheme::G90);
        // `text-error`, not `support-error` (#ff8389 on G90).
        assert_eq!(hex(MenuLook::label(danger)), "#ffb3b8");
        crate::theme::tests_set(CarbonTheme::G100);
        assert_eq!(hex(MenuLook::label(danger)), "#ff8389");
        crate::theme::tests_set(CarbonTheme::G90);
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

    #[test]
    fn opening_focuses_the_first_enabled_item_or_the_picked_option() {
        // Carbon `Menu`: the first enabled item ("Restart", then ↓ is "Resize").
        let actions = rows(&[
            MenuItem::item("Restart").disabled(true),
            MenuItem::item("Resize"),
            MenuItem::divider(),
            MenuItem::item("Delete").danger(true),
        ]);
        assert_eq!(initial_row(&actions), Some(1));
        // A mixed menu ignores its radio group's pick.
        let mixed = rows(&[
            MenuItem::checkbox("Show archived"),
            MenuItem::divider(),
            MenuItem::radio_group("Sort", ["Name", "Date"]).default_selected_item(1),
        ]);
        assert_eq!(initial_row(&mixed), Some(0));
        // A lone radio group is a select: it starts on the picked option.
        let select = rows(&[MenuItem::radio_group("Page", ["1", "2", "3"]).selected_item(Some(2))]);
        assert_eq!(initial_row(&select), Some(2));
        let none = rows(&[MenuItem::radio_group("Page", ["1", "2"]).selected_item(None)]);
        assert_eq!(initial_row(&none), Some(0));
    }
}
