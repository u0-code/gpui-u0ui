//! Table: shadcn table primitives drawn as Carbon's `.cds--data-table`
//! (`Table`, `TableHeader` = `<thead>`, `TableHead` = `<th>`, `TableBody`,
//! `TableRow`, `TableCell`, `TableFooter` = `.cu-tfoot`, `TableCaption` =
//! `.cu-caption`), with Carbon's `TableContainer` title/description.
//!
//! This module is also the **shared row/cell geometry** of
//! [`crate::components::data_table`]: row heights per size × density, cell and
//! header-label paddings, the per-state body row colors ([`RowLook`]) and the
//! row/cell render helpers.
//!
//! - **Why not Kit's `table`**: Kit's `Table` is a virtualized grid driven by a
//!   `TableDelegate` with its own fixed row height, header chrome, column
//!   resize handles and selection colors from Kit's theme. Carbon's rows need
//!   per-state top/bottom border colors (collapsed borders, zebra, selected
//!   first/last rows), size-dependent label paddings that can grow the header,
//!   and arbitrary elements (tags, checkboxes) in cells. The tables here are
//!   small, so rows are plain flex rows; nothing is virtualized.
//! - **Columns** ([`ColumnWidth`], set on the [`TableHead`]): by default sized
//!   from content like an HTML table's automatic layout. GPUI has no table
//!   layout, so [`Table`] measures each auto column's widest cell (max-content
//!   width, plus padding) while laying out and lays the columns out in
//!   proportion to those widths; content does not wrap. A frame laid out with
//!   widths that the measurement then changes is not shown ([`paints_frame`]):
//!   the table appears with its measured columns, with no visible jump. A column
//!   can instead take a flex weight or a fixed width. Every row splits the
//!   width identically ([`WEIGHT_BASIS`]), so header and body cells align.
//! - **Density**: `[data-density="compact"] .cds--data-table tbody tr, … thead tr
//!   { block-size: var(--density-row-md) }` (specificity 0,2,2) beats the size
//!   classes (`.cds--data-table--xs tbody tr`, 0,1,2), so in compact density
//!   every size is 32px tall and body cells lose their block padding
//!   (`[data-density="compact"] .cds--data-table td { padding-block: 0 }`).
//!   Header labels keep their size padding (`.cds--table-header-label` is not
//!   reset), so an `xl` header still grows to 16 + 18 + 16px. Comfortable
//!   density uses Carbon's size classes (24/32/40/48/64px).
//! - **Borders**: `border-collapse: collapse` resolves a shared row edge to the
//!   upper cell's bottom border (CSS 2.1 §17.6.2.1: "further to the top wins"),
//!   so each row draws its own bottom border and only the first row its top.
//! - **Motion**: `.cds--data-table tbody tr { transition: background-color 70ms
//!   cubic-bezier(0, 0, 0.38, 0.9) }` → [`Motion::HOVER`].
use crate::{
    density::{self, Density},
    motion::{self, Motion},
    theme::{LayerLevel, color},
    tokens::{self, ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    AnyElement, App, Bounds, ContentMask, Div, Element, ElementId, GlobalElementId, Hsla,
    InspectorElementId, InteractiveElement as _, IntoElement, LayoutId, ParentElement, Pixels,
    RenderOnce, SharedString, Stateful, StatefulInteractiveElement as _, Styled, Window, canvas,
    div, prelude::FluentBuilder as _, px, rems,
};
use std::{cell::RefCell, rc::Rc};

/// `index.d.ts` `TableProps.size` (Carbon `size`, default `lg`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TableSize {
    /// 24px rows.
    Xs,
    /// 32px rows.
    Sm,
    /// 40px rows.
    Md,
    /// 48px rows (the default).
    #[default]
    Lg,
    /// 64px rows, top-aligned.
    Xl,
}

impl TableSize {
    /// Every size.
    pub const ALL: [TableSize; 5] = [Self::Xs, Self::Sm, Self::Md, Self::Lg, Self::Xl];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Xs => "xs",
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
            Self::Xl => "xl",
        }
    }
}

/// Row height (px at 100%) of header and body rows.
///
/// Comfortable: `.cds--data-table tr { block-size: 3rem }` and
/// `.cds--data-table--{xs,sm,md,xl} … tr { block-size: 1.5/2/2.5/4rem }`.
/// Compact: `[data-density="compact"] .cds--data-table tbody tr, … thead tr
/// { block-size: var(--density-row-md) }` wins over every size class.
pub fn row_height(size: TableSize, density: Density) -> f32 {
    match density {
        Density::Compact => tokens::DENSITY_ROW_MD.px_value(),
        Density::Comfortable => match size {
            TableSize::Xs => tokens::SIZE_XS.px_value(),
            TableSize::Sm => tokens::SIZE_SM.px_value(),
            TableSize::Md => tokens::SIZE_MD.px_value(),
            TableSize::Lg => tokens::SIZE_LG.px_value(),
            TableSize::Xl => tokens::SIZE_XL.px_value(),
        },
    }
}

