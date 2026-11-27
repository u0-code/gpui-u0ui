//! EditableText: inline rename (`EditableText` in `src/ghost/primitives.jsx`).
//! A pencil button appears on hover; pressing it swaps the text for a field.
//!
//! - **Behavior**: the field opens focused with its text selected. Enter or
//!   blur commits the trimmed text (an empty result keeps the old value);
//!   Escape cancels. A `validate` error keeps the field open, outlined in
//!   `support-error`, with the message under it. Kit's [`InputState`] edits.
//! - **Appearance**: `.g-edit-text` (text, then a 24px `icon-secondary` pencil
//!   at opacity 0 until the row is hovered or the button has keyboard focus;
//!   placeholder in `text-placeholder`), `.g-edit-input` (`density-field-sm`
//!   high, `field-01`, 2px `focus` outline).
use super::parts::{self, rem};
use crate::{
    components::form,
    density,
    icons::{CarbonIcon, Icon},
    theme::color,
    tokens::ColorRole,
};
use gpui_kit::{
    App, AppContext as _, Context, CursorStyle, ElementId, Entity, Focusable as _, Hsla,
    InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement as _, Styled, Subscription, Window,
    component::input::{Escape, InputEvent, InputState},
    div,
    prelude::FluentBuilder as _,
    px,
};
use std::rc::Rc;

/// A validator: `validate(text) → error message`.
pub type ValidateFn = dyn Fn(&str) -> Option<SharedString>;
type Validate = Rc<ValidateFn>;
type Handler = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;

/// The outcome of committing a draft.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Commit {
    /// Close and set this value.
    Set(String),
    /// Close and keep the value (the trimmed draft was empty).
    Keep,
    /// Stay open with this error.
    Error(SharedString),
}

/// `commit`: trim, validate, then set a non-empty result.
pub fn commit(draft: &str, validate: Option<&ValidateFn>) -> Commit {
    let t = draft.trim();
    if let Some(err) = validate.and_then(|v| v(t)).filter(|e| !e.is_empty()) {
        return Commit::Error(err);
    }
    if t.is_empty() {
        Commit::Keep
    } else {
        Commit::Set(t.to_string())
    }
}

/// `.g-edit-input` outline: `focus`, or `support-error` with an error.
pub fn input_outline(error: bool) -> Hsla {
    color(if error {
        ColorRole::SupportError
    } else {
        ColorRole::Focus
    })
}

struct Model {
    value: SharedString,
    editing: bool,
    error: Option<SharedString>,
    input: Entity<InputState>,
    validate: Option<Validate>,
    on_change: Option<Handler>,
    controlled: bool,
    _subscription: Subscription,
}

impl Model {
    fn start(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editing = true;
        self.error = None;
        let v = self.value.clone();
        self.input.update(cx, |s, cx| {
            s.set_value(v, window, cx);
            // `onFocus={(e) => e.target.select()}`.
            s.select_all(window, cx);
        });
        let focus = self.input.read(cx).focus_handle(cx);
        focus.focus(window, cx);
        cx.notify();
    }

    fn commit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.editing {
            return;
        }
        let draft = self.input.read(cx).value().to_string();
        match commit(&draft, self.validate.as_deref()) {
            Commit::Error(e) => self.error = Some(e),
            outcome => {
                if let Commit::Set(t) = outcome {
                    let t: SharedString = t.into();
                    if !self.controlled {
                        self.value = t.clone();
                    }
                    if let Some(f) = self.on_change.clone() {
                        f(t, window, cx);
                    }
                }
                self.editing = false;
                self.error = None;
            }
        }
        cx.notify();
    }

    fn cancel(&mut self, cx: &mut Context<Self>) {
        self.editing = false;
        self.error = None;
        cx.notify();
    }
}

/// Inline-renamable text. See the module docs.
#[derive(IntoElement)]
pub struct EditableText {
    id: ElementId,
    value: Option<SharedString>,
    default_value: SharedString,
    placeholder: SharedString,
    validate: Option<Validate>,
    on_value_change: Option<Handler>,
    input_label: SharedString,
    rename_label: Option<SharedString>,
}

