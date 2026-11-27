//! Ghost · Tasks & workflows › WorkflowRow: a live row driven by the
//! configuration, and the two reference workflows.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, prelude::FluentBuilder as _, rems,
};
use u0ui::ghost::tasks::WorkflowRow;
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        steps: 2,
        trigger: 0,
        description: true,
        last_run: true,
        started: 0,
    })
    .into()
}

struct Page {
    steps: usize,
    trigger: usize,
    description: bool,
    last_run: bool,
    started: usize,
}

const STEPS: [usize; 4] = [1, 2, 4, 12];
const TRIGGERS: [&str; 3] = ["Manual", "On webhook", "Weekdays at 09:00"];

/// `components/WorkflowRow/preview.html`: two rows, 620px wide on `layer-01`.
fn reference() -> impl IntoElement {
    div()
        .w_full()
        .max_w(rems(38.75))
        .bg(color(ColorRole::Layer01))
        .child(
            WorkflowRow::new("wfr-0", "Release")
                .description("Tests, notes, tag")
                .steps(4)
                .trigger("Manual")
                .last_run("2d ago"),
        )
        .child(
            WorkflowRow::new("wfr-1", "Triage new issues")
                .description("Label and assign")
                .steps(2)
                .trigger("On webhook")
                .last_run("1h ago"),
        )
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(
            WorkflowRow::new("wfr-thumb", "Release")
                .description("Tests, notes, tag")
                .steps(4),
        )
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let started = entity.clone();
        let live = div()
            .w_full()
            .max_w(rems(38.75))
            .flex()
            .flex_col()
            .gap(tokens::SPACING_03)
            .child(
                WorkflowRow::new("wfr-live", "Release")
                    .when(self.description, |r| r.description("Tests, notes, tag"))
                    .steps(STEPS[self.steps])
                    .trigger(TRIGGERS[self.trigger])
                    .when(self.last_run, |r| r.last_run("2d ago"))
                    .on_start(move |_, cx| {
                        started.update(cx, |p, cx| {
                            p.started += 1;
                            cx.notify();
                        })
                    }),
            )
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(format!("onStart × {}", self.started)),
            );
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::field(
                "steps",
                controls::choices(
                    "wfr-steps",
                    &["1", "2", "4", "12"],
                    self.steps,
                    controls::setter(&entity, |this: &mut Page, i| this.steps = i),
                ),
            ))
            .child(controls::field(
                "trigger",
                controls::choices(
                    "wfr-trigger",
                    &TRIGGERS,
                    self.trigger,
                    controls::setter(&entity, |this: &mut Page, i| this.trigger = i),
                ),
            ))
            .child(controls::toggle(
                "wfr-description",
                "description",
                self.description,
                controls::setter(&entity, |this: &mut Page, v| this.description = v),
            ))
            .child(controls::toggle(
                "wfr-last-run",
                "lastRun",
                self.last_run,
                controls::setter(&entity, |this: &mut Page, v| this.last_run = v),
            ));
        Body::new()
            .playground(live)
            .hint("Start fires onStart")
            .config(config)
            .example(controls::block("Reference preview", reference()))
    }
}
