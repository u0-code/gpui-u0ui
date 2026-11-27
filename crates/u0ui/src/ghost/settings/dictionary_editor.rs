//! DictionaryEditor: the words dictation should know (`DictionaryEditor` in
//! `src/ghost/admin.jsx`, `.g-dict` in `src/ghost.css`): an input with a
//! secondary "Add" button, the words as removable chips and a count.
//!
//! - **Behavior**: Enter or Add puts the trimmed word into the list, sorted
//!   with `localeCompare`; duplicates are ignored and the input clears either
//!   way. Add is disabled while the input is blank. Each chip's × removes it.
//!   The list is the editor's own state, seeded from [`DictionaryEditor::words`].
//!
//! ```ignore
//! DictionaryEditor::new("dict").label("Dictation dictionary").words(["Carbon", "pnpm"])
//! ```
use crate::{
    components::{Button, ButtonSize, ButtonVariant},
    ghost::{
        fields,
        primitives::{IconButton, parts},
    },
    icons::CarbonIcon,
    t3::craft::{rem, sans},
    theme::color,
    tokens::{self, ColorRole},
};
use gpui_kit::{
    App, AppContext as _, Context, ElementId, Entity, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, Subscription, Window,
    component::input::{InputEvent, InputState},
    div, px,
};
use std::rc::Rc;

/// The editor's user-facing strings (the reference's English defaults).
#[derive(Clone)]
pub struct DictionaryTexts {
    /// The input's placeholder.
    pub placeholder: SharedString,
    /// The input's accessible name: `"New word"`.
    pub input: SharedString,
    /// `"Add"`.
    pub add: SharedString,
    /// A chip's × label: `"Remove {word}"`.
    pub remove: Rc<dyn Fn(&str) -> SharedString>,
    /// The count line: `"{n} words"`.
    pub count: Rc<dyn Fn(usize) -> SharedString>,
}

impl Default for DictionaryTexts {
    fn default() -> Self {
        Self {
            placeholder: "Add a word or name the transcriber should know".into(),
            input: "New word".into(),
            add: "Add".into(),
            remove: Rc::new(|w| format!("Remove {w}").into()),
            count: Rc::new(|n| format!("{n} words").into()),
        }
    }
}

/// `[...w, t].sort((a, b) => a.localeCompare(b))` unless `t` is blank or
/// already there (`None`). `localeCompare` orders case-insensitively first.
pub fn add_word(words: &[SharedString], word: &str) -> Option<Vec<SharedString>> {
    let t = word.trim();
    if t.is_empty() || words.iter().any(|w| w.as_ref() == t) {
        return None;
    }
    let mut next = words.to_vec();
    next.push(t.to_string().into());
    next.sort_by(|a, b| locale_cmp(a, b));
    Some(next)
}

/// An approximation of `String.prototype.localeCompare` (root collation):
/// case-folded comparison first, lowercase before uppercase on a tie.
pub fn locale_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    a.to_lowercase()
        .cmp(&b.to_lowercase())
        .then_with(|| b.cmp(a))
}

struct Model {
    words: Vec<SharedString>,
    input: Entity<InputState>,
    _sub: Subscription,
}

impl Model {
    fn add(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let q = self.input.read(cx).value().to_string();
        if let Some(next) = add_word(&self.words, &q) {
            self.words = next;
        }
        self.input.update(cx, |s, cx| s.set_value("", window, cx));
        cx.notify();
    }
}

/// A dictionary editor. See the module docs.
#[derive(IntoElement)]
pub struct DictionaryEditor {
    id: ElementId,
    words: Vec<SharedString>,
    label: SharedString,
    texts: DictionaryTexts,
}

impl DictionaryEditor {
    /// An empty dictionary.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            words: Vec::new(),
            label: "Dictionary".into(),
            texts: DictionaryTexts::default(),
        }
    }

    /// The starting words (`words`).
    pub fn words(mut self, words: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.words = words.into_iter().map(Into::into).collect();
        self
    }

    /// The list's accessible name (`label`, default "Dictionary").
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: DictionaryTexts) -> Self {
        self.texts = texts;
        self
    }
}

