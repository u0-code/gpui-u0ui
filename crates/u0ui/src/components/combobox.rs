//! Combobox: shadcn `Combobox`, a filterable dropdown. Single → Carbon
//! `ComboBox` (`.cds--combo-box`); `multiple` → Carbon `FilterableMultiSelect`
//! (`.cds--multi-select--filterable`), whose selection shows as a count tag.
//!
//! - **Behavior**: the filter text is Kit's [`InputState`] (editing, IME,
//!   selection, clipboard); the menu is placed by Kit's
//!   [`gpui_kit::base::Popup`]. Kit has no combobox that can be restyled to
//!   Carbon's list box, so the combobox keyboard is written here, captured
//!   ahead of the editor: ↓/↑ open the menu and move the highlight, Enter
//!   picks it, Escape closes the menu (and, when closed, clears a single
//!   value). Typing opens the menu. A single combobox keeps every item
//!   (Carbon `ComboBox`'s default `shouldFilterItem` returns `true`; the
//!   reference passes none) and highlights the first whose label contains
//!   the text ([`highlight_match`], Carbon's `findHighlightedIndex`).
//!   Leaving it restores the text of its value (no custom values, Carbon's
//!   default). A multiple combobox filters (case-insensitive substring,
//!   [`filter`]) and lists the items selected when the menu opened first,
//!   each part in label order ([`sort_shown`], `FilterableMultiSelect`'s
//!   `defaultSortItems`); it keeps the menu open while picking, with a
//!   checkbox per row.
//! - **Appearance**: the Carbon list-box field ([`super::select::TriggerLook`])
//!   with the text input inside; a 24px clear button (`.cds--list-box__selection`)
//!   45px from the end, split from the chevron by a 1×16px `border-subtle`
//!   rule; focus/invalid outlines; the multi-select count tag is a 24px pill on
//!   `background-inverse` with `text-inverse` and a close icon.
//!
//! ```ignore
//! Combobox::new("framework", &filter).label("Framework")
//!     .items(["React", "Angular", "Vue"]).default_value(["React"])
//! ```
use super::form::{self, Validation, rem};
use super::input::{FIELD_MOTION, InputState};
use super::list_box::{self, ListItem, ListState, MENU_MOTION, Menu};
use super::select::{TriggerLook, TriggerState};
use crate::{
    density::{self, Density},
    icons::{CarbonIcon, Icon, IconSize},
    motion,
    theme::{LayerLevel, color},
    tokens::{ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    App, CursorStyle, ElementId, Entity, Focusable as _, Hsla, InteractiveElement as _,
    IntoElement, MouseButton, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement as _, Styled, Window,
    component::{
        Sizable as _, Size,
        input::{Enter, Escape, Input as KitInput, MoveDown, MoveUp},
    },
    div,
    prelude::FluentBuilder as _,
    px,
};
use std::rc::Rc;

use super::form::FieldSize;

/// The indices of `items` whose label contains `query` (case-insensitive);
/// all items when `query` is empty or is exactly the current single value's
/// label (reopening a filled combobox lists everything).
pub fn filter(items: &[ListItem], query: &str, current_label: Option<&str>) -> Vec<usize> {
    let query = query.trim();
    if query.is_empty() || current_label == Some(query) {
        return (0..items.len()).collect();
    }
    let needle = query.to_lowercase();
    items
        .iter()
        .enumerate()
        .filter(|(_, i)| i.label.to_lowercase().contains(&needle))
        .map(|(k, _)| k)
        .collect()
}

/// The first item whose label contains `query`, case-insensitively, or
/// `None` for empty text (Carbon `ComboBox` `findHighlightedIndex`).
pub fn highlight_match(items: &[ListItem], query: &str) -> Option<usize> {
    if query.is_empty() {
        return None;
    }
    let needle = query.to_lowercase();
    items
        .iter()
        .position(|i| i.label.to_lowercase().contains(&needle))
}

/// Order a multi-select's shown rows like `FilterableMultiSelect`'s
/// `defaultSortItems`: the items in `top` (the selection when the menu
/// opened) first, then the rest, each part by label
/// (`localeCompare(…, { numeric: true })`).
pub fn sort_shown(items: &[ListItem], shown: &mut [usize], top: &[SharedString]) {
    shown.sort_by(|&a, &b| {
        let (ia, ib) = (&items[a], &items[b]);
        let (ta, tb) = (top.contains(&ia.value), top.contains(&ib.value));
        tb.cmp(&ta).then_with(|| natural_cmp(&ia.label, &ib.label))
    });
}

/// A case-insensitive comparison that orders digit runs by number
/// (`"Item 2" < "Item 10"`), like `localeCompare` with `numeric: true`.
pub fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let (mut a, mut b) = (a.chars().peekable(), b.chars().peekable());
    loop {
        match (a.peek().copied(), b.peek().copied()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(x), Some(y)) if x.is_ascii_digit() && y.is_ascii_digit() => {
                let take = |it: &mut std::iter::Peekable<std::str::Chars>| {
                    let mut n = String::new();
                    while let Some(c) = it.peek().copied().filter(char::is_ascii_digit) {
                        n.push(c);
                        it.next();
                    }
                    n.trim_start_matches('0').to_string()
                };
                let (na, nb) = (take(&mut a), take(&mut b));
                let ord = na.len().cmp(&nb.len()).then_with(|| na.cmp(&nb));
                if ord != Ordering::Equal {
                    return ord;
                }
            }
            (Some(x), Some(y)) => {
                let ord = x.to_lowercase().cmp(y.to_lowercase());
                if ord != Ordering::Equal {
                    return ord;
                }
                a.next();
                b.next();
            }
        }
    }
}

