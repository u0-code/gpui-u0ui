//! [`Composer`] (`.t3-composer`, "Composer" in `t3.css` with the gemini-A
//! and r2-gal passes): the message box.
//!
//! - **Box**: a closed square `field-01` box with a 1px `border-subtle-01`
//!   frame that turns `focus` while anything inside it has focus
//!   (`:focus-within`: the input, a chip ×, a footer control, Send).
//! - **Context row** (16px in): context chips and the attachment strip in
//!   their 32px composer forms.
//! - **Input**: Kit's multi-line text, 14/20, two rows (72px with padding),
//!   `text-helper` placeholder, at least 72px tall. Enter sends (Shift+Enter
//!   is a new line); while running Enter queues and ⌘/Ctrl+Enter steers.
//!   Tab and Shift+Tab move focus as in a plain `<textarea>` (no indenting).
//!   A `suggestion` shows as italic ghost text with a "Tab to accept" hint;
//!   while it shows, Tab accepts it instead.
//! - **Footer** (40px): attach, model, effort and permissions controls on
//!   the left; the meter, the voice pill or the mic, and the primary actions
//!   on the right.
use super::actions::{ComposerPrimaryActions, ComposerState};
use super::attachments::{AttachmentFile, AttachmentImage, ComposerAttachmentStrip};
use super::chip::ContextChip;
use super::controls::{ComposerControl, EffortGlyph, EffortLevel};
use super::voice::VoiceInputPill;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::kbd::Kbd;
use crate::t3::press::{Handler, rem};
use crate::t3::shell::{IdentitySize, ProviderIcon};
use crate::{icons::CarbonIcon, theme::color, tokens::ColorRole};
use gpui_kit::component::Sizable as _;
use gpui_kit::component::input::{
    Enter, IndentInline, OutdentInline, Textarea as KitTextarea, TextareaState,
};
use gpui_kit::{
    AnyElement, App, AppContext as _, ElementId, Entity, InteractiveElement as _, IntoElement,
    MouseButton, ParentElement, RenderOnce, SharedString, Styled, Window, div,
    prelude::FluentBuilder as _,
};
use std::rc::Rc;

/// Which footer control's menu is open (`openControl`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ComposerMenu {
    /// The model picker.
    Model,
    /// Reasoning effort.
    Effort,
    /// Permissions.
    Permissions,
}

type TextHandler = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;

/// The composer's strings (the reference's English defaults).
#[derive(Clone)]
pub struct ComposerTexts {
    /// The input placeholder.
    pub placeholder: SharedString,
    /// The input's accessible name.
    pub message: SharedString,
    /// Attach files.
    pub attach: SharedString,
    /// Dictate (⌥Space).
    pub dictate: SharedString,
    /// The model control's name.
    pub model: SharedString,
    /// The effort control's name.
    pub effort: SharedString,
    /// The permissions control's name.
    pub permissions: SharedString,
    /// "to accept" after the Tab key.
    pub accept: SharedString,
}

impl Default for ComposerTexts {
    fn default() -> Self {
        Self {
            placeholder: "Ask anything, @ to mention, / for commands".into(),
            message: "Message".into(),
            attach: "Attach files".into(),
            dictate: "Dictate (⌥Space)".into(),
            model: "Model".into(),
            effort: "Reasoning effort".into(),
            permissions: "Permissions".into(),
            accept: "to accept".into(),
        }
    }
}

/// What Enter does: `Some(true)` steers, `Some(false)` sends or queues,
/// `None` lets the input take it (Shift+Enter, or steering unavailable).
pub fn enter_action(shift: bool, command: bool, running: bool, can_steer: bool) -> Option<bool> {
    if shift {
        None
    } else if running && command && can_steer {
        Some(true)
    } else {
        Some(false)
    }
}