impl RenderOnce for DictionaryEditor {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let initial = self.words.clone();
        let model = window.use_keyed_state((id.clone(), "dict"), cx, |window, cx| {
            let input = cx.new(|cx| InputState::new(window, cx));
            let sub = cx.subscribe_in(&input, window, |m: &mut Model, _, e, window, cx| match e {
                InputEvent::PressEnter { .. } => m.add(window, cx),
                _ => cx.notify(),
            });
            Model {
                words: initial,
                input,
                _sub: sub,
            }
        });
        let (words, input) = {
            let m = model.read(cx);
            (m.words.clone(), m.input.clone())
        };
        let blank = input.read(cx).value().trim().is_empty();
        let count = words.len();
        let add = model.clone();

        // `.g-dict { display: flex; flex-direction: column; gap: var(--spacing-03) }`.
        div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_03)
            .child(
                // `.g-dict-add { display: flex; gap: var(--spacing-03); align-items: center }`.
                div()
                    .flex()
                    .items_center()
                    .gap(tokens::SPACING_03)
                    .child(div().flex_1().min_w_0().child(fields::line_input(
                        &input,
                        false,
                        fields::field_fill(false),
                        Some(self.texts.placeholder.clone()),
                        window,
                        cx,
                    )))
                    .child(
                        div().flex_none().child(
                            Button::new((id.clone(), "add"), self.texts.add.clone())
                                .variant(ButtonVariant::Secondary)
                                .size(ButtonSize::Sm)
                                .disabled(blank)
                                .on_click(move |_, w, cx| {
                                    add.update(cx, |m, cx| m.add(w, cx));
                                }),
                        ),
                    ),
            )
            .child(
                // `.g-dict-list { display: flex; flex-wrap: wrap; gap: var(--spacing-02) }`.
                div().flex().flex_wrap().gap(tokens::SPACING_02).children(
                    words.into_iter().enumerate().map(|(i, w)| {
                        let model = model.clone();
                        let word = w.clone();
                        // `.g-dict-list li { display: inline-flex; align-items:
                        // center; gap: 2px; height: 1.5rem; padding-left:
                        // var(--spacing-03); background: var(--layer-01);
                        // font-size: 0.8125rem; box-shadow: inset 0 0 0 1px
                        // var(--border-subtle-01) }`.
                        sans(div(), 13., 400)
                            .flex()
                            .items_center()
                            .gap(px(2.))
                            .h(rem(24.))
                            .pl(tokens::SPACING_03)
                            .bg(color(ColorRole::Layer01))
                            .border_1()
                            .border_color(color(ColorRole::BorderSubtle01))
                            .text_color(color(ColorRole::TextPrimary))
                            .child(w.clone())
                            .child(
                                // `.g-dict-list .g-iconbtn { width: 1.5rem; height: 1.5rem }`
                                // with `<I.Close size={14} />`.
                                IconButton::new(
                                    parts_id(&id, i),
                                    CarbonIcon::Close,
                                    (self.texts.remove)(&w),
                                )
                                .size_px(22.)
                                .icon_px(14.)
                                .on_click(move |_, _, cx| {
                                    let word = word.clone();
                                    model.update(cx, |m, cx| {
                                        m.words.retain(|x| *x != word);
                                        cx.notify();
                                    })
                                }),
                            )
                    }),
                ),
            )
            .child(parts::helper((self.texts.count)(count)))
    }
}

fn parts_id(id: &ElementId, i: usize) -> ElementId {
    (id.clone(), SharedString::from(format!("rm-{i}"))).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<SharedString> {
        v.iter()
            .map(|x| SharedString::from(x.to_string()))
            .collect()
    }

    #[test]
    fn adds_sorted_and_ignores_duplicates() {
        let w = s(&["Carbon", "g100", "Kubernetes", "pnpm"]);
        assert_eq!(
            add_word(&w, " Docker "),
            Some(s(&["Carbon", "Docker", "g100", "Kubernetes", "pnpm"]))
        );
        assert_eq!(add_word(&w, "pnpm"), None);
        assert_eq!(add_word(&w, "   "), None);
    }

    #[test]
    fn locale_order_is_case_insensitive() {
        let mut v = s(&["pnpm", "Carbon", "g100", "Kubernetes"]);
        v.sort_by(|a, b| locale_cmp(a, b));
        assert_eq!(v, s(&["Carbon", "g100", "Kubernetes", "pnpm"]));
    }
}