/// `.cds--table-header-label` block padding (top, bottom) in px:
/// xs `0.125rem`, sm/md `0.4375rem`, xl `1rem`, lg none. Not reset by density.
pub fn header_label_padding(size: TableSize) -> (f32, f32) {
    match size {
        TableSize::Xs => (2., 2.),
        TableSize::Sm | TableSize::Md => (7., 7.),
        TableSize::Lg => (0., 0.),
        TableSize::Xl => (16., 16.),
    }
}

/// Body cell block padding (top, bottom) in px: `.cds--data-table--xs td
/// { padding-block: 0.125rem }`, `--md td { 0.4375rem 0.375rem }`, `--xl td
/// { 1rem }`; compact density resets them (`[data-density="compact"]
/// .cds--data-table td { padding-block: 0 }`).
pub fn body_padding(size: TableSize, density: Density) -> (f32, f32) {
    if density == Density::Compact {
        return (0., 0.);
    }
    match size {
        TableSize::Xs => (2., 2.),
        TableSize::Md => (7., 6.),
        TableSize::Xl => (16., 16.),
        TableSize::Sm | TableSize::Lg => (0., 0.),
    }
}

/// `.cds--data-table--xl th, .cds--data-table--xl td { vertical-align: top }`
/// (otherwise `vertical-align: middle`).
pub fn top_aligned(size: TableSize) -> bool {
    size == TableSize::Xl
}

/// Cell inline padding: `.cds--data-table th, td { padding-inline: 1rem 1rem }`.
pub const CELL_PADDING_X: f32 = 16.;

/// Footer cell height: `.cu-tfoot td { height: var(--size-lg) }` (not density-aware).
pub fn footer_height() -> f32 {
    tokens::SIZE_LG.px_value()
}

/// The geometry of one table (size × density).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Geometry {
    /// Row height.
    pub row: f32,
    /// Header label padding (top, bottom).
    pub label_padding: (f32, f32),
    /// Body cell padding (top, bottom).
    pub body_padding: (f32, f32),
    /// Cells align to the top (`xl`).
    pub top_aligned: bool,
}

impl Geometry {
    /// The geometry for `size` in `density`.
    pub fn of(size: TableSize, density: Density) -> Self {
        Self {
            row: row_height(size, density),
            label_padding: header_label_padding(size),
            body_padding: body_padding(size, density),
            top_aligned: top_aligned(size),
        }
    }
}

/// The state of a body row.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RowState {
    /// `useZebraStyles`.
    pub zebra: bool,
    /// 0-based index in the body (`nth-child(odd)` is index 0, 2, …).
    pub index: usize,
    /// The first row of the body.
    pub first: bool,
    /// The last row of the body.
    pub last: bool,
    /// Pointer over the row.
    pub hovered: bool,
    /// `.cds--data-table--selected`.
    pub selected: bool,
}

/// Colors of a body row.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RowLook {
    /// Row background.
    pub background: Hsla,
    /// The 1px top border, drawn only on the first row (collapsed borders).
    pub border_top: Hsla,
    /// The 1px bottom border.
    pub border_bottom: Hsla,
    /// Cell text.
    pub text: Hsla,
}

