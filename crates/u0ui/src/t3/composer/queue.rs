//! [`QueuedRunsControl`] (`.t3-queue`, "Queue" in `t3.css`): messages queued
//! while the agent runs, numbered, each with Send now, Edit and Remove.
//!
//! The header toggles the list (open by default, as `useState(true)`); its
//! chevron turns −90° when closed (`.t3-rot`, no transition in the reference).
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::t3::press::{self, rem};
use crate::{icons::CarbonIcon, theme::color, tokens::ColorRole};
use gpui_kit::{
    App, CursorStyle, ElementId, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    Window, div, prelude::FluentBuilder as _,
};
use std::rc::Rc;

/// One queued message (`{ text, editing, error, from }`).
#[derive(Clone, Default)]
pub struct QueuedRun {
    /// `text`.
    pub text: SharedString,
    /// `editing`: being edited in the composer.
    pub editing: bool,
    /// `error`: could not be sent.
    pub error: bool,
    /// `from`: where it was queued from.
    pub from: Option<SharedString>,
}

impl QueuedRun {
    /// A queued `text`.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            ..Default::default()
        }
    }

    /// `editing`.
    pub fn editing(mut self, editing: bool) -> Self {
        self.editing = editing;
        self
    }

    /// `error`.
    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }

    /// `from`.
    pub fn from(mut self, from: impl Into<SharedString>) -> Self {
        self.from = Some(from.into());
        self
    }
}

/// The control's user-facing strings (the reference's English defaults).
#[derive(Clone)]
pub struct QueuedRunsTexts {
    /// Header: `"{n} queued"`.
    pub count: Rc<dyn Fn(usize) -> SharedString>,
    /// `error` note.
    pub error: SharedString,
    /// `editing` note.
    pub editing: SharedString,
    /// Send now.
    pub send_now: SharedString,
    /// Edit (tooltip).
    pub edit: SharedString,
    /// Remove (tooltip).
    pub remove: SharedString,
}

impl Default for QueuedRunsTexts {
    fn default() -> Self {
        Self {
            count: Rc::new(|n| format!("{n} queued").into()),
            error: "Could not send queued message".into(),
            editing: "Editing in the composer".into(),
            send_now: "Send now".into(),
            edit: "Edit".into(),
            remove: "Remove from queue".into(),
        }
    }
}

type IndexHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// The queued-runs list. See the module docs.
#[derive(IntoElement)]
pub struct QueuedRunsControl {
    id: ElementId,
    items: Vec<QueuedRun>,
    texts: QueuedRunsTexts,
    on_send_now: Option<IndexHandler>,
    on_edit: Option<IndexHandler>,
    on_remove: Option<IndexHandler>,
}

impl QueuedRunsControl {
    /// The control for `items`.
    pub fn new(id: impl Into<ElementId>, items: Vec<QueuedRun>) -> Self {
        Self {
            id: id.into(),
            items,
            texts: QueuedRunsTexts::default(),
            on_send_now: None,
            on_edit: None,
            on_remove: None,
        }
    }

    /// Replace the strings.
    pub fn texts(mut self, texts: QueuedRunsTexts) -> Self {
        self.texts = texts;
        self
    }

    /// `onSendNow(i)`.
    pub fn on_send_now(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_send_now = Some(Rc::new(f));
        self
    }

    /// `onEdit(i)`.
    pub fn on_edit(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_edit = Some(Rc::new(f));
        self
    }

