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
//! - **Columns**: HTML tables size columns from content; here every column is a
//!   flex weight (default 1, equal widths) or a fixed width, set on its
//!   [`TableHead`] ([`ColumnWidth`]).
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
    AnyElement, App, Div, ElementId, Hsla, InteractiveElement as _, IntoElement, ParentElement,
    RenderOnce, SharedString, Stateful, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _, px, rems,
};

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
    let d = div()
        .flex()
        .flex_row()
        .min_w_0()
        .px(rem(CELL_PADDING_X))
        .map(|d| match align {
            CellAlign::Start => d.justify_start(),
            CellAlign::Center => d.justify_center(),
            CellAlign::End => d.justify_end(),
        });
    match width {
        ColumnWidth::Fixed(w) => d.flex_none().w(rem(w)),
        ColumnWidth::Weight(w) => d.flex_basis(px(0.)).map(|mut d| {
            d.style().flex_grow = Some(w);
            d.style().flex_shrink = Some(1.);
            d
        }),
    }
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
            width: ColumnWidth::default(),
            align: CellAlign::Start,
        }
    }

    /// The column width (applies to the whole column).
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

        // Column widths and alignments come from the first header row.
        let mut columns: Vec<(ColumnWidth, CellAlign)> = Vec::new();
        let mut table = div().flex().flex_col().w_full();

        if let Some(header) = self.header {
            for row in header.rows {
                let cells = row
                    .cells
                    .into_iter()
                    .enumerate()
                    .map(|(i, cell)| {
                        let (content, width, align) = match cell {
                            Cell::Head(h) => (h.content, h.width, h.align),
                            Cell::Data(d) => (
                                d.content,
                                ColumnWidth::default(),
                                d.align.unwrap_or_default(),
                            ),
                        };
                        if columns.len() <= i {
                            columns.push((width, align));
                        }
                        cell_box(width, align)
                            .child(header_label(geometry, content))
                            .into_any_element()
                    })
                    .collect();
                table = table.child(header_row(geometry, level, cells));
            }
        }
        let column = |i: usize| columns.get(i).copied().unwrap_or_default();

        let place = |cells: Vec<Cell>| -> Vec<AnyElement> {
            cells
                .into_iter()
                .enumerate()
                .map(|(i, cell)| {
                    let (width, col_align) = column(i);
                    let (content, align) = match cell {
                        Cell::Head(h) => (h.content, h.align),
                        Cell::Data(d) => (d.content, d.align.unwrap_or(col_align)),
                    };
                    cell_box(width, align).child(content).into_any_element()
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

        let table = table.when_some(self.caption, |t, c| t.child(caption(c.text)));
        if self.title.is_some() || self.description.is_some() {
            container()
                .child(container_header(self.title, self.description, level))
                .child(table)
                .into_any_element()
        } else {
            table.into_any_element()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

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
