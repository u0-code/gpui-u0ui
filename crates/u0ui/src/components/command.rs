//! Command: a command palette, a Carbon `Search` (lg) over a filterable list
//! whose items look like Carbon menu items (`.cu-command*` in `custom.css`).
//! [`CommandDialog`] puts it in a passive Modal.
//!
//! - **Behavior**: text editing is Kit's `InputState`/`Input`. Typing filters
//!   items by their value (or label), case-insensitively; groups without a
//!   match disappear, separators hide while a query is typed, and the empty
//!   text shows when nothing matches. ↑/↓ move the highlighted item (wrapping,
//!   skipping disabled ones), Enter runs it, Escape clears the query. Pointer
//!   hover highlights, click runs.
//! - **Appearance**: `layer-01` box with the floating shadow, max 32rem; the
//!   search field is 3rem tall with a 16px magnifier, `border-subtle-01`
//!   bottom line and a clear button; the list scrolls past 18rem; headings
//!   are label-01 `text-secondary` (0.75rem 1rem 0.25rem); items are
//!   `size-md` (2.5rem) rows with a 16px `icon-secondary` icon, label, and a
//!   mono 12px shortcut; highlighted rows take `layer-hover-01` and a 2px inset
//!   `focus` outline when keyboard-driven.
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
        }
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
        let state = window.use_keyed_state((id.clone(), "command"), cx, |window, cx| {
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
        });
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

        let on_key = {
            let state = state.clone();
            let flat = flat.clone();
            let item_at = item_at.clone();
            let input = input.clone();
            move |event: &KeyDownEvent, window: &mut Window, cx: &mut App| {
                let n = flat.len();
                let step = |from: Option<usize>, dir: isize| -> Option<usize> {
                    let mut i = from.map_or(if dir > 0 { -1 } else { n as isize }, |i| i as isize);
                    for _ in 0..n {
                        i = (i + dir).rem_euclid(n as isize);
                        if !item_at(flat[i as usize]).disabled {
                            return Some(i as usize);
                        }
                    }
                    None
                };
                let current = state.read(cx).active;
                match event.keystroke.key.as_str() {
                    "down" | "up" if n > 0 => {
                        let next =
                            step(current, if event.keystroke.key == "down" { 1 } else { -1 });
                        state.update(cx, |s, cx| {
                            s.active = next;
                            s.keyboard = true;
                            cx.notify();
                        });
                        cx.stop_propagation();
                    }
                    "enter" => {
                        if let Some(i) = current
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
            .when(input_focused, |this| {
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

        let mut list = div()
            .id((id.clone(), "list"))
            .flex()
            .flex_col()
            .max_h(rem(288.))
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
                        let highlighted = active == Some(index) && !item.disabled;
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
                                        if *hovered {
                                            state_hover.update(cx, |s, cx| {
                                                s.active = Some(index);
                                                s.keyboard = false;
                                                cx.notify();
                                            })
                                        }
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
                                .when(highlighted && keyboard && input_focused, |this| {
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
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let on_change = self.on_open_change.clone();
        Dialog::new(self.id)
            .open(self.open)
            .on_open_change(move |open, window, cx| on_change(open, window, cx))
            .passive(true)
            .size(DialogSize::Sm)
            .child(div().pt(rem(32.)).child(self.command.into_any_element()))
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
    fn colors_follow_cu_command() {
        crate::theme::tests_set(crate::tokens::CarbonTheme::White);
        let look = CommandLook::current();
        assert_eq!(look.hover, color(ColorRole::LayerHover01));
        assert_eq!(look.icon, color(ColorRole::IconSecondary));
    }
}
