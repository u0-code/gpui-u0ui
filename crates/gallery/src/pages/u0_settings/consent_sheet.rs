//! u0 · Settings › ConsentSheet: the first-run consent, live (the slider
//! repaints the payload; both buttons report the level they pass).
use super::note;
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, rems,
};
use u0ui::prelude::*;
use u0ui::u0::settings::ConsentSheet;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        app: 0,
        last: "—".into(),
    })
    .into()
}

struct Page {
    app: usize,
    last: SharedString,
}

const APPS: [&str; 2] = ["u0", "Carbon Desk"];

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(
            div()
                .type_style(TypeStyle::Heading03)
                .text_color(color(ColorRole::TextPrimary))
                .child("Welcome to u0"),
        )
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let done = entity.clone();
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_04)
            .child(
                ConsentSheet::new(("consent-live", self.app))
                    .app_name(APPS[self.app])
                    .on_continue(move |level, _, cx| {
                        done.update(cx, |p, cx| {
                            p.last = format!("onContinue({level})").into();
                            cx.notify();
                        })
                    }),
            )
            .child(note(format!("Last: {}", self.last)));
        let config = controls::field(
            "appName",
            controls::choices(
                "consent-app",
                &APPS,
                self.app,
                controls::setter(&entity, |p: &mut Page, i| p.app = i),
            ),
        );
        Body::new()
            .playground(live)
            .hint("Drag or arrow the slider; the JSON follows the level")
            .config(config)
            .example(controls::block(
                "Reference preview",
                ConsentSheet::new("consent-ref").app_name("u0"),
            ))
    }
}