impl RowLook {
    /// The colors for `state` in a table on `level`.
    pub fn of(state: RowState, level: LayerLevel) -> Self {
        let c = |role: ColorRole| color(role);
        let layer = c(level.layer());
        if state.selected {
            if state.hovered {
                // `.cds--data-table tbody .cds--data-table--selected:hover { background-color:
                // layer-selected-hover }` and `… :hover td { border-block: layer-selected-hover }`.
                let bg = c(level.layer_selected_hover());
                return Self {
                    background: bg,
                    border_top: bg,
                    border_bottom: bg,
                    text: c(ColorRole::TextPrimary),
                };
            }
            // `tr.cds--data-table--selected { background-color: layer-selected }`,
            // `… td { border-block-end: layer-active; border-block-start: layer-selected }`,
            // `:first-of-type td { border-block-start: border-subtle-selected }`,
            // `:last-of-type td { border-block-end: layer-selected }`.
            let bg = c(level.layer_selected());
            return Self {
                background: bg,
                border_top: if state.first {
                    c(level.border_subtle_selected())
                } else {
                    bg
                },
                border_bottom: if state.last {
                    bg
                } else {
                    c(level.layer_active())
                },
                text: c(ColorRole::TextPrimary),
            };
        }
        if state.hovered {
            // `.cds--data-table tbody tr:hover { background-color: layer-hover }`,
            // `tr:hover td { border-block: 1px solid layer-hover; color: text-primary }`.
            let bg = c(level.layer_hover());
            return Self {
                background: bg,
                border_top: bg,
                border_bottom: bg,
                text: c(ColorRole::TextPrimary),
            };
        }
        let text = c(ColorRole::TextSecondary);
        if state.zebra {
            if state.index % 2 == 1 {
                // `.cds--data-table--zebra tbody tr:nth-child(even) { background-color:
                // layer-accent }`, `td { border-block: layer-accent }`.
                let accent = c(level.layer_accent());
                return Self {
                    background: accent,
                    border_top: accent,
                    border_bottom: accent,
                    text,
                };
            }
            // `tr:nth-child(odd) td { border-block-end: 1px solid var(--layer) }`.
            return Self {
                background: layer,
                border_top: layer,
                border_bottom: layer,
                text,
            };
        }
        // `.cds--data-table td { border-block-end: 1px solid border-subtle-01;
        // border-block-start: 1px solid var(--layer); color: text-secondary }`
        // (`.cds--layer-two` → border-subtle-02, `-three` → -03).
        Self {
            background: layer,
            border_top: layer,
            border_bottom: c(level.border_subtle()),
            text,
        }
    }
}

/// A column's width.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ColumnWidth {
    /// A share of the free width (flex grow; default 1).
    Weight(f32),
    /// A fixed width in px at 100%.
    Fixed(f32),
    /// Sized from content like an HTML table's automatic layout (the
    /// [`TableHead`] default): [`Table`] measures each column's widest cell
    /// and splits the width in proportion ([`auto_column_flex`]). Elsewhere
    /// it behaves as `Weight(1.)`.
    Auto,
}

/// The flex basis (px) of `Weight` columns: at least any cell's inline
/// padding (16 + 16), so padding never changes a cell's hypothetical size
/// and every row (header, body, footer) splits the free width identically,
/// keeping columns aligned even when their cells' paddings differ.
pub const WEIGHT_BASIS: f32 = 2. * CELL_PADDING_X;

/// The flex `(basis px, grow)` of an auto column whose widest cell is
/// `content` px wide (padding included): `basis = grow = content`, so the
/// columns share the table width in proportion to their content (CSS 2.1
/// §17.5.2.2 automatic layout gives each column its max-content width plus a
/// share of the extra proportional to it), and shrink alike when it is short.
pub fn auto_column_flex(content: f32) -> (f32, f32) {
    let content = content.max(WEIGHT_BASIS);
    (content, content)
}

impl Default for ColumnWidth {
    fn default() -> Self {
        Self::Weight(1.)
    }
}

/// Horizontal alignment of a cell (`th[align]`, `td[align]`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CellAlign {
    /// `text-align: start` (default).
    #[default]
    Start,
    /// `align=center`.
    Center,
    /// `align=right`.
    End,
}

pub(crate) fn rem(px_value: f32) -> gpui_kit::Rems {
    rems(px_value / tokens::REM_PX)
}

/// A cell box of `width` with Carbon's 16px inline padding.
pub(crate) fn cell_box(width: ColumnWidth, align: CellAlign) -> Div {
    cell_box_padded(width, align, CELL_PADDING_X, CELL_PADDING_X)
}

/// The max-content widths (px, padding included) of a [`Table`]'s auto
/// columns: collected from every cell while painting a frame, adopted for the
/// next layout.
#[derive(Default)]
struct ColumnMeasure {
    settled: RefCell<Vec<f32>>,
    frame: RefCell<Vec<f32>>,
    /// This frame was laid out with stale widths (set by [`Self::finish`]).
    stale: std::cell::Cell<bool>,
}

impl ColumnMeasure {
    fn get(&self, column: usize) -> Option<f32> {
        self.settled
            .borrow()
            .get(column)
            .copied()
            .filter(|w| *w > 0.)
    }

    fn report(&self, column: usize, width: f32) {
        let mut frame = self.frame.borrow_mut();
        if frame.len() <= column {
            frame.resize(column + 1, 0.);
        }
        frame[column] = frame[column].max(width);
    }

    /// Adopt this frame's widths; true when they changed.
    fn finish(&self) -> bool {
        let frame = std::mem::take(&mut *self.frame.borrow_mut());
        let changed = widths_changed(&self.settled.borrow(), &frame);
        if changed {
            *self.settled.borrow_mut() = frame;
        }
        self.stale.set(changed);
        changed
    }
}

/// Whether a frame is painted: not when its columns were laid out with
/// widths that this frame's measurement changed (it is laid out again at
/// once, so the table shows up a frame later with its final columns).
pub fn paints_frame(stale: bool) -> bool {
    !stale
}

