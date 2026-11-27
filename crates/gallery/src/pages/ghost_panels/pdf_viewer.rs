//! Ghost · Right column › PdfViewer: the live frame (page count, default
//! page, a host-rendered page) and the reference document.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, prelude::FluentBuilder as _, rems,
};
use u0ui::ghost::panels::PdfViewer;
use u0ui::prelude::*;

const PAGES: [&str; 3] = ["1", "4", "12"];

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        pages: 1,
        custom: false,
        last: None,
    })
    .into()
}

struct Page {
    pages: usize,
    custom: bool,
    last: Option<u32>,
}

/// `components/PdfViewer/preview.html`.
fn reference() -> impl IntoElement {
    PdfViewer::new("pdf-ref", "design-review.pdf", 4).default_page(2)
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .h(rems(10.))
        .overflow_hidden()
        .child(PdfViewer::new("pdf-thumb", "review.pdf", 3))
        .into_any_element()
}

/// A stand-in for a host-rendered page (what a pdf.js or pdfium raster
/// would put in the sheet).
fn rendered(n: u32) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap(rems(0.5))
        .size_full()
        .child(
            div()
                .type_style(TypeStyle::HeadingCompact01)
                .child(format!("Section {n}")),
        )
        .child(div().h(rems(2.5)).bg(color(ColorRole::Charts1)))
        .child(
            div()
                .type_style(TypeStyle::Label01)
                .child("Rendered by the host through renderPage(n)."),
        )
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let e = cx.entity();
        let pages: u32 = PAGES[self.pages].parse().unwrap_or(4);
        let report = e.clone();
        let live = div()
            .w_full()
            .max_w(rems(40.))
            .flex()
            .flex_col()
            .gap(tokens::SPACING_03)
            .child(
                PdfViewer::new(("pdf-live", self.pages), "design-review.pdf", pages)
                    .default_page(2.min(pages))
                    .when(self.custom, |v| v.render_page(|n, _, _| rendered(n)))
                    .on_page_change(move |n, _, cx| {
                        report.update(cx, |p, cx| {
                            p.last = Some(n);
                            cx.notify();
                        })
                    }),
            )
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(format!(
                        "onPageChange: {}",
                        self.last.map_or("—".to_string(), |n| n.to_string())
                    )),
            );
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::field(
                "pages",
                controls::choices(
                    "pdf-pages",
                    &PAGES,
                    self.pages,
                    controls::setter(&e, |p: &mut Page, v| p.pages = v),
                ),
            ))
            .child(controls::toggle(
                "pdf-render",
                "renderPage",
                self.custom,
                controls::setter(&e, |p: &mut Page, v| p.custom = v),
            ));
        Body::new()
            .playground(live)
            .hint("Type a page number and press Enter; thumbnails, chevrons and zoom work too")
            .config(config)
            .example(controls::block("Reference preview", reference()))
    }
}
