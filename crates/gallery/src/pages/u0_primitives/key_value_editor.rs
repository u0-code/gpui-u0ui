//! u0 · Primitives › KeyValueEditor: the reference rows (with the
//! duplicate `API_URL`), live, reporting the row count.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, rems,
};
use u0ui::prelude::*;
use u0ui::u0::primitives::{EnvVar, KeyValueEditor};

fn rows() -> Vec<EnvVar> {
    vec![
        EnvVar::new("API_URL", "https://api.example.com"),
        EnvVar::new("OPENAI_API_KEY", "sk-live-abc").secret(true),
        EnvVar::new("API_URL", "http://localhost:3000"),
    ]
}

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| KeyValueEditorPage {
        count: 3,
        secrets: 1,
    })
    .into()
}

struct KeyValueEditorPage {
    count: usize,
    secrets: usize,
}

/// `components/KeyValueEditor/preview.html`: 520px wide.
pub fn reference(id: &'static str) -> impl IntoElement {
    div()
        .w(rems(32.5))
        .max_w_full()
        .child(KeyValueEditor::new(id, rows()))
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(KeyValueEditor::new("thumb-kv", rows().into_iter().take(2)))
        .into_any_element()
}

impl Render for KeyValueEditorPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let live = div()
            .w(rems(32.5))
            .max_w_full()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(
                KeyValueEditor::new("kv-live", rows()).on_change(controls::setter(
                    &entity,
                    |this: &mut Self, vars: Vec<EnvVar>| {
                        this.count = vars.len();
                        this.secrets = vars.iter().filter(|v| v.secret).count();
                    },
                )),
            )
            .child(
                div()
                    .type_style(TypeStyle::Code01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(format!("{} rows, {} secret", self.count, self.secrets)),
            );
        Body::new()
            .playground(live)
            .hint("Names become UPPER_SNAKE; duplicates turn red; Paste .env reads KEY=value lines from the clipboard")
            .example(controls::block("Reference preview", reference("kv-ref")))
    }
}
