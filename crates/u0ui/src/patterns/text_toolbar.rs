//! TextToolbar: Carbon's text-toolbar pattern (`.cu-ptbar`,
//! `components/PatternTextToolbar`). A single `layer-01` row of ghost icon
//! buttons in groups split by short vertical rules, with inline controls
//! (font and size dropdowns) between them and an overflow menu at the end.
//!
//! - Toggles (bold, italic…) are icon buttons in their selected state; each
//!   button's label is its tooltip and accessible name, so callers include
//!   the shortcut (`"Bold (⌘B)"`).
//! - `role="toolbar"`: ←/→ move focus between the buttons, the inline
//!   dropdowns ([`ToolbarItem::dropdown`]) and the overflow button
//!   (wrapping), skipping disabled buttons; Home/End jump to the ends.
//! - When the row is too narrow, items leave from the right into the
//!   overflow menu (`"More"`), as the reference README asks. Buttons become
//!   menu items; inline controls that do not fit are hidden (a menu cannot
//!   hold them). The row's width is measured each frame, so this follows
//!   window resizes and zoom.
//!
//! ```ignore
//! TextToolbar::new("fmt")
//!     .overflow_label("More")
//!     .item(ToolbarItem::action(ToolbarAction::new("undo", "Undo", CarbonIcon::Undo)))
//!     .item(ToolbarItem::Separator)
//!     .item(ToolbarItem::action(ToolbarAction::new("bold", "Bold (⌘B)", CarbonIcon::TextBold).selected(true)))
//! ```
use crate::components::{
    Align, Button, ButtonKind, ButtonSize, ButtonVariant, DropdownMenu, DropdownMenuSize, MenuItem,
    Select, button::Geometry,
};
use crate::{
    density::{self, Density},
    icons::CarbonIcon,
    theme::color,
    tokens::{self, ColorRole},
};
use gpui_kit::{
    AnyElement, App, Bounds, ElementId, FocusHandle, InteractiveElement as _, IntoElement,
    KeyDownEvent, ParentElement, Pixels, RenderOnce, SharedString, Styled, Window, canvas, div,
    prelude::FluentBuilder as _, px, rems,
};
use std::rc::Rc;

/// `.cu-ptbar { padding: 0 var(--spacing-02) }`, each side.
pub const BAR_PADDING: tokens::TokenLength = tokens::SPACING_02;
/// `.cu-ptsep { width: 1px; height: 1.25rem }`.
pub const SEPARATOR_SIZE: (f32, f32) = (1., 20.);
/// `.cu-ptsep { margin: 0 var(--spacing-02) }`, each side.
pub const SEPARATOR_MARGIN: tokens::TokenLength = tokens::SPACING_02;

type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

/// One icon button of the toolbar (`IconButton kind="ghost" size="md"
/// align="bottom"`).
#[derive(Clone)]
pub struct ToolbarAction {
    id: SharedString,
    label: SharedString,
    icon: CarbonIcon,
    selected: bool,
    disabled: bool,
    on_click: Option<Handler>,
}

impl ToolbarAction {
    /// An action with a stable `id`, its `label` (tooltip, accessible name
    /// and overflow menu text) and its `icon`.
    pub fn new(
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        icon: CarbonIcon,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon,
            selected: false,
            disabled: false,
            on_click: None,
        }
    }

    /// Selected state of a toggle (`isSelected`): `background-selected`.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Disabled.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Called on click, Enter or Space, and from the overflow menu.
    pub fn on_click(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
}

/// An entry of the toolbar row.
pub enum ToolbarItem {
    /// An icon button.
    Action(ToolbarAction),
    /// A group rule (`.cu-ptsep`).
    Separator,
    /// An inline control `width` px wide at 100% zoom, such as a font
    /// dropdown (`<div style={{width:140}}><Dropdown type="inline"/></div>`).
    Control {
        /// Its fixed width in px (rem-scaled when drawn).
        width: f32,
        /// The control.
        element: AnyElement,
    },
    /// Free space that pushes what follows to the end (`flex: 1`).
    Spacer,
    /// An inline dropdown `width` px wide that the arrow keys reach
    /// (`<div style={{width:140}}><Dropdown type="inline"/></div>`).
    Dropdown {
        /// Its slot's width in px (the dropdown itself may run past it,
        /// as an inline `Dropdown` is as wide as its label).
        width: f32,
        /// The dropdown.
        select: Box<Select>,
    },
}

impl ToolbarItem {
    /// An icon button.
    pub fn action(action: ToolbarAction) -> Self {
        Self::Action(action)
    }

    /// An inline control `width` px wide.
    pub fn control(width: f32, element: impl IntoElement) -> Self {
        Self::Control {
            width,
            element: element.into_any_element(),
        }
    }