/// Lays out and prepaints the table as usual (the cells measure themselves
/// in prepaint), then paints it only when the columns were already settled
/// ([`paints_frame`]); a stale frame is painted under an empty content mask.
struct Measured {
    child: AnyElement,
    measure: Rc<ColumnMeasure>,
}

impl Element for Measured {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        (self.child.request_layout(window, cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        self.child.prepaint(window, cx);
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        if paints_frame(self.measure.stale.get()) {
            self.child.paint(window, cx);
        } else {
            let hidden = ContentMask {
                bounds: Bounds::default(),
            };
            window.with_content_mask(Some(hidden), |window| self.child.paint(window, cx));
        }
    }
}

/// Whether newly measured column widths differ from the settled ones by
/// more than half a pixel (so subpixel jitter never relayouts).
pub fn widths_changed(settled: &[f32], measured: &[f32]) -> bool {
    settled.len() != measured.len()
        || settled
            .iter()
            .zip(measured)
            .any(|(a, b)| (a - b).abs() > 0.5)
}

/// An auto column's cell content at its max-content width (no wrapping),
/// reporting that width plus the cell's 16px + 16px padding to `measure`.
fn measured(content: AnyElement, column: usize, measure: Rc<ColumnMeasure>) -> Div {
    div()
        .relative()
        .flex_none()
        .whitespace_nowrap()
        .child(content)
        .child(
            canvas(
                move |bounds, _, _| {
                    measure.report(column, f32::from(bounds.size.width) + 2. * CELL_PADDING_X)
                },
                |_, _, _, _| {},
            )
            .absolute()
            .top_0()
            .left_0()
            .size_full(),
        )
}

/// A cell box of `width` with explicit inline padding (start, end) in px.
pub(crate) fn cell_box_padded(width: ColumnWidth, align: CellAlign, start: f32, end: f32) -> Div {
    debug_assert!(start + end <= WEIGHT_BASIS);
    let d = cell_div(align, start, end);
    match width {
        ColumnWidth::Fixed(w) => d.flex_none().w(rem(w)),
        ColumnWidth::Weight(w) => flex_share(d, rem(WEIGHT_BASIS), w),
        ColumnWidth::Auto => flex_share(d, rem(WEIGHT_BASIS), 1.),
    }
}

/// A 16px-padded cell of an auto column whose widest cell is `content` px
/// wide (`None` before the first measurement: an equal share).
fn auto_cell_box(content: Option<f32>, align: CellAlign) -> Div {
    match content {
        Some(content) => {
            // Measured in window pixels, so the basis is in px (not rem).
            let (basis, grow) = auto_column_flex(content);
            flex_share(
                cell_div(align, CELL_PADDING_X, CELL_PADDING_X).overflow_hidden(),
                px(basis),
                grow,
            )
        }
        None => cell_box(ColumnWidth::Auto, align),
    }
}

fn cell_div(align: CellAlign, start: f32, end: f32) -> Div {
    div()
        .flex()
        .flex_row()
        .min_w_0()
        .pl(rem(start))
        .pr(rem(end))
        .map(|d| match align {
            CellAlign::Start => d.justify_start(),
            CellAlign::Center => d.justify_center(),
            CellAlign::End => d.justify_end(),
        })
}

/// `flex: grow 1 basis`.
fn flex_share(d: Div, basis: impl Into<gpui_kit::Length>, grow: f32) -> Div {
    d.flex_basis(basis).map(|mut d| {
        d.style().flex_grow = Some(grow);
        d.style().flex_shrink = Some(1.);
        d
    })
}

/// The header row (`<thead><tr>`): `layer-accent` background,
/// `heading-compact-01` in `text-primary`, at least one row tall; each cell's
/// label gets the size's block padding.
pub(crate) fn header_row(geometry: Geometry, level: LayerLevel, cells: Vec<AnyElement>) -> Div {
    div()
        .flex()
        .flex_row()
        .w_full()
        .min_h(rem(geometry.row))
        // `.cds--data-table thead { background-color: var(--layer-accent) }`;
        // `th { background-color: layer-accent; color: text-primary }`.
        .bg(color(level.layer_accent()))
        .text_color(color(ColorRole::TextPrimary))
        .type_style(TypeStyle::HeadingCompact01)
        .map(|d| {
            if geometry.top_aligned {
                d.items_start()
            } else {
                d.items_center()
            }
        })
        .children(cells)
}

/// Wrap a header label with `.cds--table-header-label` padding.
pub(crate) fn header_label(geometry: Geometry, label: impl IntoElement) -> Div {
    div()
        .pt(rem(geometry.label_padding.0))
        .pb(rem(geometry.label_padding.1))
        .child(label)
}

/// A body row: background/borders from `look` (background animated with
/// [`Motion::HOVER`]), hover tracked into `hovered`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn body_row(
    id: ElementId,
    geometry: Geometry,
    look: RowLook,
    first: bool,
    on_hover: impl Fn(bool, &mut Window, &mut App) + 'static,
    cells: Vec<AnyElement>,
    window: &mut Window,
    cx: &mut App,
) -> Stateful<Div> {
    let bg = motion::animate_color(
        (id.clone(), "bg"),
        look.background,
        Motion::HOVER,
        window,
        cx,
    );
    let bottom = motion::animate_color(
        (id.clone(), "bb"),
        look.border_bottom,
        Motion::HOVER,
        window,
        cx,
    );
    let top = motion::animate_color(
        (id.clone(), "bt"),
        look.border_top,
        Motion::HOVER,
        window,
        cx,
    );
    div()
        .id(id)
        .flex()
        .flex_row()
        .w_full()
        .min_h(rem(geometry.row))
        .bg(bg)
        .border_b_1()
        .border_color(bottom)
        // The first row's top edge (`border-block-start`; later rows' top
        // edges collapse into the bottom border of the row above).
        .when(first, |d| {
            d.relative().child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .h(px(1.))
                    .bg(top),
            )
        })
        .text_color(look.text)
        .type_style(TypeStyle::BodyCompact01)
        .pt(rem(geometry.body_padding.0))
        .pb(rem(geometry.body_padding.1))
        .map(|d| {
            if geometry.top_aligned {
                d.items_start()
            } else {
                d.items_center()
            }
        })
        .on_hover(move |hovered, window, cx| on_hover(*hovered, window, cx))
        .children(cells)
}

