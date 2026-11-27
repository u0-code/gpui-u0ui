//! Ghost · Settings › DictionaryEditor: the reference dictionary, live (Enter
//! adds, duplicates are ignored, the list stays sorted).
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, rems,
};
use u0ui::ghost::settings::DictionaryEditor;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page { seed: 0, epoch: 0 }).into()
}

struct Page {
    seed: usize,
    epoch: usize,
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
        // A new id re-seeds the editor's own state.
        let words: Vec<&str> = if self.seed == 0 {
            WORDS.to_vec()
        } else {
            vec![]
        };
        let live = div().w(rems(30.)).max_w_full().child(
            DictionaryEditor::new(("dict-live", self.epoch * 2 + self.seed))
                .label("Dictation dictionary")
                .words(words),
        );
        let config = controls::field(
            "words",
            controls::choices(
                "dict-seed",
                &SEEDS,
                self.seed,
                controls::setter(&entity, |p: &mut Page, i| {
                    p.seed = i;
                    p.epoch += 1;
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