    /// An inline dropdown in a `width` px slot, a stop of the arrow keys.
    pub fn dropdown(width: f32, select: Select) -> Self {
        Self::Dropdown {
            width,
            select: Box::new(select),
        }
    }
}

/// What the layout needs to know of an item.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Slot {
    /// A button or control of this width.
    Fixed(f32),
    /// A separator (its width including margins).
    Rule(f32),
    /// The spacer (no width of its own).
    Spacer,
}

/// The width of a toolbar icon button under `density`: Carbon `md`,
/// square (`.cds--btn--icon-only { inline-size: block-size }`).
pub fn button_width(density: Density) -> f32 {
    let g = Geometry::of(ButtonSize::Md, density, ButtonKind::Ghost, true);
    g.width.unwrap_or(g.height)
}

/// A separator's width with its margins.
pub fn separator_width() -> f32 {
    SEPARATOR_SIZE.0 + 2. * SEPARATOR_MARGIN.px_value()
}

/// Which slots stay in the row when `available` px are free and the
/// overflow button takes `more` px (`always_more`: it shows even when
/// nothing overflows). Items leave from the right; a rule left at the end
/// of what stays, or next to another rule, goes too. Returns the kept
/// indices and whether the overflow button shows.
pub fn fit(slots: &[Slot], available: f32, more: f32, always_more: bool) -> (Vec<usize>, bool) {
    let width = |s: &Slot| match s {
        Slot::Fixed(w) | Slot::Rule(w) => *w,
        Slot::Spacer => 0.,
    };
    let total: f32 = slots.iter().map(width).sum();
    let budget = if always_more {
        available - more
    } else {
        available
    };
    if total <= budget + 0.5 {
        return ((0..slots.len()).collect(), always_more);
    }
    let budget = available - more;
    let mut used = 0.;
    let mut kept = Vec::new();
    for (i, slot) in slots.iter().enumerate() {
        used += width(slot);
        if used > budget + 0.5 {
            break;
        }
        kept.push(i);
    }
    // Trailing spacers and rules have nothing after them any more.
    while kept
        .last()
        .is_some_and(|&i| !matches!(slots[i], Slot::Fixed(_)))
    {
        kept.pop();
    }
    (kept, true)
}

/// The next focused button for an arrow key, wrapping (`role="toolbar"`).
pub fn step(key: &str, len: usize, current: Option<usize>) -> Option<usize> {
    if len == 0 {
        return None;
    }
    let n = len as isize;
    let i = current.map(|c| c as isize);
    match key {
        "right" => Some(i.map_or(0, |i| (i + 1).rem_euclid(n)) as usize),
        "left" => Some(i.map_or(n - 1, |i| (i - 1).rem_euclid(n)) as usize),
        "home" => Some(0),
        "end" => Some(len - 1),
        _ => None,
    }
}

/// A text toolbar. See the module docs.
#[derive(IntoElement)]
pub struct TextToolbar {
    id: ElementId,
    items: Vec<ToolbarItem>,
    overflow_label: SharedString,
    overflow_items: Vec<MenuItem>,
    density: Option<Density>,
}

impl TextToolbar {
    /// A toolbar with a stable `id`.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            items: Vec::new(),
            overflow_label: SharedString::default(),
            overflow_items: Vec::new(),
            density: None,
        }
    }

    /// Append an item.
    pub fn item(mut self, item: ToolbarItem) -> Self {
        self.items.push(item);
        self
    }

    /// Append items.
    pub fn items(mut self, items: impl IntoIterator<Item = ToolbarItem>) -> Self {
        self.items.extend(items);
        self
    }

    /// The overflow button's tooltip and accessible name (`"More"`).
    pub fn overflow_label(mut self, label: impl Into<SharedString>) -> Self {
        self.overflow_label = label.into();
        self
    }

    /// Menu items always in the overflow menu, after any that overflowed.
    /// With any, the overflow button always shows.
    pub fn overflow_items(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.overflow_items.extend(items);
        self
    }

    /// Density override (default: the active one).
    pub fn density(mut self, density: Density) -> Self {
        self.density = Some(density);
        self
    }
}

fn rem_px(v: f32) -> gpui_kit::Rems {
    rems(v / tokens::REM_PX)
}