/// Carbon `TableContainer` header: `.cds--data-table-header { background-color:
/// var(--layer); padding-block: 1rem 1.5rem; padding-inline: 1rem }` with a
/// `heading-03` title in `text-primary` and a `body-compact-01` description
/// in `text-secondary`.
pub(crate) fn container_header(
    title: Option<SharedString>,
    description: Option<SharedString>,
    level: LayerLevel,
) -> Div {
    div()
        .bg(color(level.layer()))
        .pt(tokens::SPACING_05)
        .pb(tokens::SPACING_06)
        .px(tokens::SPACING_05)
        .flex()
        .flex_col()
        .when_some(title, |d, t| {
            d.child(
                div()
                    .type_style(TypeStyle::Heading03)
                    .text_color(color(ColorRole::TextPrimary))
                    .child(t),
            )
        })
        .when_some(description, |d, t| {
            d.child(
                div()
                    .type_style(TypeStyle::BodyCompact01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(t),
            )
        })
}

/// `.cds--data-table-container { padding-block-start: 0.125rem }`.
pub(crate) fn container() -> Div {
    div().flex().flex_col().w_full().pt(tokens::SPACING_01)
}

/// One cell of a [`TableRow`]: a header cell ([`TableHead`]) or a data cell
/// ([`TableCell`]).
pub enum Cell {
    /// `<th>`.
    Head(TableHead),
    /// `<td>`.
    Data(TableCell),
}

impl From<TableHead> for Cell {
    fn from(h: TableHead) -> Self {
        Cell::Head(h)
    }
}

impl From<TableCell> for Cell {
    fn from(c: TableCell) -> Self {
        Cell::Data(c)
    }
}

/// `TableHead` (`<th>`, Carbon `TableHeader`): a column header.
pub struct TableHead {
    content: AnyElement,
    width: ColumnWidth,
    align: CellAlign,
}

impl TableHead {
    /// A header cell showing `content` (usually text).
    pub fn new(content: impl IntoElement) -> Self {
        Self {
            content: content.into_any_element(),
            width: ColumnWidth::Auto,
            align: CellAlign::Start,
        }
    }

    /// The column width (applies to the whole column; default
    /// [`ColumnWidth::Auto`], sized from content).
    pub fn width(mut self, width: ColumnWidth) -> Self {
        self.width = width;
        self
    }

    /// The column alignment (`align`).
    pub fn align(mut self, align: CellAlign) -> Self {
        self.align = align;
        self
    }
}

/// `TableCell` (`<td>`).
pub struct TableCell {
    content: AnyElement,
    align: Option<CellAlign>,
}

impl TableCell {
    /// A data cell showing `content`.
    pub fn new(content: impl IntoElement) -> Self {
        Self {
            content: content.into_any_element(),
            align: None,
        }
    }

    /// The alignment (defaults to the column's).
    pub fn align(mut self, align: CellAlign) -> Self {
        self.align = Some(align);
        self
    }
}

/// `TableRow` (`<tr>`).
#[derive(Default)]
pub struct TableRow {
    cells: Vec<Cell>,
    selected: bool,
}

impl TableRow {
    /// An empty row.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a cell.
    pub fn child(mut self, cell: impl Into<Cell>) -> Self {
        self.cells.push(cell.into());
        self
    }

    /// Add cells.
    pub fn children(mut self, cells: impl IntoIterator<Item = impl Into<Cell>>) -> Self {
        self.cells.extend(cells.into_iter().map(Into::into));
        self
    }

    /// Carbon `isSelected` (`.cds--data-table--selected`).
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

macro_rules! section {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Default)]
        pub struct $name {
            rows: Vec<TableRow>,
        }

        impl $name {
            /// An empty section.
            pub fn new() -> Self {
                Self::default()
            }

            /// Add a row.
            pub fn child(mut self, row: TableRow) -> Self {
                self.rows.push(row);
                self
            }

            /// Add rows.
            pub fn children(mut self, rows: impl IntoIterator<Item = TableRow>) -> Self {
                self.rows.extend(rows);
                self
            }
        }
    };
}

