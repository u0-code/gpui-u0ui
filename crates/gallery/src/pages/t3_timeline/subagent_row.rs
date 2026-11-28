//! T3 · Timeline › SubagentRow: the summary bar over a running, a completed
//! and a failed subagent (`components/SubagentRow/preview.html`), plus a live
//! row in every status and the dots on their own.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div,
};
use u0ui::prelude::*;
use u0ui::t3::timeline::{SubagentBar, SubagentRow, SubagentStatus, SubagentStatusDot};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| SubagentPage {
        status: 0,
        provider: true,
        last: "".into(),
    })
    .into()
}

struct SubagentPage {
    status: usize,
    provider: bool,
    last: SharedString,
}

/// The reference preview (`cu-stack`, gap 8, max 680px).
fn reference() -> impl IntoElement {
    use SubagentStatus::*;
    div()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_03)
        .w_full()
        .max_w(gpui_kit::rems(42.5))
        .child(SubagentBar::new(
            "sub-ref-bar",
            vec![Running, Running, Completed, Failed],
        ))
        .child(
            SubagentRow::new(
                "sub-ref-0",
                "researcher",
                "Find every --cds- variable without a token",
            )
            .status(Running)
            .elapsed(48)
            .provider("Claude"),
        )
        .child(
            SubagentRow::new("sub-ref-1", "tester", "Run visual regression")
                .status(Completed)
                .elapsed(95),
        )
        .child(
            SubagentRow::new("sub-ref-2", "linter", "Lint changed files")
                .status(Failed)
                .elapsed(12),
        )
}

impl Render for SubagentPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let note = |what: &'static str| {
            let entity = entity.clone();
            move |_: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    this.last = what.into();
                    cx.notify();
                })
            }
        };
        let mut row = SubagentRow::new(
            "sub-live",
            "researcher",
            "Find every --cds- variable without a token",
        )
        .status(SubagentStatus::ALL[self.status])
        .elapsed(48)
        .on_open(note("onOpen"))
        .on_stop(note("onStop"));
        if self.provider {
            row = row.provider("Claude");
        }
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .w_full()
            .max_w(gpui_kit::rems(42.5))
            .child(row)
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(self.last.clone()),
            );
        let statuses = SubagentStatus::ALL.map(|s| s.name());
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "status",
                controls::choices(
                    "sub-status",
                    &statuses,
                    self.status,
                    controls::setter(&entity, |this: &mut Self, i| this.status = i),
                ),
            ))
            .child(controls::toggle(
                "sub-provider",
                "provider",
                self.provider,
                controls::setter(&entity, |this: &mut Self, on| this.provider = on),
            ));
        let dots = div()
            .flex()
            .flex_row()
            .flex_wrap()
            .gap(tokens::SPACING_05)
            .children(SubagentStatus::ALL.into_iter().map(|s| {
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(tokens::SPACING_03)
                    .text_size(gpui_kit::rems(0.75))
                    .text_color(color(ColorRole::TextSecondary))
                    .child(SubagentStatusDot::new(
                        SharedString::from(format!("sub-dot-{}", s.name())),
                        s,
                    ))
                    .child(s.name())
            }));
        Body::new()
            .playground(live)
            .config(config)
            .example(controls::block("Reference preview", reference()))
            .example(controls::block("SubagentStatusDot", dots))
            .example(controls::block("Many subagents", many()))
    }
}

/// A bar of 24 subagents in a narrow column: ten marks at most, the rest
/// a count.
fn many() -> impl IntoElement {
    use SubagentStatus::*;
    let statuses = (0..24)
        .map(|i| match i % 4 {
            0 => Running,
            1 => Completed,
            2 => Waiting,
            _ => Failed,
        })
        .collect();
    div()
        .w(gpui_kit::rems(26.))
        .child(SubagentBar::new("sub-many", statuses))
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    use SubagentStatus::*;
    div()
        .w(gpui_kit::rems(15.))
        .child(SubagentBar::new("sub-th", vec![Running, Completed, Failed]))
        .into_any_element()
}
