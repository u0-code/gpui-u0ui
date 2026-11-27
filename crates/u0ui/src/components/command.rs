//! Command: a command palette, a Carbon `Search` (lg) over a filterable list
//! whose items look like Carbon menu items (`.cu-command*` in `custom.css`).
//! [`CommandDialog`] puts it in a passive Modal.
//!
//! - **Behavior**: text editing is Kit's `InputState`/`Input`. Typing filters
//!   items by their value (or label), case-insensitively; groups without a
//!   match disappear, separators hide while a query is typed, and the empty
//!   text shows when nothing matches. ↑/↓ move the highlighted item (wrapping,
//!   skipping disabled ones), Enter runs it (the first enabled match when
//!   nothing is highlighted, as cmdk selects its first item), Escape clears
//!   the query. Pointer hover highlights, click runs. [`CommandDialog`] opens
//!   with an empty query and focus in the search field.
//! - **Appearance**: `layer-01` box with the floating shadow, max 32rem; the
//!   search field is 3rem tall with a 16px magnifier, `border-subtle-01`
//!   bottom line and a clear button; the list scrolls past 18rem; headings
//!   are label-01 `text-secondary` (0.75rem 1rem 0.25rem); items are
//!   `size-md` (2.5rem) rows with a 16px `icon-secondary` icon, label, and a
//!   mono 12px shortcut.
//! - **Item states** (`.cu-command-item:hover, :focus-visible { background:
//!   layer-hover-01 }`, `:focus-visible { box-shadow: inset 0 0 0 2px focus }`):
//!   a row takes `layer-hover-01` only while the pointer is over it or while it
//!   is the keyboard item (↑/↓ from the focused field). The keyboard item
//!   stands in for the reference's focused `<button>`, so it also gets the
//!   ring and the field drops its focus outline meanwhile: in the reference
//!   DOM focus is in either the search input or an item, never both, so the
//!   CSS never draws the two outlines at once. Typing clears the keyboard item
//!   and the field's outline returns; once focus leaves the palette no row
//!   stays highlighted.
//!
//! ```ignore
//! Command::new("palette")
//!     .group(CommandGroup::new("Suggestions")
//!         .item(CommandItem::new("Open dashboard").icon(CarbonIcon::Dashboard)))
//! ```
use super::dialog::{Dialog, DialogSize};
use super::overlay::{self, Callback, rem};
use crate::{
    icons::{CarbonIcon, Icon, IconSize},
    motion::{self, Motion},
    theme::color,
    tokens::{ColorRole, FontFamilyKey, SIZE_LG, SIZE_MD, TypeStyle},
    typography::{self, TypeStyled as _},
};
use gpui_kit::{
    App, AppContext as _, CursorStyle, ElementId, Entity, Focusable as _, Hsla, InteractiveElement,
    IntoElement, KeyDownEvent, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement as _, Styled, Subscription, Window,
    component::Sizable as _,
    component::input::{Input, InputEvent, InputState},
    div,
    prelude::FluentBuilder as _,
    px,
};
use std::rc::Rc;

/// One command.
#[derive(Clone)]
pub struct CommandItem {
    label: SharedString,
    value: Option<SharedString>,
    icon: Option<Icon>,
    shortcut: Option<SharedString>,
    disabled: bool,
    on_select: Option<Callback>,
}

impl CommandItem {
    /// A command with a visible `label`.
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            value: None,
            icon: None,
            shortcut: None,
            disabled: false,
            on_select: None,
        }
    }

    /// Text to filter on instead of the label.
    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Leading icon (`.cu-command-icon`).
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Shortcut text (`.cu-command-shortcut`).
    pub fn shortcut(mut self, shortcut: impl Into<SharedString>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// Disable the command.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Run when chosen.
    pub fn on_select(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }

    /// Whether the item matches `query` (`index.jsx`: `(value || text).toLowerCase().includes(q)`).
    pub fn matches(&self, query: &str) -> bool {
        self.value
            .as_ref()
            .unwrap_or(&self.label)
            .to_lowercase()
            .contains(&query.to_lowercase())
    }
}

