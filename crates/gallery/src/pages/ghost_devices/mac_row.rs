//! Ghost · Devices › MacRow: a live Mac whose "Reach again" goes through
//! reaching to online, and the three reference Macs.
use crate::controls;
use crate::page::Body;
use crate::pages::ghost_settings::{list, note};
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, prelude::FluentBuilder as _,
};
use std::time::Duration;
use u0ui::ghost::devices::{MacRow, MacState};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        state: 3,
        latency: true,
        last: "—".into(),
    })
    .into()
}

struct Page {
    state: usize,
    latency: bool,
    last: SharedString,
}

const STATES: [&str; 4] = ["online", "reaching", "turned-away", "offline"];

/// `components/MacRow/preview.html`: three Macs, 600px on `layer-01`.
fn reference() -> impl IntoElement {
    list(600.)
        .child(
            MacRow::new("mac-0", "MacBook Pro")
                .state(MacState::Online)
                .version("1.4.2")
                .latency(12),
        )
        .child(
            MacRow::new("mac-1", "build-box")
                .state(MacState::Offline)
                .version("1.4.1"),
        )
        .child(
            MacRow::new("mac-2", "studio-mac")
                .state(MacState::TurnedAway)
                .version("1.3.9"),
        )
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    list(240.)
        .child(
            MacRow::new("mac-thumb", "MacBook Pro")
                .version("1.4.2")
                .latency(12),
        )
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let reach = entity.clone();
        let forget = entity.clone();
        let live = div()
            .w_full()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_03)
            .child(
                list(600.).child(
                    MacRow::new("mac-live", "build-box")
                        .state(MacState::ALL[self.state])
                        .version("1.4.1")
                        .when(self.latency, |r| r.latency(24))
                        .on_reach(move |window, cx| {
                            reach.update(cx, |p, cx| {
                                p.state = 1;
                                p.last = "onReach".into();
                                cx.notify();
                            });
                            // Reaching settles to online after a moment.
                            let e = reach.downgrade();
                            window
                                .spawn(cx, async move |cx| {
                                    cx.background_executor()
                                        .timer(Duration::from_millis(1500))
                                        .await;
                                    let _ = e.update(cx, |p, cx| {
                                        p.state = 0;
                                        cx.notify();
                                    });
                                })
                                .detach();
                        })
                        .on_forget(move |_, cx| {
                            forget.update(cx, |p, cx| {
                                p.last = "onForget".into();
                                cx.notify();
                            })
                        }),
                ),
            )
            .child(note(format!("Last: {}", self.last)));
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::field(
                "state",
                controls::choices(
                    "mac-state",
                    &STATES,
                    self.state,
                    controls::setter(&entity, |p: &mut Page, i| p.state = i),
                ),
            ))
            .child(controls::toggle(
                "mac-latency",
                "latency",
                self.latency,
                controls::setter(&entity, |p: &mut Page, v| p.latency = v),
            ));
        Body::new()
            .playground(live)
            .hint("Reach again shows the spinner, then the Mac comes online")
            .config(config)
            .example(controls::block("Reference preview", reference()))
    }
}