section!(
    /// `TableHeader` (`<thead>`, Carbon `TableHead`).
    TableHeader
);
section!(
    /// `TableBody` (`<tbody>`).
    TableBody
);
section!(
    /// `TableFooter` (`<tfoot class="cu-tfoot">`).
    TableFooter
);

/// `TableCaption` (`<caption class="cu-caption">`, shown below the table).
pub struct TableCaption {
    text: SharedString,
}

impl TableCaption {
    /// A caption.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self { text: text.into() }
    }
}

/// A table. See the module docs.
#[derive(IntoElement)]
pub struct Table {
    id: ElementId,
    title: Option<SharedString>,
    description: Option<SharedString>,
    size: TableSize,
    zebra: bool,
    level: LayerLevel,
    density: Option<Density>,
    header: Option<TableHeader>,
    body: Option<TableBody>,
    footer: Option<TableFooter>,
    caption: Option<TableCaption>,
}

impl Table {
    /// An empty `lg` table.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            title: None,
            description: None,
            size: TableSize::Lg,
            zebra: false,
            level: LayerLevel::One,
            density: None,
            header: None,
            body: None,
            footer: None,
            caption: None,
        }
    }

    /// `title` (wraps the table in a `TableContainer`).
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// `description` (wraps the table in a `TableContainer`).
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// `size`.
    pub fn size(mut self, size: TableSize) -> Self {
        self.size = size;
        self
    }

    /// `zebra`.
    pub fn zebra(mut self, zebra: bool) -> Self {
        self.zebra = zebra;
        self
    }

    /// The layer the table sits on (One = on `background`).
    pub fn layer(mut self, level: LayerLevel) -> Self {
        self.level = level;
        self
    }

    /// Override the global density.
    pub fn density(mut self, density: Density) -> Self {
        self.density = Some(density);
        self
    }

    /// The `<thead>`.
    pub fn header(mut self, header: TableHeader) -> Self {
        self.header = Some(header);
        self
    }

    /// The `<tbody>`.
    pub fn body(mut self, body: TableBody) -> Self {
        self.body = Some(body);
        self
    }

    /// The `<tfoot>`.
    pub fn footer(mut self, footer: TableFooter) -> Self {
        self.footer = Some(footer);
        self
    }

    /// The `<caption>`.
    pub fn caption(mut self, caption: TableCaption) -> Self {
        self.caption = Some(caption);
        self
    }
}

/// `.cu-caption { padding: spacing-03 spacing-05; font-size: 0.75rem; color: text-helper }`.
pub(crate) fn caption(text: SharedString) -> Div {
    div()
        .px(tokens::SPACING_05)
        .py(tokens::SPACING_03)
        .type_style(TypeStyle::Label01)
        .text_color(color(ColorRole::TextHelper))
        .child(text)
}

impl RenderOnce for Table {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let density = self.density.unwrap_or_else(density::current);
        let geometry = Geometry::of(self.size, density);
        let level = self.level;
        let hovered = window.use_keyed_state((self.id.clone(), "hover"), cx, |_, _| None::<usize>);
        let hovered_row = *hovered.read(cx);

        // Column widths and alignments come from the first header row; auto
        // columns take the widths measured in the previous frame.
        let measure = window
            .use_keyed_state((self.id.clone(), "columns"), cx, |_, _| {
                Rc::new(ColumnMeasure::default())
            })
            .read(cx)
            .clone();
        let mut columns: Vec<(ColumnWidth, CellAlign)> = Vec::new();
        let mut table = div().flex().flex_col().w_full();
        let cell = {
            let measure = measure.clone();
            move |i: usize, width: ColumnWidth, align: CellAlign, content: AnyElement| {
                if width != ColumnWidth::Auto {
                    return cell_box(width, align).child(content).into_any_element();
                }
                auto_cell_box(measure.get(i), align)
                    .child(measured(content, i, measure.clone()))
                    .into_any_element()
            }
        };

