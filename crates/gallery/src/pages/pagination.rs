//! Navigation › Pagination: the reference preview (Pagination and
//! PaginationNav) and configurable live ones.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, SharedString,
    Styled, Window, div,
};
use u0ui::components::{Pagination, PaginationNav};
use u0ui::prelude::*;

const TOTALS: [(&str, usize); 3] = [("8", 8), ("103", 103), ("1,000", 1000)];
const SHOWN: [(&str, usize); 3] = [("5", 5), ("7", 7), ("9", 9)];
const NAV_TOTALS: [(&str, usize); 3] = [("5", 5), ("12", 12), ("40", 40)];

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| PaginationPage {
        total: 1,
        shown: 1,
        nav_total: 1,
        page: 1,
        nav_page: 3,
    })
    .into()
}

struct PaginationPage {
    total: usize,
    shown: usize,
    nav_total: usize,
    page: usize,
    nav_page: usize,
}

impl Render for PaginationPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let pick = |f: fn(&mut PaginationPage, usize)| {
            let entity = entity.clone();
            move |i: usize, _: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    f(this, i);
                    cx.notify();
                })
            }
        };
        // `components/Pagination/preview.html` (`cu-stack`).
        let reference = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::section("Reference preview"))
            .child(
                Pagination::new("ref-pagination", 103)
                    .page_size(10)
                    .default_page(2),
            )
            .child(PaginationNav::new("ref-pagination-nav", 12).page(3));

        let total = TOTALS[self.total].1;
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(
                Pagination::new(
                    SharedString::from(format!("live-pagination-{total}")),
                    total,
                )
                .page(self.page)
                .on_page_change(pick(|this, p| this.page = p)),
            )
            .child(
                PaginationNav::new(
                    SharedString::from(format!("live-nav-{}", self.nav_total)),
                    NAV_TOTALS[self.nav_total].1,
                )
                .items_shown(SHOWN[self.shown].1)
                .page(self.nav_page)
                .on_page_change(pick(|this, p| this.nav_page = p)),
            )
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(format!(
                        "Pagination page {} · PaginationNav page {}",
                        self.page,
                        self.nav_page + 1
                    )),
            );

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "totalItems (Pagination)",
                controls::choices(
                    "pg-total",
                    &TOTALS.map(|t| t.0),
                    self.total,
                    pick(|this, i| {
                        this.total = i;
                        this.page = 1;
                    }),
                ),
            ))
            .child(controls::field(
                "totalItems (PaginationNav)",
                controls::choices(
                    "pg-nav-total",
                    &NAV_TOTALS.map(|t| t.0),
                    self.nav_total,
                    pick(|this, i| {
                        this.nav_total = i;
                        this.nav_page = 0;
                    }),
                ),
            ))
            .child(controls::field(
                "itemsShown",
                controls::choices(
                    "pg-shown",
                    &SHOWN.map(|s| s.0),
                    self.shown,
                    pick(|this, i| this.shown = i),
                ),
            ));

        Body::new()
            .playground(live)
            .config(config)
            .example(reference)
    }
}
