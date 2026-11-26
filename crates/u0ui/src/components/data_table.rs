//! DataTable: shadcn `Data Table` → Carbon `DataTable` + `TableToolbar` +
//! `TableToolbarSearch` + `TableBatchActions` + `TableSelectAll`/`TableSelectRow`
//! + `TableExpandRow` + `Pagination`.
//!
//! - **Rows and cells** reuse [`super::table`]'s geometry, row colors and row
//!   helpers (row heights per size × density, label paddings, zebra/hover/
//!   selected borders, the 70ms background transition). Kit's virtualized
//!   `Table` is not used for the reasons given there.
//! - **Behavior** follows Carbon's `DataTable` state machine: sorting cycles
//!   ascending → descending → none per header (another header starts at
//!   ascending; [`SortDirection::next`]), with Carbon's default comparator
//!   (numbers numerically, other strings case-insensitively; [`compare_values`]);
//!   the toolbar search filters rows whose cell text contains the query
//!   (case-insensitive, trimmed; [`row_matches`]) and resets to page 1;
//!   select-all toggles the filtered rows (deselects them if any is selected)
//!   and is indeterminate while only some are; rows expand with a chevron.
//!   The search text is Kit's `InputState` (editing, IME, clipboard).
//! - **Sort header** (`.cds--table-sort`): the whole `th` is a button,
//!   `layer-accent`, hover and active `layer-selected-hover`, a 2px inset
//!   `focus` outline; the unsorted `ArrowsVertical` icon shows only on hover or
//!   focus; the sorted `ArrowUp` turns 180° for descending (`transition:
//!   transform 250ms cubic-bezier(0.5, 0, 0.1, 1)`, a cut under reduced
//!   motion). Its height is the size's (`.cds--data-table--sort th { block-size:
//!   3rem }` …), which compact density does not override (it sets the `tr`).
//! - **Batch actions** (`.cds--batch-actions`): while rows are selected a
//!   `background-brand` bar replaces the toolbar: it slides up from the
//!   toolbar's bottom while its clip grows and it fades in (110ms productive
//!   standard), and the toolbar content drops out of view; leaving runs the
//!   reverse. Reduced motion keeps the fade and drops the slide. Shown only
//!   when the caller gives batch actions (the reference renders none).
//! - **Toolbar search** (`.cds--toolbar-search-container-persistent`): no
//!   border, `field` fill, hover/filled `field-hover`, 2px `focus` outline,
//!   magnifier 16px `icon-secondary` at `(48 - 16) / 2`, a 48px clear button.
//!   The persistent classes sit on the `.cds--search` flexbox itself
//!   (`display: flex; align-items: center`), stretched to the 48px toolbar; in
//!   compact density `.cds--search-input` is 32px (`[data-density="compact"]
//!   .cds--search-input`) and is centered in it, with the magnifier and the
//!   48px clear button positioned on the 48px box.
//! - **Expandable rows** (`.cds--table-expand`): a 40px chevron column
//!   (`ChevronRight` at `rotate(90deg)`, `270deg` when expanded, 150ms
//!   productive standard); the expanded row (`tr[data-child-row]`) is indented
//!   3.5rem (5.5rem with selection), padded 1rem / 1.5rem, and hovers together
//!   with its parent.
//!
//! ```ignore
//! DataTable::new("servers")
//!     .title("Virtual servers")
//!     .columns(vec![DataTableColumn::new("name", "Name")])
//!     .rows(vec![DataTableRow::new("1").cell("name", "vm-101")])
//!     .selectable(true)
//!     .page_size(5)
//! ```
use super::{
    checkbox::Checkbox,
    form::{inset_outline, rem},
    pagination::Pagination,
    table::{
        self, CellAlign, ColumnWidth, Geometry, RowLook, RowState, TableSize, body_row,
        cell_box_padded, container, container_header, header_label, header_row,
    },
};
use crate::{
    density::{self, Density},
    icons::{CarbonIcon, Icon, IconSize},
    motion::{self, Curve, Motion, Speed},
    theme::{LayerLevel, color},
    tokens::{self, ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    AnyElement, App, CursorStyle, ElementId, Entity, Focusable as _, Hsla, InteractiveElement,
    IntoElement, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled,
    Window,
    base::{self, Easing},
    component::{Sizable as _, Size, input::Input as KitInput},
    div,
    prelude::FluentBuilder as _,
    px, radians,
};
use std::{cmp::Ordering, collections::HashSet, rc::Rc, sync::Arc, time::Duration};

pub use super::input::InputState;

/// A Carbon sort direction (`DataTable.sortStates`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SortDirection {
    /// Original row order.
    #[default]
    None,
    /// `ASC`.
    Ascending,
    /// `DESC`.
    Descending,
}

impl SortDirection {
    /// Carbon's `getNextSortDirection`: a new header starts at ascending; the
    /// same header cycles ascending → descending → none → ascending.
    pub fn next(self, same_header: bool) -> Self {
        if !same_header {
            return Self::Ascending;
        }
        match self {
            Self::None => Self::Ascending,
            Self::Ascending => Self::Descending,
            Self::Descending => Self::None,
        }
    }
}

/// Carbon's default `sortRow` comparator: two numbers compare numerically,
/// anything else as text (case-insensitive, like `localeCompare`'s primary
/// strength, then by the raw text to keep the order total).
pub fn compare_values(a: &str, b: &str) -> Ordering {
    match (a.trim().parse::<f64>(), b.trim().parse::<f64>()) {
        (Ok(x), Ok(y)) => x.partial_cmp(&y).unwrap_or(Ordering::Equal),
        _ => a
            .to_lowercase()
            .cmp(&b.to_lowercase())
            .then_with(|| a.cmp(b)),
    }
}

/// Carbon's `defaultFilterRows`: some cell's text, trimmed and lower-cased,
/// contains the trimmed, lower-cased query.
pub fn row_matches(row: &DataTableRow, keys: &[SharedString], query: &str) -> bool {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return true;
    }
    keys.iter()
        .any(|k| row.get(k).trim().to_lowercase().contains(&q))
}