        if let Some(header) = self.header {
            for row in header.rows {
                let cells = row
                    .cells
                    .into_iter()
                    .enumerate()
                    .map(|(i, c)| {
                        let (content, width, align) = match c {
                            Cell::Head(h) => (h.content, h.width, h.align),
                            Cell::Data(d) => {
                                (d.content, ColumnWidth::Auto, d.align.unwrap_or_default())
                            }
                        };
                        if columns.len() <= i {
                            columns.push((width, align));
                        }
                        cell(
                            i,
                            width,
                            align,
                            header_label(geometry, content).into_any_element(),
                        )
                    })
                    .collect();
                table = table.child(header_row(geometry, level, cells));
            }
        }
        let column = |i: usize| {
            columns
                .get(i)
                .copied()
                .unwrap_or((ColumnWidth::Auto, CellAlign::Start))
        };

        let place = |cells: Vec<Cell>| -> Vec<AnyElement> {
            cells
                .into_iter()
                .enumerate()
                .map(|(i, c)| {
                    let (width, col_align) = column(i);
                    let (content, align) = match c {
                        Cell::Head(h) => (h.content, h.align),
                        Cell::Data(d) => (d.content, d.align.unwrap_or(col_align)),
                    };
                    cell(i, width, align, content)
                })
                .collect()
        };

        if let Some(body) = self.body {
            let count = body.rows.len();
            for (index, row) in body.rows.into_iter().enumerate() {
                let state = RowState {
                    zebra: self.zebra,
                    index,
                    first: index == 0,
                    last: index + 1 == count,
                    hovered: hovered_row == Some(index),
                    selected: row.selected,
                };
                let look = RowLook::of(state, level);
                let hovered = hovered.clone();
                table = table.child(body_row(
                    ElementId::NamedChild(
                        std::sync::Arc::new(self.id.clone()),
                        SharedString::from(format!("row-{index}")),
                    ),
                    geometry,
                    look,
                    index == 0,
                    move |on, _, cx| {
                        hovered.update(cx, |h, cx| {
                            let next = if on {
                                Some(index)
                            } else if *h == Some(index) {
                                None
                            } else {
                                *h
                            };
                            if *h != next {
                                *h = next;
                                cx.notify();
                            }
                        })
                    },
                    place(row.cells),
                    window,
                    cx,
                ));
            }
        }

