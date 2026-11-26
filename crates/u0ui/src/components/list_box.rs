//! Carbon `ListBox` (`.cds--list-box*`): the field frame and the floating
//! option menu shared by [`super::Select`] (Dropdown), [`super::Combobox`]
//! (ComboBox / FilterableMultiSelect) and [`super::NativeSelect`].
//!
//! - **Menu**: `layer` fill with the floating shadow, rows as tall as the field
//!   (`density-row-md` in compact), at most 5.5 rows before it scrolls. An
//!   option sits 16px in with a 1px `border-subtle` rule on top (hidden on the
//!   first row and around hovered/selected/highlighted rows), `text-secondary`;
//!   hover `layer-hover` + `text-primary`; selected `layer-selected` +
//!   `text-primary` + a checkmark 16px from the end; the keyboard-highlighted
//!   row has a 2px inset `focus` outline; disabled rows `text-disabled`.
//! - **Motion**: the menu opens and closes by height over 110ms productive
//!   (`transition: max-height 110ms cubic-bezier(0.2, 0, 0.38, 0.9)`); spatial,
//!   so reduced motion cuts it. The chevron turns 180° over 70ms.
//! - **Placement** is Kit's [`gpui_kit::base::Popup`] (deferred top layer,
//!   window-edge clamping), anchored to the field's bottom-left.
use super::checkbox::{CheckboxLook, CheckboxLookState, CheckboxState, checkbox_box};
use super::form::rem;
use super::overlay;
use crate::{
    density::Density,
    icons::{CarbonIcon, Icon, IconSize},
    motion::{self, Curve, Motion, Speed},
    theme::{LayerLevel, color},
    tokens::{self, ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    AnyElement, App, ElementId, Hsla, InteractiveElement as _, IntoElement, MouseButton,
    ParentElement, SharedString, StatefulInteractiveElement as _, Styled, Window, canvas, div,
    prelude::FluentBuilder as _, px,
};
use std::{cell::Cell, rc::Rc, time::Instant};

/// `.cds--list-box__menu { transition: max-height 110ms cubic-bezier(0.2, 0, 0.38, 0.9) }`.
pub const MENU_MOTION: Motion = Motion::new(Curve::ProductiveStandard, Speed::Fast02).spatial(true);
/// `.cds--list-box__menu-icon { transition: transform 70ms cubic-bezier(0.2, 0, 0.38, 0.9) }`.
pub const CHEVRON_MOTION: Motion =
    Motion::new(Curve::ProductiveStandard, Speed::Fast01).spatial(true);
/// `max-block-size: calc(var(--layout-size-height-local) * 5.5)`.
pub const MAX_VISIBLE_ROWS: f32 = 5.5;

/// One option (`Opt` in `index.d.ts`: a string or `{ value, label, disabled }`).
#[derive(Clone, Debug, PartialEq)]
pub struct ListItem {
    /// The value.
    pub value: SharedString,
    /// The shown label.
    pub label: SharedString,
    /// Disabled.
    pub disabled: bool,
    /// The group heading shown above this item when it starts a group
    /// (`<optgroup label>`), for the native select.
    pub group: Option<SharedString>,
}

impl ListItem {
    /// An item whose label is its value.
    pub fn new(value: impl Into<SharedString>) -> Self {
        let value = value.into();
        Self {
            label: value.clone(),
            value,
            disabled: false,
            group: None,
        }
    }

    /// An item with a separate label.
    pub fn labeled(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
            group: None,
        }
    }

    /// Disabled.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl From<&str> for ListItem {
    fn from(value: &str) -> Self {
        Self::new(SharedString::from(value.to_string()))
    }
}

impl From<(&str, &str)> for ListItem {
    fn from((value, label): (&str, &str)) -> Self {
        Self::labeled(value.to_string(), label.to_string())
    }
}

/// Row height for a field `height` under `density`
/// (`[data-density="compact"] .cds--list-box__menu-item { block-size: var(--density-row-md) }`).
pub fn row_height(field_height: f32, density: Density) -> f32 {
    match density {
        Density::Compact => tokens::DENSITY_ROW_MD.px_value(),
        Density::Comfortable => field_height,
    }
}

/// The state an option look is computed for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ItemState {
    /// The current value (`--active`).
    pub selected: bool,
    /// Keyboard/pointer highlight (`--highlighted`).
    pub highlighted: bool,
    /// Pointer over it.
    pub hovered: bool,
    /// Disabled.
    pub disabled: bool,
}