/// A headed group of commands (`CommandGroup heading`).
#[derive(Clone)]
pub struct CommandGroup {
    heading: Option<SharedString>,
    items: Vec<CommandItem>,
}

impl CommandGroup {
    /// A group under `heading`.
    pub fn new(heading: impl Into<SharedString>) -> Self {
        Self {
            heading: Some(heading.into()),
            items: Vec::new(),
        }
    }

    /// A group without a heading.
    pub fn untitled() -> Self {
        Self {
            heading: None,
            items: Vec::new(),
        }
    }

    /// Append a command.
    pub fn item(mut self, item: CommandItem) -> Self {
        self.items.push(item);
        self
    }
}

#[derive(Clone)]
enum Part {
    Group(CommandGroup),
    Separator,
}

/// What the list shows for a query: `(part index, visible item indices)` per
/// visible group, separators (only without a query), and whether it is empty.
#[derive(Debug, PartialEq)]
pub(crate) enum Visible {
    Group(usize, Vec<usize>),
    Separator,
}

fn visible(parts: &[Part], query: &str) -> Vec<Visible> {
    let mut out = Vec::new();
    for (i, part) in parts.iter().enumerate() {
        match part {
            // `CommandSeparator`: `c.q ? null : <div className="cu-command-sep" />`.
            Part::Separator if query.is_empty() => out.push(Visible::Separator),
            Part::Separator => {}
            Part::Group(group) => {
                let items: Vec<usize> = group
                    .items
                    .iter()
                    .enumerate()
                    .filter(|(_, item)| item.matches(query))
                    .map(|(j, _)| j)
                    .collect();
                if !items.is_empty() {
                    out.push(Visible::Group(i, items));
                }
            }
        }
    }
    out
}

/// The next enabled index from `from` by `dir` (±1), wrapping; `None` starts
/// before the first (down) or after the last (up) item.
fn step(from: Option<usize>, dir: isize, disabled: &[bool]) -> Option<usize> {
    let n = disabled.len() as isize;
    let mut i = from.map_or(if dir > 0 { -1 } else { n }, |i| i as isize);
    for _ in 0..n {
        i = (i + dir).rem_euclid(n);
        if !disabled[i as usize] {
            return Some(i as usize);
        }
    }
    None
}

/// The palette's keyed state (shared with [`CommandDialog`], which focuses
/// its field).
fn command_state(id: &ElementId, window: &mut Window, cx: &mut App) -> Entity<CommandState> {
    window.use_keyed_state((id.clone(), "command"), cx, |window, cx| {
        let input = cx.new(|cx| InputState::new(window, cx));
        let subscription = cx.subscribe(&input, |this: &mut CommandState, _, event, cx| {
            if matches!(event, InputEvent::Change) {
                this.active = None;
                cx.notify();
            }
        });
        CommandState {
            input,
            active: None,
            keyboard: false,
            _subscription: subscription,
        }
    })
}

struct CommandState {
    input: Entity<InputState>,
    active: Option<usize>,
    keyboard: bool,
    _subscription: Subscription,
}

/// A command palette. See the module docs.
#[derive(IntoElement)]
pub struct Command {
    id: ElementId,
    parts: Vec<Part>,
    placeholder: SharedString,
    empty: SharedString,
    label: SharedString,
    clear_label: SharedString,
    stable_height: bool,
    /// State owned by a host ([`CommandDialog`] keys it outside its modal,
    /// so it can reach the field's focus handle).
    state: Option<Entity<CommandState>>,
}

