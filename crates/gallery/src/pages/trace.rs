//! Chat & AI › Trace: the reference agent trace (search, tool call with its
//! output, a running step, a pending step) and a configurable live line.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled, Window,
    div, rems,
};
use u0ui::components::code_block::{CodeBlock, CodeBlockSize};
use u0ui::components::trace::{
    Trace, TraceCollapsible, TraceDescription, TraceLine, TraceStatus, TraceTitle,
};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| TracePage {
        status: 1,
        description: true,
        collapsible: true,
    })
    .into()
}

struct TracePage {
    status: usize,
    description: bool,
    collapsible: bool,
}

impl Render for TracePage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();

        // `components/Trace/preview.html`.
        let reference = Trace::new()
            .max_w(rems(30.))
            .line(
                TraceLine::new("trace-ref-0", TraceStatus::Done)
                    .title(TraceTitle::new("Searched the knowledge base"))
                    .description(TraceDescription::new("4 documents matched")),
            )
            .line(
                TraceLine::new("trace-ref-1", TraceStatus::Done)
                    .title(TraceTitle::new("Called list_instances"))
                    .child(
                        TraceCollapsible::new("trace-ref-output", "Show output").child(
                            CodeBlock::new("trace-ref-code", "{ \"count\": 42 }")
                                .size(CodeBlockSize::Sm),
                        ),
                    ),
            )
            .line(
                TraceLine::new("trace-ref-2", TraceStatus::Running)
                    .title(TraceTitle::new("Summarizing results")),
            )
            .line(
                TraceLine::new("trace-ref-3", TraceStatus::Pending)
                    .title(TraceTitle::new("Draft answer")),
            );

        let status = TraceStatus::ALL[self.status];
        let mut line =
            TraceLine::new("trace-live-1", status).title(TraceTitle::new(match status {
                TraceStatus::Done => "Called list_instances",
                TraceStatus::Running => "Calling list_instances",
                TraceStatus::Error => "list_instances failed",
                TraceStatus::Pending => "Call list_instances",
            }));
        if self.description {
            line = line.description(TraceDescription::new("region: eu-de"));
        }
        if self.collapsible {
            line = line.child(
                TraceCollapsible::new("trace-live-output", "Show output").child(
                    CodeBlock::new("trace-live-code", "{ \"count\": 42 }").size(CodeBlockSize::Sm),
                ),
            );
        }
        let live = Trace::new()
            .max_w(rems(30.))
            .line(
                TraceLine::new("trace-live-0", TraceStatus::Done)
                    .title(TraceTitle::new("Searched the knowledge base")),
            )
            .line(line)
            .line(
                TraceLine::new("trace-live-2", TraceStatus::Pending)
                    .title(TraceTitle::new("Draft answer")),
            );

        let statuses = TraceStatus::ALL.map(|s| s.name());
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "status (middle line)",
                controls::choices(
                    "trace-status",
                    &statuses,
                    self.status,
                    controls::setter(&entity, |this: &mut TracePage, i| this.status = i),
                ),
            ))
            .child(controls::toggle(
                "trace-description",
                "TraceDescription",
                self.description,
                controls::setter(&entity, |this: &mut TracePage, on| this.description = on),
            ))
            .child(controls::toggle(
                "trace-collapsible",
                "TraceCollapsible",
                self.collapsible,
                controls::setter(&entity, |this: &mut TracePage, on| this.collapsible = on),
            ));

        Body::new()
            .playground(live)
            .config(config)
            .example(controls::block("Reference preview", reference))
    }
}
