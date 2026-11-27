//! T3 · Mobile › MobileTabDock: the reference dock (Threads current with a
//! needs-you badge of 2), a live dock, and the flat phone-chrome tab bar.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, rems,
};
use u0ui::prelude::*;
use u0ui::t3::mobile::{DockTab, MobileTabDock};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        value: "threads".into(),
        badge: 2,
    })
    .into()
}

struct Page {
    value: SharedString,
    badge: usize,
}

fn tabs(badge: usize) -> Vec<DockTab> {
    let threads = DockTab::new("threads", "Threads", CarbonIcon::Chat);
    vec![
        if badge > 0 {
            threads.badge(badge)
        } else {
            threads
        },
        DockTab::new("prs", "PRs", CarbonIcon::PullRequest),
        DockTab::new("issues", "Issues", CarbonIcon::Ticket),
        DockTab::new("settings", "Settings", CarbonIcon::Settings),
    ]
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    MobileTabDock::new("mtd-thumb")
        .tabs(tabs(2).into_iter().take(3))
        .value("threads")
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let set = controls::setter(&entity, |this: &mut Page, v: SharedString| this.value = v);
        let config = controls::field(
            "badge",
            controls::choices(
                "mtd-badge",
                &["none", "2", "12"],
                [0, 2, 12]
                    .iter()
                    .position(|n| *n == self.badge)
                    .unwrap_or(0),
                controls::setter(&entity, |this: &mut Page, i| this.badge = [0, 2, 12][i]),
            ),
        );
        Body::new()
            .playground(
                div().flex().justify_center().w_full().child(
                    MobileTabDock::new("mtd-live")
                        .tabs(tabs(self.badge))
                        .value(self.value.clone())
                        .on_value_change(set),
                ),
            )
            .config(config)
            .example(controls::block(
                "Reference preview",
                div()
                    .flex()
                    .w_full()
                    .child(MobileTabDock::new("mtd-ref").tabs(tabs(2)).value("threads")),
            ))
            .example(controls::block(
                "Phone chrome ([data-mb])",
                div().w(rems(24.375)).child(
                    MobileTabDock::new("mtd-mb")
                        .tabs(tabs(self.badge))
                        .value(self.value.clone())
                        .on_value_change(controls::setter(
                            &entity,
                            |this: &mut Page, v: SharedString| this.value = v,
                        ))
                        .phone(true),
                ),
            ))
    }
}
