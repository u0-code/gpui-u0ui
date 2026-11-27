//! Ghost · Tasks & workflows › ScheduleEditor: a live editor (controlled by
//! the page or keeping its own state, starting at any kind) with the value
//! it reports, and the reference pair (daily, webhook).
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, rems,
};
use u0ui::ghost::tasks::{Schedule, ScheduleEditor, ScheduleKind};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        kind: 0,
        controlled: true,
        value: Schedule::default(),
        reported: None,
    })
    .into()
}

struct Page {
    kind: usize,
    controlled: bool,
    value: Schedule,
    reported: Option<Schedule>,
}

const KINDS: [&str; 3] = ["daily", "interval", "webhook"];

/// One line describing a schedule's fields.
fn describe(s: &Schedule) -> SharedString {
    let days: Vec<String> = s.days.iter().map(u8::to_string).collect();
    format!(
        "{{ kind: '{}', at: '{}', days: [{}], every: {}, unit: '{}' }}",
        s.kind.name(),
        s.at,
        days.join(", "),
        s.every_or_default(),
        s.unit.name()
    )
    .into()
}

/// `components/ScheduleEditor/preview.html`: two editors side by side
/// (`grid-template-columns: 1fr 1fr; gap: 32px`).
fn reference() -> impl IntoElement {
    div()
        .w_full()
        .flex()
        .flex_wrap()
        .gap(tokens::SPACING_07)
        .child(
            div()
                .flex_1()
                .min_w(rems(20.))
                .child(ScheduleEditor::new("sched-ref-0")),
        )
        .child(
            div().flex_1().min_w(rems(20.)).child(
                ScheduleEditor::new("sched-ref-1")
                    .default_value(Schedule::of_kind(ScheduleKind::Webhook)),
            ),
        )
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(ScheduleEditor::new("sched-thumb"))
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let report = entity.clone();
        let kind = ScheduleKind::ALL[self.kind];
        let editor = if self.controlled {
            ScheduleEditor::new("sched-live-c").value(self.value.clone())
        } else {
            // A new id per starting kind, so a new `defaultValue` remounts it.
            ScheduleEditor::new(("sched-live-u", self.kind)).default_value(Schedule::of_kind(kind))
        }
        .on_value_change(move |s, _, cx| {
            let s = s.clone();
            report.update(cx, |p, cx| {
                if p.controlled {
                    p.value = s.clone();
                }
                p.reported = Some(s);
                cx.notify();
            })
        });
        let live = div()
            .w_full()
            .max_w(rems(30.))
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(editor)
            .child(
                div()
                    .type_style(TypeStyle::Code01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(match &self.reported {
                        Some(s) => format!("onValueChange {}", describe(s)),
                        None => "onValueChange —".to_string(),
                    }),
            );
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::field(
                "kind",
                controls::choices(
                    "sched-kind",
                    &KINDS,
                    self.kind,
                    controls::setter(&entity, |this: &mut Page, i| {
                        this.kind = i;
                        this.value = Schedule {
                            kind: ScheduleKind::ALL[i],
                            ..this.value.clone()
                        };
                    }),
                ),
            ))
            .child(controls::toggle(
                "sched-controlled",
                "controlled (value)",
                self.controlled,
                controls::setter(&entity, |this: &mut Page, v| this.controlled = v),
            ));
        Body::new()
            .playground(live)
            .hint("←/→ switch the kind; each day is a toggle")
            .config(config)
            .example(controls::block("Reference preview", reference()))
    }
}