impl Command {
    /// A command palette with a stable `id`.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            parts: Vec::new(),
            // `index.jsx` defaults; override per locale.
            placeholder: "Type a command or search…".into(),
            empty: "No results found.".into(),
            label: "Command menu".into(),
            clear_label: "Clear".into(),
            stable_height: false,
            state: None,
        }
    }

    /// Hold the list at its full `.cu-command-list` height (18rem) while the
    /// query filters it, instead of shrinking to the matches, so a
    /// [`CommandDialog`] around it keeps its size and does not re-center as
    /// the user types. Off by default (the reference list shrinks).
    pub fn stable_height(mut self, stable: bool) -> Self {
        self.stable_height = stable;
        self
    }

    /// The search placeholder (`CommandInput placeholder`).
    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = text.into();
        self
    }

    /// The text shown when nothing matches (`CommandEmpty`).
    pub fn empty_text(mut self, text: impl Into<SharedString>) -> Self {
        self.empty = text.into();
        self
    }

    /// The palette's accessible name.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    /// The clear button's accessible name.
    pub fn clear_label(mut self, label: impl Into<SharedString>) -> Self {
        self.clear_label = label.into();
        self
    }

    /// Append a group.
    pub fn group(mut self, group: CommandGroup) -> Self {
        self.parts.push(Part::Group(group));
        self
    }

    /// Append a separator (`CommandSeparator`).
    pub fn separator(mut self) -> Self {
        self.parts.push(Part::Separator);
        self
    }
}

/// The colors of the palette.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CommandLook {
    /// `.cu-command { background: var(--layer-01) }`.
    pub background: Hsla,
    /// `.cu-command .cds--search-input { border-bottom-color: var(--border-subtle-01) }`.
    pub search_border: Hsla,
    /// `.cu-command-heading`, shortcut, empty text: `text-secondary`.
    pub secondary: Hsla,
    /// Item text: `text-primary`.
    pub text: Hsla,
    /// `.cu-command-item:hover { background: var(--layer-hover-01) }`.
    pub hover: Hsla,
    /// `.cu-command-icon { fill: var(--icon-secondary) }`.
    pub icon: Hsla,
    /// `:disabled { color: var(--text-disabled) }`.
    pub disabled: Hsla,
    /// `.cu-command-sep { background: var(--border-subtle-01) }`.
    pub separator: Hsla,
}

impl CommandLook {
    /// The look on the active theme.
    pub fn current() -> Self {
        use ColorRole::*;
        Self {
            background: color(Layer01),
            search_border: color(BorderSubtle01),
            secondary: color(TextSecondary),
            text: color(TextPrimary),
            hover: color(LayerHover01),
            icon: color(IconSecondary),
            disabled: color(TextDisabled),
            separator: color(BorderSubtle01),
        }
    }
}

impl RenderOnce for Command {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let state = match self.state.clone() {
            Some(state) => state,
            None => command_state(&id, window, cx),
        };
        let input = state.read(cx).input.clone();
        let query = input.read(cx).value().to_string();
        let parts = Rc::new(self.parts);
        let shown = visible(&parts, &query);
        // Flat list of (part, item) for keyboard movement.
        let flat: Rc<Vec<(usize, usize)>> = Rc::new(
            shown
                .iter()
                .filter_map(|v| match v {
                    Visible::Group(p, items) => {
                        Some(items.iter().map(|i| (*p, *i)).collect::<Vec<_>>())
                    }
                    Visible::Separator => None,
                })
                .flatten()
                .collect(),
        );
        let item_at = {
            let parts = parts.clone();
            move |(p, i): (usize, usize)| match &parts[p] {
                Part::Group(group) => group.items[i].clone(),
                Part::Separator => unreachable!(),
            }
        };
        let (active, keyboard) = {
            let s = state.read(cx);
            (s.active, s.keyboard)
        };
        let look = CommandLook::current();
        // Focus lives in the search field; the list is driven from it.
        let input_focused = input.read(cx).focus_handle(cx).contains_focused(window, cx);
        // An item is "focused" (see the module docs): the field's outline steps aside.
        let keyboard_item = input_focused && keyboard && active.is_some();