/// The colors of one option row.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ItemLook {
    /// Row fill.
    pub background: Hsla,
    /// Label color.
    pub text: Hsla,
    /// The 2px highlight outline.
    pub outline: Option<Hsla>,
}

impl ItemLook {
    /// The look of an option (`.cds--list-box__menu-item*`).
    pub fn of(level: LayerLevel, state: ItemState) -> Self {
        if state.disabled {
            return Self {
                background: Hsla::transparent_black(),
                text: color(ColorRole::TextDisabled),
                outline: None,
            };
        }
        let background = match (state.selected, state.hovered) {
            // `--active:hover { background-color: var(--layer-selected-hover) }`.
            (true, true) => color(level.layer_selected_hover()),
            (true, false) => color(level.layer_selected()),
            (false, true) => color(level.layer_hover()),
            (false, false) => Hsla::transparent_black(),
        };
        let text = if state.selected || state.hovered || state.highlighted {
            color(ColorRole::TextPrimary)
        } else {
            color(ColorRole::TextSecondary)
        };
        Self {
            background,
            text,
            outline: state.highlighted.then(|| color(ColorRole::Focus)),
        }
    }
}

/// Whether row `i`'s top rule shows: not on the first row, nor on or right
/// after a hovered, selected or highlighted row.
pub fn rule_visible(i: usize, states: &[ItemState]) -> bool {
    if i == 0 {
        return false;
    }
    let busy = |s: &ItemState| s.selected || s.hovered || s.highlighted;
    !(busy(&states[i]) || busy(&states[i - 1]))
}

/// The next enabled index from `from` by `delta`, clamped (no wrap, like
/// Downshift); `None` starts from the ends.
pub fn move_highlight(from: Option<usize>, delta: isize, disabled: &[bool]) -> Option<usize> {
    let n = disabled.len() as isize;
    if n == 0 {
        return None;
    }
    let mut i = match from {
        Some(i) => i as isize,
        None if delta > 0 => -1,
        None => n,
    };
    loop {
        i += delta;
        if i < 0 || i >= n {
            return from;
        }
        if !disabled[i as usize] {
            return Some(i as usize);
        }
    }
}

/// Type-ahead: the first enabled item after `from` (wrapping) whose label
/// starts with `prefix`, case-insensitively.
pub fn type_ahead(items: &[ListItem], prefix: &str, from: Option<usize>) -> Option<usize> {
    if prefix.is_empty() || items.is_empty() {
        return None;
    }
    let prefix = prefix.to_lowercase();
    let n = items.len();
    // A repeated single letter cycles; a longer prefix stays on a match.
    let start = match from {
        Some(i) if prefix.chars().count() == 1 => i + 1,
        Some(i) => i,
        None => 0,
    };
    (0..n)
        .map(|k| (start + k) % n)
        .find(|&i| !items[i].disabled && items[i].label.to_lowercase().starts_with(&prefix))
}

/// Per-instance list-box state kept across frames.
pub(crate) struct ListState {
    /// The menu is open.
    pub open: bool,
    /// The keyboard/pointer-highlighted row.
    pub highlighted: Option<usize>,
    /// The row under the pointer.
    pub hovered: Option<usize>,
    /// Type-ahead buffer and the time of its last key.
    pub typed: String,
    pub typed_at: Option<Instant>,
    /// The field's measured width in px (the menu matches it).
    pub width: Rc<Cell<f32>>,
}

impl Default for ListState {
    fn default() -> Self {
        Self {
            open: false,
            highlighted: None,
            hovered: None,
            typed: String::new(),
            typed_at: None,
            width: Rc::new(Cell::new(0.)),
        }
    }
}

impl ListState {
    /// Add `ch` to the type-ahead buffer (reset after 500ms idle) and return it.
    pub fn push_typed(&mut self, ch: &str) -> String {
        let now = Instant::now();
        if self
            .typed_at
            .is_none_or(|t| now.duration_since(t).as_millis() > 500)
        {
            self.typed.clear();
        }
        self.typed.push_str(ch);
        self.typed_at = Some(now);
        self.typed.clone()
    }
}

/// A zero-size canvas that records its parent's width into `width`.
pub(crate) fn measure_width(width: Rc<Cell<f32>>) -> impl IntoElement {
    canvas(
        move |bounds, _, _| width.set(f32::from(bounds.size.width)),
        |_, _, _, _| {},
    )
    .absolute()
    .top_0()
    .left_0()
    .size_full()
}

