//! T3 · Thread › ThreadDetailsPanel: the reference panel (Pull requests,
//! Context, Servers, a closed Linked issues) with sections that collapse,
//! and a live ContextMeter.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, rems,
};
use u0ui::prelude::*;
use u0ui::t3::review::{
    ChecksStatus, ContextMeter, PullRequestRow, PullRequestState, ThreadDetailsSection,
};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page { used: 1 }).into()
}

struct Page {
    used: usize,
}

const USED: [u64; 4] = [40_000, 142_000, 186_000, 200_000];

fn panel(prefix: &'static str, used: u64) -> gpui_kit::Div {
    // `width: 340; background: var(--layer-01)`.
    div()
        .flex()
        .flex_col()
        .w(rems(21.25))
        .bg(color(ColorRole::Layer01))
        .child(
            ThreadDetailsSection::new((prefix, 0usize), "Pull requests")
                .count(1)
                .child(
                    PullRequestRow::new((prefix, 1usize), 482, "Dark theme tokens")
                        .state(PullRequestState::Open)
                        .author("jd")
                        .branches("feat/dark-tokens", "main")
                        .checks(ChecksStatus::Running)
                        .time("2h"),
                ),
        )
        .child(
            ThreadDetailsSection::new((prefix, 2usize), "Context")
                .child(ContextMeter::new(used, 200_000)),
        )
        .child(ThreadDetailsSection::new((prefix, 3usize), "Servers").count(2))
        .child(
            ThreadDetailsSection::new((prefix, 4usize), "Linked issues")
                .count(0)
                .default_open(false),
        )
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .bg(color(ColorRole::Layer01))
        .child(
            ThreadDetailsSection::new("tdp-thumb", "Context")
                .child(ContextMeter::new(142_000, 200_000)),
        )
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let config = controls::field(
            "ContextMeter used",
            controls::choices(
                "tdp-used",
                &["40k", "142k", "186k", "200k"],
                self.used,
                controls::setter(&entity, |this: &mut Page, i| this.used = i),
            ),
        );
        Body::new()
            .playground(panel("tdp-live", USED[self.used]))
            .hint("Section heads collapse and expand their bodies")
            .config(config)
            .example(controls::block(
                "Reference preview",
                panel("tdp-ref", 142_000),
            ))
    }
}