/// The next value list after picking `value`: single replaces, multiple toggles.
pub fn toggled(values: &[SharedString], value: &SharedString, multiple: bool) -> Vec<SharedString> {
    if !multiple {
        return vec![value.clone()];
    }
    if values.contains(value) {
        values.iter().filter(|v| *v != value).cloned().collect()
    } else {
        let mut next = values.to_vec();
        next.push(value.clone());
        next
    }
}

type ChangeHandler = Rc<dyn Fn(&[SharedString], &mut Window, &mut App)>;
type CommitFn = Rc<dyn Fn(Vec<SharedString>, &mut Window, &mut App)>;

#[derive(Default)]
struct Typing {
    /// The text last seen, to notice typing.
    last: Option<SharedString>,
    /// Whether the menu was open last frame, to notice it opening.
    was_open: bool,
    /// The multi-select's selection when the menu opened (listed first).
    top: Vec<SharedString>,
}

/// A Carbon combo box. See the module docs.
#[derive(IntoElement)]
pub struct Combobox {
    id: ElementId,
    state: Entity<InputState>,
    label: Option<SharedString>,
    items: Vec<ListItem>,
    multiple: bool,
    value: Option<Vec<SharedString>>,
    default_value: Vec<SharedString>,
    placeholder: Option<SharedString>,
    helper_text: Option<SharedString>,
    invalid: bool,
    invalid_text: Option<SharedString>,
    warn: bool,
    warn_text: Option<SharedString>,
    size: FieldSize,
    density: Option<Density>,
    disabled: bool,
    clear_label: Option<SharedString>,
    level: LayerLevel,
    on_value_change: Option<ChangeHandler>,
}

impl Combobox {
    /// A combobox with a stable `id` whose filter text is `state`.
    pub fn new(id: impl Into<ElementId>, state: &Entity<InputState>) -> Self {
        Self {
            id: id.into(),
            state: state.clone(),
            label: None,
            items: Vec::new(),
            multiple: false,
            value: None,
            default_value: Vec::new(),
            placeholder: None,
            helper_text: None,
            invalid: false,
            invalid_text: None,
            warn: false,
            warn_text: None,
            size: FieldSize::Md,
            density: None,
            disabled: false,
            clear_label: None,
            level: LayerLevel::One,
            on_value_change: None,
        }
    }

    /// `label` (Carbon `titleText`).
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// `items`: strings or `(value, label)` pairs.
    pub fn items<I: Into<ListItem>>(mut self, items: impl IntoIterator<Item = I>) -> Self {
        self.items = items.into_iter().map(Into::into).collect();
        self
    }

