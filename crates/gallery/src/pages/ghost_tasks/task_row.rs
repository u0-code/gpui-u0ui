//! Ghost · Tasks & workflows › TaskRow: a live row driven by the
//! configuration (status, enabled, every optional run), and the three
//! reference tasks.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, prelude::FluentBuilder as _, rems,
};
use u0ui::ghost::tasks::{TaskRow, TaskStatus};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        status: 1,
        enabled: true,
        next_run: true,
        agent: 0,
        runs: 0,
        toggles: Vec::new(),
    })
    .into()
}

struct Page {
    /// 0 = none, then `TaskStatus::ALL`.
    status: usize,
    enabled: bool,
    next_run: bool,
    agent: usize,
    runs: usize,
    toggles: Vec<bool>,
}

const STATUSES: [&str; 3] = ["none", "ok", "failed"];
const AGENTS: [&str; 3] = ["Claude", "Codex", "Gemini"];

/// `components/TaskRow/preview.html`: three rows, 620px wide on `layer-01`.
fn reference() -> impl IntoElement {
    div()
        .w_full()
        .max_w(rems(38.75))
        .bg(color(ColorRole::Layer01))
        .child(
            TaskRow::new("tr-0", "Morning triage")
                .schedule("Weekdays at 09:00")
                .agent("Claude")
                .project("carbon-ui")
                .last_run("today 09:00")
                .last_status(TaskStatus::Ok)
                .next_run("tomorrow 09:00"),
        )
        .child(
            TaskRow::new("tr-1", "Dependency updates")
                .schedule("Every Monday")
                .agent("Codex")
                .project("api-gateway")
                .last_run("Mon 08:00")
                .last_status(TaskStatus::Failed)
                .next_run("Mon 08:00"),
        )
        .child(
            TaskRow::new("tr-2", "Release notes draft")
                .schedule("On webhook")
                .agent("Claude")
                .project("docs")
                .last_run("2d ago")
                .enabled(false),
        )
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(
            TaskRow::new("tr-thumb", "Morning triage")
                .schedule("Weekdays at 09:00")
                .last_run("09:00")
                .last_status(TaskStatus::Ok),
        )
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let runs = entity.clone();
        let toggles = entity.clone();
        let status = self.status.checked_sub(1).map(|i| TaskStatus::ALL[i]);
        let log: Vec<&str> = self
            .toggles
            .iter()
            .rev()
            .take(4)
            .map(|on| if *on { "true" } else { "false" })
            .collect();
        let live = div()
            .w_full()
            .max_w(rems(38.75))
            .flex()
            .flex_col()
            .gap(tokens::SPACING_03)
            .child(
                TaskRow::new("tr-live", "Morning triage")
                    .schedule("Weekdays at 09:00")
                    .agent(AGENTS[self.agent])
                    .project("carbon-ui")
                    .last_run("today 09:00")
                    .when_some(status, |r, s| r.last_status(s))
                    .when(self.next_run, |r| r.next_run("tomorrow 09:00"))
                    .enabled(self.enabled)
                    .on_run(move |_, cx| {
                        runs.update(cx, |p, cx| {
                            p.runs += 1;
                            cx.notify();
                        })
                    })
                    .on_toggle(move |on, _, cx| {
                        toggles.update(cx, |p, cx| {
                            p.toggles.push(on);
                            cx.notify();
                        })
                    }),
            )
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(format!(
                        "onRun × {} · onToggle: {}",
                        self.runs,
                        if log.is_empty() {
                            "—".to_string()
                        } else {
                            log.join(", ")
                        }
                    )),
            );
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::field(
                "lastStatus",
                controls::choices(
                    "tr-status",
                    &STATUSES,
                    self.status,
                    controls::setter(&entity, |this: &mut Page, i| this.status = i),
                ),
            ))
            .child(controls::field(
                "agent",
                controls::choices(
                    "tr-agent",
                    &AGENTS,
                    self.agent,
                    controls::setter(&entity, |this: &mut Page, i| this.agent = i),
                ),
            ))
            .child(controls::toggle(
                "tr-enabled",
                "enabled",
                self.enabled,
                controls::setter(&entity, |this: &mut Page, v| this.enabled = v),
            ))
            .child(controls::toggle(
                "tr-next",
                "nextRun",
                self.next_run,
                controls::setter(&entity, |this: &mut Page, v| this.next_run = v),
            ));
        Body::new()
            .playground(live)
            .hint("Run now fires onRun; the switch pauses the task")
            .config(config)
            .example(controls::block("Reference preview", reference()))
    }
}