        if let Some(footer) = self.footer {
            for row in footer.rows {
                // `.cu-tfoot td { height: size-lg; padding: 0 spacing-05;
                // background: layer-accent-01; font-weight: 600 }` plus
                // `.cds--data-table td { border-block-end: border-subtle-01;
                // color: text-secondary }`.
                table = table.child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .w_full()
                        .h(rem(footer_height()))
                        .bg(color(ColorRole::LayerAccent01))
                        .border_b_1()
                        .border_color(color(level.border_subtle()))
                        .type_style(TypeStyle::HeadingCompact01)
                        .text_color(color(ColorRole::TextSecondary))
                        .children(place(row.cells)),
                );
            }
        }

        // Prepainted after every cell: adopt this frame's measurements and lay
        // out again when they changed (content does not depend on the column
        // widths, so this settles after one extra frame, which is not shown).
        let finish = measure.clone();
        let table = table.child(
            canvas(
                move |_, window, _| {
                    let measure = &finish;
                    if measure.finish() {
                        window.on_next_frame(|window, _| window.refresh());
                    }
                },
                |_, _, _, _| {},
            )
            .w_full()
            .h(px(0.)),
        );
        let table = table.when_some(self.caption, |t, c| t.child(caption(c.text)));
        let table = if self.title.is_some() || self.description.is_some() {
            container()
                .child(container_header(self.title, self.description, level))
                .child(table)
                .into_any_element()
        } else {
            table.into_any_element()
        };
        Measured {
            child: table,
            measure,
        }
        .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    #[test]
    fn columns_size_from_content_and_rows_align() {
        // Auto layout: columns share the width in proportion to content.
        assert_eq!(auto_column_flex(120.), (120., 120.));
        assert_eq!(auto_column_flex(10.), (WEIGHT_BASIS, WEIGHT_BASIS));
        assert_eq!(TableHead::new("Invoice").width, ColumnWidth::Auto);
        // The weight basis covers every cell padding, so padding never
        // changes how a row splits its width.
        assert_eq!(WEIGHT_BASIS, 32.);
        let m = ColumnMeasure::default();
        m.report(0, 80.);
        m.report(0, 120.);
        m.report(1, 60.);
        assert!(m.finish());
        assert_eq!(m.get(0), Some(120.));
        m.report(0, 120.2);
        m.report(1, 60.);
        assert!(!m.finish(), "subpixel jitter does not relayout");
        assert!(widths_changed(&[120., 60.], &[120., 61.]));
    }

    #[test]
    fn a_frame_laid_out_with_stale_widths_is_not_painted() {
        let m = ColumnMeasure::default();
        // First frame: laid out with equal shares, then measured.
        m.report(0, 200.);
        m.report(1, 60.);
        assert!(m.finish());
        assert!(
            !paints_frame(m.stale.get()),
            "the equal-width frame flashes"
        );
        // Next frame: laid out with the measured widths, which hold.
        m.report(0, 200.);
        m.report(1, 60.);
        assert!(!m.finish());
        assert!(paints_frame(m.stale.get()));
    }

    #[test]
    fn row_heights_follow_size_and_compact_wins() {
        use TableSize::*;
        let c = Density::Comfortable;
        assert_eq!(
            [Xs, Sm, Md, Lg, Xl].map(|s| row_height(s, c)),
            [24., 32., 40., 48., 64.]
        );
        // `[data-density="compact"] .cds--data-table tbody tr` (0,2,2) beats
        // `.cds--data-table--xl tbody tr` (0,1,2).
        assert!(
            [Xs, Sm, Md, Lg, Xl]
                .iter()
                .all(|s| row_height(*s, Density::Compact) == 32.)
        );
        assert_eq!(body_padding(Xl, Density::Compact), (0., 0.));
        assert_eq!(body_padding(Md, c), (7., 6.));
        assert_eq!(body_padding(Xl, c), (16., 16.));
        assert_eq!(header_label_padding(Xl), (16., 16.));
        assert_eq!(header_label_padding(Sm), (7., 7.));
        assert_eq!(header_label_padding(Lg), (0., 0.));
        assert!(top_aligned(Xl) && !top_aligned(Lg));
        assert_eq!(footer_height(), 48.);
        assert_eq!(TableSize::default().name(), "lg");
    }

    #[test]
    fn row_colors_match_bundle() {
        crate::theme::tests_set(CarbonTheme::White);
        let one = LayerLevel::One;
        let base = RowState {
            index: 1,
            ..Default::default()
        };
        let plain = RowLook::of(base, one);
        assert_eq!(plain.background, color(ColorRole::Layer01));
        assert_eq!(plain.border_bottom, color(ColorRole::BorderSubtle01));
        assert_eq!(plain.text, color(ColorRole::TextSecondary));
        let hover = RowLook::of(
            RowState {
                hovered: true,
                ..base
            },
            one,
        );
        assert_eq!(hover.background, color(ColorRole::LayerHover01));
        assert_eq!(hover.border_bottom, color(ColorRole::LayerHover01));
        assert_eq!(hover.text, color(ColorRole::TextPrimary));
        // Zebra: even rows (index 1) layer-accent, odd rows layer with layer borders.
        let even = RowLook::of(
            RowState {
                zebra: true,
                ..base
            },
            one,
        );
        assert_eq!(even.background, color(ColorRole::LayerAccent01));
        assert_eq!(even.border_bottom, color(ColorRole::LayerAccent01));
        let odd = RowLook::of(
            RowState {
                zebra: true,
                index: 0,
                ..base
            },
            one,
        );
        assert_eq!(odd.background, color(ColorRole::Layer01));
        assert_eq!(odd.border_bottom, color(ColorRole::Layer01));
        // Selected: layer-selected, layer-active bottom; first row border-subtle-selected top.
        let sel = RowLook::of(
            RowState {
                selected: true,
                first: true,
                ..base
            },
            one,
        );
        assert_eq!(sel.background, color(ColorRole::LayerSelected01));
        assert_eq!(sel.border_bottom, color(ColorRole::LayerActive01));
        assert_eq!(sel.border_top, color(ColorRole::BorderSubtleSelected01));
        let last = RowLook::of(
            RowState {
                selected: true,
                last: true,
                ..base
            },
            one,
        );
        assert_eq!(last.border_bottom, color(ColorRole::LayerSelected01));
        let sel_hover = RowLook::of(
            RowState {
                selected: true,
                hovered: true,
                ..base
            },
            one,
        );
        assert_eq!(sel_hover.background, color(ColorRole::LayerSelectedHover01));
        // `.cds--layer-two .cds--data-table td { border-block-end: border-subtle-02 }`.
        assert_eq!(
            RowLook::of(base, LayerLevel::Two).border_bottom,
            color(ColorRole::BorderSubtle02)
        );
    }
}