/// The row indices shown, in order: filtered by `query`, then sorted by
/// column `sort` (stable; `None` keeps the original order).
pub fn visible_rows(
    rows: &[DataTableRow],
    keys: &[SharedString],
    query: &str,
    sort: Option<(SharedString, SortDirection)>,
) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..rows.len())
        .filter(|&i| row_matches(&rows[i], keys, query))
        .collect();
    if let Some((key, dir)) = sort {
        match dir {
            SortDirection::None => {}
            SortDirection::Ascending => {
                idx.sort_by(|&a, &b| compare_values(rows[a].get(&key), rows[b].get(&key)))
            }
            SortDirection::Descending => {
                idx.sort_by(|&a, &b| compare_values(rows[b].get(&key), rows[a].get(&key)))
            }
        }
    }
    idx
}

/// The select-all checkbox state: `(checked, indeterminate)` for
/// `selected` of `total` (filtered) rows.
pub fn select_all_state(selected: usize, total: usize) -> (bool, bool) {
    (
        total > 0 && selected == total,
        selected > 0 && selected < total,
    )
}

/// Whether the pagination bar shows: `rs.length > ps || pageSize < rs.length`.
pub fn shows_pagination(filtered: usize, page_size: usize, prop_page_size: usize) -> bool {
    filtered > page_size || prop_page_size < filtered
}

/// Height (px) of a sortable header: `.cds--data-table--sort th { block-size:
/// 3rem }`, xs 1.5rem, sm 2rem, md 2.5rem, xl 4rem (no density override).
pub fn sort_header_height(size: TableSize) -> f32 {
    table::row_height(size, Density::Comfortable)
}

/// `.cds--table-toolbar { block-size: var(--layout-size-height-local) }` =
/// `layout-size-height-lg` (3rem).
pub const TOOLBAR_HEIGHT: f32 = 48.;

/// The toolbar search input height: `[data-density="compact"] .cds--search-input
/// { block-size: var(--density-field-md) }`, else the `lg` search (3rem).
pub fn search_height(density: Density) -> f32 {
    match density {
        Density::Compact => tokens::DENSITY_FIELD_MD.px_value(),
        Density::Comfortable => TOOLBAR_HEIGHT,
    }
}

/// Batch-actions transition: `transition: transform 110ms cubic-bezier(0.2, 0,
/// 0.38, 0.9), clip-path 110ms …, opacity 110ms …`.
pub const BATCH_MOTION: Motion = Motion::new(Curve::ProductiveStandard, Speed::Fast02);

/// Where the batch bar is at slide progress `t` (0 hidden → 1 shown), as
/// `(top offset, visible height)` in px inside the toolbar: `translate3d(0,
/// height·(1−t), 0)` with a `clip-path` growing from the zero-height
/// `polygon(0 0, 100% 0, 100% 0, 0 0)` to `polygon(0 0, 300% 0, 300% 300%, 0 300%)`.
pub fn batch_reveal(t: f32, height: f32) -> (f32, f32) {
    let t = t.clamp(0., 1.);
    (height * (1. - t), (3. * t).min(1.) * height)
}

/// Sort icon rotation: `.cds--table-sort--descending .cds--table-sort__icon
/// { transform: rotate(180deg) }`.
pub fn sort_icon_rotation(direction: SortDirection) -> f32 {
    if direction == SortDirection::Descending {
        std::f32::consts::PI
    } else {
        0.
    }
}

/// Expand chevron rotation: `.cds--table-expand__svg { transform: rotate(90deg) }`;
/// expanded (`[data-previous-value=collapsed]`) `rotate(270deg)`.
pub fn expand_rotation(expanded: bool) -> f32 {
    if expanded {
        1.5 * std::f32::consts::PI
    } else {
        0.5 * std::f32::consts::PI
    }
}

/// Leading-column geometry: `(expand column width, select column (width,
/// padding start, padding end), first data cell padding start)` in px.
///
/// - `.cds--data-table td.cds--table-expand { padding: 0.5rem; padding-inline-end: 0 }`
///   around a 2rem button → 40px.
/// - `td.cds--table-column-checkbox { min-inline-size: 2.5rem; padding-inline: 1rem 0.25rem }`;
///   after the expand column `padding-inline: 0.375rem` and the next cell
///   `padding-inline-start: 0.5rem`.
pub fn leading_columns(expandable: bool, selectable: bool) -> (f32, (f32, f32, f32), f32) {
    match (expandable, selectable) {
        (true, true) => (40., (40., 6., 6.), 8.),
        (false, true) => (0., (40., 16., 4.), table::CELL_PADDING_X),
        (true, false) => (40., (0., 0., 0.), table::CELL_PADDING_X),
        (false, false) => (0., (0., 0., 0.), table::CELL_PADDING_X),
    }
}

/// Expanded row indent: `tr[data-child-row] td { padding-inline-start: 3.5rem }`,
/// with a checkbox column `5.5rem`.
pub fn child_indent(selectable: bool) -> f32 {
    if selectable { 88. } else { 56. }
}

/// Renders one cell's content.
pub type CellRender = Rc<dyn Fn(&SharedString, &DataTableRow) -> AnyElement>;
/// Renders an expanded row's content.
pub type ExpandRender = Rc<dyn Fn(&DataTableRow) -> AnyElement>;
type BatchHandler = Rc<dyn Fn(&[SharedString], &mut Window, &mut App)>;
type Action = Rc<dyn Fn(&mut Window, &mut App)>;

/// `index.d.ts` `DataTableColumn`: `{ key, header, render? }`.
#[derive(Clone)]
pub struct DataTableColumn {
    key: SharedString,
    header: SharedString,
    render: Option<CellRender>,
    width: ColumnWidth,
}

impl DataTableColumn {
    /// A column showing field `key` under `header`.
    pub fn new(key: impl Into<SharedString>, header: impl Into<SharedString>) -> Self {
        Self {
            key: key.into(),
            header: header.into(),
            render: None,
            width: ColumnWidth::default(),
        }
    }

    /// `render(value, row)`.
    pub fn render(
        mut self,
        f: impl Fn(&SharedString, &DataTableRow) -> AnyElement + 'static,
    ) -> Self {
        self.render = Some(Rc::new(f));
        self
    }

    /// The column width (default: an equal share).
    pub fn width(mut self, width: ColumnWidth) -> Self {
        self.width = width;
        self
    }
}

/// One row: a unique `id` and its field values (as text, like Carbon's cells).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DataTableRow {
    /// The unique row id.
    pub id: SharedString,
    cells: Vec<(SharedString, SharedString)>,
}

