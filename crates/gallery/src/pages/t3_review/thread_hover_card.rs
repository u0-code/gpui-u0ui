//! T3 · Thread › ThreadHoverCard: the reference card, and the card in the
//! HoverCard it pops from when a thread reference is hovered.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, rems,
};
use u0ui::components::{Button, ButtonVariant, HoverCard};
use u0ui::t3::Status;
use u0ui::t3::review::ThreadHoverCard;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page { status: 1 }).into()
}

struct Page {
    status: usize,
}

const STATUSES: [Status; 4] = [
    Status::Working,
    Status::NeedsYou,
    Status::Done,
    Status::Failed,
];

fn card(status: Status) -> ThreadHoverCard {
    ThreadHoverCard::new("Fix flaky date picker test", "carbon-ui")
        .status(status)
        .provider("Codex \u{b7} GPT")
        .time("3m ago")
        .last_message(
            "I need approval to run the full e2e suite against the staging database. It takes about 6 minutes.",
        )
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(card(Status::NeedsYou))
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let status = STATUSES[self.status];
        let hover = HoverCard::new(
            "thc-live",
            Button::new("thc-trigger", "Fix flaky date picker test").variant(ButtonVariant::Link),
        )
        // `.t3-hovercard` is the popup surface itself (20rem, 16px padding,
        // layer-01, `0 2px 6px` shadow): no popover surface around it.
        .bare(true)
        .child(card(status));
        let config = controls::field(
            "status",
            controls::choices(
                "thc-status",
                &["working", "needs-you", "done", "failed"],
                self.status,
                controls::setter(&entity, |this: &mut Page, i| this.status = i),
            ),
        );
        Body::new()
            .playground(
                div()
                    .flex()
                    .flex_col()
                    // Children keep their own width: the link (and its focus
                    // ring) hugs its text instead of stretching across.
                    .items_start()
                    .gap(rems(1.))
                    .child(card(status))
                    .child(hover),
            )
            .hint("Hover the link to pop the card")
            .config(config)
            .example(controls::block("Reference preview", card(Status::NeedsYou)))
    }
}
