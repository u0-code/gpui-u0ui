//! T3 · Timeline › TimelineDivider: model switch with a time, new session,
//! hand-off, compaction and the red "new since your last visit" line
//! (`components/TimelineDivider/preview.html`), plus every kind live.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div,
};
use u0ui::prelude::*;
use u0ui::t3::timeline::{DividerKind, TimelineDivider};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| DividerPage {
        kind: 0,
        time: true,
    })
    .into()
}

struct DividerPage {
    kind: usize,
    time: bool,
}

/// The reference preview (`cu-stack`, gap 16, max 640px).
fn reference() -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_05)
        .w_full()
        .max_w(gpui_kit::rems(40.))
        .child(TimelineDivider::new(DividerKind::Model, "Switched to Sonnet").time("10:51"))
        .child(TimelineDivider::new(
            DividerKind::Session,
            "Fresh session · context cleared",
        ))
        .child(TimelineDivider::new(
            DividerKind::Handoff,
            "Handed off from Codex",
        ))
        .child(TimelineDivider::new(
            DividerKind::Compact,
            "Context compacted",
        ))
        .child(TimelineDivider::new(
            DividerKind::Now,
            "New since your last visit",
        ))
}

impl Render for DividerPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let kind = DividerKind::ALL[self.kind];
        let mut divider = TimelineDivider::new(
            kind,
            match kind {
                DividerKind::Model => "Switched to Sonnet",
                DividerKind::Session => "Fresh session · context cleared",
                DividerKind::Handoff => "Handed off from Codex",
                DividerKind::Now => "New since your last visit",
                DividerKind::Compact => "Context compacted",
                DividerKind::Worktree => "Moved to worktree feature/tokens",
                DividerKind::Unread => "3 unread messages",
            },
        );
        if self.time {
            divider = divider.time("10:51");
        }
        let kinds = DividerKind::ALL.map(|k| k.name());
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "kind",
                controls::choices(
                    "div-kind",
                    &kinds,
                    self.kind,
                    controls::setter(&entity, |this: &mut Self, i| this.kind = i),
                ),
            ))
            .child(controls::toggle(
                "div-time",
                "time",
                self.time,
                controls::setter(&entity, |this: &mut Self, on| this.time = on),
            ));
        Body::new()
            .playground(div().w_full().max_w(gpui_kit::rems(40.)).child(divider))
            .config(config)
            .example(controls::block("Reference preview", reference()))
    }
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(gpui_kit::rems(15.))
        .flex()
        .flex_col()
        .gap(tokens::SPACING_04)
        .child(TimelineDivider::new(
            DividerKind::Model,
            "Switched to Sonnet",
        ))
        .child(TimelineDivider::new(DividerKind::Now, "New"))
        .into_any_element()
}
