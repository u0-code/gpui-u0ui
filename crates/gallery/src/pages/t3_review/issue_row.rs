//! T3 · Pull requests & issues › IssueRow: the reference issues (in
//! progress / high, open / low, done / medium, cancelled) and a live row.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, prelude::FluentBuilder as _, rems, rgb,
};
use u0ui::prelude::*;
use u0ui::t3::review::{IssueRow, IssueState, IssueStateGlyph, Label, Priority};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        state: 1,
        priority: 3,
        label: true,
        assignee: true,
        links: 1,
        dash_unprioritized: false,
        interactive_progress: false,
    })
    .into()
}

struct Page {
    state: usize,
    priority: usize,
    label: bool,
    assignee: bool,
    links: usize,
    dash_unprioritized: bool,
    interactive_progress: bool,
}

/// `components/IssueRow/preview.html`.
fn reference() -> [IssueRow; 4] {
    [
        IssueRow::new("ir-0", "CUI-142", "Date picker loses focus on month change")
            .state(IssueState::Progress)
            .priority(Priority::High)
            .labels([Label::new("bug", rgb(0xda1e28))])
            .assignee("JD")
            .time("2d")
            .linked_threads(1),
        IssueRow::new("ir-1", "CUI-139", "Add G10 theme to docs")
            .state(IssueState::Open)
            .priority(Priority::Low)
            .time("4d"),
        IssueRow::new("ir-2", "CUI-120", "Cover uses wrong radius token")
            .state(IssueState::Done)
            .priority(Priority::Medium)
            .assignee("AK")
            .time("1w"),
        IssueRow::new("ir-3", "CUI-101", "Explore motion tokens")
            .state(IssueState::Cancelled)
            .time("3w"),
    ]
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    let [a, b, ..] = reference();
    div().w(rems(15.)).child(a).child(b).into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let priority = [
            None,
            Some(Priority::Low),
            Some(Priority::Medium),
            Some(Priority::High),
            Some(Priority::Urgent),
        ][self.priority];
        let live = div().w_full().max_w(rems(51.25)).child(
            IssueRow::new(
                "ir-live",
                "CUI-142",
                "Date picker loses focus on month change",
            )
            .state(IssueState::ALL[self.state])
            .when_some(priority, |r, p| r.priority(p))
            .when(self.label, |r| r.labels([Label::new("bug", rgb(0xda1e28))]))
            .when(self.assignee, |r| r.assignee("JD"))
            .dash_when_unprioritized(self.dash_unprioritized)
            .interactive_progress(self.interactive_progress)
            .linked_threads(self.links)
            .time("2d"),
        );
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::field(
                "state",
                controls::choices(
                    "ir-state",
                    &["open", "progress", "done", "cancelled", "backlog"],
                    self.state,
                    controls::setter(&entity, |this: &mut Page, i| this.state = i),
                ),
            ))
            .child(controls::field(
                "priority",
                controls::choices(
                    "ir-priority",
                    &["none", "low", "medium", "high", "urgent"],
                    self.priority,
                    controls::setter(&entity, |this: &mut Page, i| this.priority = i),
                ),
            ))
            .child(controls::field(
                "linkedThreads",
                controls::choices(
                    "ir-links",
                    &["0", "1", "3"],
                    [0, 1, 2][self.links.min(2)],
                    controls::setter(&entity, |this: &mut Page, i| this.links = [0, 1, 3][i]),
                ),
            ))
            .child(controls::toggle(
                "ir-label",
                "labels",
                self.label,
                controls::setter(&entity, |this: &mut Page, v| this.label = v),
            ))
            .child(controls::toggle(
                "ir-assignee",
                "assignee",
                self.assignee,
                controls::setter(&entity, |this: &mut Page, v| this.assignee = v),
            ))
            .child(controls::toggle(
                "ir-dash",
                "dashWhenUnprioritized",
                self.dash_unprioritized,
                controls::setter(&entity, |this: &mut Page, v| this.dash_unprioritized = v),
            ))
            .child(controls::toggle(
                "ir-interactive-progress",
                "interactiveProgress",
                self.interactive_progress,
                controls::setter(&entity, |this: &mut Page, v| this.interactive_progress = v),
            ));
        let glyphs =
            div()
                .flex()
                .flex_wrap()
                .gap(tokens::SPACING_05)
                .children(IssueState::ALL.map(|s| {
                    div()
                        .flex()
                        .items_center()
                        .gap(rems(0.375))
                        .text_size(rems(0.75))
                        .text_color(color(ColorRole::TextPrimary))
                        .child(IssueStateGlyph::new(s))
                        .child(s.name())
                }));
        Body::new()
            .playground(live)
            .config(config)
            .example(controls::block(
                "Reference preview",
                div().w_full().max_w(rems(51.25)).children(reference()),
            ))
            .example(controls::block("IssueStateGlyph", glyphs))
    }
}
