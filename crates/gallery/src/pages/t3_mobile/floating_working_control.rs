//! T3 · Mobile › FloatingWorkingControl: the reference control (1m 14s,
//! two queued) ticking live, and a configurable one whose Stop pauses it.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div,
};
use u0ui::prelude::*;
use u0ui::t3::mobile::FloatingWorkingControl;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        queued: 2,
        running: true,
    })
    .into()
}

struct Page {
    queued: usize,
    running: bool,
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    FloatingWorkingControl::new("fwc-thumb", 74)
        .queued(2)
        .running(false)
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let live = FloatingWorkingControl::new("fwc-live", 74)
            .queued(self.queued)
            .running(self.running)
            .on_stop({
                let entity = entity.clone();
                move |_: &mut Window, cx: &mut App| {
                    entity.update(cx, |this, cx| {
                        this.running = false;
                        cx.notify();
                    })
                }
            });
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::field(
                "queued",
                controls::choices(
                    "fwc-queued",
                    &["0", "1", "2", "5"],
                    [0, 1, 2, 5]
                        .iter()
                        .position(|n| *n == self.queued)
                        .unwrap_or(0),
                    controls::setter(&entity, |this: &mut Page, i| this.queued = [0, 1, 2, 5][i]),
                ),
            ))
            .child(controls::toggle(
                "fwc-running",
                "timer running",
                self.running,
                controls::setter(&entity, |this: &mut Page, v| this.running = v),
            ));
        Body::new()
            .playground(live)
            .hint("Stop pauses the timer")
            .config(config)
            .example(controls::block(
                "Reference preview",
                FloatingWorkingControl::new("fwc-ref", 74).queued(2),
            ))
    }
}
