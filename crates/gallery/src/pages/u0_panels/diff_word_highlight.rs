//! u0 · Right column › DiffWordHighlight: `DiffView`'s word-level
//! highlight (`.g-word-add` / `.g-word-del`, the `diff-*-word-background`
//! tokens, a strike-through on removed words), live with word segments on
//! or off, and the reference hunk. The behavior lives in
//! [`u0ui::t3::review::DiffView`] ([`Segment::Word`]); this page shows it.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, rems,
};
use u0ui::t3::review::{DiffHunk, DiffLine, DiffView, Segment};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page { words: true }).into()
}

struct Page {
    words: bool,
}

/// `components/DiffWordHighlight/preview.html`'s hunk; without `words` the
/// changed lines are plain text.
fn hunk(words: bool) -> DiffHunk {
    let removed = DiffLine::removed(13, "  layer01: '#393939',");
    let added = DiffLine::added(13, "  layer01: '#333333', // v11.86");
    DiffHunk::new(
        "@@ -12,3 +12,3 @@ export const g90 = {",
        [
            DiffLine::context(12, 12, "  background: '#262626',"),
            if words {
                removed.segments([
                    Segment::Text("  layer01: ".into()),
                    Segment::Word("'#393939'".into()),
                    Segment::Text(",".into()),
                ])
            } else {
                removed
            },
            if words {
                added.segments([
                    Segment::Text("  layer01: ".into()),
                    Segment::Word("'#333333'".into()),
                    Segment::Text(", // v11.86".into()),
                ])
            } else {
                added
            },
            DiffLine::context(14, 14, "  focus: '#ffffff',"),
        ],
    )
}

/// `{ width: 600 }`.
fn frame(content: impl IntoElement) -> gpui_kit::Div {
    div().w_full().max_w(rems(37.5)).child(content)
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .overflow_hidden()
        .child(DiffView::new("dwh-thumb").hunks([hunk(true)]))
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let e = cx.entity();
        let live =
            frame(DiffView::new(("dwh-live", self.words as usize)).hunks([hunk(self.words)]));
        let config = div().child(controls::toggle(
            "dwh-words",
            "word segments ({ w })",
            self.words,
            controls::setter(&e, |p: &mut Page, v| p.words = v),
        ));
        Body::new()
            .playground(live)
            .config(config)
            .example(controls::block(
                "Reference preview",
                frame(DiffView::new("dwh-ref").hunks([hunk(true)])),
            ))
    }
}
