//! Ghost · Tasks & workflows › WorkflowStepEditor: the reference workflow,
//! live (move, remove and add steps; edits report through onChange).
use crate::controls;
use crate::page::Body;
use crate::pages::ghost_settings::note;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, rems,
};
use u0ui::ghost::tasks::{WorkflowStep, WorkflowStepEditor};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        summary: "3 steps".into(),
    })
    .into()
}

struct Page {
    summary: String,
}

/// `components/WorkflowStepEditor/preview.html`'s steps.
fn steps() -> Vec<WorkflowStep> {
    vec![
        WorkflowStep::agent("Claude", "Summarise yesterday's merged PRs.").with_mode("Plan"),
        WorkflowStep::command("pnpm test --reporter=dot").with_continue_on_error(true),
        WorkflowStep::question("Ship the release notes?", ["Yes", "Edit first"]),
    ]
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(
            WorkflowStepEditor::new("wfe-thumb")
                .steps([WorkflowStep::command("pnpm test --reporter=dot")]),
        )
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let live = div()
            .w(rems(35.))
            .max_w_full()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(
                WorkflowStepEditor::new("wfe-live")
                    .steps(steps())
                    .on_change(move |s, _, cx| {
                        let kinds: Vec<_> = s.iter().map(|s| s.kind.name()).collect();
                        entity.update(cx, |p, cx| {
                            p.summary = format!("{} steps: {}", s.len(), kinds.join(", "));
                            cx.notify();
                        })
                    }),
            )
            .child(note(format!("onChange → {}", self.summary)));
        Body::new()
            .playground(live)
            .hint("Move up / down is the keyboard alternative to dragging")
            .example(controls::block(
                "Reference preview",
                div()
                    .w(rems(35.))
                    .max_w_full()
                    .child(WorkflowStepEditor::new("wfe-ref").steps(steps())),
            ))
    }
}
