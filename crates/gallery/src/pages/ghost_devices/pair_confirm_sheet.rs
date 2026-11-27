//! Ghost · Devices › PairConfirmSheet: the reference request from
//! "iPhone 16", live (the control box and both buttons report back).
use crate::controls;
use crate::page::Body;
use crate::pages::ghost_settings::note;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, rems,
};
use u0ui::ghost::devices::PairConfirmSheet;
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        device: 0,
        last: "—".into(),
    })
    .into()
}

struct Page {
    device: usize,
    last: SharedString,
}

const DEVICES: [&str; 2] = ["iPhone 16", "studio-mac"];

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(
            div()
                .type_style(TypeStyle::Heading03)
                .text_color(color(ColorRole::TextPrimary))
                .child("Pair “iPhone 16”?"),
        )
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let (allow, deny) = (entity.clone(), entity.clone());
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_04)
            .child(
                PairConfirmSheet::new(("pair-confirm-live", self.device))
                    .device(DEVICES[self.device])
                    .fingerprint("A1F3 9C2E 7B04 51D8")
                    .on_allow(move |control, _, cx| {
                        allow.update(cx, |p, cx| {
                            p.last = format!("onAllow({control})").into();
                            cx.notify();
                        })
                    })
                    .on_deny(move |_, cx| {
                        deny.update(cx, |p, cx| {
                            p.last = "onDeny".into();
                            cx.notify();
                        })
                    }),
            )
            .child(note(format!("Last: {}", self.last)));
        let config = controls::field(
            "device",
            controls::choices(
                "pair-confirm-device",
                &DEVICES,
                self.device,
                controls::setter(&entity, |p: &mut Page, i| p.device = i),
            ),
        );
        Body::new()
            .playground(live)
            .hint("Control stays off unless the box is ticked")
            .config(config)
            .example(controls::block(
                "Reference preview",
                PairConfirmSheet::new("pair-confirm-ref")
                    .device("iPhone 16")
                    .fingerprint("A1F3 9C2E 7B04 51D8"),
            ))
    }
}