/// The chevron (`.cds--list-box__menu-icon`), turning 180° when open.
pub(crate) fn chevron(
    id: ElementId,
    open: bool,
    disabled: bool,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let angle = motion::animate(
        (id, "chevron"),
        if open { std::f32::consts::PI } else { 0. },
        CHEVRON_MOTION,
        window,
        cx,
    );
    Icon::carbon(CarbonIcon::ChevronDown)
        .size(IconSize::S16)
        .color(color(if disabled {
            ColorRole::IconDisabled
        } else {
            ColorRole::IconPrimary
        }))
        .rotate(gpui_kit::radians(angle))
}

/// What a menu row does when used.
pub(crate) type PickFn = Rc<dyn Fn(usize, &mut Window, &mut App)>;
/// Pointer hover over a row (`None` on leave).
pub(crate) type HoverFn = Rc<dyn Fn(Option<usize>, &mut Window, &mut App)>;

/// Everything a menu needs to draw one frame.
pub(crate) struct Menu<'a> {
    pub id: ElementId,
    pub items: &'a [ListItem],
    /// Indices into `items` to show (filtering), in order.
    pub shown: Vec<usize>,
    pub selected: &'a [bool],
    pub highlighted: Option<usize>,
    pub hovered: Option<usize>,
    pub row_height: f32,
    pub width: f32,
    pub level: LayerLevel,
    /// Draw a checkbox per row (multi-select).
    pub checkboxes: bool,
    /// Shown when `shown` is empty.
    pub empty: Option<SharedString>,
    pub on_pick: PickFn,
    pub on_hover: HoverFn,
    pub on_close: overlay::Callback,
}

