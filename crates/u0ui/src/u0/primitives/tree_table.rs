//! TreeTable: a table whose first column expands (`TreeTable` in
//! `src/ghost/primitives.jsx`; Carbon `DataTable` expandable rows go one
//! level deep, this goes any depth). For process trees, storage breakdowns
//! and file lists.
//!
//! - **Behavior**: each branch's chevron button expands or collapses it
//!   (`aria-expanded`); rows start open when `open`.
//! - **Appearance** (`.g-ttable-*`): a `density-row-md` header on
//!   `layer-accent-01` (12px, 600); `density-row-md` rows on `layer-01` with a
//!   `border-subtle-01` rule and `layer-hover-01` hover; cells 12px padded,
//!   truncated; the first column indents 16px per level from 8px; a column
//!   is fixed (`width`) or shares the rest, aligned start/end.
use super::parts::{self, rem};
use super::tree::CHEVRON_MOTION;
use crate::{
    density,
    icons::{CarbonIcon, Icon},
    motion,
    theme::color,
    tokens::{ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    AnyElement, App, CursorStyle, ElementId, FontWeight, InteractiveElement as _, IntoElement,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _, px,
};
use std::{collections::HashSet, rc::Rc};

/// `textAlign`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CellAlign {
    /// Start (default).
    #[default]
    Start,
    /// `right`.
    End,
}

type CellRender = Rc<dyn Fn(&SharedString, &TreeTableRow) -> AnyElement>;

/// A column (`{ key, header, width, align, render }`).
#[derive(Clone)]
pub struct TreeTableColumn {
    /// The row field it shows.
    pub key: SharedString,
    /// Header text.
    pub header: SharedString,
    /// Fixed width in rems (`flex: 0 0 <width>`); `None` shares the rest.
    pub width: Option<f32>,
    /// Alignment.
    pub align: CellAlign,
    render: Option<CellRender>,
}

impl TreeTableColumn {
    /// A flexible, start-aligned column.
    pub fn new(key: impl Into<SharedString>, header: impl Into<SharedString>) -> Self {
        Self {
            key: key.into(),
            header: header.into(),
            width: None,
            align: CellAlign::Start,
            render: None,
        }
    }

    /// Fixed width in rems.
    pub fn width(mut self, rems: f32) -> Self {
        self.width = Some(rems);
        self
    }

    /// Alignment.
    pub fn align(mut self, align: CellAlign) -> Self {
        self.align = align;
        self
    }

    /// Custom cell content (`render(value, row)`).
    pub fn render(
        mut self,
        f: impl Fn(&SharedString, &TreeTableRow) -> AnyElement + 'static,
    ) -> Self {
        self.render = Some(Rc::new(f));
        self
    }
}

/// A row: its cells by column key, open state and children.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TreeTableRow {
    /// Stable id.
    pub id: SharedString,
    /// `(key, value)` cells.
    pub cells: Vec<(SharedString, SharedString)>,
    /// Initially expanded.
    pub open: bool,
    /// Children (`None` for a leaf).
    pub children: Option<Vec<TreeTableRow>>,
}

impl TreeTableRow {
    /// A leaf row.
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            ..Default::default()
        }
    }

    /// Set a cell.
    pub fn cell(mut self, key: impl Into<SharedString>, value: impl Into<SharedString>) -> Self {
        self.cells.push((key.into(), value.into()));
        self
    }

    /// Initially expanded.
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// Children (makes it a branch).
    pub fn children(mut self, children: impl IntoIterator<Item = TreeTableRow>) -> Self {
        self.children = Some(children.into_iter().collect());
        self
    }

    /// The value of `key` (empty when missing).
    pub fn get(&self, key: &str) -> SharedString {
        self.cells
            .iter()
            .find(|(k, _)| k.as_ref() == key)
            .map(|(_, v)| v.clone())
            .unwrap_or_default()
    }
}

