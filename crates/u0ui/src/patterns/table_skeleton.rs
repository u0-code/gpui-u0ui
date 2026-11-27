//! `DataTableSkeleton` (`.cds--data-table.cds--skeleton`): the stand-in for
//! a table whose data has not arrived, with neither title nor toolbar
//! (`showHeader: false, showToolbar: false`). The Filtering pattern puts it
//! next to its filter panel.
//!
//! - A header row on `layer-accent-01` and `row_count` body rows on
//!   `layer-01`, each [`crate::components::table::row_height`] tall (48px
//!   comfortable) with a 1px `border-subtle-01` rule under it.
//! - Each cell holds a 4rem × 1rem bar, 1rem from the start
//!   (`th span, td span { display: block; block-size: 1rem; inline-size: 4rem }`,
//!   `th { padding-inline-start: 1rem }`); body bars are the shimmering
//!   [`Skeleton`] (`skeleton-background`, stopped under reduced motion), header
//!   bars `skeleton-element` (`th span { background: var(--skeleton-element) }`).
use crate::{
    components::{
        Skeleton, SkeletonLength,
        table::{TableSize, row_height},
    },
    density,
    theme::color,
    tokens::{ColorRole, REM_PX},
};
use gpui_kit::{App, IntoElement, ParentElement, RenderOnce, Styled, Window, div, px, rems};

/// The bar in each cell, in rem (width, height).
pub const BAR: (f32, f32) = (4., 1.);
/// `.cds--data-table.cds--skeleton th { padding-inline-start: 1rem }`, in rem.
pub const CELL_PADDING: f32 = 1.;

/// The skeleton's colors.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TableSkeletonLook {
    /// `.cds--data-table th { background-color: var(--layer-accent) }`.
    pub header: gpui_kit::Hsla,
    /// `.cds--data-table tbody tr { background-color: var(--layer) }`.
    pub row: gpui_kit::Hsla,
    /// `.cds--data-table td { border-bottom: 1px solid var(--border-subtle) }`.
    pub rule: gpui_kit::Hsla,
    /// `.cds--data-table.cds--skeleton th span { background: var(--skeleton-element) }`.
    pub header_bar: gpui_kit::Hsla,
}

impl TableSkeletonLook {
    /// The look on layer 01.
    pub fn current() -> Self {
        Self {
            header: color(ColorRole::LayerAccent01),
            row: color(ColorRole::Layer01),
            rule: color(ColorRole::BorderSubtle01),
            header_bar: color(ColorRole::SkeletonElement),
        }
    }
}

/// A data-table skeleton. See the module docs.
#[derive(IntoElement)]
pub struct DataTableSkeleton {
    columns: usize,
    rows: usize,
    header: bool,
}

impl DataTableSkeleton {
    /// `columnCount` × `rowCount` (Carbon's defaults are 5 × 5).
    pub fn new(columns: usize, rows: usize) -> Self {
        Self {
            columns: columns.max(1),
            rows,
            header: true,
        }
    }

    /// Draw the column-header row (`showHeader` is the title, not this row).
    pub fn header(mut self, header: bool) -> Self {
        self.header = header;
        self
    }
}

impl RenderOnce for DataTableSkeleton {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let look = TableSkeletonLook::current();
        let height = px(row_height(TableSize::Lg, density::current()));
        let bar = |head: bool| {
            if head {
                div()
                    .w(rems(BAR.0))
                    .h(rems(BAR.1))
                    .bg(look.header_bar)
                    .into_any_element()
            } else {
                Skeleton::block()
                    .width(SkeletonLength::Rem(BAR.0))
                    .height(SkeletonLength::Px(BAR.1 * REM_PX))
                    .into_any_element()
            }
        };
        let row = |head: bool| {
            div()
                .flex()
                .flex_row()
                .flex_none()
                .w_full()
                .h(height)
                .bg(if head { look.header } else { look.row })
                .border_b_1()
                .border_color(look.rule)
                .children((0..self.columns).map(|_| {
                    div()
                        .flex()
                        .flex_1()
                        .min_w_0()
                        .items_center()
                        .pl(rems(CELL_PADDING))
                        .overflow_hidden()
                        .child(bar(head))
                }))
        };
        div()
            .flex()
            .flex_col()
            .w_full()
            .overflow_hidden()
            .children(self.header.then(|| row(true)))
            .children((0..self.rows).map(|_| row(false)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry_matches_the_css() {
        assert_eq!(BAR.0 * REM_PX, 64.);
        assert_eq!(BAR.1 * REM_PX, 16.);
        assert_eq!(CELL_PADDING * REM_PX, 16.);
        assert_eq!(
            row_height(TableSize::Lg, density::Density::Comfortable),
            48.
        );
    }

    #[test]
    fn colors_come_from_the_css_roles() {
        let look = TableSkeletonLook::current();
        assert_eq!(look.header, color(ColorRole::LayerAccent01));
        assert_eq!(look.row, color(ColorRole::Layer01));
        assert_eq!(look.rule, color(ColorRole::BorderSubtle01));
        assert_eq!(look.header_bar, color(ColorRole::SkeletonElement));
    }
}
