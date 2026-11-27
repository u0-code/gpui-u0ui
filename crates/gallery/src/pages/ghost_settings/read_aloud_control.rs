//! Ghost · Settings › ReadAloudControl: a live control (play / pause, voice,
//! speed) and the two reference controls.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div,
};
use u0ui::ghost::settings::ReadAloudControl;
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        playing: false,
        rate: 1,
        epoch: 0,
    })
    .into()
}

struct Page {
    playing: bool,
    rate: usize,
    epoch: usize,
}

const RATES: [&str; 6] = ["0.75", "1", "1.25", "1.5", "1.75", "2"];

/// `components/ReadAloudControl/preview.html`: two controls 16px apart.
fn reference() -> impl IntoElement {
    div()
        .flex()
        .flex_wrap()
        .gap(tokens::SPACING_05)
        .child(ReadAloudControl::new("read-0").voices(["Samantha", "Tingting", "Daniel"]))
        .child(
            ReadAloudControl::new("read-1")
                .voices(["Tingting"])
                .playing(true)
                .rate(1.25),
        )
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .child(ReadAloudControl::new("read-thumb").voices(["Samantha"]))
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let live = ReadAloudControl::new(("read-live", self.epoch))
            .voices(["Samantha", "Tingting", "Daniel"])
            .playing(self.playing)
            .rate(RATES[self.rate].parse().unwrap_or(1.));
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::toggle(
                "read-playing",
                "playing",
                self.playing,
                controls::setter(&entity, |p: &mut Page, v| {
                    p.playing = v;
                    p.epoch += 1;
                }),
            ))
            .child(controls::field(
                "rate",
                controls::choices(
                    "read-rate",
                    &RATES,
                    self.rate,
                    controls::setter(&entity, |p: &mut Page, i| {
                        p.rate = i;
                        p.epoch += 1;
                    }),
                ),
            ));
        Body::new()
            .playground(live)
            .hint("The speed button steps 0.25× up to 2×, then wraps to 0.75×")
            .config(config)
            .example(controls::block("Reference preview", reference()))
    }
}