/// The visible rows with their depth (`walk(rows, 0)`).
pub fn flatten(rows: &[TreeTableRow], open: &HashSet<SharedString>) -> Vec<(TreeTableRow, usize)> {
    fn walk(
        rs: &[TreeTableRow],
        d: usize,
        open: &HashSet<SharedString>,
        out: &mut Vec<(TreeTableRow, usize)>,
    ) {
        for r in rs {
            out.push((r.clone(), d));
            if let Some(c) = &r.children
                && open.contains(&r.id)
            {
                walk(c, d + 1, open, out);
            }
        }
    }
    let mut out = Vec::new();
    walk(rows, 0, open, &mut out);
    out
}

/// A tree table. See the module docs.
#[derive(IntoElement)]
pub struct TreeTable {
    id: ElementId,
    columns: Vec<TreeTableColumn>,
    rows: Vec<TreeTableRow>,
    label: SharedString,
    toggle_labels: (SharedString, SharedString),
}

impl TreeTable {
    /// A tree table with a stable `id`.
    pub fn new(
        id: impl Into<ElementId>,
        columns: impl IntoIterator<Item = TreeTableColumn>,
        rows: impl IntoIterator<Item = TreeTableRow>,
    ) -> Self {
        Self {
            id: id.into(),
            columns: columns.into_iter().collect(),
            rows: rows.into_iter().collect(),
            // The reference's texts; override per locale.
            label: "Tree table".into(),
            toggle_labels: ("Expand".into(), "Collapse".into()),
        }
    }

    /// The accessible name.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    /// The toggle's verbs (`Expand <name>` / `Collapse <name>`).
    pub fn toggle_labels(
        mut self,
        expand: impl Into<SharedString>,
        collapse: impl Into<SharedString>,
    ) -> Self {
        self.toggle_labels = (expand.into(), collapse.into());
        self
    }
}

fn sized(col: &TreeTableColumn) -> gpui_kit::Div {
    let d = div()
        .min_w_0()
        .overflow_hidden()
        .whitespace_nowrap()
        .text_ellipsis();
    let d = match col.width {
        Some(w) => d.flex_none().w(gpui_kit::rems(w)),
        None => d.flex_1().flex_basis(px(0.)),
    };
    match col.align {
        CellAlign::Start => d,
        CellAlign::End => d.text_right(),
    }
}

impl RenderOnce for TreeTable {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let init = self.rows.clone();
        let open = window.use_keyed_state((id.clone(), "open"), cx, move |_, _| {
            fn opened(rs: &[TreeTableRow], out: &mut HashSet<SharedString>) {
                for r in rs {
                    if r.open {
                        out.insert(r.id.clone());
                    }
                    if let Some(c) = &r.children {
                        opened(c, out);
                    }
                }
            }
            let mut s = HashSet::new();
            opened(&init, &mut s);
            s
        });
        let opened = open.read(cx).clone();
        let row_h = density::current().row().px_value();
        let flat = flatten(&self.rows, &opened);
        let first_key = self
            .columns
            .first()
            .map(|c| c.key.clone())
            .unwrap_or_default();

        let head = div()
            .flex()
            .items_center()
            .h(rem(row_h))
            .bg(color(ColorRole::LayerAccent01))
            .text_size(rem(12.))
            .line_height(rem(16.))
            .font_weight(FontWeight::SEMIBOLD)
            .children(
                self.columns
                    .iter()
                    .map(|c| sized(c).px(rem(12.)).child(c.header.clone())),
            );

