//! u0 · Shell server pages: ServerHeader (the sidebar group title per
//! Mac) and ServerChip (the small server tag).
use super::{KIND_NAMES, KINDS, last_action};
use crate::controls;
use crate::page::Body;
use crate::pages::t3_shell::{eid, panel, row, stack};
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, px,
};
use u0ui::prelude::*;
use u0ui::t3::shell::MachineKind;
use u0ui::u0::shell::{ServerChip, ServerChipSize, ServerHeader, ServerState};

const STATE_NAMES: [&str; 4] = ["online", "offline", "turned-away", "reaching"];
const SIZE_NAMES: [&str; 2] = ["md", "sm"];

// ── ServerHeader ────────────────────────────────────────────────────────

struct HeaderPage {
    state: usize,
    kind: usize,
    version: bool,
    count: bool,
    expanded: bool,
    action: SharedString,
}

/// Build the ServerHeader page.
pub fn build_server_header(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| HeaderPage {
        state: 1,
        kind: 2,
        version: true,
        count: true,
        expanded: true,
        action: "none".into(),
    })
    .into()
}

/// `components/ServerHeader/preview.html`: four servers in a 320px column
/// on `layer-01`.
fn header_reference(prefix: &str) -> impl IntoElement {
    let id = |n: &str| eid(prefix, n);
    panel(320.)
        .child(
            ServerHeader::new(id("a"), "MacBook Pro")
                .state(ServerState::Online)
                .version("1.4.2")
                .count(6),
        )
        .child(
            ServerHeader::new(id("b"), "build-box")
                .kind(MachineKind::Server)
                .state(ServerState::Offline)
                .reason("Not reachable since 10:42. It may be asleep.")
                .count(3),
        )
        .child(
            ServerHeader::new(id("c"), "studio-mac")
                .kind(MachineKind::Desktop)
                .state(ServerState::TurnedAway)
                .reason("It turned this Mac away: pairing was revoked."),
        )
        .child(
            ServerHeader::new(id("d"), "mini")
                .kind(MachineKind::Desktop)
                .state(ServerState::Reaching),
        )
}

/// Overview specimen.
pub fn header_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    panel(240.)
        .child(
            ServerHeader::new("thumb-shead-a", "MacBook Pro")
                .version("1.4.2")
                .count(6),
        )
        .child(
            ServerHeader::new("thumb-shead-b", "mini")
                .kind(MachineKind::Desktop)
                .state(ServerState::Reaching),
        )
        .into_any_element()
}

fn reason_for(state: ServerState) -> &'static str {
    match state {
        ServerState::TurnedAway => "It turned this Mac away: pairing was revoked.",
        _ => "Not reachable since 10:42. It may be asleep.",
    }
}

impl Render for HeaderPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let state = ServerState::ALL[self.state];
        let mut header = ServerHeader::new("live-shead", "build-box")
            .kind(KINDS[self.kind])
            .state(state)
            .reason(reason_for(state))
            .expanded(self.expanded)
            .on_toggle({
                let entity = entity.clone();
                move |_, cx| {
                    entity.update(cx, |this, cx| {
                        this.expanded = !this.expanded;
                        this.action = "Toggle".into();
                        cx.notify();
                    })
                }
            })
            .on_reach({
                let entity = entity.clone();
                move |_, cx| {
                    entity.update(cx, |this, cx| {
                        // Reaching again: it is connecting now.
                        this.state = 3;
                        this.action = "Reach again".into();
                        cx.notify();
                    })
                }
            });
        if self.version {
            header = header.version("1.4.2");
        }
        if self.count {
            header = header.count(3);
        }
        let live = stack()
            .gap(px(16.))
            .child(panel(320.).child(header))
            .child(last_action(&self.action));
        let config = stack()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "state",
                controls::choices(
                    "shead-state",
                    &STATE_NAMES,
                    self.state,
                    controls::setter(&entity, |this: &mut HeaderPage, i| this.state = i),
                ),
            ))
            .child(controls::field(
                "kind",
                controls::choices(
                    "shead-kind",
                    &KIND_NAMES,
                    self.kind,
                    controls::setter(&entity, |this: &mut HeaderPage, i| this.kind = i),
                ),
            ))
            .child(controls::toggle(
                "shead-version",
                "version (shown while online)",
                self.version,
                controls::setter(&entity, |this: &mut HeaderPage, on| this.version = on),
            ))
            .child(controls::toggle(
                "shead-count",
                "count",
                self.count,
                controls::setter(&entity, |this: &mut HeaderPage, on| this.count = on),
            ))
            .child(controls::toggle(
                "shead-expanded",
                "expanded",
                self.expanded,
                controls::setter(&entity, |this: &mut HeaderPage, on| this.expanded = on),
            ));
        Body::new()
            .playground(live)
            .hint("Click the name to fold the group; Reach again starts connecting")
            .config(config)
            .example(controls::block(
                "Reference preview",
                header_reference("ref-shead"),
            ))
    }
}

