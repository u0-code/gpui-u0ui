//! [`QueuedRunsControl`] (`.t3-queue`, "Queue" in `t3.css`): messages queued
//! while the agent runs, numbered, each with Send now, Edit and Remove.
//!
//! The header toggles the list (open by default, as `useState(true)`); its
//! chevron turns −90° when closed (`.t3-rot`, no transition in the reference).
//!
//! Options for the u0 board `ThreadComposer` (`.tc-later*`): a head icon
//! and the caller's head text, a head that does not collapse (no chevron),
//! unnumbered rows on one line with an inline note, an "Earlier" action and
//! a hairline frame. Left unset, the control draws as above.
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
    /// A note of the caller's (e.g. "Saved"), in `text-error` with `error`.
    pub note: Option<SharedString>,
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

    /// A note of the caller's, in place of the `error` / `editing` / `from`
    /// note (`.tc-later-row small`; `.tc-later-err` with `error`).
    pub fn note(mut self, note: impl Into<SharedString>) -> Self {
        self.note = Some(note.into());
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
    /// Earlier: move the row up one (with `on_earlier`).
    pub earlier: SharedString,
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
            earlier: "Earlier".into(),
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
    on_earlier: Option<IndexHandler>,
    icon: CarbonIcon,
    title: Option<SharedString>,
    collapsible: bool,
    numbered: bool,
    single_line: bool,
    bordered: bool,
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
            on_earlier: None,
            icon: CarbonIcon::ListChecked,
            title: None,
            collapsible: true,
            numbered: true,
            single_line: false,
            bordered: false,
        }
    }

    /// The head's icon (default `ListChecked`; the u0 board's is `Time`).
    pub fn icon(mut self, icon: CarbonIcon) -> Self {
        self.icon = icon;
        self
    }

    /// The head's text, in place of `texts.count` (e.g. "2 wait their turn ·
    /// sent in order when Claude’s turn ends").
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Whether the head opens and closes the list (default true). Off, the
    /// head is plain text with no chevron and the list always shows.
    pub fn collapsible(mut self, collapsible: bool) -> Self {
        self.collapsible = collapsible;
        self
    }

    /// Whether each row shows its number (default true).
    pub fn numbered(mut self, numbered: bool) -> Self {
        self.numbered = numbered;
        self
    }

    /// Each row on one line (`.tc-later-row`: min-height 32px, the text
    /// ellipsized, its note after it); default false (the note under the text).
    pub fn single_line(mut self, single_line: bool) -> Self {
        self.single_line = single_line;
        self
    }

    /// A 1px `border-subtle-01` frame (`.tc-later { box-shadow: inset 0 0 0 1px
    /// border-subtle-01 }`); default false.
    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }

    /// `onEarlier(i)`: an "Earlier" ghost action before Send now, on every
    /// row but the first.
    pub fn on_earlier(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_earlier = Some(Rc::new(f));
        self
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
        let count = self
            .title
            .clone()
            .unwrap_or_else(|| (texts.count)(self.items.len()));
        let collapsible = self.collapsible;
        let open = open || !collapsible;
        let single = self.single_line;
        let numbered = self.numbered;
        // `.tc-later-h svg { fill: icon-secondary }`.
        let head_ink = if single {
            color(ColorRole::IconSecondary)
        } else {
            secondary
        };
        // `.t3-queue-head { display: flex; align-items: center; gap: spacing-03; width: 100%;
        // height: size-sm; padding: 0 spacing-04; color: text-secondary; font-size: 0.75rem }`.
        let head = if collapsible {
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
                .child(press::icon(self.icon, 16., head_ink))
                .child(count)
                // `.t3-rot { transform: rotate(-90deg) }` when closed.
                .child(press::icon(CarbonIcon::ChevronDown, 12., secondary).rotate(
                    gpui_kit::radians(if open {
                        0.
                    } else {
                        -std::f32::consts::FRAC_PI_2
                    }),
                ))
                .when(ring, |this| this.child(press::ring()))
                .into_any_element()
        } else {
            // `.tc-later-h { display: flex; align-items: center; gap: 8px; height: 32px;
            // padding: 0 8px 0 12px; font-size: 12px; line-height: 16px; color: text-secondary }`.
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(rem(8.))
                .w_full()
                .h(rem(32.))
                .pl(rem(12.))
                .pr(rem(8.))
                .text_color(secondary)
                .text_size(rem(12.))
                .line_height(rem(16.))
                .child(press::icon(self.icon, 16., head_ink))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .text_ellipsis()
                        .child(count),
                )
                .into_any_element()
        };
        let rows = open.then(|| {
            self.items
                .into_iter()
                .enumerate()
                .map(|(i, item)| {
                    let sub = if let Some(note) = item.note.clone() {
                        // `.tc-later-row small { font-size: 12px; line-height: 16px;
                        // color: text-secondary }`, `.tc-later-err { color: text-error }`.
                        Some(
                            div()
                                .flex()
                                .flex_none()
                                .text_size(rem(12.))
                                .line_height(rem(16.))
                                .text_color(if item.error {
                                    color(ColorRole::TextError)
                                } else {
                                    secondary
                                })
                                .when(single, |this| this.mr(rem(4.)))
                                .child(note),
                        )
                    } else if item.error {
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
                                .flex()
                                .when(single, |this| this.flex_none().mr(rem(4.)))
                                .text_size(rem(12.))
                                .line_height(rem(16.))
                                .text_color(secondary)
                                .child(note)
                        })
                    };
                    let send = self.on_send_now.clone();
                    let edit = self.on_edit.clone();
                    let remove = self.on_remove.clone();
                    let earlier = self.on_earlier.clone().filter(|_| i > 0);
                    // `.t3-queue-item { display: flex; align-items: center; gap: spacing-03;
                    // padding: spacing-02 spacing-02 spacing-02 spacing-04; border-top: 1px solid border-subtle-01 }`.
                    // `.tc-later-row { min-height: 32px; padding: 0 4px 0 12px }` on one line.
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(rem(8.))
                        .map(|this| {
                            if single {
                                this.min_h(rem(32.))
                            } else {
                                this.pt(rem(4.)).pb(rem(4.))
                            }
                        })
                        .pl(rem(12.))
                        .pr(rem(4.))
                        .border_t_1()
                        .border_color(color(ColorRole::BorderSubtle01))
                        // `.t3-queue-n { width: 1.25rem; height: 1.25rem; background: layer-accent-01; font-size: 0.6875rem }`.
                        .when(numbered, |this| {
                            this.child(
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
                        })
                        .map(|this| {
                            if single {
                                // `.tc-later-row > span { flex: 1; min-width: 0; white-space: nowrap;
                                // overflow: hidden; text-overflow: ellipsis }` (14/20).
                                this.child(
                                    div()
                                        .flex_1()
                                        .min_w_0()
                                        .overflow_hidden()
                                        .whitespace_nowrap()
                                        .text_ellipsis()
                                        .text_size(rem(14.))
                                        .line_height(rem(20.))
                                        .text_color(color(ColorRole::TextPrimary))
                                        .child(item.text),
                                )
                                .children(sub)
                            } else {
                                // `.t3-queue-text { display: flex; flex: 1; min-width: 0;
                                // flex-direction: column; font-size: 0.875rem }`.
                                this.child(
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
                            }
                        })
                        .when_some(earlier, |this, f| {
                            this.child(
                                Button::new(
                                    (id.clone(), SharedString::from(format!("earlier-{i}"))),
                                    texts.earlier.clone(),
                                )
                                .variant(ButtonVariant::Ghost)
                                .size(ButtonSize::Sm)
                                .on_click(move |_, w, cx| f(i, w, cx)),
                            )
                        })
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
            .when(self.bordered, |this| {
                this.shadow(press::inset_frame(color(ColorRole::BorderSubtle01)))
            })
            .child(head)
            .when_some(rows, |this, rows| this.children(rows))
            .min_w_0()
    }
}