        let rows = flat.into_iter().map(|(r, d)| {
            let hover = window.use_keyed_state(
                (id.clone(), SharedString::from(format!("hover-{}", r.id))),
                cx,
                |_, _| false,
            );
            let hovered = *hover.read(cx);
            let is_open = opened.contains(&r.id);
            let chevron = motion::animate(
                (id.clone(), SharedString::from(format!("chev-{}", r.id))),
                if is_open {
                    std::f32::consts::FRAC_PI_2
                } else {
                    0.
                },
                CHEVRON_MOTION,
                window,
                cx,
            );
            // `.g-ttable-tog:focus-visible` (ghost.css:17): the 2px `focus`
            // outline at offset -2px, for keyboard focus only.
            let tog_focus = parts::keyed_focus(
                (
                    id.clone(),
                    SharedString::from(format!("tog-focus-{}", r.id)),
                ),
                window,
                cx,
            );
            let tog_ring = parts::focus_visible(&tog_focus, window);
            let cells = self.columns.iter().enumerate().map(|(ci, c)| {
                let value = r.get(&c.key);
                let content = match &c.render {
                    Some(f) => f(&value, &r),
                    None => value.clone().into_any_element(),
                };
                let cell = sized(c).px(rem(12.));
                if ci > 0 {
                    return cell.child(content);
                }
                // `.g-ttable-row > span:first-child { display: flex; gap: spacing-02;
                // padding-left: 8 + d × 16 }`.
                let toggle = if r.children.is_some() {
                    let open = open.clone();
                    let rid = r.id.clone();
                    let (expand, collapse) = self.toggle_labels.clone();
                    let name = r.get(&first_key);
                    gpui_kit::base::Button::new((
                        id.clone(),
                        SharedString::from(format!("tog-{}", r.id)),
                    ))
                    .accessibility_label(format!(
                        "{} {name}",
                        if is_open { collapse } else { expand }
                    ))
                    .track_focus(&tog_focus)
                    .relative()
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .size(rem(20.))
                    .cursor(CursorStyle::PointingHand)
                    .child(
                        Icon::carbon(CarbonIcon::ChevronRight)
                            .size_length(px(14.))
                            .color(color(ColorRole::IconSecondary))
                            .rotate(gpui_kit::radians(chevron)),
                    )
                    .when(tog_ring, |b| b.child(parts::focus_ring()))
                    .on_click(move |_, _, cx| {
                        open.update(cx, |s, cx| {
                            if !s.remove(&rid) {
                                s.insert(rid.clone());
                            }
                            cx.notify();
                        })
                    })
                    .into_any_element()
                } else {
                    div().w(rem(14.)).flex_none().into_any_element()
                };
                cell.flex()
                    .items_center()
                    .gap(rem(4.))
                    .pl(rem(8. + d as f32 * 16.))
                    .child(toggle)
                    .child(
                        div()
                            .min_w_0()
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(content),
                    )
            });
            div()
                .id((id.clone(), SharedString::from(format!("row-{}", r.id))))
                .flex()
                .items_center()
                .h(rem(row_h))
                .border_b(px(1.))
                .border_color(color(ColorRole::BorderSubtle01))
                .bg(color(if hovered {
                    ColorRole::LayerHover01
                } else {
                    ColorRole::Layer01
                }))
                .on_hover(move |h, _, cx| {
                    hover.update(cx, |v, cx| {
                        *v = *h;
                        cx.notify();
                    })
                })
                .children(cells)
        });

        div()
            .id(id.clone())
            .role(gpui_kit::Role::TreeGrid)
            .aria_label(self.label.clone())
            .flex()
            .flex_col()
            .w_full()
            .type_style(TypeStyle::BodyCompact01)
            .text_color(color(ColorRole::TextPrimary))
            .child(head)
            .children(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flatten_follows_open_rows() {
        let rows = vec![
            TreeTableRow::new("1")
                .cell("name", "src")
                .open(true)
                .children([
                    TreeTableRow::new("1a").cell("name", "index.ts"),
                    TreeTableRow::new("1b")
                        .cell("name", "theme")
                        .children([TreeTableRow::new("1b1").cell("name", "g100.ts")]),
                ]),
            TreeTableRow::new("2").cell("name", "package.json"),
        ];
        let open: HashSet<SharedString> = ["1".into()].into();
        let flat = flatten(&rows, &open);
        let ids: Vec<(&str, usize)> = flat.iter().map(|(r, d)| (r.id.as_ref(), *d)).collect();
        assert_eq!(ids, [("1", 0), ("1a", 1), ("1b", 1), ("2", 0)]);
        assert_eq!(rows[1].get("name").as_ref(), "package.json");
        assert_eq!(rows[1].get("size").as_ref(), "");
    }
}
