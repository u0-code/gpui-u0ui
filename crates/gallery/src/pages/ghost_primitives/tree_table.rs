//! Ghost · Primitives › TreeTable: the reference file table, live.
use crate::controls;

use gpui_kit::{AnyElement, AnyView, App, IntoElement, ParentElement, Styled, Window, div, rems};
use u0ui::ghost::primitives::{CellAlign, TreeTable, TreeTableColumn, TreeTableRow};

fn columns() -> Vec<TreeTableColumn> {
    vec![
        TreeTableColumn::new("name", "Name"),
        TreeTableColumn::new("size", "Size")
            .width(6.)
            .align(CellAlign::End),
        TreeTableColumn::new("mod", "Modified").width(8.),
    ]
}

fn file(id: &str, name: &str, size: &str, modified: &str) -> TreeTableRow {
    TreeTableRow::new(id.to_string())
        .cell("name", name.to_string())
        .cell("size", size.to_string())
        .cell("mod", modified.to_string())
}

fn rows() -> Vec<TreeTableRow> {
    vec![
        file("1", "src", "—", "2m ago").open(true).children([
            file("1a", "index.ts", "4 KB", "2m ago"),
            file("1b", "theme", "—", "1h ago").children([file("1b1", "g100.ts", "2 KB", "1h ago")]),
        ]),
        file("2", "package.json", "1 KB", "3d ago"),
        file("3", "README.md", "6 KB", "3d ago"),
    ]
}

/// `components/TreeTable/preview.html`.
pub fn reference(id: &'static str) -> impl IntoElement {
    TreeTable::new(id, columns(), rows()).label("Files")
}

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    use gpui_kit::AppContext as _;
    cx.new(|_| TreeTablePage).into()
}

struct TreeTablePage;

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(reference("thumb-ttable"))
        .into_any_element()
}

impl gpui_kit::Render for TreeTablePage {
    fn render(&mut self, _: &mut Window, _: &mut gpui_kit::Context<Self>) -> impl IntoElement {
        crate::page::Body::new()
            .playground(div().w_full().child(reference("ttable-live")))
            .hint("Press a chevron to expand or collapse")
            .example(controls::block(
                "Reference preview",
                reference("ttable-ref"),
            ))
    }
}
