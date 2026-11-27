//! u0 · Diagnostics › ProcessTree: the reference process tree, live
//! (chevrons collapse branches; Stop reports the pid).
use crate::controls;
use crate::page::Body;
use crate::pages::u0_settings::note;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, rems,
};
use u0ui::prelude::*;
use u0ui::u0::diagnostics::{Process, ProcessTree};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page { stopped: None }).into()
}

struct Page {
    stopped: Option<u32>,
}

/// `components/ProcessTree/preview.html`'s processes.
fn processes() -> Vec<Process> {
    vec![
        Process::new(4120, "u0-server", 3.2, 180.).children([
            Process::new(4188, "claude", 42.5, 610.)
                .kind("Claude")
                .children([Process::new(4301, "pnpm test", 88.1, 1540.)]),
            Process::new(4190, "codex", 0.4, 220.).kind("Codex"),
        ]),
        Process::new(5011, "cloudflared", 0.8, 38.),
    ]
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .overflow_hidden()
        .child(
            div()
                .w(rems(30.))
                .child(ProcessTree::new("ptree-thumb", processes())),
        )
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let live = div()
            .w_full()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_03)
            .child(
                ProcessTree::new("ptree-live", processes()).on_stop(move |pid, _, cx| {
                    entity.update(cx, |p, cx| {
                        p.stopped = Some(pid);
                        cx.notify();
                    })
                }),
            )
            .child(note(match self.stopped {
                Some(pid) => format!("Stop pressed for pid {pid}"),
                None => "Stop: —".into(),
            }));
        Body::new()
            .playground(live)
            .hint("Bars turn red past 70% CPU or 1.4 GB")
            .example(controls::block(
                "Reference preview",
                ProcessTree::new("ptree-ref", processes()),
            ))
    }
}
