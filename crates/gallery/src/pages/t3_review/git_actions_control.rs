//! T3 · Thread › GitActionsControl: the reference states (6 changed files,
//! 2 commits ahead, pushed, behind upstream, pushing) and a live control
//! for every state, note and progress.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, prelude::FluentBuilder as _, rems,
};
use u0ui::prelude::*;
use u0ui::t3::review::{GitAction, GitActionsControl, GitState};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        state: 0,
        files: 1,
        busy: false,
        note: false,
        last: None,
    })
    .into()
}

struct Page {
    state: usize,
    files: usize,
    busy: bool,
    note: bool,
    last: Option<SharedString>,
}

const FILES: [usize; 3] = [0, 6, 1];

fn reference() -> gpui_kit::Div {
    // `cu-stack` with `gap: 12`.
    div()
        .flex()
        .flex_col()
        .items_start()
        .gap(rems(0.75))
        .child(GitActionsControl::new("ga-ref-0", GitState::Changes).files(6))
        .child(GitActionsControl::new("ga-ref-1", GitState::Ahead).ahead(2))
        .child(GitActionsControl::new("ga-ref-2", GitState::Pushed))
        .child(GitActionsControl::new("ga-ref-3", GitState::Behind))
        .child(
            GitActionsControl::new("ga-ref-4", GitState::Changes)
                .progress("Pushing to origin\u{2026}"),
        )
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap(rems(0.75))
        .child(GitActionsControl::new("ga-thumb-0", GitState::Changes).files(6))
        .child(GitActionsControl::new("ga-thumb-1", GitState::Ahead).ahead(2))
        .into_any_element()
}

fn action_name(a: GitAction) -> &'static str {
    match a {
        GitAction::Commit => "commit",
        GitAction::CommitBranch => "commit-branch",
        GitAction::Push => "push",
        GitAction::Pr => "pr",
        GitAction::Discard => "discard",
    }
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let mut control = GitActionsControl::new("ga-live", GitState::ALL[self.state])
            .files(FILES[self.files])
            .ahead(2)
            .on_primary({
                let entity = entity.clone();
                move |_, cx| {
                    entity.update(cx, |this, cx| {
                        this.last = Some("onPrimary()".into());
                        cx.notify();
                    })
                }
            })
            .on_action(controls::setter(
                &entity,
                |this: &mut Page, a: GitAction| {
                    this.last = Some(format!("onAction(\"{}\")", action_name(a)).into())
                },
            ));
        if self.busy {
            control = control.progress("Pushing to origin\u{2026}");
        }
        if self.note {
            control = control.note("2 files have conflicts");
        }
        let live = div()
            .flex()
            .flex_col()
            .items_start()
            .gap(tokens::SPACING_05)
            .child(control)
            .when_some(self.last.clone(), |d, l| {
                d.child(
                    div()
                        .type_style(TypeStyle::Label01)
                        .text_color(color(ColorRole::TextSecondary))
                        .child(l),
                )
            });
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::field(
                "state",
                controls::choices(
                    "ga-state",
                    &[
                        "changes", "ahead", "pushed", "pr", "behind", "none", "init", "publish",
                    ],
                    self.state,
                    controls::setter(&entity, |this: &mut Page, i| this.state = i),
                ),
            ))
            .child(controls::field(
                "files",
                controls::choices(
                    "ga-files",
                    &["0", "6", "1"],
                    self.files,
                    controls::setter(&entity, |this: &mut Page, i| this.files = i),
                ),
            ))
            .child(controls::toggle(
                "ga-note",
                "note",
                self.note,
                controls::setter(&entity, |this: &mut Page, v| this.note = v),
            ))
            .child(controls::toggle(
                "ga-busy",
                "progress",
                self.busy,
                controls::setter(&entity, |this: &mut Page, v| this.busy = v),
            ));
        Body::new()
            .playground(live)
            .config(config)
            .example(controls::block("Reference preview", reference()))
    }
}
