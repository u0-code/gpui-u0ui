//! Ghost · Thread › ThinkingBlock: a live block with every prop, then the
//! two reference blocks (`components/ThinkingBlock/preview.html`).
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, rems,
};
use u0ui::ghost::thread::ThinkingBlock;
use u0ui::prelude::*;

/// The reference's reasoning text.
const THOUGHT: &str = "The G100 theme has no layer-accent-03, so the third nesting level should fall back to layer-accent-02. I will check whether any component reaches level three before adding a token.";

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| ThinkingBlockPage {
        live: false,
        seconds: 1,
        default_open: true,
    })
    .into()
}

struct ThinkingBlockPage {
    live: bool,
    seconds: usize,
    default_open: bool,
}

/// The `seconds` options.
const SECONDS: [&str; 3] = ["9", "23", "74"];

/// `width: 520, display: grid, gap: 8`: a live block, then a finished one.
pub fn reference(prefix: &'static str) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_03)
        .w_full()
        .max_w(rems(32.5))
        .child(ThinkingBlock::new((prefix, 0usize)).live(true))
        .child(
            ThinkingBlock::new((prefix, 1usize))
                .seconds(23)
                .default_open(true)
                .child(THOUGHT),
        )
}

impl Render for ThinkingBlockPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let seconds: u64 = SECONDS[self.seconds].parse().unwrap_or(0);
        // Re-key on `defaultOpen` so the switch re-seeds the fold.
        let live = ThinkingBlock::new(("think-live", self.default_open as usize))
            .live(self.live)
            .seconds(seconds)
            .default_open(self.default_open)
            .child(THOUGHT);
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::toggle(
                "think-live-toggle",
                "live",
                self.live,
                controls::setter(&entity, |this: &mut Self, on| this.live = on),
            ))
            .child(controls::field(
                "seconds",
                controls::choices(
                    "think-seconds",
                    &SECONDS,
                    self.seconds,
                    controls::setter(&entity, |this: &mut Self, i| this.seconds = i),
                ),
            ))
            .child(controls::toggle(
                "think-open",
                "defaultOpen",
                self.default_open,
                controls::setter(&entity, |this: &mut Self, on| this.default_open = on),
            ));
        Body::new()
            .playground(div().w_full().max_w(rems(32.5)).child(live))
            .hint("Click the head to open or close the reasoning; turn on reduced motion to see the pulse.")
            .config(config)
            .example(controls::block("Reference preview", reference("think-ref")))
    }
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .flex()
        .flex_col()
        .gap(tokens::SPACING_03)
        .child(ThinkingBlock::new("think-thumb-0").live(true))
        .child(ThinkingBlock::new("think-thumb-1").seconds(23))
        .into_any_element()
}
