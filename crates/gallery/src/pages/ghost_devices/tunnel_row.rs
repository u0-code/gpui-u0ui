//! Ghost · Devices › TunnelRow: a live tunnel that starts (through
//! "Starting…") and stops, and the four reference states.
use crate::controls;
use crate::page::Body;
use crate::pages::ghost_settings::list;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Window,
};
use std::time::Duration;
use u0ui::ghost::devices::{TunnelRow, TunnelState};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page { state: 0 }).into()
}

struct Page {
    state: usize,
}

const STATES: [&str; 5] = ["running", "starting", "stopped", "error", "missing"];
const ADDRESS: &str = "https://quiet-river-1234.trycloudflare.com";

/// `components/TunnelRow/preview.html`: four states, 600px on `layer-01`.
fn reference() -> impl IntoElement {
    list(600.)
        .child(
            TunnelRow::new("tun-0")
                .state(TunnelState::Running)
                .address(ADDRESS)
                .uptime("3h 12m"),
        )
        .child(TunnelRow::new("tun-1").state(TunnelState::Starting))
        .child(TunnelRow::new("tun-2").state(TunnelState::Stopped))
        .child(TunnelRow::new("tun-3").state(TunnelState::Missing))
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    list(240.)
        .child(TunnelRow::new("tun-thumb").state(TunnelState::Stopped))
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let (start, stop) = (entity.clone(), entity.clone());
        let live = list(600.).child(
            TunnelRow::new("tun-live")
                .state(TunnelState::ALL[self.state])
                .address(ADDRESS)
                .uptime("3h 12m")
                .on_start(move |window, cx| {
                    start.update(cx, |p, cx| {
                        p.state = 1;
                        cx.notify();
                    });
                    let e = start.downgrade();
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
                .on_stop(move |_, cx| {
                    stop.update(cx, |p, cx| {
                        p.state = 2;
                        cx.notify();
                    })
                }),
        );
        let config = controls::field(
            "state",
            controls::choices(
                "tun-state",
                &STATES,
                self.state,
                controls::setter(&entity, |p: &mut Page, i| p.state = i),
            ),
        );
        Body::new()
            .playground(live)
            .hint("Stop, then Start: it reports Starting… before it runs again")
            .config(config)
            .example(controls::block("Reference preview", reference()))
    }
}