/// The menu surface, clipped to `progress` of its height (0 closed → 1 open).
pub(crate) fn menu(menu: Menu<'_>, progress: f32, window: &mut Window, cx: &mut App) -> AnyElement {
    let Menu {
        id,
        items,
        shown,
        selected,
        highlighted,
        hovered,
        row_height,
        width,
        level,
        checkboxes,
        empty,
        on_pick,
        on_hover,
        on_close,
    } = menu;
    let states: Vec<ItemState> = shown
        .iter()
        .map(|&i| ItemState {
            selected: selected.get(i).copied().unwrap_or(false),
            highlighted: highlighted == Some(i),
            hovered: hovered == Some(i),
            disabled: items[i].disabled,
        })
        .collect();
    let groups = shown.iter().filter(|&&i| items[i].group.is_some()).count();
    let rows = (shown.len().max(1) + groups) as f32;
    let full = (rows.min(MAX_VISIBLE_ROWS)) * row_height;
    let pad = (row_height - 16.) / 2. - 1.;
    let mut children: Vec<AnyElement> = Vec::new();
    for (k, &i) in shown.iter().enumerate() {
        let item = &items[i];
        if let Some(group) = &item.group {
            // `<optgroup>` heading: `label-01` in `text-secondary`.
            children.push(
                div()
                    .h(rem(row_height))
                    .px(tokens::SPACING_05)
                    .flex()
                    .items_center()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(group.clone())
                    .into_any_element(),
            );
        }
        let state = states[k];
        let look = ItemLook::of(level, state);
        let rule = rule_visible(k, &states) && item.group.is_none();
        let on_pick = on_pick.clone();
        let on_hover = on_hover.clone();
        let label = item.label.clone();
        let mark = checkboxes.then(|| {
            let value = if state.selected {
                CheckboxState::Checked
            } else {
                CheckboxState::Unchecked
            };
            checkbox_box(
                (id.clone(), format!("cb-{i}")).into(),
                CheckboxLook::of(CheckboxLookState {
                    value,
                    disabled: state.disabled,
                    ..Default::default()
                }),
                value,
                false,
                window,
                cx,
            )
        });
        children.push(
            div()
                .id((id.clone(), format!("item-{i}")))
                .relative()
                .h(rem(row_height))
                .flex_shrink_0()
                .bg(look.background)
                .cursor(if state.disabled {
                    gpui_kit::CursorStyle::OperationNotAllowed
                } else {
                    gpui_kit::CursorStyle::PointingHand
                })
                .on_hover(move |over, window, cx| {
                    on_hover(if *over { Some(i) } else { None }, window, cx)
                })
                .when(!state.disabled, |this| {
                    this.on_click(move |_, window, cx| on_pick(i, window, cx))
                })
                // `.cds--list-box__menu-item__option { margin: 0 1rem; border-block-start:
                // 1px solid var(--border-subtle); padding-inline-end: 1.5rem }`.
                .child(
                    div()
                        .mx(tokens::SPACING_05)
                        .h_full()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(rem(12.))
                        .pt(rem(pad))
                        .pb(rem(pad))
                        .pr(rem(24.))
                        .border_t_1()
                        .border_color(if rule {
                            color(level.border_subtle())
                        } else {
                            Hsla::transparent_black()
                        })
                        .type_style(TypeStyle::BodyCompact01)
                        .text_color(look.text)
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .children(mark)
                        .child(label),
                )
                // `.cds--list-box__menu-item__selected-icon { inset-inline-end: 1rem }`.
                .when(state.selected && !checkboxes, |this| {
                    this.child(
                        div()
                            .absolute()
                            .top_0()
                            .bottom_0()
                            .right(rem(16.))
                            .flex()
                            .items_center()
                            .child(
                                Icon::carbon(CarbonIcon::Checkmark)
                                    .size(IconSize::S16)
                                    .color(color(ColorRole::IconPrimary)),
                            ),
                    )
                })
                .when_some(look.outline, |this, outline| {
                    this.child(super::form::inset_outline(2., outline))
                })
                .into_any_element(),
        );
    }
    if shown.is_empty()
        && let Some(text) = empty
    {
        children.push(
            div()
                .h(rem(row_height))
                .px(tokens::SPACING_05)
                .flex()
                .items_center()
                .type_style(TypeStyle::BodyCompact01)
                .text_color(color(ColorRole::TextSecondary))
                .child(text)
                .into_any_element(),
        );
    }

    div()
        .id((id.clone(), "menu"))
        .w(px(width.max(1.)))
        .h(rem(full * progress.clamp(0., 1.)))
        .overflow_hidden()
        .bg(color(level.layer()))
        .shadow(overlay::floating_shadow())
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .on_mouse_down_out(move |_, window, cx| on_close(window, cx))
        .child(
            div()
                .id((id, "scroll"))
                .h(rem(full))
                .overflow_y_scroll()
                .flex()
                .flex_col()
                .children(children),
        )
        .into_any_element()
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
    fn item_colors_follow_list_box_css() {
        crate::theme::tests_set(CarbonTheme::White);
        let rest = ItemLook::of(LayerLevel::One, ItemState::default());
        assert_eq!(rest.background.a, 0.);
        assert_eq!(hex(rest.text), "#525252");
        let hover = ItemLook::of(
            LayerLevel::One,
            ItemState {
                hovered: true,
                ..Default::default()
            },
        );
        assert_eq!(hex(hover.background), "#e8e8e8");
        let selected = ItemLook::of(
            LayerLevel::One,
            ItemState {
                selected: true,
                ..Default::default()
            },
        );
        assert_eq!(hex(selected.background), "#e0e0e0");
        assert_eq!(hex(selected.text), "#161616");
        let highlighted = ItemLook::of(
            LayerLevel::One,
            ItemState {
                highlighted: true,
                ..Default::default()
            },
        );
        assert_eq!(highlighted.outline.map(hex).as_deref(), Some("#0f62fe"));
    }

    #[test]
    fn rows_and_rules() {
        assert_eq!(row_height(40., Density::Comfortable), 40.);
        assert_eq!(row_height(48., Density::Compact), 32.);
        let mut states = vec![ItemState::default(); 3];
        assert!(!rule_visible(0, &states));
        assert!(rule_visible(1, &states));
        states[1].hovered = true;
        assert!(!rule_visible(1, &states) && !rule_visible(2, &states));
    }

    #[test]
    fn keyboard_moves_and_type_ahead() {
        let disabled = [false, true, false];
        assert_eq!(move_highlight(None, 1, &disabled), Some(0));
        assert_eq!(move_highlight(Some(0), 1, &disabled), Some(2));
        assert_eq!(move_highlight(Some(2), 1, &disabled), Some(2));
        assert_eq!(move_highlight(None, -1, &disabled), Some(2));
        let items: Vec<ListItem> = ["React", "Angular", "Vue", "Svelte"]
            .into_iter()
            .map(ListItem::from)
            .collect();
        assert_eq!(type_ahead(&items, "v", None), Some(2));
        assert_eq!(type_ahead(&items, "s", Some(0)), Some(3));
        assert_eq!(type_ahead(&items, "x", None), None);
    }
}