/// What Tab does in the input: accept the showing suggestion, or move focus
/// (`forward` = Tab, otherwise Shift+Tab). The input never indents.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabAction {
    /// Put the suggestion into the input.
    Accept,
    /// Move focus to the next stop.
    FocusNext,
    /// Move focus to the previous stop.
    FocusPrev,
}

/// Tab accepts only while a suggestion is showing (`suggestion && !text`);
/// otherwise Tab and Shift+Tab move focus, as in a plain `<textarea>`.
pub fn tab_action(forward: bool, suggestion_showing: bool) -> TabAction {
    match (forward, suggestion_showing) {
        (true, true) => TabAction::Accept,
        (true, false) => TabAction::FocusNext,
        (false, _) => TabAction::FocusPrev,
    }
}

/// Move focus one stop through Base's `root::Tab` / `root::TabPrev`, so an
/// open dialog's focus trap still holds; plain GPUI traversal otherwise.
fn move_focus(forward: bool, window: &mut Window, cx: &mut App) {
    let name = if forward {
        "root::Tab"
    } else {
        "root::TabPrev"
    };
    match cx.build_action(name, None) {
        Ok(action) => window.dispatch_action(action, cx),
        Err(_) if forward => window.focus_next(cx),
        Err(_) => window.focus_prev(cx),
    }
}

/// The composer. See the module docs.
#[derive(IntoElement)]
pub struct Composer {
    id: ElementId,
    value: SharedString,
    input: Option<Entity<TextareaState>>,
    contexts: Vec<ContextChip>,
    attachments: Option<(Vec<AttachmentImage>, Vec<AttachmentFile>)>,
    model: SharedString,
    provider: SharedString,
    effort: EffortLevel,
    fast: bool,
    permissions: SharedString,
    state: ComposerState,
    voice: Option<VoiceInputPill>,
    banners: Option<AnyElement>,
    meter: Option<AnyElement>,
    open_control: Option<ComposerMenu>,
    send_disabled: bool,
    suggestion: Option<SharedString>,
    send_label: Option<SharedString>,
    texts: ComposerTexts,
    on_send: Option<TextHandler>,
    on_queue: Option<TextHandler>,
    on_steer: Option<TextHandler>,
    on_stop: Option<Handler>,
    on_attach: Option<Handler>,
    on_model_click: Option<Handler>,
    on_effort_click: Option<Handler>,
    on_permissions_click: Option<Handler>,
    on_remove_attachment: Option<TextHandler>,
}

