//! Ghost · Diagnostics › LogTroubleRow: a live row for each level and count,
//! and the three reference problems.
use crate::controls;
use crate::page::Body;
use crate::pages::ghost_settings::{list, note};
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div,
};
use u0ui::ghost::diagnostics::{LogLevel, LogTroubleRow};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        level: 0,
        count: 1,
        last: "—".into(),
    })
    .into()
}

struct Page {
    level: usize,
    count: usize,
    last: SharedString,
}

const LEVELS: [&str; 3] = ["error", "warning", "info"];
const COUNTS: [&str; 3] = ["1", "3", "12"];

/// `components/LogTroubleRow/preview.html`: three problems, 600px on `layer-01`.
fn reference() -> impl IntoElement {
    list(600.)
        .child(
            LogTroubleRow::new("log-0", "Process exited with code 137 (out of memory)")
                .level(LogLevel::Error)
                .source("agent/claude")
                .count(3)
                .time("10:42"),
        )
        .child(
            LogTroubleRow::new("log-1", "Reconnecting to Cloudflare edge")
                .level(LogLevel::Warning)
                .source("tunnel")
                .count(12)
                .time("10:31"),
        )
        .child(
            LogTroubleRow::new("log-2", "Update 1.4.3 downloaded")
                .level(LogLevel::Info)
                .source("updater")
                .time("09:00"),
        )
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    list(240.)
        .child(
            LogTroubleRow::new("log-thumb", "Process exited with code 137")
                .source("agent/claude")
                .count(3),
        )
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let fired = |what: &'static str| {
            let e = entity.clone();
            move |_: &mut Window, cx: &mut App| {
                e.update(cx, |p, cx| {
                    p.last = what.into();
                    cx.notify();
                })
            }
        };
        let live = div()
            .w_full()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_03)
            .child(
                list(600.).child(
                    LogTroubleRow::new("log-live", "Process exited with code 137 (out of memory)")
                        .level(LogLevel::ALL[self.level])
                        .source("agent/claude")
                        .count(COUNTS[self.count].parse().unwrap_or(1))
                        .time("10:42")
                        .on_open(fired("onOpen"))
                        .on_copy(fired("onCopy")),
                ),
            )
            .child(note(format!("Last: {}", self.last)));
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::field(
                "level",
                controls::choices(
                    "log-level",
                    &LEVELS,
                    self.level,
                    controls::setter(&entity, |p: &mut Page, i| p.level = i),
                ),
            ))
            .child(controls::field(
                "count",
                controls::choices(
                    "log-count",
                    &COUNTS,
                    self.count,
                    controls::setter(&entity, |p: &mut Page, i| p.count = i),
                ),
            ));
        Body::new()
            .playground(live)
            .config(config)
            .example(controls::block("Reference preview", reference()))
    }
}
