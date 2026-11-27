//! u0 · Thread › TurnFooter: the reference footer
//! (`components/TurnFooter/preview.html`), live: each button reports its
//! click, and every piece of metadata can be turned off.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, prelude::FluentBuilder as _,
};
use u0ui::prelude::*;
use u0ui::u0::thread::TurnFooter;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| TurnFooterPage {
        model: true,
        tokens: true,
        time: true,
        last: None,
    })
    .into()
}

struct TurnFooterPage {
    model: bool,
    tokens: bool,
    time: bool,
    last: Option<&'static str>,
}

/// `h(C.TurnFooter, { model: 'claude-sonnet', tokens: '12.4k tokens', time: '10:43' })`.
fn reference(id: &'static str) -> TurnFooter {
    TurnFooter::new(id)
        .model("claude-sonnet")
        .tokens("12.4k tokens")
        .time("10:43")
}

impl Render for TurnFooterPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let report = |what: &'static str| {
            let entity = entity.clone();
            move |_: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    this.last = Some(what);
                    cx.notify();
                })
            }
        };
        let footer = TurnFooter::new("turn-footer-live")
            .when(self.model, |f| f.model("claude-sonnet"))
            .when(self.tokens, |f| f.tokens("12.4k tokens"))
            .when(self.time, |f| f.time("10:43"))
            .on_copy(report("onCopy"))
            .on_fork(report("onFork"))
            .on_retry(report("onRetry"));
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(footer)
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(match self.last {
                        Some(what) => format!("Last action: {what}"),
                        None => "No action yet".to_string(),
                    }),
            );
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::toggle(
                "turn-footer-model",
                "model",
                self.model,
                controls::setter(&entity, |this: &mut Self, on| this.model = on),
            ))
            .child(controls::toggle(
                "turn-footer-tokens",
                "tokens",
                self.tokens,
                controls::setter(&entity, |this: &mut Self, on| this.tokens = on),
            ))
            .child(controls::toggle(
                "turn-footer-time",
                "time",
                self.time,
                controls::setter(&entity, |this: &mut Self, on| this.time = on),
            ));
        Body::new()
            .playground(live)
            .hint("Hover or Tab into the row to bring it to full opacity.")
            .config(config)
            .example(controls::block(
                "Reference preview",
                reference("turn-footer-reference"),
            ))
    }
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    reference("turn-footer-thumb").into_any_element()
}