        let on_key = {
            let state = state.clone();
            let flat = flat.clone();
            let item_at = item_at.clone();
            let input = input.clone();
            move |event: &KeyDownEvent, window: &mut Window, cx: &mut App| {
                let n = flat.len();
                let disabled: Vec<bool> = flat.iter().map(|&at| item_at(at).disabled).collect();
                let current = state.read(cx).active;
                match event.keystroke.key.as_str() {
                    "down" | "up" if n > 0 => {
                        let dir = if event.keystroke.key == "down" { 1 } else { -1 };
                        let next = step(current, dir, &disabled);
                        state.update(cx, |s, cx| {
                            s.active = next;
                            s.keyboard = true;
                            cx.notify();
                        });
                        cx.stop_propagation();
                    }
                    "enter" => {
                        if let Some(i) = current.or_else(|| step(None, 1, &disabled))
                            && let item = item_at(flat[i])
                            && !item.disabled
                            && let Some(f) = item.on_select
                        {
                            f(window, cx);
                        }
                        cx.stop_propagation();
                    }
                    "escape" if !input.read(cx).value().is_empty() => {
                        input.update(cx, |s, cx| s.set_value("", window, cx));
                        cx.stop_propagation();
                    }
                    _ => {}
                }
            }
        };

        // `.cds--search--lg`: 3rem; magnifier at 1rem; input padded 3rem.
        let search = div()
            .relative()
            .h(SIZE_LG.rems())
            .flex()
            .items_center()
            .flex_shrink_0()
            .bg(look.background)
            .border_b_1()
            .border_color(look.search_border)
            .child(
                div()
                    .absolute()
                    .left(rem(16.))
                    .top_0()
                    .bottom_0()
                    .flex()
                    .items_center()
                    .child(
                        Icon::carbon(CarbonIcon::Search)
                            .size(IconSize::S16)
                            .color(look.icon),
                    ),
            )
            .child(
                div()
                    .relative()
                    .flex_1()
                    .h_full()
                    .pl(rem(48.))
                    .pr(rem(48.))
                    .flex()
                    .items_center()
                    .type_style(TypeStyle::BodyCompact01)
                    .text_color(look.text)
                    .when(query.is_empty(), |this| {
                        this.child(
                            div()
                                .absolute()
                                .left(rem(48.))
                                .text_color(color(ColorRole::TextPlaceholder))
                                .child(self.placeholder.clone()),
                        )
                    })
                    .child(
                        Input::new(&input)
                            .appearance(false)
                            .with_size(gpui_kit::component::Size::XSmall)
                            .aria_label(self.label.clone())
                            .p_0()
                            .h_full()
                            .w_full()
                            .text_size(rem(14.))
                            .line_height(rem(18.)),
                    ),
            )
            .when(!query.is_empty(), |this| {
                let input = input.clone();
                this.child(
                    gpui_kit::base::Button::new((id.clone(), "clear"))
                        .accessibility_label(self.clear_label.clone())
                        .absolute()
                        .right_0()
                        .top_0()
                        .size(SIZE_LG.rems())
                        .flex()
                        .items_center()
                        .justify_center()
                        .hover(|s| s.bg(look.hover))
                        .cursor(CursorStyle::PointingHand)
                        .on_click(move |_, window, cx| {
                            input.update(cx, |s, cx| s.set_value("", window, cx))
                        })
                        .child(
                            Icon::carbon(CarbonIcon::Close)
                                .size(IconSize::S16)
                                .color(look.icon),
                        ),
                )
            })
            .when(input_focused && !keyboard_item, |this| {
                // `.cds--search-input:focus { outline: 2px solid var(--focus);
                // outline-offset: -2px }`.
                this.child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .border_2()
                        .border_color(color(ColorRole::Focus)),
                )
            });

        // `.cu-command-list { max-height: 18rem }`.
        let mut list = div()
            .id((id.clone(), "list"))
            .flex()
            .flex_col()
            .max_h(rem(288.))
            .when(self.stable_height, |this| this.h(rem(288.)))
            .overflow_y_scroll()
            .pb(rem(8.));
        let mut flat_index = 0usize;
        if flat.is_empty() {
            // `.cu-command-empty { padding: spacing-06 spacing-05; text-align: center }`.
            list = list.child(
                div()
                    .py(rem(24.))
                    .px(rem(16.))
                    .flex()
                    .justify_center()
                    .type_style(TypeStyle::BodyCompact01)
                    .text_color(look.secondary)
                    .child(self.empty.clone()),
            );
        }
        for v in &shown {
            match v {
                Visible::Separator => {
                    list = list.child(div().my(rem(4.)).h(px(1.)).bg(look.separator));
                }
                Visible::Group(p, items) => {
                    let Part::Group(group) = &parts[*p] else {
                        continue;
                    };
                    if let Some(heading) = &group.heading {
                        list = list.child(
                            div()
                                .pt(rem(12.))
                                .px(rem(16.))
                                .pb(rem(4.))
                                .type_style(TypeStyle::Label01)
                                .text_color(look.secondary)
                                .child(heading.clone()),
                        );
                    }
                    for &i in items {
                        let item = group.items[i].clone();
                        let index = flat_index;
                        flat_index += 1;
                        let is_active = active == Some(index) && !item.disabled;
                        // The keyboard item (the reference's `:focus-visible` button)
                        // or the row under the pointer (`:hover`).
                        let focus_visible = is_active && keyboard && input_focused;
                        let highlighted = is_active && (focus_visible || !keyboard);
                        let row_id = ElementId::NamedChild(
                            std::sync::Arc::new(id.clone()),
                            SharedString::from(format!("item-{p}-{i}")),
                        );
                        let bg = motion::animate_color(
                            (row_id.clone(), "bg"),
                            if highlighted {
                                look.hover
                            } else {
                                Hsla::transparent_black()
                            },
                            Motion::HOVER,
                            window,
                            cx,
                        );
                        let text = if item.disabled {
                            look.disabled
                        } else {
                            look.text
                        };
                        let state_hover = state.clone();
                        let on_select = item.on_select.clone();
                        list = list.child(
                            div()
                                .id(row_id)
                                .relative()
                                .flex_shrink_0()
                                .h(SIZE_MD.rems())
                                .px(rem(16.))
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(rem(12.))
                                .bg(bg)
                                .text_color(text)
                                .type_style(TypeStyle::BodyCompact01)
                                .cursor(if item.disabled {
                                    CursorStyle::OperationNotAllowed
                                } else {
                                    CursorStyle::PointingHand
                                })
                                .when(!item.disabled, |this| {
                                    this.on_hover(move |hovered, _, cx| {
                                        state_hover.update(cx, |s, cx| {
                                            if *hovered {
                                                s.active = Some(index);
                                                s.keyboard = false;
                                            } else if s.active == Some(index) && !s.keyboard {
                                                // `:hover` ends with the pointer.
                                                s.active = None;
                                            }
                                            cx.notify();
                                        })
                                    })
                                    .on_click(
                                        move |_, window, cx| {
                                            if let Some(f) = &on_select {
                                                f(window, cx)
                                            }
                                        },
                                    )
                                })
                                .when_some(item.icon.clone(), |this, icon| {
                                    this.child(icon.size(IconSize::S16).color(if item.disabled {
                                        look.disabled
                                    } else {
                                        look.icon
                                    }))
                                })
                                .child(div().flex_1().min_w_0().child(item.label.clone()))
                                .when_some(item.shortcut.clone(), |this, shortcut| {
                                    // `.cu-command-shortcut`: mono 0.75rem, text-secondary.
                                    this.child(
                                        div()
                                            .font(typography::font(FontFamilyKey::Mono, 400))
                                            .text_size(rem(12.))
                                            .text_color(look.secondary)
                                            .child(shortcut),
                                    )
                                })
                                .when(focus_visible, |this| {
                                    // `.cu-command-item:focus-visible { box-shadow: inset 0 0 0 2px var(--focus) }`.
                                    this.child(
                                        div()
                                            .absolute()
                                            .top_0()
                                            .left_0()
                                            .size_full()
                                            .border_2()
                                            .border_color(color(ColorRole::Focus)),
                                    )
                                }),
                        );
                    }
                }
            }
        }

        div()
            .id(id.clone())
            .capture_key_down(on_key)
            .flex()
            .flex_col()
            .w_full()
            .max_w(rem(512.))
            .bg(look.background)
            .shadow(overlay::floating_shadow())
            .child(search)
            .child(list)
    }
}

