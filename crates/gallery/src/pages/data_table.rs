//! Data display › DataTable: the reference "Virtual servers" table (sort,
//! search, select, pagination, a Create action) and a live table with every
//! `DataTableProps` prop plus batch actions and expandable rows.
use crate::controls;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, SharedString,
    Styled, Window, div,
};
use u0ui::components::badge::{Badge, BadgeSize, BadgeVariant};
use u0ui::components::data_table::{BatchAction, DataTable, DataTableColumn, DataTableRow};
use u0ui::components::table::TableSize;
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| DataTablePage {
        size: 3,
        sortable: true,
        selectable: true,
        search: true,
        page_size: 1,
        zebra: false,
        title: true,
        actions: true,
        batch: true,
        expandable: true,
        last_action: None,
    })
    .into()
}

struct DataTablePage {
    size: usize,
    sortable: bool,
    selectable: bool,
    search: bool,
    page_size: usize,
    zebra: bool,
    title: bool,
    actions: bool,
    batch: bool,
    expandable: bool,
    last_action: Option<SharedString>,
}

const PAGE_SIZES: [(&str, usize); 3] = [("3", 3), ("5", 5), ("10", 10)];

/// `components/DataTable/preview.html` rows: `vm-101…112`, status
/// `['Running','Stopped','Failed'][i % 3]`, zone by `i % 3`, cpu `(i * 7) % 100 + '%'`.
fn rows() -> Vec<DataTableRow> {
    const STATUS: [&str; 3] = ["Running", "Stopped", "Failed"];
    const ZONE: [&str; 3] = ["us-south-1", "eu-de-2", "jp-tok-3"];
    (1..=12)
        .map(|i| {
            DataTableRow::new(i.to_string())
                .cell("name", format!("vm-{}", 100 + i))
                .cell("status", STATUS[i % 3])
                .cell("zone", ZONE[i % 3])
                .cell("cpu", format!("{}%", (i * 7) % 100))
        })
        .collect()
}

/// `tone = { Running: 'green', Stopped: 'gray', Failed: 'red' }`, `size: 'sm'`.
fn columns() -> Vec<DataTableColumn> {
    vec![
        DataTableColumn::new("name", "Name"),
        DataTableColumn::new("status", "Status").render(|v, _| {
            let tone = match v.as_ref() {
                "Running" => BadgeVariant::Green,
                "Stopped" => BadgeVariant::Gray,
                _ => BadgeVariant::Red,
            };
            Badge::new(v.clone())
                .variant(tone)
                .size(BadgeSize::Sm)
                .into_any_element()
        }),
        DataTableColumn::new("zone", "Zone"),
        DataTableColumn::new("cpu", "CPU"),
    ]
}

/// `toolbarActions: Button size sm, icon Add, "Create"`. Inside the toolbar
/// `.cds--table-toolbar .cds--btn--primary { block-size:
/// var(--layout-size-height-local) }` (0,2,0) makes it 48px tall, and compact
/// density does not shrink `sm` buttons, so it renders as a 48px button.
fn create_button(id: &'static str) -> impl IntoElement {
    Button::new(id, "Create")
        .icon(CarbonIcon::Add)
        .size(ButtonSize::Lg)
        .density(Density::Comfortable)
}

impl Render for DataTablePage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();

        let reference = DataTable::new("ref-data-table")
            .title("Virtual servers")
            .description("Sort, search and select.")
            .selectable(true)
            .page_size(5)
            .columns(columns())
            .rows(rows())
            .toolbar_actions(create_button("dt-ref-create"));

        let page_size = PAGE_SIZES[self.page_size].1;
        let mut live = DataTable::new(SharedString::from(format!(
            "live-data-table-{}-{}",
            self.selectable, self.expandable
        )))
        .size(TableSize::ALL[self.size])
        .sortable(self.sortable)
        .selectable(self.selectable)
        .search(self.search)
        .page_size(page_size)
        .zebra(self.zebra)
        .columns(columns())
        .rows(rows());
        if self.title {
            live = live
                .title("Virtual servers")
                .description("Sort, search and select.");
        }
        if self.actions {
            live = live.toolbar_actions(create_button("dt-live-create"));
        }
        if self.batch {
            let e1 = entity.clone();
            let e2 = entity.clone();
            live = live.batch_actions(vec![
                BatchAction::new("Delete", move |ids, _, cx| {
                    let text = format!("Delete: {}", ids.join(", "));
                    e1.update(cx, |this, cx| {
                        this.last_action = Some(text.into());
                        cx.notify();
                    })
                })
                .icon(CarbonIcon::TrashCan),
                BatchAction::new("Download", move |ids, _, cx| {
                    let text = format!("Download: {}", ids.join(", "));
                    e2.update(cx, |this, cx| {
                        this.last_action = Some(text.into());
                        cx.notify();
                    })
                })
                .icon(CarbonIcon::Download),
            ]);
        }
        if self.expandable {
            live = live.expandable(|row| {
                div()
                    .flex()
                    .flex_col()
                    .gap(tokens::SPACING_02)
                    .child(
                        div()
                            .type_style(TypeStyle::HeadingCompact01)
                            .child(SharedString::from(row.get("name").to_string())),
                    )
                    .child(SharedString::from(format!(
                        "Zone {} · CPU {} · status {}",
                        row.get("zone"),
                        row.get("cpu"),
                        row.get("status")
                    )))
                    .into_any_element()
            });
        }

        let sizes = TableSize::ALL.map(|s| s.name());
        let page_sizes = PAGE_SIZES.map(|p| p.0);
        let toggle =
            |id: &'static str, label: &'static str, on: bool, f: fn(&mut DataTablePage, bool)| {
                controls::toggle(id, label, on, controls::setter(&entity, f))
            };
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "size",
                controls::choices(
                    "dt-size",
                    &sizes,
                    self.size,
                    controls::setter(&entity, |this: &mut DataTablePage, i| this.size = i),
                ),
            ))
            .child(controls::field(
                "pageSize",
                controls::choices(
                    "dt-page-size",
                    &page_sizes,
                    self.page_size,
                    controls::setter(&entity, |this: &mut DataTablePage, i| this.page_size = i),
                ),
            ))
            .child(toggle("dt-sortable", "sortable", self.sortable, |t, on| {
                t.sortable = on
            }))
            .child(toggle(
                "dt-selectable",
                "selectable",
                self.selectable,
                |t, on| t.selectable = on,
            ))
            .child(toggle("dt-search", "search", self.search, |t, on| {
                t.search = on
            }))
            .child(toggle("dt-zebra", "zebra", self.zebra, |t, on| {
                t.zebra = on
            }))
            .child(toggle(
                "dt-title",
                "title / description",
                self.title,
                |t, on| t.title = on,
            ))
            .child(toggle(
                "dt-actions",
                "toolbarActions",
                self.actions,
                |t, on| t.actions = on,
            ))
            .child(toggle("dt-batch", "batch actions", self.batch, |t, on| {
                t.batch = on
            }))
            .child(toggle(
                "dt-expand",
                "expandable rows",
                self.expandable,
                |t, on| t.expandable = on,
            ))
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(
                        self.last_action
                            .clone()
                            .unwrap_or_else(|| "Select rows to show the batch actions.".into()),
                    ),
            );

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
