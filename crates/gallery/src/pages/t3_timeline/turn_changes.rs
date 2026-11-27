//! T3 · Timeline › TurnChanges (`src/t3/chat.jsx`, `.cu-turnchg`; no card
//! folder): the end-of-turn summary of four changed files with Review
//! changes and Undo. Rows open the file; the page reports each action.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div,
};
use u0ui::prelude::*;
use u0ui::t3::chat::{TurnChanges, TurnFile};
use u0ui::t3::review::FileStatus;

/// The page description (TurnChanges has no reference README).
pub const DESCRIPTION: &str = "End-of-turn change summary for coding-agent threads: the file count with the summed diff stat, Review changes and an optional Undo, and one row per file that opens it.";

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| TurnPage {
        undo: true,
        single: false,
        last: "".into(),
    })
    .into()
}

struct TurnPage {
    undo: bool,
    single: bool,
    last: SharedString,
}

fn files() -> Vec<TurnFile> {
    vec![
        TurnFile::new("src/tokens/g90.json", 217, 0).status(FileStatus::Added),
        TurnFile::new("src/tokens/index.ts", 6, 2),
        TurnFile::new("scripts/generate-cover.ts", 14, 3),
        TurnFile::new("docs/legacy-themes.md", 0, 41).status(FileStatus::Deleted),
    ]
}

impl Render for TurnPage {
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
        let mut shown = files();
        if self.single {
            shown.truncate(1);
        }
        let mut changes = TurnChanges::new("turn-live", shown)
            .on_review(note("onReview"))
            .on_open_file({
                let entity = entity.clone();
                move |path, _, cx| {
                    entity.update(cx, |this, cx| {
                        this.last = format!("onOpenFile({path})").into();
                        cx.notify();
                    })
                }
            });
        if self.undo {
            changes = changes.on_undo(note("onUndo"));
        }
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .w_full()
            .max_w(gpui_kit::rems(42.5))
            .child(changes)
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(self.last.clone()),
            );
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::toggle(
                "turn-undo",
                "onUndo",
                self.undo,
                controls::setter(&entity, |this: &mut Self, on| this.undo = on),
            ))
            .child(controls::toggle(
                "turn-single",
                "One file",
                self.single,
                controls::setter(&entity, |this: &mut Self, on| this.single = on),
            ));
        Body::new()
            .playground(live)
            .config(config)
            .example(controls::block(
                "Changes in this turn",
                div()
                    .w_full()
                    .max_w(gpui_kit::rems(42.5))
                    .child(TurnChanges::new("turn-ref", files())),
            ))
    }
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(gpui_kit::rems(15.))
        .child(TurnChanges::new(
            "turn-th",
            files().into_iter().take(2).collect(),
        ))
        .into_any_element()
}
