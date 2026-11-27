//! u0 · Settings › ScriptRow: a live script row (Run starts it, Stop
//! stops it) with its kind, shortcut and state switchable, and the
//! reference's three scripts.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, rems,
};
use u0ui::prelude::*;
use u0ui::u0::settings::ScriptRow;

const KINDS: [&str; 4] = ["run", "test", "setup", "build"];

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| ScriptRowPage {
        kind: 0,
        shortcut: true,
        running: true,
        last: "Running".into(),
    })
    .into()
}

struct ScriptRowPage {
    kind: usize,
    shortcut: bool,
    running: bool,
    last: SharedString,
}

/// `components/ScriptRow/preview.html`: three scripts, 600px on `layer-01`.
pub fn reference(prefix: &'static str) -> impl IntoElement {
    div()
        .w(rems(37.5))
        .max_w_full()
        .bg(color(ColorRole::Layer01))
        .child(
            ScriptRow::new((prefix, 0usize), "Dev server", "pnpm dev --port 5173")
                .kind("run")
                .shortcut("⌘ R")
                .running(true),
        )
        .child(ScriptRow::new((prefix, 1usize), "Test", "pnpm test").kind("test"))
        .child(
            ScriptRow::new(
                (prefix, 2usize),
                "Setup worktree",
                "pnpm install && cp .env.example .env",
            )
            .kind("setup"),
        )
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .bg(color(ColorRole::Layer01))
        .child(ScriptRow::new("thumb-script", "Test", "pnpm test").kind("test"))
        .into_any_element()
}

impl Render for ScriptRowPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let set_running = |on: bool, what: &'static str| {
            let e = entity.clone();
            move |_: &mut Window, cx: &mut App| {
                e.update(cx, |this, cx| {
                    this.running = on;
                    this.last = what.into();
                    cx.notify();
                })
            }
        };
        let edit = {
            let e = entity.clone();
            move |_: &mut Window, cx: &mut App| {
                e.update(cx, |this, cx| {
                    this.last = "Edit pressed".into();
                    cx.notify();
                })
            }
        };
        let mut row = ScriptRow::new("script-live", "Dev server", "pnpm dev --port 5173")
            .kind(KINDS[self.kind])
            .running(self.running)
            .on_run(set_running(true, "Running"))
            .on_stop(set_running(false, "Stopped"))
            .on_edit(edit);
        if self.shortcut {
            row = row.shortcut("⌘ R");
        }
        let live = div()
            .w(rems(37.5))
            .max_w_full()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_03)
            .child(div().bg(color(ColorRole::Layer01)).child(row))
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(self.last.clone()),
            );
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::field(
                "kind",
                controls::choices(
                    "script-kind",
                    &KINDS,
                    self.kind,
                    controls::setter(&entity, |this: &mut Self, i| this.kind = i),
                ),
            ))
            .child(controls::toggle(
                "script-shortcut",
                "shortcut",
                self.shortcut,
                controls::setter(&entity, |this: &mut Self, on| this.shortcut = on),
            ))
            .child(controls::toggle(
                "script-running",
                "running",
                self.running,
                controls::setter(&entity, |this: &mut Self, on| this.running = on),
            ));
        Body::new()
            .playground(live)
            .hint("Run starts the script; Stop stops it")
            .config(config)
            .example(controls::block(
                "Reference preview",
                reference("script-ref"),
            ))
    }
}