/// `CommandDialog`: the palette in a passive Carbon Modal (`size="sm"`, no
/// heading).
///
/// Follows shadcn's `CommandDialog` semantics, which the design's API mirrors.
#[derive(IntoElement)]
pub struct CommandDialog {
    id: ElementId,
    open: bool,
    on_open_change: overlay::OpenChange,
    command: Command,
}

impl CommandDialog {
    /// A controlled command dialog.
    pub fn new(
        id: impl Into<ElementId>,
        open: bool,
        on_open_change: impl Fn(bool, &mut Window, &mut App) + 'static,
        command: Command,
    ) -> Self {
        Self {
            id: id.into(),
            open,
            on_open_change: Rc::new(on_open_change),
            command,
        }
    }
}

impl RenderOnce for CommandDialog {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let on_change = self.on_open_change.clone();
        // Focus opens in the search field, not on the modal's close button.
        // Keyed here (keyed state is scoped by the element path, which the
        // modal extends) and handed to the palette.
        let state = command_state(&self.command.id, window, cx);
        let field = state.read(cx).input.read(cx).focus_handle(cx);
        // Each opening starts from an empty query, as the reference's modal
        // mounts a fresh palette.
        let was_open = window.use_keyed_state((self.id.clone(), "was-open"), cx, |_, _| false);
        if self.open != *was_open.read(cx) {
            was_open.update(cx, |w, _| *w = self.open);
            if self.open {
                let input = state.read(cx).input.clone();
                input.update(cx, |s, cx| s.set_value("", window, cx));
                state.update(cx, |s, _| {
                    s.active = None;
                    s.keyboard = false;
                });
            }
        }
        let mut command = self.command;
        command.state = Some(state);
        Dialog::new(self.id)
            .initial_focus(&field)
            .open(self.open)
            .on_open_change(move |open, window, cx| on_change(open, window, cx))
            .passive(true)
            .size(DialogSize::Sm)
            .child(div().pt(rem(32.)).child(command.into_any_element()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parts() -> Vec<Part> {
        vec![
            Part::Group(
                CommandGroup::new("Suggestions")
                    .item(CommandItem::new("Open dashboard"))
                    .item(CommandItem::new("Create resource"))
                    .item(CommandItem::new("Search docs")),
            ),
            Part::Separator,
            Part::Group(
                CommandGroup::new("Settings")
                    .item(CommandItem::new("Profile"))
                    .item(CommandItem::new("Preferences").value("settings prefs")),
            ),
        ]
    }

    #[test]
    fn filtering_hides_empty_groups_and_separators() {
        let all = visible(&parts(), "");
        assert_eq!(
            all,
            vec![
                Visible::Group(0, vec![0, 1, 2]),
                Visible::Separator,
                Visible::Group(2, vec![0, 1]),
            ]
        );
        assert_eq!(visible(&parts(), "PRO"), vec![Visible::Group(2, vec![0])]);
        // The value replaces the label for matching.
        assert_eq!(visible(&parts(), "prefs"), vec![Visible::Group(2, vec![1])]);
        assert!(visible(&parts(), "zzz").is_empty());
    }

    #[test]
    fn keyboard_steps_wrap_and_skip_disabled() {
        let disabled = [false, true, false];
        // Enter with nothing highlighted runs the first enabled match.
        assert_eq!(step(None, 1, &disabled), Some(0));
        assert_eq!(step(None, 1, &[true, false]), Some(1));
        assert_eq!(step(Some(0), 1, &disabled), Some(2));
        assert_eq!(step(Some(2), 1, &disabled), Some(0));
        assert_eq!(step(None, -1, &disabled), Some(2));
        assert_eq!(step(None, 1, &[true]), None);
        assert_eq!(step(None, 1, &[]), None);
    }

    #[test]
    fn colors_follow_cu_command() {
        crate::theme::tests_set(crate::tokens::CarbonTheme::White);
        let look = CommandLook::current();
        assert_eq!(look.hover, color(ColorRole::LayerHover01));
        assert_eq!(look.icon, color(ColorRole::IconSecondary));
    }
}