impl Composer {
    /// A composer (model "Model", provider "Agent", medium effort,
    /// "Ask before edits", idle: the reference defaults).
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            value: SharedString::default(),
            input: None,
            contexts: Vec::new(),
            attachments: None,
            model: "Model".into(),
            provider: "Agent".into(),
            effort: EffortLevel::Medium,
            fast: false,
            permissions: "Ask before edits".into(),
            state: ComposerState::Idle,
            voice: None,
            banners: None,
            meter: None,
            open_control: None,
            send_disabled: false,
            suggestion: None,
            send_label: None,
            texts: ComposerTexts::default(),
            on_send: None,
            on_queue: None,
            on_steer: None,
            on_stop: None,
            on_attach: None,
            on_model_click: None,
            on_effort_click: None,
            on_permissions_click: None,
            on_remove_attachment: None,
        }
    }

    /// `value`: the initial text.
    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = value.into();
        self
    }

    /// The input's text state, when the caller owns it (to read or set the
    /// text, or to focus it). Without it the composer keeps its own, keyed
    /// by the composer's id and seeded from [`Self::value`].
    pub fn input(mut self, state: Entity<TextareaState>) -> Self {
        self.input = Some(state);
        self
    }

    /// `contexts`: chips above the input (drawn in their composer form).
    pub fn contexts(mut self, contexts: Vec<ContextChip>) -> Self {
        self.contexts = contexts;
        self
    }

    /// `attachments`: images and files above the input.
    pub fn attachments(mut self, images: Vec<AttachmentImage>, files: Vec<AttachmentFile>) -> Self {
        self.attachments = Some((images, files));
        self
    }

    /// `model` and `provider`.
    pub fn model(
        mut self,
        model: impl Into<SharedString>,
        provider: impl Into<SharedString>,
    ) -> Self {
        self.model = model.into();
        self.provider = provider.into();
        self
    }

    /// `effort` and `fast`.
    pub fn effort(mut self, effort: EffortLevel, fast: bool) -> Self {
        self.effort = effort;
        self.fast = fast;
        self
    }

    /// `permissions`.
    pub fn permissions(mut self, permissions: impl Into<SharedString>) -> Self {
        self.permissions = permissions.into();
        self
    }

    /// `state`.
    pub fn state(mut self, state: ComposerState) -> Self {
        self.state = state;
        self
    }

    /// `voice`: the dictation pill replaces the mic.
    pub fn voice(mut self, voice: VoiceInputPill) -> Self {
        self.voice = Some(voice);
        self
    }

    /// `banners`: above the box (usually a [`super::ComposerBannerStack`]).
    pub fn banners(mut self, banners: impl IntoElement) -> Self {
        self.banners = Some(banners.into_any_element());
        self
    }

    /// `meter`: before the voice control.
    pub fn meter(mut self, meter: impl IntoElement) -> Self {
        self.meter = Some(meter.into_any_element());
        self
    }

    /// `openControl`: highlights that footer control.
    pub fn open_control(mut self, control: Option<ComposerMenu>) -> Self {
        self.open_control = control;
        self
    }

    /// `sendDisabled`.
    pub fn send_disabled(mut self, disabled: bool) -> Self {
        self.send_disabled = disabled;
        self
    }

    /// `suggestion`: ghost text while the input is empty.
    pub fn suggestion(mut self, suggestion: impl Into<SharedString>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// `sendLabel`: answering, with this button text.
    pub fn send_label(mut self, label: impl Into<SharedString>) -> Self {
        self.send_label = Some(label.into());
        self
    }

    /// Replace the strings (`placeholder`, …).
    pub fn texts(mut self, texts: ComposerTexts) -> Self {
        self.texts = texts;
        self
    }

    /// `onSend(text)`.
    pub fn on_send(mut self, f: impl Fn(SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_send = Some(Rc::new(f));
        self
    }

    /// `onQueue(text)`.
    pub fn on_queue(mut self, f: impl Fn(SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_queue = Some(Rc::new(f));
        self
    }

    /// `onSteer(text)`: running shows Steer / Queue / Stop.
    pub fn on_steer(mut self, f: impl Fn(SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_steer = Some(Rc::new(f));
        self
    }

    /// `onStop`.
    pub fn on_stop(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_stop = Some(Rc::new(f));
        self
    }

    /// `onAttach`.
    pub fn on_attach(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_attach = Some(Rc::new(f));
        self
    }

    /// `onModelClick`.
    pub fn on_model_click(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_model_click = Some(Rc::new(f));
        self
    }

    /// `onEffortClick`.
    pub fn on_effort_click(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_effort_click = Some(Rc::new(f));
        self
    }

    /// `onPermissionsClick`.
    pub fn on_permissions_click(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_permissions_click = Some(Rc::new(f));
        self
    }

    /// The attachment strip's remove ×.
    pub fn on_remove_attachment(
        mut self,
        f: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_remove_attachment = Some(Rc::new(f));
        self
    }
}

/// Take the text and hand it to `f` when not blank, then clear the input
/// (`send` / `queue` / `steer` in `composer.jsx`).
fn submit(
    state: &Entity<TextareaState>,
    f: Option<TextHandler>,
    window: &mut Window,
    cx: &mut App,
) {
    let text = state.read(cx).value().to_string();
    if text.trim().is_empty() {
        return;
    }
    if let Some(f) = f {
        f(text.into(), window, cx);
    }
    state.update(cx, |s, cx| s.set_value("", window, cx));
}

impl RenderOnce for Composer {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let seed = self.value.clone();
        let input = match self.input.clone() {
            Some(input) => input,
            None => window
                .use_keyed_state((id.clone(), "input"), cx, move |window, cx| {
                    cx.new(|cx| TextareaState::new(window, cx).default_value(seed))
                })
                .read(cx)
                .clone(),
        };
        let text = input.read(cx).value().to_string();
        let empty = text.trim().is_empty();
        // `:focus-within`: a handle on the box that is never a tab stop itself.
        let within = window
            .use_keyed_state((id.clone(), "within"), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let focused = within.contains_focused(window, cx);
        let texts = self.texts.clone();
        let running = self.state == ComposerState::Running;
        let has_ctx = !self.contexts.is_empty() || self.attachments.is_some();
        let show_suggestion = self.suggestion.is_some() && text.is_empty();

        let send = {
            let input = input.clone();
            let f = self.on_send.clone();
            Rc::new(move |w: &mut Window, cx: &mut App| submit(&input, f.clone(), w, cx)) as Handler
        };
        let queue = {
            let input = input.clone();
            let f = self.on_queue.clone();
            Rc::new(move |w: &mut Window, cx: &mut App| submit(&input, f.clone(), w, cx)) as Handler
        };
        let steer = self.on_steer.clone().map(|f| {
            let input = input.clone();
            Rc::new(move |w: &mut Window, cx: &mut App| submit(&input, Some(f.clone()), w, cx))
                as Handler
        });

        // Kit's input binds Enter (new line) and Tab (indent) as actions,
        // which GPUI dispatches before any key-down listener, so the box
        // intercepts the actions in the capture phase instead.
        let on_enter = {
            let (send, queue, steer) = (send.clone(), queue.clone(), steer.clone());
            move |action: &Enter, window: &mut Window, cx: &mut App| {
                let Some(steering) =
                    enter_action(action.shift, action.secondary, running, steer.is_some())
                else {
                    return;
                };
                cx.stop_propagation();
                if steering {
                    if let Some(f) = steer.as_ref() {
                        f(window, cx)
                    }
                } else if running {
                    queue(window, cx)
                } else {
                    send(window, cx)
                }
            }
        };
        let on_tab = {
            let suggestion = self.suggestion.clone();
            let input = input.clone();
            move |forward: bool, window: &mut Window, cx: &mut App| {
                cx.stop_propagation();
                let showing = suggestion.is_some() && input.read(cx).value().is_empty();
                match tab_action(forward, showing) {
                    TabAction::Accept => {
                        if let Some(s) = suggestion.clone() {
                            input.update(cx, |state, cx| state.insert(s, window, cx));
                        }
                    }
                    TabAction::FocusNext => move_focus(true, window, cx),
                    TabAction::FocusPrev => move_focus(false, window, cx),
                }
            }
        };

        let ctx_row = has_ctx.then(|| {
            // `.t3-composer-ctx { display: flex; flex-wrap: wrap; align-items: center;
            // padding: spacing-05 spacing-05 0; gap: spacing-03 }`.
            div()
                .flex()
                .flex_row()
                .flex_wrap()
                .items_center()
                .gap(rem(8.))
                .pt(rem(16.))
                .px(rem(16.))
                .children(self.contexts.into_iter().map(|c| c.in_composer(true)))
                .when_some(self.attachments.clone(), |this, (images, files)| {
                    let remove = self.on_remove_attachment.clone();
                    this.child(
                        ComposerAttachmentStrip::new((id.clone(), "attachments"), images, files)
                            .in_composer(true)
                            .when_some(remove, |s, f| {
                                s.on_remove(move |name, w, cx| f(name, w, cx))
                            }),
                    )
                })
        });

        // `.t3-composer-input { min-height: 4.5rem; padding: spacing-05 spacing-05 spacing-03;
        // font-size: 0.875rem; line-height: 1.42857 }`; after the context row `padding-top: spacing-04`.
        let pad_top = if has_ctx { 12. } else { 16. };
        let editor = div()
            .relative()
            .min_h(rem(72.))
            .pt(rem(pad_top - 2.))
            .pb(rem(8. - 2.))
            .px(rem(16. - 4.))
            .text_size(rem(14.))
            .line_height(rem(20.))
            .text_color(color(ColorRole::TextPrimary))
            .when(text.is_empty() && !show_suggestion, |this| {
                // `.t3-composer-input::placeholder { color: text-helper }` (gemini-A).
                this.child(
                    div()
                        .absolute()
                        .top(rem(pad_top))
                        .left(rem(16.))
                        .right(rem(16.))
                        .text_color(color(ColorRole::TextHelper))
                        .child(texts.placeholder.clone()),
                )
            })
            .child(
                KitTextarea::new(&input)
                    .appearance(false)
                    .with_size(gpui_kit::component::Size::XSmall)
                    .aria_label(texts.message.clone())
                    // The rest of the 72px `min-height` after the padding.
                    .h(rem(72. - pad_top - 8. + 4.))
                    .w_full()
                    .text_size(rem(14.))
                    .line_height(rem(20.)),
            );
        // r2-gal: `.t3-composer-suggest { position: absolute; top: spacing-05; left: spacing-05;
        // right: spacing-05; gap: spacing-03; font-size: 0.875rem; line-height: 1.25rem }`.
        let suggestion = show_suggestion.then(|| {
            div()
                .absolute()
                .top(rem(pad_top))
                .left(rem(16.))
                .right(rem(16.))
                .flex()
                .flex_row()
                .items_center()
                .gap(rem(8.))
                .text_size(rem(14.))
                .line_height(rem(20.))
                // `.t3-composer-suggest-text { color: text-secondary; font-style: italic; ellipsis }`.
                .child(
                    div()
                        .min_w_0()
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .text_ellipsis()
                        .italic()
                        .text_color(color(ColorRole::TextSecondary))
                        .children(self.suggestion.clone()),
                )
                // `.t3-composer-suggest-key { gap: spacing-02; flex: none; font-size: 0.75rem; color: text-helper }`.
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .flex_none()
                        .items_center()
                        .gap(rem(4.))
                        .text_size(rem(12.))
                        .text_color(color(ColorRole::TextHelper))
                        .child(Kbd::new("Tab"))
                        .child(texts.accept.clone()),
                )
        });

        let control =
            |key: &'static str, menu: ComposerMenu, label: SharedString, f: Option<Handler>| {
                ComposerControl::new((id.clone(), key), label)
                    .active(self.open_control == Some(menu))
                    .when_some(f, |c, f| c.on_click(move |w, cx| f(w, cx)))
            };
        let attach = self.on_attach.clone();
        let left = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(rem(2.))
            .min_w_0()
            .overflow_hidden()
            .child(
                Button::new((id.clone(), "attach"), texts.attach.clone())
                    .icon(CarbonIcon::Add)
                    .icon_only(true)
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm)
                    .when_some(attach, |b, f| b.on_click(move |_, w, cx| f(w, cx))),
            )
            .child(
                control(
                    "model",
                    ComposerMenu::Model,
                    texts.model.clone(),
                    self.on_model_click.clone(),
                )
                .child(ProviderIcon::new(self.provider.clone()).size(IdentitySize::Xs))
                .child(self.model.clone()),
            )
            .child(
                control(
                    "effort",
                    ComposerMenu::Effort,
                    texts.effort.clone(),
                    self.on_effort_click.clone(),
                )
                .child(EffortGlyph::new(self.effort).fast(self.fast)),
            )
            .child(
                control(
                    "permissions",
                    ComposerMenu::Permissions,
                    texts.permissions.clone(),
                    self.on_permissions_click.clone(),
                )
                .icon(CarbonIcon::Locked)
                .child(self.permissions.clone()),
            );
        let primary_state = if self.send_label.is_some() {
            ComposerState::Answering
        } else {
            self.state
        };
        let mut primary = ComposerPrimaryActions::new((id.clone(), "primary"), primary_state)
            .disabled(self.send_disabled || (empty && self.state == ComposerState::Idle))
            .empty(empty)
            .on_send({
                let send = send.clone();
                move |w, cx| send(w, cx)
            })
            .on_queue({
                let queue = queue.clone();
                move |w, cx| queue(w, cx)
            });
        if let Some(label) = self.send_label.clone() {
            primary = primary.answer_label(label);
        }
        if let Some(f) = self.on_stop.clone() {
            primary = primary.on_stop(move |w, cx| f(w, cx));
        }
        if let Some(f) = steer.clone() {
            primary = primary.on_steer(move |w, cx| f(w, cx));
        }
        let right = div()
            .flex()
            .flex_row()
            .flex_none()
            .items_center()
            .gap(rem(2.))
            .children(self.meter)
            .child(match self.voice {
                Some(voice) => voice.into_any_element(),
                None => Button::new((id.clone(), "mic"), texts.dictate.clone())
                    .icon(CarbonIcon::Microphone)
                    .icon_only(true)
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm)
                    .into_any_element(),
            })
            .child(primary);
        // `.t3-composer-foot { display: flex; align-items: center; justify-content: space-between;
        // gap: spacing-03; min-height: 2.5rem; padding: 0 spacing-03 spacing-03 }`.
        let foot = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .gap(rem(8.))
            .min_h(rem(40.))
            .px(rem(8.))
            .pb(rem(8.))
            .child(left)
            .child(right);
        // `.t3-composer-box { background: field-01; border: 1px solid border-subtle-01 }`,
        // `:focus-within { border-color: focus }`.
        let frame = color(if focused {
            ColorRole::Focus
        } else {
            ColorRole::BorderSubtle01
        });
        let tab = Rc::new(on_tab);
        let content = div()
            .id((id.clone(), "box"))
            .track_focus(&within)
            .relative()
            .flex()
            .flex_col()
            .w_full()
            .bg(color(ColorRole::Field01))
            // A real 1px border (outside the 48 + 72 + 40px content), not an inset line.
            .border_1()
            .border_color(frame)
            .capture_action(on_enter)
            .capture_action({
                let tab = tab.clone();
                move |_: &IndentInline, window, cx| tab(true, window, cx)
            })
            .capture_action(move |_: &OutdentInline, window, cx| tab(false, window, cx))
            .child(
                // A press on the box's own padding takes no focus (`<div>`),
                // so `track_focus` above only observes focus within.
                div()
                    .flex()
                    .flex_col()
                    .w_full()
                    .on_mouse_down(MouseButton::Left, |_, window, _| window.prevent_default())
                    .children(ctx_row)
                    .child(div().relative().child(editor).children(suggestion))
                    .child(foot),
            );
        // `.t3-composer { display: flex; flex-direction: column; gap: spacing-03; width: 100% }`.
        div()
            .flex()
            .flex_col()
            .w_full()
            .min_w_0()
            .gap(rem(8.))
            .children(self.banners)
            .child(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tab_accepts_only_a_showing_suggestion() {
        assert_eq!(tab_action(true, true), TabAction::Accept);
        assert_eq!(tab_action(true, false), TabAction::FocusNext);
        assert_eq!(tab_action(false, true), TabAction::FocusPrev);
        assert_eq!(tab_action(false, false), TabAction::FocusPrev);
    }

    #[test]
    fn enter_sends_queues_or_steers() {
        assert_eq!(enter_action(false, false, false, false), Some(false));
        assert_eq!(enter_action(true, false, false, false), None);
        assert_eq!(enter_action(false, true, true, true), Some(true));
        // ⌘↵ without onSteer queues.
        assert_eq!(enter_action(false, true, true, false), Some(false));
    }
}
