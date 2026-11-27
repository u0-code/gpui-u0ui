//! u0 · Devices › PairingCard: a live card whose countdown runs (pick a
//! short expiry to watch it expire; "New link" makes a fresh one), and the
//! reference card.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, rems,
};
use u0ui::u0::devices::PairingCard;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        expiry: 0,
        serial: 0,
    })
    .into()
}

struct Page {
    expiry: usize,
    serial: usize,
}

const EXPIRY: [&str; 3] = ["600", "65", "5"];
const LINKS: [&str; 4] = [
    "u0://pair/7f3a-9c2e-41b0",
    "u0://pair/2b91-c07d-88e4",
    "u0://pair/e5a0-1f6b-93cd",
    "u0://pair/4c3e-a7d2-0b19",
];

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(
            div()
                .w(rems(35.))
                .child(PairingCard::new("pair-thumb").expires_in(600)),
        )
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let fresh = entity.clone();
        let live = div().w(rems(35.)).max_w_full().child(
            PairingCard::new("pair-live")
                .link(LINKS[self.serial % LINKS.len()])
                .expires_in(EXPIRY[self.expiry].parse().unwrap_or(600))
                .on_new_link(move |_, cx| {
                    fresh.update(cx, |p, cx| {
                        p.serial += 1;
                        cx.notify();
                    })
                }),
        );
        let config = controls::field(
            "expiresIn (s)",
            controls::choices(
                "pair-expiry",
                &EXPIRY,
                self.expiry,
                controls::setter(&entity, |p: &mut Page, i| p.expiry = i),
            ),
        );
        Body::new()
            .playground(live)
            .hint("At zero the QR code turns into “Link expired”")
            .config(config)
            .example(controls::block(
                "Reference preview",
                div()
                    .w(rems(35.))
                    .max_w_full()
                    .child(PairingCard::new("pair-ref").expires_in(600)),
            ))
    }
}
