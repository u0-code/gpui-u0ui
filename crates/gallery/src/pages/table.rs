//! Data display › Table: the reference "Services" table (title, description,
//! status tags, caption) and a live table with every `TableProps` prop plus
//! footer and caption parts.
use crate::controls;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled, Window, div,
};
use u0ui::components::badge::{Badge, BadgeSize, BadgeVariant};
use u0ui::components::table::{
    CellAlign, Table, TableBody, TableCaption, TableCell, TableFooter, TableHead, TableHeader,
    TableRow, TableSize,
};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| TablePage {
        size: 3,
        zebra: false,
        title: true,
        description: true,
        footer: true,
        caption: true,
        selected: false,
    })
    .into()
}

struct TablePage {
    size: usize,
    zebra: bool,
    title: bool,
    description: bool,
    footer: bool,
    caption: bool,
    selected: bool,
}

/// `components/Table/preview.html` rows.
const ROWS: [[&str; 4]; 3] = [
    ["api-gateway", "Running", "us-south", "3"],
    ["auth-service", "Running", "eu-de", "2"],
    ["billing-worker", "Failed", "us-east", "0"],
];

/// More rows for the live table (zebra needs several).
const MORE: [[&str; 4]; 3] = [
    ["search-indexer", "Running", "jp-tok", "4"],
    ["mail-relay", "Running", "eu-gb", "1"],
    ["report-cron", "Failed", "us-south", "0"],
];

fn header() -> TableHeader {
    TableHeader::new().child(
        TableRow::new().children(["Name", "Status", "Region", "Replicas"].map(TableHead::new)),
    )
}

fn row(r: [&'static str; 4]) -> TableRow {
    // `tone = { Running: 'green', Failed: 'red' }`, `size: 'sm'`.
    let tone = if r[1] == "Running" {
        BadgeVariant::Green
    } else {
        BadgeVariant::Red
    };
    TableRow::new()
        .child(TableCell::new(r[0]))
        .child(TableCell::new(
            Badge::new(r[1]).variant(tone).size(BadgeSize::Sm),
        ))
        .child(TableCell::new(r[2]))
        .child(TableCell::new(r[3]))
}

impl Render for TablePage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();

        let reference = Table::new("ref-table")
            .title("Services")
            .description("Workloads in the production project.")
            .header(header())
            .body(TableBody::new().children(ROWS.map(row)))
            .caption(TableCaption::new("Updated 2 minutes ago"));

        let mut live = Table::new("live-table")
            .size(TableSize::ALL[self.size])
            .zebra(self.zebra)
            .header(header())
            .body(
                TableBody::new().children(
                    ROWS.into_iter()
                        .chain(MORE)
                        .enumerate()
                        .map(|(i, r)| row(r).selected(self.selected && i == 1)),
                ),
            );
        if self.title {
            live = live.title("Services");
        }
        if self.description {
            live = live.description("Workloads in the production project.");
        }
        if self.footer {
            live = live.footer(
                TableFooter::new().child(
                    TableRow::new()
                        .child(TableCell::new("Total"))
                        .child(TableCell::new(""))
                        .child(TableCell::new(""))
                        .child(TableCell::new("10").align(CellAlign::Start)),
                ),
            );
        }
        if self.caption {
            live = live.caption(TableCaption::new("Updated 2 minutes ago"));
        }

        let sizes = TableSize::ALL.map(|s| s.name());
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "size",
                controls::choices(
                    "table-size",
                    &sizes,
                    self.size,
                    controls::setter(&entity, |this: &mut TablePage, i| this.size = i),
                ),
            ))
            .child(
                div()
                    .text_color(color(ColorRole::TextSecondary))
                    .type_style(TypeStyle::Label01)
                    .child("Compact density sets every size to 32px rows (density-row-md)."),
            )
            .child(controls::toggle(
                "table-zebra",
                "zebra",
                self.zebra,
                controls::setter(&entity, |this: &mut TablePage, on| this.zebra = on),
            ))
            .child(controls::toggle(
                "table-title",
                "title",
                self.title,
                controls::setter(&entity, |this: &mut TablePage, on| this.title = on),
            ))
            .child(controls::toggle(
                "table-description",
                "description",
                self.description,
                controls::setter(&entity, |this: &mut TablePage, on| this.description = on),
            ))
            .child(controls::toggle(
                "table-footer",
                "TableFooter",
                self.footer,
                controls::setter(&entity, |this: &mut TablePage, on| this.footer = on),
            ))
            .child(controls::toggle(
                "table-caption",
                "TableCaption",
                self.caption,
                controls::setter(&entity, |this: &mut TablePage, on| this.caption = on),
            ))
            .child(controls::toggle(
                "table-selected",
                "TableRow selected (row 2)",
                self.selected,
                controls::setter(&entity, |this: &mut TablePage, on| this.selected = on),
            ));

        controls::page_body(
            div()
                .flex()
                .flex_col()
                .gap(tokens::SPACING_07)
                .child(controls::block("Reference preview", reference))
                .child(controls::block("Live", live)),
            Some(config.into_any_element()),
        )
    }
}