// ── ServerChip ──────────────────────────────────────────────────────────

struct ChipPage {
    state: usize,
    kind: usize,
    size: usize,
    clickable: bool,
    clicks: usize,
}

/// Build the ServerChip page.
pub fn build_server_chip(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| ChipPage {
        state: 0,
        kind: 2,
        size: 0,
        clickable: true,
        clicks: 0,
    })
    .into()
}

/// `components/ServerChip/preview.html`: five chips in a wrapping row, 8px
/// apart (`{ display: flex; gap: 8; flexWrap: wrap }`). The row's default
/// `align-items: stretch` leaves the 16px small chip (fixed height) at the
/// top of the 20px line, not centred.
fn chip_reference(prefix: &str) -> impl IntoElement {
    let id = |n: &str| eid(prefix, n);
    row()
        .items_start()
        .gap(tokens::SPACING_03)
        .child(ServerChip::new(id("a"), "MacBook Pro"))
        .child(
            ServerChip::new(id("b"), "build-box")
                .kind(MachineKind::Server)
                .state(ServerState::Offline),
        )
        .child(
            ServerChip::new(id("c"), "studio-mac")
                .kind(MachineKind::Desktop)
                .state(ServerState::TurnedAway),
        )
        .child(
            ServerChip::new(id("d"), "mini")
                .kind(MachineKind::Desktop)
                .state(ServerState::Reaching)
                .on_click(|_, _| {}),
        )
        .child(
            ServerChip::new(id("e"), "build-box")
                .kind(MachineKind::Server)
                .size(ServerChipSize::Sm),
        )
}

/// Overview specimen.
pub fn chip_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    row()
        .gap(tokens::SPACING_03)
        .child(ServerChip::new("thumb-chip-a", "MacBook Pro"))
        .child(
            ServerChip::new("thumb-chip-b", "build-box")
                .kind(MachineKind::Server)
                .state(ServerState::Offline),
        )
        .into_any_element()
}

impl Render for ChipPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let mut chip = ServerChip::new("live-chip", "build-box")
            .kind(KINDS[self.kind])
            .state(ServerState::ALL[self.state])
            .size(ServerChipSize::ALL[self.size]);
        if self.clickable {
            let entity = entity.clone();
            chip = chip.on_click(move |_, cx| {
                entity.update(cx, |this, cx| {
                    this.clicks += 1;
                    cx.notify();
                })
            });
        }
        let live = stack().gap(px(16.)).child(div().flex().child(chip)).child(
            div()
                .type_style(TypeStyle::Label01)
                .text_color(color(ColorRole::TextSecondary))
                .child(format!("Clicked {} times", self.clicks)),
        );
        let config = stack()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "state",
                controls::choices(
                    "chip-state",
                    &STATE_NAMES,
                    self.state,
                    controls::setter(&entity, |this: &mut ChipPage, i| this.state = i),
                ),
            ))
            .child(controls::field(
                "kind",
                controls::choices(
                    "chip-kind",
                    &KIND_NAMES,
                    self.kind,
                    controls::setter(&entity, |this: &mut ChipPage, i| this.kind = i),
                ),
            ))
            .child(controls::field(
                "size",
                controls::choices(
                    "chip-size",
                    &SIZE_NAMES,
                    self.size,
                    controls::setter(&entity, |this: &mut ChipPage, i| this.size = i),
                ),
            ))
            .child(controls::toggle(
                "chip-click",
                "onClick (renders a button)",
                self.clickable,
                controls::setter(&entity, |this: &mut ChipPage, on| this.clickable = on),
            ));
        Body::new()
            .playground(live)
            .hint("Hover for name · state; with onClick it is a focusable button")
            .config(config)
            .example(controls::block(
                "Reference preview",
                chip_reference("ref-chip"),
            ))
    }
}