    /// `multiple` → `FilterableMultiSelect`.
    pub fn multiple(mut self, multiple: bool) -> Self {
        self.multiple = multiple;
        self
    }

    /// Controlled `value` (one entry when single).
    pub fn value<S: Into<SharedString>>(mut self, value: impl IntoIterator<Item = S>) -> Self {
        self.value = Some(value.into_iter().map(Into::into).collect());
        self
    }

    /// `defaultValue`.
    pub fn default_value<S: Into<SharedString>>(
        mut self,
        value: impl IntoIterator<Item = S>,
    ) -> Self {
        self.default_value = value.into_iter().map(Into::into).collect();
        self
    }

    /// `placeholder` (drawn in `text-placeholder`).
    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    /// `helperText`.
    pub fn helper_text(mut self, text: impl Into<SharedString>) -> Self {
        self.helper_text = Some(text.into());
        self
    }

    /// `invalid`.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// `invalidText`.
    pub fn invalid_text(mut self, text: impl Into<SharedString>) -> Self {
        self.invalid_text = Some(text.into());
        self
    }

    /// `warn`.
    pub fn warn(mut self, warn: bool) -> Self {
        self.warn = warn;
        self
    }

    /// `warnText`.
    pub fn warn_text(mut self, text: impl Into<SharedString>) -> Self {
        self.warn_text = Some(text.into());
        self
    }

    /// `size`.
    pub fn size(mut self, size: FieldSize) -> Self {
        self.size = size;
        self
    }

    /// Override the app density.
    pub fn density(mut self, density: Density) -> Self {
        self.density = Some(density);
        self
    }

    /// Disabled.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Accessible name of the clear button (`translateWithId('clear.all')`).
    pub fn clear_label(mut self, label: impl Into<SharedString>) -> Self {
        self.clear_label = Some(label.into());
        self
    }

    /// The layer the field sits on.
    pub fn layer(mut self, level: LayerLevel) -> Self {
        self.level = level;
        self
    }

    /// `onValueChange` with the selected values.
    pub fn on_value_change(
        mut self,
        handler: impl Fn(&[SharedString], &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_value_change = Some(Rc::new(handler));
        self
    }
}

fn set_open(state: &Entity<ListState>, open: bool, highlight: Option<usize>, cx: &mut App) {
    state.update(cx, |s, cx| {
        if s.open != open || (open && s.highlighted.is_none()) {
            s.open = open;
            if open {
                s.highlighted = highlight;
                s.hovered = None;
            }
            cx.notify();
        }
    });
}

impl RenderOnce for Combobox {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let multiple = self.multiple;
        let density = self.density.unwrap_or_else(density::current);
        let height = self.size.height(density);
        let row = list_box::row_height(height, density);
        let input = self.state.clone();
        let (values, store) = form::controlled(
            (id.clone(), "value"),
            self.value.clone(),
            self.default_value.clone(),
            window,
            cx,
        );
        let items = Rc::new(self.items.clone());
        let list = window.use_keyed_state((id.clone(), "list"), cx, |_, _| ListState::default());
        let typing = window.use_keyed_state((id.clone(), "typing"), cx, |_, _| Typing::default());
        let hovered = window.use_keyed_state((id.clone(), "hover"), cx, |_, _| false);
        let focus = input.read(cx).focus_handle(cx);
        let focused = focus.is_focused(window);
        // Focus that leaves the field (a global shortcut opening a dialog,
        // Tab) closes the menu, as Downshift's blur does, so it never floats
        // over whatever took focus.
        let was_focused = window.use_keyed_state((id.clone(), "was-focused"), cx, |_, _| false);
        if *was_focused.read(cx) != focused {
            was_focused.update(cx, |w, _| *w = focused);
            if !focused && list.read(cx).open {
                set_open(&list, false, None, cx);
            }
        }
        let query = input.read(cx).value();
        let current = (!multiple)
            .then(|| values.first())
            .flatten()
            .and_then(|v| items.iter().find(|i| &i.value == v))
            .map(|i| i.label.clone());

