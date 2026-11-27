//! Ghost · Settings › TelemetryPreview: what each telemetry level would
//! send, live (pick a level, copy the JSON), and the reference level-2
//! payload.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, rems,
};
use u0ui::ghost::settings::{Payload, TelemetryLevel, TelemetryPreview};
use u0ui::prelude::*;

const LEVELS: [&str; 4] = ["1", "2", "3", "Crash reports"];

/// `components/TelemetryPreview/preview.html`'s payload (level 2).
pub fn reference_payload() -> Payload {
    Payload::object([
        ("app", Payload::str("1.4.2")),
        ("os", Payload::str("macOS 15.2")),
        (
            "crash",
            Payload::object([
                ("signal", Payload::str("SIGABRT")),
                ("frames", Payload::Number(12.)),
            ]),
        ),
        (
            "features",
            Payload::object([
                ("palette", Payload::Number(14.)),
                ("sideChat", Payload::Number(3.)),
            ]),
        ),
    ])
}

/// The demo payload of each level: 1 crash only, 2 adds feature counts
/// (the reference), 3 adds timings; the named level is the crash report.
fn payload(level: usize) -> Payload {
    let crash = || {
        Payload::object([
            ("signal", Payload::str("SIGABRT")),
            ("frames", Payload::Number(12.)),
        ])
    };
    match level {
        0 | 3 => Payload::object([
            ("app", Payload::str("1.4.2")),
            ("os", Payload::str("macOS 15.2")),
            ("crash", crash()),
        ]),
        1 => reference_payload(),
        _ => Payload::object([
            ("app", Payload::str("1.4.2")),
            ("os", Payload::str("macOS 15.2")),
            ("crash", crash()),
            (
                "features",
                Payload::object([
                    ("palette", Payload::Number(14.)),
                    ("sideChat", Payload::Number(3.)),
                ]),
            ),
            (
                "timings",
                Payload::object([
                    ("launchMs", Payload::Number(412.)),
                    ("firstTokenMs", Payload::Number(980.)),
                ]),
            ),
            (
                "agents",
                Payload::Array(vec![Payload::str("claude"), Payload::str("codex")]),
            ),
            ("optedIn", Payload::Bool(true)),
        ]),
    }
}

fn level(i: usize) -> TelemetryLevel {
    match i {
        0..=2 => TelemetryLevel::Number(i as u32 + 1),
        _ => TelemetryLevel::Named(LEVELS[3].into()),
    }
}

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| TelemetryPreviewPage { level: 1 }).into()
}

struct TelemetryPreviewPage {
    level: usize,
}

/// `components/TelemetryPreview/preview.html`: 480px wide.
pub fn reference(id: &'static str) -> impl IntoElement {
    div()
        .w(rems(30.))
        .max_w_full()
        .child(TelemetryPreview::new(id, 2, reference_payload()))
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(TelemetryPreview::new("thumb-tele", 1, payload(0)))
        .into_any_element()
}

impl Render for TelemetryPreviewPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let live = div().w(rems(30.)).max_w_full().child(TelemetryPreview::new(
            ("tele-live", self.level),
            level(self.level),
            payload(self.level),
        ));
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::field(
                "level",
                controls::choices(
                    "tele-level",
                    &LEVELS,
                    self.level,
                    controls::setter(&entity, |this: &mut Self, i| this.level = i),
                ),
            ));
        Body::new()
            .playground(live)
            .hint("Level 3 scrolls past 10rem; the copy button copies the JSON")
            .config(config)
            .example(controls::block("Reference preview", reference("tele-ref")))
    }
}
