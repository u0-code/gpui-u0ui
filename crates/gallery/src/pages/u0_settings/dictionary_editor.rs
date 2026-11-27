//! u0 · Settings › DictionaryEditor: the reference dictionary, live (Enter
//! adds, duplicates are ignored, the list stays sorted).
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, rems,
};
use u0ui::u0::settings::DictionaryEditor;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        seed: 0,
        words: WORDS.iter().map(|&w| w.into()).collect(),
    })
    .into()
}

struct Page {
    seed: usize,
    words: Vec<gpui_kit::SharedString>,
}

const SEEDS: [&str; 2] = ["reference words", "empty"];
const WORDS: [&str; 5] = ["Carbon", "g100", "Kubernetes", "pnpm", "雨夹雪"];

/// `components/DictionaryEditor/preview.html`: 480px wide.
fn reference() -> impl IntoElement {
    div().w(rems(30.)).max_w_full().child(
        DictionaryEditor::new("dict-ref")
            .label("Dictation dictionary")
            .words(WORDS),
    )
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(DictionaryEditor::new("dict-thumb").words(["Carbon", "pnpm"]))
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let on_change_entity = entity.clone();
        let live = div().w(rems(30.)).max_w_full().child(
            DictionaryEditor::new("dict-live")
                .label("Dictation dictionary")
                .words(self.words.clone())
                .on_change(move |next, _, cx| {
                    let next = next.to_vec();
                    on_change_entity.update(cx, |p, cx| {
                        p.words = next;
                        cx.notify();
                    });
                }),
        );
        let config = controls::field(
            "words",
            controls::choices(
                "dict-seed",
                &SEEDS,
                self.seed,
                controls::setter(&entity, |p: &mut Page, i| {
                    p.seed = i;
                    p.words = if i == 0 {
                        WORDS.iter().map(|&w| w.into()).collect()
                    } else {
                        Vec::new()
                    };
                }),
            ),
        );
        Body::new()
            .playground(live)
            .hint("Type a word and press Enter; × removes one")
            .config(config)
            .example(controls::block("Reference preview", reference()))
    }
}