        // A single combobox shows its value's label: seed it once, and restore it
        // when focus leaves with the menu closed (no custom values).
        let open_now = list.read(cx).open && !self.disabled;
        if !multiple && !focused && !open_now {
            let shown = current.clone().unwrap_or_default();
            if query != shown && (typing.read(cx).last.is_some() || query.is_empty()) {
                input.update(cx, |s, cx| s.set_value(shown.clone(), window, cx));
                typing.update(cx, |t, _| t.last = Some(shown));
            }
        }
        let query = input.read(cx).value();
        // Snapshot the multi-select's selection as the menu opens.
        let open_before = list.read(cx).open && !self.disabled;
        if open_before != typing.read(cx).was_open {
            typing.update(cx, |t, _| {
                t.was_open = open_before;
                if open_before {
                    t.top = values.clone();
                }
            });
        }
        let mut shown = if multiple {
            filter(&items, &query, None)
        } else {
            (0..items.len()).collect()
        };
        if multiple {
            sort_shown(&items, &mut shown, &typing.read(cx).top);
        }

        // Typing opens the menu: a single combobox highlights the first match,
        // a multi-select its first enabled filtered row.
        let changed = typing.read(cx).last.as_ref() != Some(&query);
        if changed {
            typing.update(cx, |t, _| t.last = Some(query.clone()));
            if focused && !self.disabled {
                let first = if multiple {
                    shown.iter().copied().find(|&i| !items[i].disabled)
                } else {
                    highlight_match(&items, query.trim())
                };
                list.update(cx, |s, _| {
                    if !s.open {
                        s.open = true;
                    }
                    s.highlighted = first;
                });
                if multiple && !typing.read(cx).was_open {
                    typing.update(cx, |t, _| {
                        t.was_open = true;
                        t.top = values.clone();
                    });
                    sort_shown(&items, &mut shown, &values);
                }
            }
        }
        let (open, highlighted, hover_row, anchor) = {
            let s = list.read(cx);
            (
                s.open && !self.disabled,
                s.highlighted,
                s.hovered,
                s.anchor.clone(),
            )
        };
        let progress = motion::animate(
            (id.clone(), "menu"),
            if open { 1f32 } else { 0. },
            MENU_MOTION,
            window,
            cx,
        );
        let validation = if self.disabled {
            Validation::None
        } else {
            Validation::from_flags(
                self.invalid,
                self.invalid_text.clone(),
                self.warn,
                self.warn_text.clone(),
            )
        };
        let look = TriggerLook::of(
            self.level,
            TriggerState {
                hovered: *hovered.read(cx),
                open,
                disabled: self.disabled,
                readonly: false,
                inline: false,
            },
        );
        // `.cds--combo-box:hover { background-color: var(--field) }`: the combo box
        // field does not take `field-hover`; the multi-select does.
        let background_target = if multiple {
            look.background
        } else {
            color(self.level.field())
        };
        let background = motion::animate_color(
            (id.clone(), "bg"),
            background_target,
            FIELD_MOTION,
            window,
            cx,
        );
        let outline = form::field_outline(focused && !self.disabled, &validation)
            .unwrap_or(Hsla::transparent_black());

        let on_change = self.on_value_change.clone();
        let commit: CommitFn = Rc::new({
            let store = store.clone();
            move |next, window, cx| {
                store.update(cx, |v, cx| {
                    *v = next.clone();
                    cx.notify();
                });
                if let Some(handler) = on_change.as_ref() {
                    handler(&next, window, cx);
                }
            }
        });
        let pick: list_box::PickFn = Rc::new({
            let items = items.clone();
            let values = values.clone();
            let commit = commit.clone();
            let list = list.clone();
            let input = input.clone();
            let typing = typing.clone();
            let focus = focus.clone();
            move |i, window, cx| {
                let item = &items[i];
                commit(toggled(&values, &item.value, multiple), window, cx);
                if !multiple {
                    let label = item.label.clone();
                    typing.update(cx, |t, _| t.last = Some(label.clone()));
                    input.update(cx, |s, cx| s.set_value(label, window, cx));
                    set_open(&list, false, None, cx);
                }
                focus.focus(window, cx);
            }
        });
        let clear: super::overlay::Callback = Rc::new({
            let commit = commit.clone();
            let input = input.clone();
            let typing = typing.clone();
            let focus = focus.clone();
            move |window, cx| {
                commit(Vec::new(), window, cx);
                if !multiple {
                    typing.update(cx, |t, _| t.last = Some(SharedString::default()));
                    input.update(cx, |s, cx| s.set_value("", window, cx));
                }
                focus.focus(window, cx);
            }
        });

