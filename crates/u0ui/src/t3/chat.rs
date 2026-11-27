//! T3 chat extensions (`src/t3/chat.jsx`, `.cu-turnchg` in `t3.css`):
//! [`TurnChanges`], the end-of-turn summary of what the agent changed.
//!
//! A `layer-01` box with a hairline frame: a 48px head ("4 files changed",
//! the summed [`DiffStat`], optional Undo, Review changes) over 32px file rows
//! (status letter, mono path, per-file stat) that open the file; rows hover
//! `layer-hover-01` and show the 2px focus ring on keyboard focus.
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::t3::press::{self, Handler, inset_frame, rem};
use crate::t3::review::{DiffFileStatus, DiffStat, FileStatus};
use crate::{
    theme::color,
    tokens::{ColorRole, FontFamilyKey},
    typography::font,
};
use gpui_kit::{
    App, ElementId, FontWeight, InteractiveElement as _, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _,
};
use std::rc::Rc;

/// One changed file (`{ path, status, added, removed }`).
#[derive(Clone, Debug, PartialEq)]
pub struct TurnFile {
    /// `path`.
    pub path: SharedString,
    /// `status` (default `M`).
    pub status: FileStatus,
    /// `added` lines.
    pub added: usize,
    /// `removed` lines.
    pub removed: usize,
}

impl TurnFile {
    /// A modified file.
    pub fn new(path: impl Into<SharedString>, added: usize, removed: usize) -> Self {
        Self {
            path: path.into(),
            status: FileStatus::Modified,
            added,
            removed,
        }
    }

    /// `status`.
    pub fn status(mut self, status: FileStatus) -> Self {
        self.status = status;
        self
    }
}

/// The head's totals: (files, added, removed).
pub fn turn_totals(files: &[TurnFile]) -> (usize, usize, usize) {
    (
        files.len(),
        files.iter().map(|f| f.added).sum(),
        files.iter().map(|f| f.removed).sum(),
    )
}

/// `TurnChanges`' strings (the reference's English defaults).
#[derive(Clone)]
pub struct TurnChangesTexts {
    /// `"{n} file(s) changed"`.
    pub changed: Rc<dyn Fn(usize) -> SharedString>,
    /// `reviewLabel`.
    pub review: SharedString,
    /// `undoLabel`.
    pub undo: SharedString,
}

impl Default for TurnChangesTexts {
    fn default() -> Self {
        Self {
            changed: Rc::new(|n| {
                format!("{n} {} changed", if n == 1 { "file" } else { "files" }).into()
            }),
            review: "Review changes".into(),
            undo: "Undo".into(),
        }
    }
}

type PathHandler = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;

/// The end-of-turn change summary. See the module docs.
#[derive(IntoElement)]
pub struct TurnChanges {
    id: ElementId,
    files: Vec<TurnFile>,
    texts: TurnChangesTexts,
    on_review: Option<Handler>,
    on_undo: Option<Handler>,
    on_open_file: Option<PathHandler>,
}

impl TurnChanges {
    /// A summary of `files`.
    pub fn new(id: impl Into<ElementId>, files: Vec<TurnFile>) -> Self {
        Self {
            id: id.into(),
            files,
            texts: TurnChangesTexts::default(),
            on_review: None,
            on_undo: None,
            on_open_file: None,
        }
    }

    /// Replace the strings (`reviewLabel`, `undoLabel`).
    pub fn texts(mut self, texts: TurnChangesTexts) -> Self {
        self.texts = texts;
        self
    }

    /// `onReview`.
    pub fn on_review(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_review = Some(Rc::new(f));
        self
    }

    /// `onUndo`: adds the ghost Undo button.
    pub fn on_undo(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_undo = Some(Rc::new(f));
        self
    }

