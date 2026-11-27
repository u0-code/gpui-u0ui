//! T3 · Timeline › AssistantMessage: the G90 reply with inline code, from
//! Claude · Opus (`components/AssistantMessage/preview.html`), plus a live
//! reply whose meta parts toggle.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div,
};
use u0ui::prelude::*;
use u0ui::t3::timeline::{AssistantMessage, Prose};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| AssistantPage {
        provider: true,
        elapsed: true,
        time: true,
        last: "".into(),
    })
    .into()
}

struct AssistantPage {
    provider: bool,
    elapsed: bool,
    time: bool,
    last: SharedString,
}

fn body(m: AssistantMessage) -> AssistantMessage {
    m.child(Prose::markdown(
        "I added the `g90` theme with 217 tokens from @carbon/themes and regenerated the cover.",
    ))
    .child(Prose::markdown(
        "All text tokens pass 4.5:1 on their documented grounds in both dark themes.",
    ))
}

impl Render for AssistantPage {
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
        let mut m = AssistantMessage::new("amsg-live").model("Opus");
        if self.provider {
            m = m.provider("Claude");
        }
        if self.elapsed {
            m = m.elapsed(134);
        }
        if self.time {
            m = m.time("10:44");
        }
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .w_full()
            .max_w(gpui_kit::rems(45.))
            .child(
                body(m)
                    .on_copy(note("onCopy"))
                    .on_reply(note("onReply"))
                    .on_fork(note("onFork")),
            )
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
                "amsg-provider",
                "provider",
                self.provider,
                controls::setter(&entity, |this: &mut Self, on| this.provider = on),
            ))
            .child(controls::toggle(
                "amsg-elapsed",
                "elapsed",
                self.elapsed,
                controls::setter(&entity, |this: &mut Self, on| this.elapsed = on),
            ))
            .child(controls::toggle(
                "amsg-time",
                "time",
                self.time,
                controls::setter(&entity, |this: &mut Self, on| this.time = on),
            ));
        Body::new()
            .playground(live)
            .config(config)
            .example(controls::block(
                "Reference preview",
                div().w_full().max_w(gpui_kit::rems(45.)).child(body(
                    AssistantMessage::new("amsg-ref")
                        .provider("Claude")
                        .model("Opus")
                        .elapsed(134)
                        .time("10:44"),
                )),
            ))
    }
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(gpui_kit::rems(15.))
        .child(
            AssistantMessage::new("amsg-th")
                .provider("Claude")
                .model("Opus")
                .child(Prose::markdown("I added the `g90` theme.")),
        )
        .into_any_element()
}