impl DataTableRow {
    /// A row with `id`.
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            cells: Vec::new(),
        }
    }

    /// Set field `key`.
    pub fn cell(mut self, key: impl Into<SharedString>, value: impl Into<SharedString>) -> Self {
        self.cells.push((key.into(), value.into()));
        self
    }

    /// The value of field `key` (empty when missing).
    pub fn get(&self, key: &str) -> &str {
        self.cells
            .iter()
            .find(|(k, _)| k.as_ref() == key)
            .map_or("", |(_, v)| v.as_ref())
    }

    fn value(&self, key: &str) -> SharedString {
        self.cells
            .iter()
            .find(|(k, _)| k.as_ref() == key)
            .map(|(_, v)| v.clone())
            .unwrap_or_default()
    }
}

/// A `TableBatchAction`: a button in the batch-actions bar.
#[derive(Clone)]
pub struct BatchAction {
    label: SharedString,
    icon: Option<CarbonIcon>,
    on_click: BatchHandler,
}

impl BatchAction {
    /// An action called with the selected row ids.
    pub fn new(
        label: impl Into<SharedString>,
        on_click: impl Fn(&[SharedString], &mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            label: label.into(),
            icon: None,
            on_click: Rc::new(on_click),
        }
    }

    /// The trailing icon (`renderIcon`).
    pub fn icon(mut self, icon: CarbonIcon) -> Self {
        self.icon = Some(icon);
        self
    }
}

/// Localized texts (Carbon's English defaults).
#[derive(Clone)]
pub struct DataTableText {
    /// Search placeholder (`TableToolbarSearch` default `Filter table`).
    pub filter_placeholder: SharedString,
    /// Clear-search button label.
    pub clear_search: SharedString,
    /// Batch summary (`1 item selected` / `N items selected`).
    pub items_selected: Rc<dyn Fn(usize) -> SharedString>,
    /// Batch cancel button (`Cancel`).
    pub cancel: SharedString,
    /// Expand button label.
    pub expand_row: SharedString,
}

impl Default for DataTableText {
    fn default() -> Self {
        Self {
            filter_placeholder: "Filter table".into(),
            clear_search: "Clear search input".into(),
            items_selected: Rc::new(|n| {
                if n == 1 {
                    "1 item selected".into()
                } else {
                    format!("{n} items selected").into()
                }
            }),
            cancel: "Cancel".into(),
            expand_row: "Expand current row".into(),
        }
    }
}

/// A data table. See the module docs.
#[derive(IntoElement)]
pub struct DataTable {
    id: ElementId,
    columns: Vec<DataTableColumn>,
    rows: Vec<DataTableRow>,
    title: Option<SharedString>,
    description: Option<SharedString>,
    size: TableSize,
    sortable: bool,
    selectable: bool,
    search: bool,
    page_size: usize,
    zebra: bool,
    toolbar_actions: Option<AnyElement>,
    batch_actions: Vec<BatchAction>,
    expand: Option<ExpandRender>,
    level: LayerLevel,
    density: Option<Density>,
    text: DataTableText,
}

impl DataTable {
    /// An empty table (`sortable`, `search`, `pageSize` 10, size `lg`).
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            columns: Vec::new(),
            rows: Vec::new(),
            title: None,
            description: None,
            size: TableSize::Lg,
            sortable: true,
            selectable: false,
            search: true,
            page_size: 10,
            zebra: false,
            toolbar_actions: None,
            batch_actions: Vec::new(),
            expand: None,
            level: LayerLevel::One,
            density: None,
            text: DataTableText::default(),
        }
    }

    /// `columns`.
    pub fn columns(mut self, columns: Vec<DataTableColumn>) -> Self {
        self.columns = columns;
        self
    }

    /// `rows` (each with a unique `id`).
    pub fn rows(mut self, rows: Vec<DataTableRow>) -> Self {
        self.rows = rows;
        self
    }

    /// `title`.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// `description`.
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// `size`.
    pub fn size(mut self, size: TableSize) -> Self {
        self.size = size;
        self
    }

    /// `sortable` (default true).
    pub fn sortable(mut self, sortable: bool) -> Self {
        self.sortable = sortable;
        self
    }

    /// `selectable`.
    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    /// `search` (default true).
    pub fn search(mut self, search: bool) -> Self {
        self.search = search;
        self
    }

    /// `pageSize` (default 10).
    pub fn page_size(mut self, size: usize) -> Self {
        self.page_size = size.max(1);
        self
    }

    /// `zebra`.
    pub fn zebra(mut self, zebra: bool) -> Self {
        self.zebra = zebra;
        self
    }

    /// `toolbarActions`: elements at the end of the toolbar.
    pub fn toolbar_actions(mut self, actions: impl IntoElement) -> Self {
        self.toolbar_actions = Some(actions.into_any_element());
        self
    }

    /// Batch actions shown while rows are selected (`TableBatchActions`).
    pub fn batch_actions(mut self, actions: Vec<BatchAction>) -> Self {
        self.batch_actions = actions;
        self
    }

    /// Make rows expandable, showing `f(row)` under an expanded row.
    pub fn expandable(mut self, f: impl Fn(&DataTableRow) -> AnyElement + 'static) -> Self {
        self.expand = Some(Rc::new(f));
        self
    }

    /// The layer the table sits on.
    pub fn layer(mut self, level: LayerLevel) -> Self {
        self.level = level;
        self
    }

    /// Override the global density.
    pub fn density(mut self, density: Density) -> Self {
        self.density = Some(density);
        self
    }

    /// Localized texts.
    pub fn text(mut self, text: DataTableText) -> Self {
        self.text = text;
        self
    }
}

/// The table's interaction state.
#[derive(Default)]
struct State {
    sort: Option<(usize, SortDirection)>,
    selected: HashSet<SharedString>,
    expanded: HashSet<SharedString>,
    page: usize,
    page_size: usize,
    query_seen: SharedString,
    hovered_row: Option<SharedString>,
}

fn child_id(id: &ElementId, name: impl Into<SharedString>) -> ElementId {
    ElementId::NamedChild(Arc::new(id.clone()), name.into())
}

/// `transition: transform 250ms cubic-bezier(0.5, 0, 0.1, 1)` (`.cds--table-sort__icon`).
fn sort_transition() -> base::Transition {
    base::Transition::new(Duration::from_millis(250))
        .easing(Easing::cubic_bezier(0.5, 0., 0.1, 1.).expect("valid bezier"))
        .reduced_motion(base::ReducedMotion::Finish)
}

/// `.cds--table-expand__svg { transition: transform 150ms cubic-bezier(0.2, 0, 0.38, 0.9) }`.
const EXPAND_MOTION: Motion =
    Motion::new(Curve::ProductiveStandard, Speed::Moderate01).spatial(true);