    /// `onOpenFile(path)`.
    pub fn on_open_file(
        mut self,
        f: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_open_file = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for TurnChanges {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let (count, added, removed) = turn_totals(&self.files);
        let texts = self.texts.clone();
        let review = self.on_review.clone();
        // `.cu-turnchg-head { display: flex; align-items: center; gap: spacing-04; min-height: size-lg;
        // padding: 0 spacing-03 0 spacing-05; border-bottom: 1px solid border-subtle-01; font-size: 0.875rem }`.
        let head = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(rem(12.))
            .min_h(rem(48.))
            .pl(rem(16.))
            .pr(rem(8.))
            .border_b_1()
            .border_color(color(ColorRole::BorderSubtle01))
            .text_size(rem(14.))
            .line_height(rem(18.))
            // `.cu-turnchg-head b { font-weight: 600; white-space: nowrap }`.
            .child(
                div()
                    .flex_none()
                    .whitespace_nowrap()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child((texts.changed)(count)),
            )
            .child(DiffStat::new(added, removed))
            .child(div().flex_1())
            .when_some(self.on_undo.clone(), |this, f| {
                this.child(
                    Button::new((id.clone(), "undo"), texts.undo.clone())
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .on_click(move |_, w, cx| f(w, cx)),
                )
            })
            .child(
                Button::new((id.clone(), "review"), texts.review.clone())
                    .variant(ButtonVariant::Tertiary)
                    .size(ButtonSize::Sm)
                    .when_some(review, |b, f| b.on_click(move |_, w, cx| f(w, cx))),
            );
        let rows: Vec<_> = self
            .files
            .iter()
            .enumerate()
            .map(|(i, file)| {
                let key = |p: &str| -> ElementId {
                    (id.clone(), SharedString::from(format!("{p}-{i}"))).into()
                };
                let (hovered, hover) = press::hover(key("hover"), window, cx);
                let (focus, ring) = press::focus(key("focus"), window, cx);
                let open = self.on_open_file.clone();
                let path = file.path.clone();
                // `.cu-turnchg-row { display: flex; align-items: center; gap: spacing-04; width: 100%;
                // height: 2rem; padding: 0 spacing-05; color: text-primary; font-size: 0.875rem }`,
                // `:hover { background: layer-hover-01 }`.
                gpui_kit::base::Button::new(key("row"))
                    .track_focus(&focus)
                    .accessibility_label(file.path.clone())
                    .relative()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_start()
                    .w_full()
                    .gap(rem(12.))
                    .h(rem(32.))
                    .px(rem(16.))
                    .text_color(color(ColorRole::TextPrimary))
                    .cursor_pointer()
                    .when(hovered, |this| this.bg(color(ColorRole::LayerHover01)))
                    .on_hover(press::track_hover(hover))
                    .when_some(open, |this, f| {
                        this.on_click(move |_, w, cx| f(path.clone(), w, cx))
                    })
                    .child(DiffFileStatus::new(file.status))
                    // `.cu-turnchg-path { flex: 1; text-overflow: ellipsis; font-family: mono; font-size: 0.8125rem }`.
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .text_ellipsis()
                            .font(font(FontFamilyKey::Mono, 400))
                            .text_size(rem(13.))
                            .child(file.path.clone()),
                    )
                    .child(DiffStat::new(file.added, file.removed))
                    .when(ring, |this| this.child(press::ring()))
            })
            .collect();
        // `.cu-turnchg { background: layer-01; outline: 1px solid border-subtle-01; outline-offset: -1px }`.
        div()
            .id(id.clone())
            .flex()
            .flex_col()
            .w_full()
            .min_w_0()
            .bg(color(ColorRole::Layer01))
            .shadow(inset_frame(color(ColorRole::BorderSubtle01)))
            .text_color(color(ColorRole::TextPrimary))
            .child(head)
            // `.cu-turnchg-list { padding: spacing-02 0 }`.
            .child(div().flex().flex_col().py(rem(4.)).children(rows))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn totals_sum_the_files() {
        let files = vec![
            TurnFile::new("a.ts", 10, 2),
            TurnFile::new("b.ts", 4, 1).status(FileStatus::Added),
        ];
        assert_eq!(turn_totals(&files), (2, 14, 3));
        let t = TurnChangesTexts::default();
        assert_eq!((t.changed)(1), "1 file changed");
        assert_eq!((t.changed)(4), "4 files changed");
    }
}