        let selected_index =
            |values: &[SharedString]| items.iter().position(|i| values.first() == Some(&i.value));
        let open_highlight = shown
            .iter()
            .copied()
            .find(|&i| Some(i) == selected_index(&values))
            .or_else(|| shown.iter().copied().find(|&i| !items[i].disabled));
        let disabled_rows: Vec<bool> = shown.iter().map(|&i| items[i].disabled).collect();
        let shown_rc = Rc::new(shown.clone());

        let has_selection = !values.is_empty() && (multiple || !query.is_empty());
        // `.cds--list-box__field .cds--text-input { padding-inline-end: 4.375rem }`,
        // `--empty { 3rem }`; a status icon adds 1rem.
        let mut pad_end = if has_selection && !multiple { 70. } else { 48. };
        if validation.has_icon() {
            pad_end += 16.;
        }
        let placeholder = self
            .placeholder
            .clone()
            .filter(|_| query.is_empty())
            .map(|text| {
                div()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .left_0()
                    .right_0()
                    .flex()
                    .items_center()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_color(color(if self.disabled {
                        ColorRole::TextDisabled
                    } else {
                        ColorRole::TextPlaceholder
                    }))
                    .child(text)
            });
        let text_color = color(if self.disabled {
            ColorRole::TextDisabled
        } else {
            ColorRole::TextPrimary
        });

        // `.cds--list-box__selection--multi`: the count tag of a multi-select.
        let tag = (multiple && !values.is_empty()).then(|| {
            let clear = clear.clone();
            div()
                .id((id.clone(), "tag"))
                .flex()
                .flex_row()
                .flex_shrink_0()
                .items_center()
                .h(rem(24.))
                .pl(rem(8.))
                .pr(rem(2.))
                .ml(rem(16.))
                .mr(rem(10.))
                .rounded(rem(12.))
                .bg(color(if self.disabled {
                    ColorRole::TextDisabled
                } else {
                    ColorRole::BackgroundInverse
                }))
                .type_style(TypeStyle::Label01)
                .text_color(color(ColorRole::TextInverse))
                .child(SharedString::from(values.len().to_string()))
                .child(
                    div()
                        .id((id.clone(), "tag-close"))
                        .ml(rem(4.))
                        .size(rem(20.))
                        .rounded_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .when(!self.disabled, |this| {
                            this.cursor(CursorStyle::PointingHand)
                                .hover(|s| s.bg(color(ColorRole::BackgroundInverseHover)))
                                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                                .on_click(move |_, window, cx| clear(window, cx))
                        })
                        .child(
                            Icon::carbon(CarbonIcon::Close)
                                .size(IconSize::S16)
                                .color(color(ColorRole::IconInverse)),
                        ),
                )
        });

        let editor = div()
            .relative()
            .flex_1()
            .min_w_0()
            .h_full()
            .flex()
            .items_center()
            .child(
                KitInput::new(&input)
                    .appearance(false)
                    .with_size(Size::XSmall)
                    .disabled(self.disabled)
                    .when_some(self.label.clone(), |this, label| this.aria_label(label))
                    .p_0()
                    .h_full()
                    .flex_1()
                    .min_w_0()
                    .text_size(rem(14.))
                    .line_height(rem(18.))
                    .text_color(text_color),
            )
            .children(placeholder);

