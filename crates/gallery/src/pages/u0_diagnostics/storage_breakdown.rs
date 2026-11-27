//! u0 · Diagnostics › StorageBreakdown: the reference breakdown, live
//! (a tidy action shrinks its item and turns it to "Kept").
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, rems,
};
use u0ui::u0::diagnostics::{StorageBreakdown, StorageItem};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page { items: items() }).into()
}

struct Page {
    items: Vec<StorageItem>,
}

/// `components/StorageBreakdown/preview.html`'s items.
fn items() -> Vec<StorageItem> {
    vec![
        StorageItem::new("Transcripts", "1,284 threads", 820e6).tidy("Archive old"),
        StorageItem::new("Checkpoints", "Snapshots for restore", 2.1e9).tidy("Remove >30 days"),
        StorageItem::new("Worktrees", "7 worktrees", 3.4e9).tidy("Clean up"),
        StorageItem::new("Agent binaries", "Claude, Codex, Gemini", 640e6),
        StorageItem::new("Logs", "Last 14 days", 95e6).tidy("Clear"),
    ]
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(StorageBreakdown::new("store-thumb", items().into_iter().take(2)).total(412e9))
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let live = div().w(rems(35.)).max_w_full().child(
            StorageBreakdown::new("store-live", self.items.clone())
                .total(412e9)
                .on_tidy(move |label, _, cx| {
                    entity.update(cx, |p, cx| {
                        if let Some(it) = p.items.iter_mut().find(|i| i.label == label) {
                            it.bytes *= 0.2;
                            it.tidy = None;
                        }
                        cx.notify();
                    })
                }),
        );
        Body::new()
            .playground(live)
            .hint("The bar's segments use charts-1… in order")
            .example(controls::block(
                "Reference preview",
                div()
                    .w(rems(35.))
                    .max_w_full()
                    .child(StorageBreakdown::new("store-ref", items()).total(412e9)),
            ))
    }
}
