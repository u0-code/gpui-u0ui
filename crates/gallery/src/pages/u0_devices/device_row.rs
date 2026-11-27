//! u0 · Devices › DeviceRow: a live paired device and the three
//! reference devices.
use crate::controls;
use crate::page::Body;
use crate::pages::u0_settings::{list, note};
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div,
};
use u0ui::prelude::*;
use u0ui::t3::MachineKind;
use u0ui::u0::devices::DeviceRow;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        kind: 4,
        current: false,
        control: true,
        revoked: 0,
    })
    .into()
}

struct Page {
    kind: usize,
    current: bool,
    control: bool,
    revoked: usize,
}

const KINDS: [&str; 5] = ["laptop", "desktop", "server", "cloud", "phone"];

/// `components/DeviceRow/preview.html`: three devices, 600px on `layer-01`
/// (`kind: 'tablet'` falls back to the laptop glyph).
fn reference() -> impl IntoElement {
    list(600.)
        .child(
            DeviceRow::new("dev-0", "MacBook Pro")
                .kind(MachineKind::Laptop)
                .fingerprint("5D21 …")
                .last_seen("Now")
                .version("1.4.2")
                .current(true),
        )
        .child(
            DeviceRow::new("dev-1", "iPhone 16")
                .kind(MachineKind::Phone)
                .fingerprint("A1F3 9C2E")
                .last_seen("2m ago")
                .version("1.4.0")
                .can_control(true),
        )
        .child(
            DeviceRow::new("dev-2", "iPad")
                .kind(MachineKind::Laptop)
                .fingerprint("77B0 13CE")
                .last_seen("3w ago")
                .version("1.2.1"),
        )
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    list(240.)
        .child(
            DeviceRow::new("dev-thumb", "iPhone 16")
                .fingerprint("A1F3 9C2E")
                .version("1.4.0"),
        )
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let revoke = entity.clone();
        let live = div()
            .w_full()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_03)
            .child(
                list(600.).child(
                    DeviceRow::new("dev-live", "iPhone 16")
                        .kind(MachineKind::ALL[self.kind])
                        .fingerprint("A1F3 9C2E")
                        .last_seen("2m ago")
                        .version("1.4.0")
                        .current(self.current)
                        .can_control(self.control)
                        .on_revoke(move |_, cx| {
                            revoke.update(cx, |p, cx| {
                                p.revoked += 1;
                                cx.notify();
                            })
                        }),
                ),
            )
            .child(note(format!("onRevoke × {}", self.revoked)));
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::field(
                "kind",
                controls::choices(
                    "dev-kind",
                    &KINDS,
                    self.kind,
                    controls::setter(&entity, |p: &mut Page, i| p.kind = i),
                ),
            ))
            .child(controls::toggle(
                "dev-current",
                "current",
                self.current,
                controls::setter(&entity, |p: &mut Page, v| p.current = v),
            ))
            .child(controls::toggle(
                "dev-control",
                "canControl",
                self.control,
                controls::setter(&entity, |p: &mut Page, v| p.control = v),
            ));
        Body::new()
            .playground(live)
            .config(config)
            .example(controls::block("Reference preview", reference()))
    }
}
