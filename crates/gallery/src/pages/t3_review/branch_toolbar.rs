//! T3 · Thread › BranchToolbar: the reference bar (MacBook Pro, new
//! worktree, main) with working menus and segmented control, and a live bar.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, rems,
};
use u0ui::t3::review::{BranchToolbar, WorkspaceMode};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        mode: WorkspaceMode::Worktree,
        environment: "MacBook Pro".into(),
        branch: "main".into(),
    })
    .into()
}

struct Page {
    mode: WorkspaceMode,
    environment: SharedString,
    branch: SharedString,
}

const ENVIRONMENTS: [&str; 2] = ["MacBook Pro", "build-box"];
const BRANCHES: [&str; 3] = ["main", "develop", "release/1.4"];

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(
            BranchToolbar::new("bt-thumb")
                .environment("MacBook Pro")
                .branch("main"),
        )
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let live = BranchToolbar::new("bt-live")
            .environments(ENVIRONMENTS)
            .environment(self.environment.clone())
            .mode(self.mode)
            .branch(self.branch.clone())
            .branches(BRANCHES)
            .on_mode_change(controls::setter(&entity, |this: &mut Page, m| {
                this.mode = m
            }))
            .on_environment(controls::setter(&entity, |this: &mut Page, e| {
                this.environment = e
            }))
            .on_branch(controls::setter(&entity, |this: &mut Page, b| {
                this.branch = b
            }));
        let config = controls::field(
            "mode",
            controls::choices(
                "bt-mode",
                &["local", "worktree"],
                WorkspaceMode::ALL
                    .iter()
                    .position(|m| *m == self.mode)
                    .unwrap_or(1),
                controls::setter(&entity, |this: &mut Page, i| {
                    this.mode = WorkspaceMode::ALL[i]
                }),
            ),
        );
        Body::new()
            .playground(live)
            .hint("Pick a machine or branch from the menus; the mode is a segmented control")
            .config(config)
            .example(controls::block(
                "Reference preview",
                BranchToolbar::new("bt-ref")
                    .environments(ENVIRONMENTS)
                    .environment("MacBook Pro")
                    .default_mode(WorkspaceMode::Worktree)
                    .branch("main")
                    .branches(BRANCHES),
            ))
    }
}
