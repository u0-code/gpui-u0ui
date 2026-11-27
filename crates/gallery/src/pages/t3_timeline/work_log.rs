//! T3 · Timeline › WorkLog: six steps (read, search, edit with a diff,
//! a failed test run, a fetch, a running build) open by default
//! (`components/WorkLog/preview.html`); click a row with detail to open it.
//! Also every `ToolActivityIcon`.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div,
};
use u0ui::prelude::*;
use u0ui::t3::timeline::{ToolActivityIcon, ToolKind, ToolStatus, WorkEntry, WorkLog};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| WorkLogPage {
        live: false,
        open: true,
    })
    .into()
}

struct WorkLogPage {
    live: bool,
    open: bool,
}

fn entries() -> Vec<WorkEntry> {
    vec![
        WorkEntry::new(ToolKind::Read, "Read", "packages/tokens/tokens.json"),
        WorkEntry::new(ToolKind::Search, "Searched", "\"layer-accent\" in src/").stat("14 matches"),
        WorkEntry::new(ToolKind::Edit, "Edited", "src/tokens.json")
            .stat("+48 −3")
            .detail("@@ -112,6 +112,54 @@\n+  \"g90\": { ... }"),
        WorkEntry::new(ToolKind::Command, "Ran", "pnpm test --filter tokens")
            .status(ToolStatus::Failed)
            .detail("1 failing: contrast(text-helper, layer-02)"),
        WorkEntry::new(
            ToolKind::Web,
            "Fetched",
            "carbondesignsystem.com/elements/themes",
        ),
        WorkEntry::new(ToolKind::Command, "Running", "pnpm build").status(ToolStatus::Running),
    ]
}

impl Render for WorkLogPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let live = div().w_full().max_w(gpui_kit::rems(42.5)).child(
            WorkLog::new(
                SharedString::from(format!("wl-live-{}", self.open)),
                entries(),
                134,
            )
            .default_open(self.open)
            .live(self.live),
        );
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::toggle(
                "wl-open",
                "defaultOpen",
                self.open,
                controls::setter(&entity, |this: &mut Self, on| this.open = on),
            ))
            .child(controls::toggle(
                "wl-live",
                "live",
                self.live,
                controls::setter(&entity, |this: &mut Self, on| this.live = on),
            ));
        let icons = div()
            .flex()
            .flex_row()
            .flex_wrap()
            .gap(tokens::SPACING_05)
            .children(ToolKind::ALL.into_iter().map(|k| {
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(tokens::SPACING_03)
                    .text_size(gpui_kit::rems(0.75))
                    .text_color(color(ColorRole::TextSecondary))
                    .child(ToolActivityIcon::new(
                        SharedString::from(format!("wl-icon-{}", k.name())),
                        k,
                        ToolStatus::Done,
                    ))
                    .child(k.name())
            }))
            .child(ToolActivityIcon::new(
                "wl-icon-running",
                ToolKind::Command,
                ToolStatus::Running,
            ));
        Body::new()
            .playground(live)
            .hint("The header folds the log; rows with detail open on click.")
            .config(config)
            .example(controls::block(
                "Reference preview",
                div()
                    .w_full()
                    .max_w(gpui_kit::rems(42.5))
                    .child(WorkLog::new("wl-ref", entries(), 134).default_open(true)),
            ))
            .example(controls::block("ToolActivityIcon", icons))
    }
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(gpui_kit::rems(15.))
        .child(
            WorkLog::new("wl-th", entries().into_iter().take(2).collect(), 134).default_open(true),
        )
        .into_any_element()
}