impl EditableText {
    /// Editable text with a stable `id` (uncontrolled, empty).
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            value: None,
            default_value: SharedString::default(),
            // The reference's defaults; override per locale.
            placeholder: "Untitled".into(),
            validate: None,
            on_value_change: None,
            input_label: "Rename".into(),
            rename_label: None,
        }
    }

    /// Control the value.
    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// The initial value when uncontrolled.
    pub fn default_value(mut self, value: impl Into<SharedString>) -> Self {
        self.default_value = value.into();
        self
    }

    /// Shown (in `text-placeholder`) while the value is empty (default "Untitled").
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// `validate(text) → error message`; `None` (or empty) accepts.
    pub fn validate(mut self, f: impl Fn(&str) -> Option<SharedString> + 'static) -> Self {
        self.validate = Some(Rc::new(f));
        self
    }

    /// Accessible names of the field (default "Rename") and of the pencil
    /// button (default `Rename <value>`).
    pub fn labels(
        mut self,
        input: impl Into<SharedString>,
        button: impl Into<SharedString>,
    ) -> Self {
        self.input_label = input.into();
        self.rename_label = Some(button.into());
        self
    }

    /// Called with each committed value.
    pub fn on_value_change(
        mut self,
        f: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_value_change = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for EditableText {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let initial = self.value.clone().unwrap_or(self.default_value.clone());
        let model = window.use_keyed_state((id.clone(), "edit"), cx, |window, cx| {
            let input = cx.new(|cx| InputState::new(window, cx));
            let subscription = cx.subscribe_in(
                &input,
                window,
                |this: &mut Model, _, e, window, cx| match e {
                    InputEvent::Blur | InputEvent::PressEnter { .. } => this.commit(window, cx),
                    InputEvent::Change if this.error.is_some() => {}
                    _ => {}
                },
            );
            Model {
                value: initial,
                editing: false,
                error: None,
                input,
                validate: None,
                on_change: None,
                controlled: false,
                _subscription: subscription,
            }
        });
        model.update(cx, |m, _| {
            m.validate = self.validate.clone();
            m.on_change = self.on_value_change.clone();
            m.controlled = self.value.is_some();
            if let Some(v) = &self.value {
                m.value = v.clone();
            }
        });
        let (value, editing, error, input) = {
            let m = model.read(cx);
            (m.value.clone(), m.editing, m.error.clone(), m.input.clone())
        };

        if editing {
            let field_sm = density::current().field_sm().px_value();
            let cancel = model.clone();
            return div()
                .id(id.clone())
                .flex()
                .flex_col()
                .gap(px(2.))
                .capture_action(move |_: &Escape, _, cx| {
                    // `Escape: setD(v); setEdit(false); setErr('')`.
                    cancel.update(cx, |m, cx| m.cancel(cx));
                    cx.stop_propagation();
                })
                .child(
                    div()
                        .relative()
                        .flex()
                        .items_center()
                        .h(rem(field_sm))
                        .px(rem(8.))
                        .bg(color(ColorRole::Field01))
                        .child(
                            gpui_kit::component::input::Input::new(&input)
                                .appearance(false)
                                .p_0()
                                .h_full()
                                .flex_1()
                                .min_w_0()
                                .aria_label(self.input_label.clone())
                                .text_color(color(ColorRole::TextPrimary)),
                        )
                        .child(form::inset_outline(2., input_outline(error.is_some()))),
                )
                .children(error.map(parts::error))
                .into_any_element();
        }

        let button_focus = parts::keyed_focus((id.clone(), "btn-focus"), window, cx);
        let ring = parts::focus_visible(&button_focus, window);
        let row_hover = window.use_keyed_state((id.clone(), "row-hover"), cx, |_, _| false);
        let btn_hover = window.use_keyed_state((id.clone(), "btn-hover"), cx, |_, _| false);
        // `.g-edit-text:hover .g-edit-btn, .g-edit-btn:focus-visible { opacity: 1 }`.
        let visible = *row_hover.read(cx) || ring;
        let rename_label = self
            .rename_label
            .clone()
            .unwrap_or_else(|| format!("Rename {value}").into());
        let start = model.clone();
        let text = if value.is_empty() {
            // `.g-edit-text em { color: var(--text-placeholder); font-style: normal }`.
            div()
                .text_color(color(ColorRole::TextPlaceholder))
                .child(self.placeholder.clone())
        } else {
            div().child(value.clone())
        };
        let bh = btn_hover.clone();
        div()
            .id(id.clone())
            .flex()
            .items_center()
            .gap(rem(4.))
            .on_hover(move |h, _, cx| {
                row_hover.update(cx, |v, cx| {
                    *v = *h;
                    cx.notify();
                })
            })
            .child(text)
            .child(
                gpui_kit::base::Button::new((id.clone(), "rename"))
                    .track_focus(&button_focus)
                    .accessibility_label(rename_label)
                    .relative()
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .size(rem(24.))
                    .opacity(if visible { 1. } else { 0. })
                    // `.g-edit-btn:hover { background: var(--layer-hover-01) }`.
                    .when(*btn_hover.read(cx), |b| {
                        b.bg(color(ColorRole::LayerHover01))
                    })
                    .cursor(CursorStyle::PointingHand)
                    .on_hover(move |h, _, cx| {
                        bh.update(cx, |v, cx| {
                            *v = *h;
                            cx.notify();
                        })
                    })
                    .child(
                        Icon::carbon(CarbonIcon::Edit)
                            .size_length(px(14.))
                            .color(color(ColorRole::IconSecondary)),
                    )
                    .when(ring, |b| b.child(parts::focus_ring()))
                    .on_click(move |_, window, cx| start.update(cx, |m, cx| m.start(window, cx))),
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commit_trims_validates_and_keeps_empty() {
        assert_eq!(commit("  carbon  ", None), Commit::Set("carbon".into()));
        assert_eq!(commit("   ", None), Commit::Keep);
        let required = |t: &str| t.is_empty().then(|| SharedString::from("Name is required"));
        assert_eq!(
            commit(" ", Some(&required)),
            Commit::Error("Name is required".into())
        );
        assert_eq!(commit("x", Some(&required)), Commit::Set("x".into()));
    }
}
