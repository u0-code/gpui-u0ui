//! T3 · Timeline › ThreadErrorBanner: the usage-limit error with two
//! actions and the unreachable-machine warning
//! (`components/ThreadErrorBanner/preview.html`), plus a configurable one.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div,
};
use u0ui::prelude::*;
use u0ui::t3::timeline::{ThreadBannerKind, ThreadErrorBanner};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| BannerPage {
        kind: 0,
        actions: true,
    })
    .into()
}

struct BannerPage {
    kind: usize,
    actions: bool,
}

fn usage(id: &'static str) -> ThreadErrorBanner {
    ThreadErrorBanner::new(ThreadBannerKind::Error, "Usage limit reached")
        .body("Send the request to your workspace admin, or continue with another model.")
        .action(
            Button::new(
                (gpui_kit::ElementId::from(id), "copy"),
                "Copy request for admin",
            )
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Sm),
        )
        .action(
            Button::new((gpui_kit::ElementId::from(id), "switch"), "Switch model")
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm),
        )
}

/// The reference preview (`cu-stack`, gap 8).
fn reference() -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_03)
        .w_full()
        .child(usage("tb-ref-0"))
        .child(
            ThreadErrorBanner::new(ThreadBannerKind::Warning, "build-box is unreachable")
                .body("Turn it back on, or move the thread to another computer."),
        )
}

impl Render for BannerPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let kind = ThreadBannerKind::ALL[self.kind];
        let mut banner = ThreadErrorBanner::new(
            kind,
            match kind {
                ThreadBannerKind::Error => "Codex exited unexpectedly",
                ThreadBannerKind::Warning => "build-box is unreachable",
            },
        )
        .body("The last turn was saved. Restart the agent to continue.");
        if self.actions {
            banner = banner.action(
                Button::new("tb-live-restart", "Restart agent")
                    .variant(ButtonVariant::Secondary)
                    .size(ButtonSize::Sm),
            );
        }
        let kinds = ThreadBannerKind::ALL.map(|k| k.name());
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "kind",
                controls::choices(
                    "tb-kind",
                    &kinds,
                    self.kind,
                    controls::setter(&entity, |this: &mut Self, i| this.kind = i),
                ),
            ))
            .child(controls::toggle(
                "tb-actions",
                "actions",
                self.actions,
                controls::setter(&entity, |this: &mut Self, on| this.actions = on),
            ));
        Body::new()
            .playground(div().w_full().child(banner))
            .config(config)
            .example(controls::block("Reference preview", reference()))
    }
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(gpui_kit::rems(15.))
        .child(
            ThreadErrorBanner::new(ThreadBannerKind::Warning, "build-box is unreachable")
                .body("Turn it back on."),
        )
        .into_any_element()
}