    /// `onRemove(i)`.
    pub fn on_remove(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_remove = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for QueuedRunsControl {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let open_state = window.use_keyed_state((id.clone(), "open"), cx, |_, _| true);
        let open = *open_state.read(cx);
        let (focus, ring) = press::focus((id.clone(), "head-focus"), window, cx);
        let secondary = color(ColorRole::TextSecondary);
        let texts = self.texts.clone();
        let count = (texts.count)(self.items.len());
        // `.t3-queue-head { display: flex; align-items: center; gap: spacing-03; width: 100%;
        // height: size-sm; padding: 0 spacing-04; color: text-secondary; font-size: 0.75rem }`.
        let head =
            gpui_kit::base::Button::new((id.clone(), "head"))
                .track_focus(&focus)
                .accessibility_label(count.clone())
                .justify_start()
                .relative()
                .flex()
                .flex_row()
                .items_center()
                .gap(rem(8.))
                .w_full()
                .h(rem(32.))
                .px(rem(12.))
                .text_color(secondary)
                .text_size(rem(12.))
                .line_height(rem(16.))
                .cursor(CursorStyle::PointingHand)
                .on_click(move |_, _, cx| {
                    open_state.update(cx, |o, cx| {
                        *o = !*o;
                        cx.notify();
                    })
                })
                .child(press::icon(CarbonIcon::ListChecked, 16., secondary))
                .child(count)
                // `.t3-rot { transform: rotate(-90deg) }` when closed.
                .child(press::icon(CarbonIcon::ChevronDown, 12., secondary).rotate(
                    gpui_kit::radians(if open {
                        0.
                    } else {
                        -std::f32::consts::FRAC_PI_2
                    }),
                ))
                .when(ring, |this| this.child(press::ring()));
        let rows = open.then(|| {
            self.items
                .into_iter()
                .enumerate()
                .map(|(i, item)| {
                    let sub = if item.error {
                        // `.t3-queue-err { display: inline-flex; gap: spacing-02; align-items: center;
                        // font-size: 0.75rem; color: text-error }`, icon `support-error`.
                        Some(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(rem(4.))
                                .text_size(rem(12.))
                                .line_height(rem(16.))
                                .text_color(color(ColorRole::TextError))
                                .child(press::icon(
                                    CarbonIcon::ErrorFilled,
                                    12.,
                                    color(ColorRole::SupportError),
                                ))
                                .child(texts.error.clone()),
                        )
                    } else {
                        let note = if item.editing {
                            Some(texts.editing.clone())
                        } else {
                            item.from.clone()
                        };
                        // `.t3-queue-sub { font-size: 0.75rem; color: text-secondary }`.
                        note.map(|note| {
                            div()
                                .text_size(rem(12.))
                                .line_height(rem(16.))
                                .text_color(secondary)
                                .child(note)
                        })
                    };
                    let send = self.on_send_now.clone();
                    let edit = self.on_edit.clone();
                    let remove = self.on_remove.clone();
                    // `.t3-queue-item { display: flex; align-items: center; gap: spacing-03;
                    // padding: spacing-02 spacing-02 spacing-02 spacing-04; border-top: 1px solid border-subtle-01 }`.
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(rem(8.))
                        .pt(rem(4.))
                        .pb(rem(4.))
                        .pl(rem(12.))
                        .pr(rem(4.))
                        .border_t_1()
                        .border_color(color(ColorRole::BorderSubtle01))
                        // `.t3-queue-n { width: 1.25rem; height: 1.25rem; background: layer-accent-01; font-size: 0.6875rem }`.
                        .child(
                            div()
                                .flex()
                                .flex_none()
                                .items_center()
                                .justify_center()
                                .size(rem(20.))
                                .bg(color(ColorRole::LayerAccent01))
                                .text_size(rem(11.))
                                .text_color(color(ColorRole::TextPrimary))
                                .child(SharedString::from((i + 1).to_string())),
                        )
                        // `.t3-queue-text { display: flex; flex: 1; min-width: 0; flex-direction: column; font-size: 0.875rem }`.
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .flex_1()
                                .min_w_0()
                                .text_size(rem(14.))
                                .line_height(rem(18.))
                                .text_color(color(ColorRole::TextPrimary))
                                .child(item.text)
                                .children(sub),
                        )
                        .child(
                            Button::new(
                                (id.clone(), SharedString::from(format!("send-{i}"))),
                                texts.send_now.clone(),
                            )
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::Sm)
                            .when_some(send, |b, f| b.on_click(move |_, w, cx| f(i, w, cx))),
                        )
                        .child(
                            Button::new(
                                (id.clone(), SharedString::from(format!("edit-{i}"))),
                                texts.edit.clone(),
                            )
                            .icon(CarbonIcon::Edit)
                            .icon_only(true)
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::Sm)
                            .when_some(edit, |b, f| b.on_click(move |_, w, cx| f(i, w, cx))),
                        )
                        .child(
                            Button::new(
                                (id.clone(), SharedString::from(format!("remove-{i}"))),
                                texts.remove.clone(),
                            )
                            .icon(CarbonIcon::TrashCan)
                            .icon_only(true)
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::Sm)
                            .when_some(remove, |b, f| b.on_click(move |_, w, cx| f(i, w, cx))),
                        )
                })
                .collect::<Vec<_>>()
        });
        // `.t3-queue { background: layer-01 }`.
        div()
            .flex()
            .flex_col()
            .w_full()
            .bg(color(ColorRole::Layer01))
            .child(head)
            .when_some(rows, |this, rows| this.children(rows))
            .min_w_0()
    }
}
