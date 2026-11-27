//! Ghost · Thread › ToolCallFold: one live call with every prop, then the
//! seven reference calls (`components/ToolCallFold/preview.html`).
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, prelude::FluentBuilder as _, px, rems,
};
use u0ui::ghost::thread::{ToolCallFold, ToolKind, ToolStatus};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| ToolCallFoldPage {
        kind: 4,
        status: 1,
        duration: true,
        locations: true,
        diff: false,
        body: true,
        default_open: true,
    })
    .into()
}

struct ToolCallFoldPage {
    kind: usize,
    status: usize,
    duration: bool,
    locations: bool,
    diff: bool,
    body: bool,
    default_open: bool,
}

/// `.g-dim`: secondary text.
fn dim(text: &'static str) -> impl IntoElement {
    div()
        .text_color(color(ColorRole::TextSecondary))
        .child(text)
}

/// The reference column: `width: 560, display: grid, gap: 2`.
pub fn reference(prefix: &'static str) -> impl IntoElement {
    let call = |i: usize, kind| ToolCallFold::new((prefix, i), kind);
    div()
        .flex()
        .flex_col()
        .gap(px(2.))
        .w_full()
        .max_w(rems(35.))
        .child(
            call(0, ToolKind::Read)
                .title("src/theme/tokens.ts")
                .duration(1),
        )
        .child(
            call(1, ToolKind::Search)
                .title("\"layer-accent\" in src/**")
                .locations(["src/theme/tokens.ts:41", "src/theme/g100.ts:12"])
                .duration(2)
                .default_open(true)
                .child(dim("2 matches")),
        )
        .child(
            call(2, ToolKind::Edit)
                .title("src/theme/g100.ts")
                .duration(3),
        )
        .child(
            call(3, ToolKind::Execute)
                .title("pnpm test")
                .status(ToolStatus::Running),
        )
        .child(
            call(4, ToolKind::Fetch)
                .title("carbondesignsystem.com/…/color")
                .duration(4),
        )
        .child(
            call(5, ToolKind::Execute)
                .title("pnpm lint")
                .status(ToolStatus::Failed)
                .duration(9),
        )
        .child(
            call(6, ToolKind::Think)
                .title("Plan the token migration")
                .duration(12),
        )
}

impl Render for ToolCallFoldPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let kind = ToolKind::ALL[self.kind];
        let status = ToolStatus::ALL[self.status];
        // Re-key on `defaultOpen` so the switch re-seeds the fold.
        let live = ToolCallFold::new(("tool-live", self.default_open as usize), kind)
            .title("\"layer-accent\" in src/**")
            .status(status)
            .default_open(self.default_open)
            .when(self.duration, |f| f.duration(2))
            .when(self.locations, |f| {
                f.locations(["src/theme/tokens.ts:41", "src/theme/g100.ts:12"])
            })
            .when(self.diff, |f| f.added(12).removed(3))
            .when(self.body, |f| f.child(dim("2 matches")));
        let kinds: Vec<&'static str> = ToolKind::ALL.iter().map(|k| k.name()).collect();
        let statuses: Vec<&'static str> = ToolStatus::ALL.iter().map(|s| s.name()).collect();
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "kind",
                controls::choices(
                    "tool-kind",
                    &kinds,
                    self.kind,
                    controls::setter(&entity, |this: &mut Self, i| this.kind = i),
                ),
            ))
            .child(controls::field(
                "status",
                controls::choices(
                    "tool-status",
                    &statuses,
                    self.status,
                    controls::setter(&entity, |this: &mut Self, i| this.status = i),
                ),
            ))
            .child(controls::toggle(
                "tool-open",
                "defaultOpen",
                self.default_open,
                controls::setter(&entity, |this: &mut Self, on| this.default_open = on),
            ))
            .child(controls::toggle(
                "tool-duration",
                "duration",
                self.duration,
                controls::setter(&entity, |this: &mut Self, on| this.duration = on),
            ))
            .child(controls::toggle(
                "tool-locations",
                "locations",
                self.locations,
                controls::setter(&entity, |this: &mut Self, on| this.locations = on),
            ))
            .child(controls::toggle(
                "tool-diff",
                "added / removed",
                self.diff,
                controls::setter(&entity, |this: &mut Self, on| this.diff = on),
            ))
            .child(controls::toggle(
                "tool-body",
                "children",
                self.body,
                controls::setter(&entity, |this: &mut Self, on| this.body = on),
            ));
        Body::new()
            .playground(div().w_full().max_w(rems(35.)).child(live))
            .hint("Click the row or press Enter/Space on it to open and close it.")
            .config(config)
            .example(controls::block("Reference preview", reference("tool-ref")))
    }
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .flex()
        .flex_col()
        .gap(px(2.))
        .child(
            ToolCallFold::new("tool-thumb-0", ToolKind::Read)
                .title("src/theme/tokens.ts")
                .duration(1),
        )
        .child(
            ToolCallFold::new("tool-thumb-1", ToolKind::Execute)
                .title("pnpm test")
                .status(ToolStatus::Running),
        )
        .into_any_element()
}
