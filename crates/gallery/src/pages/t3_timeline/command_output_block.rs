//! T3 · Timeline › CommandOutputBlock: a failed test run with its cwd and
//! output, and a running build (`components/CommandOutputBlock/preview.html`).
//! The head folds the output.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div,
};
use u0ui::prelude::*;
use u0ui::t3::timeline::{CommandDuration, CommandOutputBlock};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| CommandPage {
        status: 1,
        meta: false,
        duration: true,
    })
    .into()
}

struct CommandPage {
    status: usize,
    meta: bool,
    duration: bool,
}

const OUTPUT: &str = "FAIL  tokens.contrast.test.ts\n  × text-helper on layer-02 (g90): 4.21 < 4.5\nTests: 1 failed, 63 passed";

/// The reference preview (`cu-stack`, gap 8, max 640px).
fn reference() -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_03)
        .w_full()
        .max_w(gpui_kit::rems(40.))
        .child(
            CommandOutputBlock::new("cmd-ref-0", "pnpm test --filter tokens", OUTPUT)
                .exit_code(1)
                .cwd("~/Developer/carbon-ui"),
        )
        .child(CommandOutputBlock::new("cmd-ref-1", "pnpm build", "").running(true))
}

const STATUSES: [&str; 4] = ["exit 0", "exit 1", "running", "stopped"];

impl Render for CommandPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let mut block = CommandOutputBlock::new(
            SharedString::from(format!("cmd-live-{}", self.status)),
            "pnpm test --filter tokens",
            OUTPUT,
        )
        .cwd("~/Developer/carbon-ui");
        block = match self.status {
            0 => block.exit_code(0),
            1 => block.exit_code(1),
            2 => block.running(true),
            _ => block.stopped(true),
        };
        if self.meta {
            block = block.meta("build-box");
        }
        if self.duration {
            block = block.duration(CommandDuration::Seconds(74));
        }
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "status",
                controls::choices(
                    "cmd-status",
                    &STATUSES,
                    self.status,
                    controls::setter(&entity, |this: &mut Self, i| this.status = i),
                ),
            ))
            .child(controls::toggle(
                "cmd-meta",
                "meta",
                self.meta,
                controls::setter(&entity, |this: &mut Self, on| this.meta = on),
            ))
            .child(controls::toggle(
                "cmd-duration",
                "duration",
                self.duration,
                controls::setter(&entity, |this: &mut Self, on| this.duration = on),
            ));
        Body::new()
            .playground(div().w_full().max_w(gpui_kit::rems(40.)).child(block))
            .config(config)
            .example(controls::block("Reference preview", reference()))
    }
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(gpui_kit::rems(15.))
        .child(CommandOutputBlock::new("cmd-th", "pnpm build", "").exit_code(0))
        .into_any_element()
}
