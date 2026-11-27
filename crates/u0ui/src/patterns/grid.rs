//! `.cu-pgrid`: the grid the pattern cards lay their examples out in, one
//! captioned example per cell.
use crate::tokens::{self, TokenLength};
use gpui_kit::{
    AnyElement, App, IntoElement, ParentElement, RenderOnce, Styled, Window, div,
    prelude::FluentBuilder as _,
};

/// `.cu-pgrid { gap: var(--spacing-06) var(--spacing-07) }`: between rows.
pub const ROW_GAP: TokenLength = tokens::SPACING_06;
/// … between columns.
pub const COLUMN_GAP: TokenLength = tokens::SPACING_07;

/// One cell: its element and whether it spans the whole row
/// (`style: { gridColumn: '1 / -1' }`).
struct Cell {
    element: AnyElement,
    full: bool,
}

/// Split cells into rows of `columns`: a full-width cell closes the open row
/// and takes one of its own. Returns, per row, the indices it holds.
pub fn rows(full: &[bool], columns: usize) -> Vec<Vec<usize>> {
    let columns = columns.max(1);
    let mut rows: Vec<Vec<usize>> = Vec::new();
    let mut open: Vec<usize> = Vec::new();
    for (i, &is_full) in full.iter().enumerate() {
        if is_full {
            if !open.is_empty() {
                rows.push(std::mem::take(&mut open));
            }
            rows.push(vec![i]);
        } else {
            open.push(i);
            if open.len() == columns {
                rows.push(std::mem::take(&mut open));
            }
        }
    }
    if !open.is_empty() {
        rows.push(open);
    }
    rows
}

/// The pattern grid (`.cu-pgrid--2`, `.cu-pgrid--3`: `repeat(n, minmax(0,
/// 1fr))`, or a custom `grid-template-columns` of fractions). Cells keep the
/// track width even when a row is short, as CSS grid does.
#[derive(IntoElement)]
pub struct PatternGrid {
    tracks: Vec<f32>,
    cells: Vec<Cell>,
    align_start: bool,
}

impl PatternGrid {
    /// A grid of `columns` equal tracks.
    pub fn new(columns: usize) -> Self {
        Self::tracks(vec![1.; columns.max(1)])
    }

    /// A grid whose tracks take these fractions (`1.3fr 1fr`).
    pub fn tracks(tracks: Vec<f32>) -> Self {
        Self {
            tracks: if tracks.is_empty() { vec![1.] } else { tracks },
            cells: Vec::new(),
            align_start: false,
        }
    }

    /// `align-items: start`: cells keep their own height instead of
    /// stretching to the row's.
    pub fn align_start(mut self, start: bool) -> Self {
        self.align_start = start;
        self
    }

    /// Append a cell taking one track.
    pub fn cell(mut self, element: impl IntoElement) -> Self {
        self.cells.push(Cell {
            element: element.into_any_element(),
            full: false,
        });
        self
    }

    /// Append a cell spanning every track (`gridColumn: '1 / -1'`).
    pub fn full(mut self, element: impl IntoElement) -> Self {
        self.cells.push(Cell {
            element: element.into_any_element(),
            full: true,
        });
        self
    }
}

/// A track taking `weight` of the row (`flex: <weight> 1 0`).
fn track(weight: f32) -> gpui_kit::Div {
    let mut d = div()
        .flex()
        .flex_col()
        .min_w_0()
        .flex_basis(gpui_kit::px(0.));
    d.style().flex_grow = Some(weight);
    d.style().flex_shrink = Some(1.);
    d
}

impl RenderOnce for PatternGrid {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let full: Vec<bool> = self.cells.iter().map(|c| c.full).collect();
        let layout = rows(&full, self.tracks.len());
        let mut cells: Vec<Option<AnyElement>> =
            self.cells.into_iter().map(|c| Some(c.element)).collect();
        let tracks = self.tracks;
        let align_start = self.align_start;
        div()
            .flex()
            .flex_col()
            .w_full()
            .gap(ROW_GAP)
            .children(layout.into_iter().map(|row| {
                let spans = row.len() == 1 && full[row[0]];
                let mut line = div()
                    .flex()
                    .flex_row()
                    .w_full()
                    .gap(COLUMN_GAP)
                    .when(align_start, |d| d.items_start());
                if spans {
                    line = line.child(track(1.).children(cells[row[0]].take()));
                } else {
                    for (slot, weight) in tracks.iter().enumerate() {
                        let element = row.get(slot).and_then(|&i| cells[i].take());
                        line = line.child(track(*weight).children(element));
                    }
                }
                line
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gaps_match_the_css() {
        assert_eq!((ROW_GAP.px_value(), COLUMN_GAP.px_value()), (24., 32.));
    }

    #[test]
    fn cells_fill_rows_and_full_cells_break_them() {
        assert_eq!(rows(&[false; 5], 2), vec![vec![0, 1], vec![2, 3], vec![4]]);
        assert_eq!(
            rows(&[false, false, true, true], 2),
            vec![vec![0, 1], vec![2], vec![3]]
        );
        assert_eq!(
            rows(&[false, true, false], 3),
            vec![vec![0], vec![1], vec![2]]
        );
    }
}