        let clear_button = (has_selection && !multiple && !self.disabled).then(|| {
            let clear = clear.clone();
            div()
                .id((id.clone(), "clear"))
                .absolute()
                .top_0()
                .bottom_0()
                .right(rem(45. - 12.))
                .flex()
                .items_center()
                .child(
                    div()
                        .id((id.clone(), "clear-button"))
                        .size(rem(24.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor(CursorStyle::PointingHand)
                        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                        .on_click(move |_, window, cx| clear(window, cx))
                        .when_some(self.clear_label.clone(), |this, label| {
                            this.tooltip(move |window, cx| {
                                gpui_kit::component::tooltip::Tooltip::new(label.clone())
                                    .build(window, cx)
                            })
                        })
                        .child(
                            Icon::carbon(CarbonIcon::Close)
                                .size(IconSize::S16)
                                .color(color(ColorRole::IconPrimary)),
                        ),
                )
                // `.cds--list-box__selection::after`: 1×16px `border-subtle-01`, 33px on.
                .child(
                    div()
                        .ml(rem(9.))
                        .w(px(1.))
                        .h(rem(16.))
                        .bg(color(ColorRole::BorderSubtle01)),
                )
        });
        let status = form::status_icon(&validation).map(|icon| {
            div()
                .absolute()
                .top_0()
                .bottom_0()
                .right(rem(if has_selection && !multiple { 82. } else { 40. }))
                .flex()
                .items_center()
                .child(icon)
        });
        let chevron = {
            let list = list.clone();
            let focus = focus.clone();
            div()
                .id((id.clone(), "chevron"))
                .absolute()
                .top_0()
                .bottom_0()
                .right(rem(12.))
                .w(rem(24.))
                .flex()
                .items_center()
                .justify_center()
                .when(!self.disabled, |this| {
                    this.cursor(CursorStyle::PointingHand)
                        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                        .on_click(move |_, window, cx| {
                            focus.focus(window, cx);
                            set_open(&list, !open, open_highlight, cx);
                        })
                })
                .child(list_box::chevron(
                    id.clone(),
                    open,
                    self.disabled,
                    window,
                    cx,
                ))
        };

        let field = div()
            .id((id.clone(), "field"))
            .relative()
            .flex()
            .flex_row()
            .items_center()
            .w_full()
            .h(rem(height))
            .when(!(multiple && tag.is_some()), |this| this.pl(rem(16.)))
            .pr(rem(pad_end))
            .bg(background)
            .pb(gpui_kit::px(form::FIELD_RULE))
            .type_style(TypeStyle::BodyCompact01)
            .text_color(text_color)
            .cursor(if self.disabled {
                CursorStyle::OperationNotAllowed
            } else {
                CursorStyle::IBeam
            })
            .on_hover({
                let hovered = hovered.clone();
                move |over, _, cx| {
                    hovered.update(cx, |h, cx| {
                        if *h != *over {
                            *h = *over;
                            cx.notify();
                        }
                    })
                }
            })
            .when(!self.disabled, |this| {
                let list = list.clone();
                let focus = focus.clone();
                this.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    focus.focus(window, cx);
                    // Downshift opens on input click.
                    set_open(&list, true, open_highlight, cx);
                })
            })
            .child(anchor.measure_trigger())
            .children(tag)
            .child(editor)
            .children(status)
            .children(clear_button)
            .child(chevron)
            .child(form::bottom_rule(look.border))
            .child(form::inset_outline(2., outline));

        let (side, room) = anchor.place(list_box::menu_height(&items, &shown, row), window);
        let popup = gpui_kit::base::Popup::new((id.clone(), "popup"), field)
            .anchor(side.anchor())
            .w_full()
            .when(open || progress > 0., |this| {
                let selected: Vec<bool> = items.iter().map(|i| values.contains(&i.value)).collect();
                this.content(list_box::menu(
                    Menu {
                        id: id.clone(),
                        items: &items,
                        shown: shown.clone(),
                        selected: &selected,
                        highlighted,
                        hovered: hover_row,
                        row_height: row,
                        width: anchor.width(),
                        max_height: room,
                        level: self.level,
                        checkboxes: multiple,
                        empty: None,
                        on_pick: pick.clone(),
                        on_hover: Rc::new({
                            let list = list.clone();
                            move |row, _, cx| {
                                list.update(cx, |s, cx| {
                                    if row.is_some() || s.hovered.is_some() {
                                        s.hovered = row;
                                        if row.is_some() {
                                            s.highlighted = row;
                                        }
                                        cx.notify();
                                    }
                                })
                            }
                        }),
                        on_close: Rc::new({
                            let list = list.clone();
                            let focus = focus.clone();
                            move |window, cx| {
                                // A press inside the field keeps it open (handled there).
                                if !focus.is_focused(window) {
                                    set_open(&list, false, None, cx);
                                }
                            }
                        }),
                    },
                    progress,
                ))
            });