struct Pointer {
    hovered: bool,
}

/// A sortable header cell (`th.cds--table-sort__header > button.cds--table-sort`).
#[allow(clippy::too_many_arguments)]
fn sort_header(
    id: ElementId,
    label: SharedString,
    direction: SortDirection,
    active: bool,
    size: TableSize,
    label_start: f32,
    on_click: Action,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let pointer = window.use_keyed_state((id.clone(), "pointer"), cx, |_, _| Pointer {
        hovered: false,
    });
    let focus = window
        .use_keyed_state((id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
        .read(cx)
        .clone();
    let hovered = pointer.read(cx).hovered;
    let focused = focus.is_focused(window);
    // `.cds--table-sort { background-color: layer-accent }`, `:hover` and
    // `--active { background: layer-selected-hover }`.
    let target = if hovered || active {
        color(ColorRole::LayerSelectedHover01)
    } else {
        color(ColorRole::LayerAccent01)
    };
    let bg = motion::animate_color((id.clone(), "bg"), target, Motion::HOVER, window, cx);
    let rotation = base::transition(
        (id.clone(), "rot"),
        sort_icon_rotation(direction),
        sort_transition(),
        window,
        cx,
    );
    let geometry = Geometry::of(size, Density::Comfortable);
    let height = sort_header_height(size);
    let xl = size == TableSize::Xl;
    let icon_color = color(ColorRole::IconPrimary);
    let icon = if active {
        Icon::carbon(CarbonIcon::ArrowUp)
            .size(IconSize::S20)
            .color(icon_color)
            .rotate(radians(rotation))
            .into_any_element()
    } else {
        // `.cds--table-sort__icon-unsorted { opacity: 0 }`, `:hover svg, :focus svg { opacity: 1 }`.
        Icon::carbon(CarbonIcon::ArrowsVertical)
            .size(IconSize::S20)
            .color(if hovered || focused {
                icon_color
            } else {
                Hsla::transparent_black()
            })
            .into_any_element()
    };
    gpui_kit::base::Button::new(id.clone())
        .track_focus(&focus)
        .accessibility_label(label.clone())
        .relative()
        .w_full()
        .h(rem(height))
        .flex()
        .flex_row()
        .justify_between()
        .map(|d| {
            if xl {
                d.items_start()
            } else {
                d.items_center()
            }
        })
        // `.cds--table-sort { padding-inline-start: 1rem }`, or the body cells'
        // start padding after the leading columns, so labels line up with cells.
        .pl(rem(label_start))
        .bg(bg)
        .text_color(color(ColorRole::TextPrimary))
        .cursor(CursorStyle::PointingHand)
        .on_hover({
            let pointer = pointer.clone();
            move |h, _, cx| {
                pointer.update(cx, |p, cx| {
                    if p.hovered != *h {
                        p.hovered = *h;
                        cx.notify();
                    }
                })
            }
        })
        .on_click(move |_, window, cx| on_click(window, cx))
        .child(
            // `.cds--table-sort { line-height: 1 }` with the size's label padding.
            header_label(geometry, label)
                .line_height(rem(14.))
                .min_w_0(),
        )
        .child(
            // `margin-inline: 0.5rem`; xl `margin-block-start: 0.8125rem`.
            div()
                .flex_none()
                .mx(rem(8.))
                .when(xl, |d| d.mt(rem(13.)))
                .child(icon),
        )
        .when(focused, |d| {
            d.child(inset_outline(2., color(ColorRole::Focus)))
        })
}

/// A `.cds--action-list .cds--btn--primary` button in the batch bar: 16px
/// inline padding, `text-on-color`, `button-primary` with hover/active, a 2px
/// `layer` focus outline; the cancel button draws the 1px × 16px divider
/// before it (`.cds--batch-summary__cancel::before`), hidden while it or the
/// action before it is hovered.
#[allow(clippy::too_many_arguments)]
fn batch_button(
    id: ElementId,
    label: SharedString,
    icon: Option<CarbonIcon>,
    divider: Option<bool>,
    hovered_state: Entity<Option<usize>>,
    index: usize,
    on_click: Action,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let pointer = window.use_keyed_state((id.clone(), "pointer"), cx, |_, _| (false, false));
    let focus = window
        .use_keyed_state((id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
        .read(cx)
        .clone();
    let (hovered, pressed) = *pointer.read(cx);
    let target = color(if pressed {
        ColorRole::ButtonPrimaryActive
    } else if hovered {
        ColorRole::ButtonPrimaryHover
    } else {
        ColorRole::ButtonPrimary
    });
    let bg = motion::animate_color((id.clone(), "bg"), target, Motion::HOVER, window, cx);
    let fg = color(ColorRole::TextOnColor);
    // `::before { opacity: 1; transition: opacity 110ms }`; hidden on hover.
    let divider_alpha = divider.map(|hidden| {
        motion::animate(
            (id.clone(), "divider"),
            if hidden { 0f32 } else { 1. },
            BATCH_MOTION,
            window,
            cx,
        )
    });
    gpui_kit::base::Button::new(id.clone())
        .track_focus(&focus)
        .accessibility_label(label.clone())
        .relative()
        .h_full()
        .flex()
        .flex_row()
        .items_center()
        .gap(rem(8.))
        .px(rem(16.))
        .whitespace_nowrap()
        .bg(bg)
        .text_color(fg)
        .type_style(TypeStyle::BodyCompact01)
        .cursor(CursorStyle::PointingHand)
        .on_hover({
            let pointer = pointer.clone();
            move |h, _, cx| {
                pointer.update(cx, |p, cx| {
                    p.0 = *h;
                    if !h {
                        p.1 = false;
                    }
                    cx.notify();
                });
                hovered_state.update(cx, |s, cx| {
                    let next = if *h {
                        Some(index)
                    } else if *s == Some(index) {
                        None
                    } else {
                        *s
                    };
                    if *s != next {
                        *s = next;
                        cx.notify();
                    }
                });
            }
        })
        .on_mouse_down(gpui_kit::MouseButton::Left, {
            let pointer = pointer.clone();
            move |_, _, cx| {
                pointer.update(cx, |p, cx| {
                    p.1 = true;
                    cx.notify();
                })
            }
        })
        .on_mouse_up(gpui_kit::MouseButton::Left, {
            let pointer = pointer.clone();
            move |_, _, cx| {
                pointer.update(cx, |p, cx| {
                    p.1 = false;
                    cx.notify();
                })
            }
        })
        .on_click(move |_, window, cx| on_click(window, cx))
        .when_some(divider_alpha, |d, alpha| {
            d.child(
                div()
                    .absolute()
                    .left_0()
                    .top(rem((TOOLBAR_HEIGHT - 16.) / 2.))
                    .w(px(1.))
                    .h(rem(16.))
                    .bg(fg.opacity(alpha)),
            )
        })
        .child(label)
        .when_some(icon, |d, icon| {
            d.child(Icon::carbon(icon).size(IconSize::S16).color(fg))
        })
        // `.cds--action-list .cds--btn--primary:focus { outline: 2px solid var(--layer) }`.
        .when(focus.is_focused(window), |d| {
            d.child(inset_outline(2., color(ColorRole::Layer01)))
        })
}

#[allow(clippy::too_many_arguments)]
/// The persistent toolbar search (`.cds--search.cds--toolbar-search-container-persistent`).
fn toolbar_search(
    id: ElementId,
    state: Entity<InputState>,
    placeholder: SharedString,
    clear_label: SharedString,
    density: Density,
    level: LayerLevel,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let pointer = window.use_keyed_state((id.clone(), "pointer"), cx, |_, _| false);
    let hovered = *pointer.read(cx);
    let focus = state.read(cx).focus_handle(cx);
    let focused = focus.is_focused(window);
    let value = state.read(cx).value();
    let empty = value.is_empty();
    // `.cds--search-input { background: var(--field) }`; persistent `:hover`
    // and `:not(:placeholder-shown) { background-color: var(--field-hover) }`.
    let target = color(if hovered || !empty {
        level.field_hover()
    } else {
        level.field()
    });
    // `transition: background-color 110ms cubic-bezier(0.2, 0, 0.38, 0.9)`.
    let bg = motion::animate_color(
        (id.clone(), "bg"),
        target,
        Motion::new(Curve::ProductiveStandard, Speed::Fast02),
        window,
        cx,
    );
    let height = search_height(density);
    let pad = TOOLBAR_HEIGHT;
    let editor = KitInput::new(&state)
        .appearance(false)
        .with_size(Size::XSmall)
        .aria_label(placeholder.clone())
        .p_0()
        .h_full()
        .flex_1()
        .min_w_0()
        .text_size(rem(14.))
        .line_height(rem(18.))
        .text_color(color(ColorRole::TextPrimary));
    let input = div()
        .id((id.clone(), "input"))
        .relative()
        .w_full()
        .h(rem(height))
        .flex()
        .items_center()
        .px(rem(pad))
        .bg(bg)
        .type_style(TypeStyle::BodyCompact01)
        .cursor(CursorStyle::IBeam)
        .on_hover({
            let pointer = pointer.clone();
            move |h, _, cx| {
                pointer.update(cx, |p, cx| {
                    *p = *h;
                    cx.notify();
                })
            }
        })
        .child(editor)
        .when(empty, |d| {
            d.child(
                div()
                    .absolute()
                    .left(rem(pad))
                    .right(rem(pad))
                    .top_0()
                    .bottom_0()
                    .flex()
                    .items_center()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_color(color(ColorRole::TextPlaceholder))
                    .child(placeholder),
            )
        })
        // `:focus:not([disabled]) { outline: 2px solid var(--focus); outline-offset: -2px }`.
        .when(focused, |d| {
            d.child(inset_outline(2., color(ColorRole::Focus)))
        });

    let clear_focus = window
        .use_keyed_state((id.clone(), "clear-focus"), cx, |_, cx| cx.focus_handle())
        .read(cx)
        .clone();
    let clear_focused = clear_focus.is_focused(window);
    let clear = (!empty).then(|| {
        let state = state.clone();
        gpui_kit::base::Button::new((id.clone(), "clear"))
            .track_focus(&clear_focus)
            .accessibility_label(clear_label)
            .absolute()
            .top_0()
            .right_0()
            .size(rem(TOOLBAR_HEIGHT))
            .flex()
            .items_center()
            .justify_center()
            .cursor(CursorStyle::PointingHand)
            .child(
                Icon::carbon(CarbonIcon::Close)
                    .size(IconSize::S16)
                    .color(color(ColorRole::IconPrimary)),
            )
            .when(clear_focused, |d| {
                d.child(inset_outline(2., color(ColorRole::Focus)))
            })
            .on_click(move |_, window, cx| {
                state.update(cx, |s, cx| s.set_value("", window, cx));
            })
    });

    div()
        .relative()
        .flex_1()
        .min_w_0()
        .h_full()
        .flex()
        .flex_col()
        .justify_center()
        .child(input)
        // `.cds--search-magnifier-icon { inset-block-start: 50%;
        // inset-inline-start: calc((var(--layout-size-height-local) - 1rem) / 2) }`.
        .child(
            div()
                .absolute()
                .left(rem((TOOLBAR_HEIGHT - 16.) / 2.))
                .top(rem((TOOLBAR_HEIGHT - 16.) / 2.))
                .child(
                    Icon::carbon(CarbonIcon::Search)
                        .size(IconSize::S16)
                        .color(color(ColorRole::IconSecondary)),
                ),
        )
        .children(clear)
}

impl RenderOnce for DataTable {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let density = self.density.unwrap_or_else(density::current);
        let geometry = Geometry::of(self.size, density);
        let level = self.level;
        let prop_page_size = self.page_size;
        let state = window.use_keyed_state((id.clone(), "state"), cx, move |_, _| State {
            page: 1,
            page_size: prop_page_size,
            ..Default::default()
        });
        let input = window.use_keyed_state((id.clone(), "search"), cx, |window, cx| {
            InputState::new(window, cx)
        });
        let query: SharedString = if self.search {
            input.read(cx).value()
        } else {
            SharedString::default()
        };
        // `onInputChange(e); setPage(1)`.
        if state.read(cx).query_seen != query {
            let q = query.clone();
            state.update(cx, |s, _| {
                s.query_seen = q;
                s.page = 1;
            });
        }

        let keys: Vec<SharedString> = self.columns.iter().map(|c| c.key.clone()).collect();
        let sort = state.read(cx).sort;
        let order = visible_rows(
            &self.rows,
            &keys,
            &query,
            sort.and_then(|(c, d)| keys.get(c).map(|k| (k.clone(), d))),
        );
        let page_size = state.read(cx).page_size.max(1);
        let pages = order.len().div_ceil(page_size).max(1);
        let page = state.read(cx).page.clamp(1, pages);
        let shown: Vec<usize> = order
            .iter()
            .copied()
            .skip((page - 1) * page_size)
            .take(page_size)
            .collect();

        let selected = state.read(cx).selected.clone();
        let expanded = state.read(cx).expanded.clone();
        let hovered_row = state.read(cx).hovered_row.clone();
        let filtered_ids: Vec<SharedString> =
            order.iter().map(|&i| self.rows[i].id.clone()).collect();
        let selected_filtered = filtered_ids
            .iter()
            .filter(|i| selected.contains(*i))
            .count();
        let (all_checked, all_indeterminate) =
            select_all_state(selected_filtered, filtered_ids.len());
        let expandable = self.expand.is_some();
        let (expand_w, (select_w, select_pl, select_pr), first_pl) =
            leading_columns(expandable, self.selectable);

        // ── Header row ──
        let mut head_cells: Vec<AnyElement> = Vec::new();
        if expandable {
            // `th.cds--table-expand` (empty without `enableToggle`).
            head_cells.push(
                cell_box_padded(ColumnWidth::Fixed(expand_w), CellAlign::Start, 0., 0.)
                    .into_any_element(),
            );
        }
        if self.selectable {
            let state = state.clone();
            let ids = filtered_ids.clone();
            head_cells.push(
                cell_box_padded(
                    ColumnWidth::Fixed(select_w),
                    CellAlign::Start,
                    select_pl,
                    select_pr,
                )
                .items_center()
                .child(
                    Checkbox::new(child_id(&id, "select-all"))
                        .checked(all_checked)
                        .indeterminate(all_indeterminate)
                        .on_checked_change(move |_, _, cx| {
                            let ids = ids.clone();
                            state.update(cx, |s, cx| {
                                // Carbon `handleSelectAll`: deselect if any is selected.
                                let any = ids.iter().any(|i| s.selected.contains(i));
                                for i in &ids {
                                    if any {
                                        s.selected.remove(i);
                                    } else {
                                        s.selected.insert(i.clone());
                                    }
                                }
                                cx.notify();
                            })
                        }),
                )
                .into_any_element(),
            );
        }
        for (c, column) in self.columns.iter().enumerate() {
            let pl = if c == 0 {
                first_pl
            } else {
                table::CELL_PADDING_X
            };
            if self.sortable {
                let (direction, active) = match sort {
                    Some((sc, d)) if sc == c && d != SortDirection::None => (d, true),
                    _ => (SortDirection::None, false),
                };
                let state = state.clone();
                let on_click: Action = Rc::new(move |_, cx| {
                    state.update(cx, |s, cx| {
                        let next = match s.sort {
                            Some((sc, d)) => d.next(sc == c),
                            None => SortDirection::Ascending,
                        };
                        s.sort = Some((c, next));
                        cx.notify();
                    })
                });
                head_cells.push(
                    // `th.cds--table-sort__header { padding: 0 }`. The button's start
                    // padding is the body cells' (`.cds--table-sort { padding-inline-start:
                    // 1rem }`, 0.5rem after the expand + checkbox columns), so labels
                    // line up with cell text in every column.
                    cell_box_padded(column.width, CellAlign::Start, 0., 0.)
                        .child(sort_header(
                            child_id(&id, format!("sort-{c}")),
                            column.header.clone(),
                            direction,
                            active,
                            self.size,
                            pl,
                            on_click,
                            window,
                            cx,
                        ))
                        .into_any_element(),
                );
            } else {
                head_cells.push(
                    cell_box_padded(column.width, CellAlign::Start, pl, table::CELL_PADDING_X)
                        .child(header_label(geometry, column.header.clone()))
                        .into_any_element(),
                );
            }
        }
        let header = header_row(geometry, level, head_cells);

        // ── Body ──
        let mut body = div().flex().flex_col().w_full();
        let count = shown.len();
        for (pos, &ri) in shown.iter().enumerate() {
            let row = &self.rows[ri];
            let rid = row.id.clone();
            let is_selected = selected.contains(&rid);
            let is_expanded = expandable && expanded.contains(&rid);
            let row_state = RowState {
                zebra: self.zebra,
                index: pos,
                first: pos == 0,
                last: pos + 1 == count,
                hovered: hovered_row.as_ref() == Some(&rid),
                selected: is_selected,
            };
            let mut look = RowLook::of(row_state, level);
            if expandable && !is_selected && !self.zebra {
                // `.cds--data-table tr.cds--parent-row:first-of-type td {
                // border-block-start: 1px solid var(--border-subtle) }`;
                // expanded: `.cds--parent-row.cds--expandable-row > td {
                // border-block-end: 1px solid var(--border-subtle) }`.
                let subtle = super::separator::line_color(level);
                if pos == 0 {
                    look.border_top = subtle;
                }
                if is_expanded || row_state.hovered {
                    look.border_bottom = subtle;
                }
            }
            let mut cells: Vec<AnyElement> = Vec::new();
            if expandable {
                let state = state.clone();
                let rid2 = rid.clone();
                let rot = motion::animate(
                    child_id(&id, format!("chev-{rid}")),
                    expand_rotation(is_expanded),
                    EXPAND_MOTION,
                    window,
                    cx,
                );
                let focus = window
                    .use_keyed_state(child_id(&id, format!("expand-focus-{rid}")), cx, |_, cx| {
                        cx.focus_handle()
                    })
                    .read(cx)
                    .clone();
                let focused = focus.is_focused(window);
                cells.push(
                    cell_box_padded(ColumnWidth::Fixed(expand_w), CellAlign::Start, 8., 0.)
                        .relative()
                        .h_full()
                        .when(geometry.top_aligned, |d| d.pt(rem(10.)))
                        .child(
                            gpui_kit::base::Button::new(child_id(&id, format!("expand-{rid}")))
                                .track_focus(&focus)
                                .accessibility_label(self.text.expand_row.clone())
                                .relative()
                                .w(rem(32.))
                                // `.cds--table-expand__button { block-size: calc(100% + 1px) }`
                                // of the row's content box (row minus its 1px border);
                                // xl: a 2rem button under `padding-block: 0.625rem`.
                                .h(rem(if geometry.top_aligned {
                                    32.
                                } else {
                                    geometry.row - 1.
                                }))
                                .flex()
                                .items_center()
                                .justify_center()
                                .cursor(CursorStyle::PointingHand)
                                .child(
                                    // `.cds--table-expand__svg { fill: var(--layer-selected-inverse) }`.
                                    Icon::carbon(CarbonIcon::ChevronRight)
                                        .size(IconSize::S16)
                                        .color(color(ColorRole::LayerSelectedInverse))
                                        .rotate(radians(rot)),
                                )
                                // `:focus { box-shadow: inset 0 0 0 2px var(--focus) }`.
                                .when(focused, |d| {
                                    d.child(inset_outline(2., color(ColorRole::Focus)))
                                })
                                .on_click(move |_, _, cx| {
                                    let rid = rid2.clone();
                                    state.update(cx, |s, cx| {
                                        if !s.expanded.remove(&rid) {
                                            s.expanded.insert(rid);
                                        }
                                        cx.notify();
                                    })
                                }),
                        )
                        // `td.cds--table-expand[data-previous-value=collapsed] {
                        // border-block-end: 1px solid transparent }`.
                        .when(is_expanded, |d| {
                            d.child(
                                div()
                                    .absolute()
                                    .left_0()
                                    .right_0()
                                    .bottom(px(-1.))
                                    .h(px(1.))
                                    .bg(look.background),
                            )
                        })
                        .into_any_element(),
                );
            }
            if self.selectable {
                let state = state.clone();
                let rid2 = rid.clone();
                cells.push(
                    cell_box_padded(
                        ColumnWidth::Fixed(select_w),
                        CellAlign::Start,
                        select_pl,
                        select_pr,
                    )
                    .items_center()
                    .child(
                        Checkbox::new(child_id(&id, format!("select-{rid}")))
                            .checked(is_selected)
                            .on_checked_change(move |on, _, cx| {
                                let rid = rid2.clone();
                                state.update(cx, |s, cx| {
                                    if on {
                                        s.selected.insert(rid);
                                    } else {
                                        s.selected.remove(&rid);
                                    }
                                    cx.notify();
                                })
                            }),
                    )
                    .into_any_element(),
                );
            }
            for (c, column) in self.columns.iter().enumerate() {
                let value = row.value(&column.key);
                let content = match &column.render {
                    Some(f) => f(&value, row),
                    None => value.into_any_element(),
                };
                let pl = if c == 0 {
                    first_pl
                } else {
                    table::CELL_PADDING_X
                };
                cells.push(
                    cell_box_padded(column.width, CellAlign::Start, pl, table::CELL_PADDING_X)
                        .child(content)
                        .into_any_element(),
                );
            }
            let hover_setter = {
                let state = state.clone();
                let rid = rid.clone();
                move |on: bool, _: &mut Window, cx: &mut App| {
                    state.update(cx, |s, cx| {
                        let next = if on {
                            Some(rid.clone())
                        } else if s.hovered_row.as_ref() == Some(&rid) {
                            None
                        } else {
                            s.hovered_row.clone()
                        };
                        if s.hovered_row != next {
                            s.hovered_row = next;
                            cx.notify();
                        }
                    })
                }
            };
            body = body.child(body_row(
                child_id(&id, format!("row-{rid}")),
                geometry,
                look,
                pos == 0,
                hover_setter.clone(),
                cells,
                window,
                cx,
            ));
            if is_expanded && let Some(render) = &self.expand {
                // `tr[data-child-row] td { padding-inline-start: 3.5rem;
                // border-block-end: 1px solid var(--border-subtle) }`, inner
                // `padding-block: 1rem 1.5rem`; it hovers with its parent
                // (`tr.cds--parent-row:hover + tr[data-child-row] td { background:
                // layer-hover; color: text-primary }`).
                let bg = if row_state.hovered {
                    look.background
                } else {
                    color(level.layer())
                };
                body = body.child(
                    div()
                        .id(child_id(&id, format!("child-{rid}")))
                        .w_full()
                        .pl(rem(child_indent(self.selectable)))
                        .pr(rem(table::CELL_PADDING_X))
                        .pt(tokens::SPACING_05)
                        .pb(tokens::SPACING_06)
                        .bg(bg)
                        .border_b_1()
                        .border_color(super::separator::line_color(level))
                        .type_style(TypeStyle::BodyCompact01)
                        .text_color(if row_state.hovered {
                            color(ColorRole::TextPrimary)
                        } else {
                            color(ColorRole::TextSecondary)
                        })
                        .on_hover(move |h, window, cx| hover_setter(*h, window, cx))
                        .child(render(row)),
                );
            }
        }

        // ── Toolbar ──
        let batch_open = !self.batch_actions.is_empty() && !selected.is_empty();
        let has_toolbar =
            self.search || self.toolbar_actions.is_some() || !self.batch_actions.is_empty();
        let toolbar = has_toolbar.then(|| {
            let target = if batch_open { 1f32 } else { 0. };
            let progress = motion::animate((id.clone(), "batch"), target, BATCH_MOTION, window, cx);
            let slide = if motion::reduced_motion(cx) {
                target
            } else {
                progress
            };
            let content = div()
                .absolute()
                .left_0()
                .right_0()
                .top(rem(TOOLBAR_HEIGHT * slide))
                .h(rem(TOOLBAR_HEIGHT))
                .flex()
                .flex_row()
                .justify_end()
                .when(self.search, |d| {
                    d.child(toolbar_search(
                        child_id(&id, "search"),
                        input.clone(),
                        self.text.filter_placeholder.clone(),
                        self.text.clear_search.clone(),
                        density,
                        level,
                        window,
                        cx,
                    ))
                })
                .when_some(self.toolbar_actions, |d, actions| {
                    d.child(div().flex().flex_row().flex_none().h_full().child(actions))
                });
            let hovered_action =
                window.use_keyed_state((id.clone(), "batch-hover"), cx, |_, _| None::<usize>);
            let hovered_index = *hovered_action.read(cx);
            let n_actions = self.batch_actions.len();
            let bar = (progress > 0.).then(|| {
                let (top, visible) = batch_reveal(slide, TOOLBAR_HEIGHT);
                let ids: Vec<SharedString> = self
                    .rows
                    .iter()
                    .filter(|r| selected.contains(&r.id))
                    .map(|r| r.id.clone())
                    .collect();
                let mut list = div().flex().flex_row().h_full();
                for (i, action) in self.batch_actions.iter().enumerate() {
                    let f = action.on_click.clone();
                    let ids = ids.clone();
                    list = list.child(batch_button(
                        child_id(&id, format!("batch-{i}")),
                        action.label.clone(),
                        action.icon,
                        None,
                        hovered_action.clone(),
                        i,
                        Rc::new(move |window, cx| f(&ids, window, cx)),
                        window,
                        cx,
                    ));
                }
                let cancel_hidden =
                    hovered_index.is_some_and(|h| h == n_actions || h + 1 == n_actions);
                let state = state.clone();
                list = list.child(batch_button(
                    child_id(&id, "batch-cancel"),
                    self.text.cancel.clone(),
                    None,
                    Some(cancel_hidden),
                    hovered_action.clone(),
                    n_actions,
                    Rc::new(move |_, cx| {
                        state.update(cx, |s, cx| {
                            s.selected.clear();
                            cx.notify();
                        })
                    }),
                    window,
                    cx,
                ));
                div()
                    .absolute()
                    .left_0()
                    .right_0()
                    .top(rem(top))
                    .h(rem(visible))
                    .overflow_hidden()
                    .opacity(progress)
                    .child(
                        div()
                            .h(rem(TOOLBAR_HEIGHT))
                            .w_full()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .bg(color(ColorRole::BackgroundBrand))
                            // `.cds--batch-summary { padding: 0 1rem; color: text-on-color }`.
                            .child(
                                div()
                                    .px(rem(16.))
                                    .type_style(TypeStyle::BodyCompact01)
                                    .text_color(color(ColorRole::TextOnColor))
                                    .whitespace_nowrap()
                                    .child((self.text.items_selected)(selected.len())),
                            )
                            .child(list),
                    )
            });
            // `.cds--table-toolbar { background-color: var(--layer);
            // block-size: var(--layout-size-height-local) }`.
            div()
                .relative()
                .w_full()
                .h(rem(TOOLBAR_HEIGHT))
                .overflow_hidden()
                .bg(color(level.layer()))
                .when(!batch_open, |d| d.child(content))
                .children(bar)
        });

        // ── Pagination ──
        let pagination = shows_pagination(order.len(), page_size, prop_page_size).then(|| {
            let s1 = state.clone();
            let s2 = state.clone();
            Pagination::new(
                child_id(&id, format!("pagination-{prop_page_size}")),
                order.len(),
            )
            .page_size(page_size)
            .page_sizes([5, 10, 20, 50])
            .page(page)
            .on_page_change(move |p, _, cx| {
                s1.update(cx, |s, cx| {
                    s.page = p;
                    cx.notify();
                })
            })
            .on_page_size_change(move |size, _, cx| {
                s2.update(cx, |s, cx| {
                    s.page_size = size;
                    s.page = 1;
                    cx.notify();
                })
            })
        });

        container()
            .when(self.title.is_some() || self.description.is_some(), |d| {
                d.child(container_header(self.title, self.description, level))
            })
            .children(toolbar)
            .child(div().flex().flex_col().w_full().child(header).child(body))
            .children(pagination)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows() -> Vec<DataTableRow> {
        (1..=12)
            .map(|i| {
                DataTableRow::new(i.to_string())
                    .cell("name", format!("vm-{}", 100 + i))
                    .cell("status", ["Running", "Stopped", "Failed"][i % 3])
                    .cell("cpu", format!("{}%", (i * 7) % 100))
            })
            .collect()
    }

    #[test]
    fn sort_cycles_like_carbon() {
        use SortDirection::*;
        assert_eq!(None.next(true), Ascending);
        assert_eq!(Ascending.next(true), Descending);
        assert_eq!(Descending.next(true), None);
        assert_eq!(Descending.next(false), Ascending);
    }

    #[test]
    fn compare_numbers_then_text() {
        assert_eq!(compare_values("9", "10"), Ordering::Less);
        assert_eq!(compare_values("apple", "Banana"), Ordering::Less);
        assert_eq!(compare_values("b", "A"), Ordering::Greater);
    }

    #[test]
    fn filter_and_sort() {
        let rows = rows();
        let keys: Vec<SharedString> = vec!["name".into(), "status".into(), "cpu".into()];
        // Case-insensitive contains.
        let failed = visible_rows(&rows, &keys, " FAIL ", Option::None);
        assert_eq!(failed.len(), 4);
        let asc = visible_rows(
            &rows,
            &keys,
            "",
            Some(("name".into(), SortDirection::Descending)),
        );
        assert_eq!(rows[asc[0]].get("name"), "vm-112");
        let none = visible_rows(&rows, &keys, "", Some(("name".into(), SortDirection::None)));
        assert_eq!(none, (0..12).collect::<Vec<_>>());
    }

    #[test]
    fn selection_and_pagination_rules() {
        assert_eq!(select_all_state(0, 5), (false, false));
        assert_eq!(select_all_state(2, 5), (false, true));
        assert_eq!(select_all_state(5, 5), (true, false));
        assert_eq!(select_all_state(0, 0), (false, false));
        // `rs.length > ps || pageSize < rs.length`.
        assert!(shows_pagination(12, 5, 5));
        assert!(!shows_pagination(3, 10, 10));
    }

    #[test]
    fn geometry_matches_bundle() {
        // `.cds--data-table--sort th { block-size: 3rem }` regardless of density.
        assert_eq!(sort_header_height(TableSize::Lg), 48.);
        assert_eq!(sort_header_height(TableSize::Xs), 24.);
        assert_eq!(search_height(Density::Compact), 32.);
        assert_eq!(search_height(Density::Comfortable), 48.);
        assert_eq!(leading_columns(false, true), (0., (40., 16., 4.), 16.));
        assert_eq!(leading_columns(true, true), (40., (40., 6., 6.), 8.));
        assert_eq!(child_indent(false), 56.);
        assert_eq!(child_indent(true), 88.);
        assert_eq!(
            sort_icon_rotation(SortDirection::Descending),
            std::f32::consts::PI
        );
        assert_eq!(expand_rotation(false), std::f32::consts::FRAC_PI_2);
        // Batch bar: hidden below the toolbar at 0, fully shown at 1; the clip
        // (300%) reveals the whole bar from a third of the way.
        assert_eq!(batch_reveal(0., 48.), (48., 0.));
        assert_eq!(batch_reveal(1., 48.), (0., 48.));
        assert_eq!(batch_reveal(0.5, 48.), (24., 48.));
        assert_eq!(BATCH_MOTION.speed.millis(), 110);
    }
}