impl RenderOnce for TextToolbar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let density = self.density.unwrap_or_else(density::current);
        let button_w = button_width(density);
        let bounds = window.use_keyed_state((id.clone(), "bounds"), cx, |_, _| {
            Bounds::<Pixels>::default()
        });
        let scale = f32::from(window.rem_size()) / tokens::REM_PX;
        let measured = f32::from(bounds.read(cx).size.width) / scale;
        // Before the first measurement everything shows.
        let available = if measured > 0. {
            measured - 2. * BAR_PADDING.px_value()
        } else {
            f32::MAX
        };
        let slots: Vec<Slot> = self
            .items
            .iter()
            .map(|item| match item {
                ToolbarItem::Action(_) => Slot::Fixed(button_w),
                ToolbarItem::Separator => Slot::Rule(separator_width()),
                ToolbarItem::Control { width, .. } | ToolbarItem::Dropdown { width, .. } => {
                    Slot::Fixed(*width)
                }
                ToolbarItem::Spacer => Slot::Spacer,
            })
            .collect();
        let always_more = !self.overflow_items.is_empty();
        let (kept, more) = fit(&slots, available, button_w, always_more);

        // Focus handles for the buttons left in the row, in order.
        let action_count = kept
            .iter()
            .filter(|&&i| matches!(self.items[i], ToolbarItem::Action(_)))
            .count();
        let handles =
            window.use_keyed_state((id.clone(), "focus"), cx, |_, _| Vec::<FocusHandle>::new());
        handles.update(cx, |h, cx| {
            while h.len() < action_count {
                h.push(cx.focus_handle());
            }
        });
        let focus: Vec<FocusHandle> = handles.read(cx)[..action_count].to_vec();
        // Handles for the inline dropdowns that bring none of their own: a
        // Select's fallback handle is keyed under the element ids of its own
        // render, so asking it from here would name a different handle than
        // the one its trigger tracks (no ring, keys lost).
        let dropdown_count = self
            .items
            .iter()
            .filter(|item| matches!(item, ToolbarItem::Dropdown { .. }))
            .count();
        let dropdown_handles =
            window.use_keyed_state((id.clone(), "dropdown-focus"), cx, |_, _| {
                Vec::<FocusHandle>::new()
            });
        dropdown_handles.update(cx, |h, cx| {
            while h.len() < dropdown_count {
                h.push(cx.focus_handle());
            }
        });
        let dropdown_focus: Vec<FocusHandle> = dropdown_handles.read(cx)[..dropdown_count].to_vec();
        let mut dropdown_slot = 0;
        let more_focus = window
            .use_keyed_state((id.clone(), "more-focus"), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        // The arrow keys' stops in row order: enabled buttons, the inline
        // dropdowns, then the overflow button (a disabled button never takes
        // focus, as it shows none).
        let mut stops: Vec<FocusHandle> = Vec::new();

        let mut row: Vec<AnyElement> = Vec::new();
        let mut menu: Vec<MenuItem> = Vec::new();
        let mut slot = 0;
        for (i, item) in self.items.into_iter().enumerate() {
            let keep = kept.contains(&i);
            match item {
                ToolbarItem::Action(action) => {
                    if keep {
                        if !action.disabled {
                            stops.push(focus[slot].clone());
                        }
                        let handler = action.on_click.clone();
                        row.push(
                            Button::new((id.clone(), action.id.clone()), action.label.clone())
                                .variant(ButtonVariant::Ghost)
                                .size(ButtonSize::Md)
                                .density(density)
                                .icon(action.icon)
                                .icon_only(true)
                                .selected(action.selected)
                                .disabled(action.disabled)
                                .tooltip_align(Align::Bottom)
                                .track_focus(&focus[slot])
                                .when_some(handler, |b, h| b.on_click(move |_, w, cx| h(w, cx)))
                                .into_any_element(),
                        );
                        slot += 1;
                    } else {
                        let mut entry = MenuItem::item(action.label)
                            .icon(action.icon)
                            .disabled(action.disabled);
                        if let Some(h) = action.on_click {
                            entry = entry.on_select(move |w, cx| h(w, cx));
                        }
                        menu.push(entry);
                    }
                }
                ToolbarItem::Separator if keep => row.push(
                    div()
                        .flex_none()
                        .w(px(SEPARATOR_SIZE.0))
                        .h(rem_px(SEPARATOR_SIZE.1))
                        .mx(SEPARATOR_MARGIN)
                        // `.cu-ptsep { background: var(--border-subtle-01) }`.
                        .bg(color(ColorRole::BorderSubtle01))
                        .into_any_element(),
                ),
                ToolbarItem::Control { width, element } if keep => row.push(
                    div()
                        .flex_none()
                        .w(rem_px(width))
                        .child(element)
                        .into_any_element(),
                ),
                ToolbarItem::Dropdown { width, select } if keep => {
                    let own = &dropdown_focus[dropdown_slot];
                    dropdown_slot += 1;
                    let select = if select.has_focus_handle() {
                        *select
                    } else {
                        select.track_focus(own)
                    };
                    stops.push(select.focus_handle(window, cx));
                    row.push(
                        div()
                            .flex_none()
                            .w(rem_px(width))
                            .child(select)
                            .into_any_element(),
                    )
                }
                ToolbarItem::Dropdown { .. } => dropdown_slot += 1,
                ToolbarItem::Spacer if keep => {
                    row.push(div().flex_1().min_w_0().into_any_element())
                }
                _ => {}
            }
        }
        if more {
            stops.push(more_focus.clone());
            menu.extend(self.overflow_items);
            // The overflow button sits at the end even when the spacer left.
            if !kept.iter().any(|&i| matches!(slots[i], Slot::Spacer)) {
                row.push(div().flex_1().min_w_0().into_any_element());
            }
            row.push(
                div()
                    .flex_none()
                    .child(
                        DropdownMenu::new((id.clone(), "more"))
                            .overflow_label(self.overflow_label.clone())
                            .overflow_icon(CarbonIcon::OverflowMenuHorizontal)
                            .size(DropdownMenuSize::Md)
                            .track_focus(&more_focus)
                            .items(menu),
                    )
                    .into_any_element(),
            );
        }

        let measure = canvas(
            move |b, window, cx| {
                if *bounds.read(cx) != b {
                    bounds.update(cx, |s, _| *s = b);
                    // A notify from inside this frame's prepaint is dropped
                    // when the frame ends, so a resize would keep the old
                    // fit (items clipped, no "…") until something else
                    // redraws. Ask for the next frame explicitly.
                    window.on_next_frame(|window, _| window.refresh());
                }
            },
            |_, _, _, _| {},
        )
        .absolute()
        .top_0()
        .left_0()
        .size_full();

        let buttons = stops;
        div()
            .id(id)
            .relative()
            .flex()
            .flex_row()
            // `.cu-ptbar { display: flex; align-items: center; flex-wrap: nowrap; gap: 0 }`.
            .items_center()
            .w_full()
            .min_w_0()
            // `.cu-ptbar { overflow: hidden }`.
            .overflow_hidden()
            // `.cu-ptbar { padding: 0 var(--spacing-02) }`.
            .px(BAR_PADDING)
            // `.cu-ptbar { background: var(--layer-01); border-bottom: 1px solid var(--border-subtle-01) }`.
            .bg(color(ColorRole::Layer01))
            .border_b_1()
            .border_color(color(ColorRole::BorderSubtle01))
            .on_key_down(move |event: &KeyDownEvent, window, cx| {
                let current = buttons.iter().position(|h| h.is_focused(window));
                if current.is_none() {
                    return;
                }
                if let Some(next) = step(event.keystroke.key.as_str(), buttons.len(), current) {
                    cx.stop_propagation();
                    buttons[next].focus(window, cx);
                }
            })
            .children(row)
            .child(measure)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry_follows_patterns_css() {
        // `.cu-ptsep`: 1px + 2 × spacing-02.
        assert_eq!(separator_width(), 9.);
        assert_eq!(SEPARATOR_SIZE.1, 20.);
        assert_eq!(BAR_PADDING.px_value(), 4.);
        // `IconButton size="md"`: 40 comfortable, density-field-md compact.
        assert_eq!(button_width(Density::Comfortable), 40.);
        assert_eq!(button_width(Density::Compact), 32.);
    }

    #[test]
    fn everything_fits_without_overflow() {
        let slots = [Slot::Fixed(40.), Slot::Rule(9.), Slot::Fixed(40.)];
        assert_eq!(fit(&slots, 100., 40., false), (vec![0, 1, 2], false));
        assert_eq!(fit(&slots, 89., 40., false), (vec![0, 1, 2], false));
        // Overflow items keep the button even when everything fits.
        assert_eq!(fit(&slots, 200., 40., true), (vec![0, 1, 2], true));
    }

    #[test]
    fn items_leave_from_the_right_and_rules_go_with_them() {
        let slots = [
            Slot::Fixed(40.),
            Slot::Fixed(40.),
            Slot::Rule(9.),
            Slot::Fixed(40.),
            Slot::Spacer,
        ];
        // 128 px: not all fits; with the 40 px overflow button, two buttons
        // and the rule fit, but a trailing rule is dropped.
        assert_eq!(fit(&slots, 128., 40., false), (vec![0, 1], true));
        assert_eq!(fit(&slots, 60., 40., false), (vec![], true));
    }

    #[test]
    fn arrow_keys_wrap_like_a_toolbar() {
        assert_eq!(step("right", 3, Some(2)), Some(0));
        assert_eq!(step("left", 3, Some(0)), Some(2));
        assert_eq!(step("home", 3, Some(1)), Some(0));
        assert_eq!(step("end", 3, Some(1)), Some(2));
        assert_eq!(step("up", 3, Some(1)), None);
        assert_eq!(step("right", 0, None), None);
    }
}
