//! T3 · Mobile › SwipeRow: the reference rows (trailing Snooze / Archive
//! revealed, leading Pin revealed) and a live row to drag sideways.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, rems,
};
use u0ui::prelude::*;
use u0ui::t3::Status;
use u0ui::t3::mobile::{Revealed, SwipeAction, SwipeRow, SwipeTone};
use u0ui::t3::shell::ThreadRow;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        revealed: Revealed::None,
    })
    .into()
}

struct Page {
    revealed: Revealed,
}

fn trailing() -> [SwipeAction; 2] {
    [
        SwipeAction::new("Snooze", CarbonIcon::Asleep, SwipeTone::Purple),
        SwipeAction::new("Archive", CarbonIcon::Archive, SwipeTone::Gray),
    ]
}

fn leading() -> [SwipeAction; 1] {
    [SwipeAction::new("Pin", CarbonIcon::Pin, SwipeTone::Blue)]
}

fn stack() -> gpui_kit::Div {
    // `cu-stack` with `gap: 8; maxWidth: 380`.
    div()
        .flex()
        .flex_col()
        .gap(rems(0.5))
        .w_full()
        .max_w(rems(23.75))
}

fn reference(prefix: &'static str) -> gpui_kit::Div {
    stack()
        .child(
            SwipeRow::new((prefix, 0usize))
                .revealed(Revealed::Trailing)
                .trailing(trailing())
                .content(
                    ThreadRow::new((prefix, 1usize), "Fix flaky date picker test")
                        .status(Status::NeedsYou)
                        .time("3m"),
                ),
        )
        .child(
            SwipeRow::new((prefix, 2usize))
                .revealed(Revealed::Leading)
                .leading(leading())
                .content(
                    ThreadRow::new((prefix, 3usize), "Write migration guide")
                        .status(Status::Done)
                        .time("1h"),
                ),
        )
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    reference("sr-thumb").w(rems(15.)).into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let live = stack().child(
            SwipeRow::new("sr-live")
                .revealed(self.revealed)
                .leading(leading())
                .trailing(trailing())
                .on_revealed(controls::setter(&entity, |this: &mut Page, r| {
                    this.revealed = r
                }))
                .content(
                    ThreadRow::new("sr-live-row", "Fix flaky date picker test")
                        .status(Status::NeedsYou)
                        .time("3m"),
                ),
        );
        let config = controls::field(
            "revealed",
            controls::choices(
                "sr-revealed",
                &["none", "leading", "trailing"],
                Revealed::ALL
                    .iter()
                    .position(|r| *r == self.revealed)
                    .unwrap_or(0),
                controls::setter(&entity, |this: &mut Page, i| {
                    this.revealed = Revealed::ALL[i]
                }),
            ),
        );
        Body::new()
            .playground(live)
            .hint("Drag the row sideways; past half the travel it stays open")
            .config(config)
            .example(controls::block("Reference preview", reference("sr-ref")))
    }
}
