//! T3 · Mobile › MobileNotice: the reference notices (warning with body,
//! info, error with Retry) over a LoadingStrip, and a live notice.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, prelude::FluentBuilder as _, rems,
};
use u0ui::components::{Button, ButtonSize, ButtonVariant};
use u0ui::prelude::*;
use u0ui::t3::mobile::{LoadingStrip, MobileNotice, NoticeTone};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        tone: 1,
        body: true,
        action: false,
    })
    .into()
}

struct Page {
    tone: usize,
    body: bool,
    action: bool,
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
            MobileNotice::new(NoticeTone::Warning, "build-box is offline")
                .body("Threads on it will resume when it reconnects."),
        )
        .child(MobileNotice::new(NoticeTone::Info, "Using cellular data"))
        .child(
            MobileNotice::new(NoticeTone::Error, "Could not load pull requests").action(
                Button::new((prefix, 0usize), "Retry")
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm),
            ),
        )
        .child(LoadingStrip::new((prefix, 1usize)))
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    stack()
        .w(rems(15.))
        .child(MobileNotice::new(
            NoticeTone::Warning,
            "build-box is offline",
        ))
        .child(LoadingStrip::new("mn-thumb-strip"))
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let live = stack().child(
            MobileNotice::new(NoticeTone::ALL[self.tone], "build-box is offline")
                .when(self.body, |n| {
                    n.body("Threads on it will resume when it reconnects.")
                })
                .when(self.action, |n| {
                    n.action(
                        Button::new("mn-live-action", "Retry")
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::Sm),
                    )
                }),
        );
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::field(
                "tone",
                controls::choices(
                    "mn-tone",
                    &["info", "warning", "error", "success"],
                    self.tone,
                    controls::setter(&entity, |this: &mut Page, i| this.tone = i),
                ),
            ))
            .child(controls::toggle(
                "mn-body",
                "children",
                self.body,
                controls::setter(&entity, |this: &mut Page, v| this.body = v),
            ))
            .child(controls::toggle(
                "mn-action",
                "action",
                self.action,
                controls::setter(&entity, |this: &mut Page, v| this.action = v),
            ));
        Body::new()
            .playground(live)
            .config(config)
            .example(controls::block("Reference preview", reference("mn-ref")))
    }
}
