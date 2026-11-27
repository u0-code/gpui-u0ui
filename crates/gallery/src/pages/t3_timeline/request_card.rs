//! T3 · Timeline › RequestCard: "Link another computer?" with build-box in
//! the preview well (`components/RequestCard/preview.html`), plus a live
//! card whose actions report back.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div,
};
use u0ui::prelude::*;
use u0ui::t3::shell::MachineKind;
use u0ui::t3::timeline::RequestCard;
use u0ui::t3::workspace::EnvironmentRow;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| RequestPage {
        preview: true,
        body: true,
        last: "".into(),
    })
    .into()
}

struct RequestPage {
    preview: bool,
    body: bool,
    last: SharedString,
}

const BODY: &str = "The agent wants to run tests on build-box. Linking lets this thread use it.";

fn build_box(id: &'static str) -> EnvironmentRow {
    EnvironmentRow::new(id, "build-box")
        .kind(MachineKind::Server)
        .transport("SSH")
        .detail("10.0.4.12")
}

impl Render for RequestPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let note = |what: &'static str| {
            let entity = entity.clone();
            move |_: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    this.last = what.into();
                    cx.notify();
                })
            }
        };
        let mut card = RequestCard::new(
            "req-live",
            "Link another computer?",
            "Link computer",
            "Decline",
        )
        .on_primary(note("onPrimary"))
        .on_secondary(note("onSecondary"));
        if self.body {
            card = card.body(BODY);
        }
        if self.preview {
            card = card.preview(build_box("req-live-env"));
        }
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .w_full()
            .max_w(gpui_kit::rems(35.))
            .child(card)
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(self.last.clone()),
            );
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::toggle(
                "req-body",
                "children",
                self.body,
                controls::setter(&entity, |this: &mut Self, on| this.body = on),
            ))
            .child(controls::toggle(
                "req-preview",
                "preview",
                self.preview,
                controls::setter(&entity, |this: &mut Self, on| this.preview = on),
            ));
        Body::new()
            .playground(live)
            .config(config)
            .example(controls::block(
                "Reference preview",
                div().w_full().max_w(gpui_kit::rems(35.)).child(
                    RequestCard::new(
                        "req-ref",
                        "Link another computer?",
                        "Link computer",
                        "Decline",
                    )
                    .body(BODY)
                    .preview(build_box("req-ref-env")),
                ),
            ))
    }
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(gpui_kit::rems(15.))
        .child(RequestCard::new(
            "req-th",
            "Link another computer?",
            "Allow",
            "Decline",
        ))
        .into_any_element()
}
