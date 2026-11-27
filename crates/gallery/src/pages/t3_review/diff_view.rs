//! T3 · Diff & review › DiffView: the reference hunk with its review
//! thread under the first added line, and a live diff where the gutter "+"
//! opens a draft thread under that line.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, rems,
};
use u0ui::prelude::*;
use u0ui::t3::review::{DiffHunk, DiffLine, DiffView, ReviewComment, ReviewThreadCard, Segment};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        drafts: vec![(0, 2)],
        words: false,
    })
    .into()
}

struct Page {
    drafts: Vec<(usize, usize)>,
    words: bool,
}

/// `components/DiffView/preview.html`.
fn hunk(words: bool) -> DiffHunk {
    let removed = DiffLine::removed(39, "  const value = props.value ?? new Date();");
    let added = DiffLine::added(39, "  const value = props.value ?? null;");
    DiffHunk::new(
        "@@ -38,7 +38,9 @@ export function DatePicker(props) {",
        [
            DiffLine::context(38, 38, "  const [open, setOpen] = useState(false);"),
            if words {
                removed.segments([
                    Segment::Text("  const value = props.value ?? ".into()),
                    Segment::Word("new Date()".into()),
                    Segment::Text(";".into()),
                ])
            } else {
                removed
            },
            if words {
                added.segments([
                    Segment::Text("  const value = props.value ?? ".into()),
                    Segment::Word("null".into()),
                    Segment::Text(";".into()),
                ])
            } else {
                added
            },
            DiffLine::added(40, "  const tz = useTimeZone();"),
            DiffLine::context(40, 41, "  return ("),
        ],
    )
}

fn frame(content: impl IntoElement) -> gpui_kit::Div {
    // `maxWidth: 760; outline: 1px solid var(--border-subtle-01)`.
    div()
        .w_full()
        .max_w(rems(47.5))
        .border_1()
        .border_color(color(ColorRole::BorderSubtle01))
        .child(content)
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .overflow_hidden()
        .child(DiffView::new("dv-thumb").hunks([hunk(false)]))
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let reference = frame(DiffView::new("dv-ref").hunks([hunk(false)]).annotation(
            0,
            2,
            ReviewThreadCard::new("dv-ref-thread").comments([ReviewComment::new(
                "jd",
                "2m",
                "Should null be allowed here? The form expects a date.",
            )]),
        ));

        let mut view = DiffView::new("dv-live")
            .hunks([hunk(self.words)])
            .on_comment({
                let entity = entity.clone();
                move |h, l, _, cx| {
                    entity.update(cx, |this, cx| {
                        if !this.drafts.contains(&(h, l)) {
                            this.drafts.push((h, l));
                        }
                        cx.notify();
                    })
                }
            });
        for (i, (h, l)) in self.drafts.iter().copied().enumerate() {
            let thread = if (h, l) == (0, 2) {
                ReviewThreadCard::new(("dv-live-thread", i)).comments([ReviewComment::new(
                    "jd",
                    "2m",
                    "Should null be allowed here? The form expects a date.",
                )])
            } else {
                ReviewThreadCard::new(("dv-live-thread", i))
                    .pending(true)
                    .comments([ReviewComment::new(
                        "you",
                        "now",
                        "New comment on this line.",
                    )])
                    .on_resolve({
                        let entity = entity.clone();
                        move |_, cx| {
                            entity.update(cx, |this, cx| {
                                this.drafts.retain(|d| *d != (h, l));
                                cx.notify();
                            })
                        }
                    })
            };
            view = view.annotation(h, l, thread);
        }
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::toggle(
                "dv-words",
                "word highlights",
                self.words,
                controls::setter(&entity, |this: &mut Page, v| this.words = v),
            ));

        Body::new()
            .playground(frame(view))
            .hint("Hover a line and press + to open a draft thread under it")
            .config(config)
            .example(controls::block("Reference preview", reference))
    }
}