        // Combobox keys, captured ahead of the editor's own bindings.
        let step = {
            let list = list.clone();
            let shown = shown_rc.clone();
            let disabled_rows = disabled_rows.clone();
            move |delta: isize, cx: &mut App| {
                list.update(cx, |s, cx| {
                    if !s.open {
                        s.open = true;
                        s.highlighted = open_highlight;
                    } else {
                        let at = s
                            .highlighted
                            .and_then(|h| shown.iter().position(|&i| i == h));
                        s.highlighted =
                            list_box::move_highlight(at, delta, &disabled_rows).map(|k| shown[k]);
                    }
                    cx.notify();
                })
            }
        };
        let step_down = step.clone();
        let root = div()
            .id((id.clone(), "root"))
            .relative()
            .w_full()
            .min_w_0()
            .when(!self.disabled, |this| {
                this.capture_action(move |_: &MoveDown, _, cx| {
                    step_down(1, cx);
                    cx.stop_propagation();
                })
                .capture_action(move |_: &MoveUp, _, cx| {
                    step(-1, cx);
                    cx.stop_propagation();
                })
                .capture_action({
                    let list = list.clone();
                    let pick = pick.clone();
                    move |action: &Enter, window, cx| {
                        let (is_open, h) = {
                            let s = list.read(cx);
                            (s.open, s.highlighted)
                        };
                        if is_open && !action.secondary {
                            if let Some(i) = h {
                                pick(i, window, cx);
                            }
                            cx.stop_propagation();
                        }
                    }
                })
                .capture_action({
                    let list = list.clone();
                    let clear = clear.clone();
                    move |_: &Escape, window, cx| {
                        if list.read(cx).open {
                            set_open(&list, false, None, cx);
                            cx.stop_propagation();
                        } else if !multiple {
                            clear(window, cx);
                            cx.stop_propagation();
                        }
                    }
                })
            })
            .child(popup);

        let label = self.label.clone().filter(|l| !l.is_empty());
        form::labeled(
            label.as_ref(),
            self.disabled,
            root,
            form::message(&validation, self.helper_text.as_ref(), self.disabled),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn items() -> Vec<ListItem> {
        ["React", "Angular", "Vue", "Svelte", "Web Components"]
            .into_iter()
            .map(ListItem::from)
            .collect()
    }

    #[test]
    fn filtering_is_a_case_insensitive_substring() {
        let items = items();
        assert_eq!(filter(&items, "", None), vec![0, 1, 2, 3, 4]);
        assert_eq!(filter(&items, "ve", None), vec![3]);
        assert_eq!(filter(&items, "E", None), vec![0, 2, 3, 4]);
        // Reopening a filled combobox lists everything.
        assert_eq!(filter(&items, "React", Some("React")).len(), 5);
    }

    #[test]
    fn single_highlights_the_first_containing_item() {
        let items = items();
        assert_eq!(highlight_match(&items, ""), None);
        assert_eq!(highlight_match(&items, "ve"), Some(3));
        assert_eq!(highlight_match(&items, "E"), Some(0));
        assert_eq!(highlight_match(&items, "zz"), None);
    }

    #[test]
    fn multi_lists_the_opening_selection_first_then_by_label() {
        let items = items();
        let mut shown: Vec<usize> = (0..items.len()).collect();
        sort_shown(&items, &mut shown, &["Vue".into(), "Angular".into()]);
        // Angular, Vue (selected), then React, Svelte, Web Components.
        assert_eq!(shown, vec![1, 2, 0, 3, 4]);
        assert!(natural_cmp("Item 2", "Item 10").is_lt());
        assert!(natural_cmp("apple", "Banana").is_lt());
    }

    #[test]
    fn picking_replaces_or_toggles() {
        let values: Vec<SharedString> = vec!["prod".into(), "eu".into()];
        assert_eq!(
            toggled(&values, &"us".into(), false),
            vec![SharedString::from("us")]
        );
        assert_eq!(
            toggled(&values, &"eu".into(), true),
            vec![SharedString::from("prod")]
        );
        assert_eq!(toggled(&values, &"us".into(), true).len(), 3);
    }
}
