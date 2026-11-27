//! The options variant of [`ApprovalCard`] (u0 board `PermissionCard`,
//! `.pmc*`): the buttons are the asker's options as data (ACP
//! `request_permission`: a name and a kind each), drawn in the order sent;
//! none is added, hidden, renamed, reordered or marked dangerous.
//!
//! - The first `allow_once` (else the first allow) is the one primary
//!   button and takes Return; the others are secondary. Escape takes the
//!   first `reject_once` (else the first reject). Each button's tooltip
//!   gives its kind, and Return / Esc where it applies.
//! - Head: a 16px icon (command → Terminal, file change → Edit, otherwise
//!   Locked; `tool` draws u0's own tools' `home_repair_service`), the bold
//!   title, and the provider text on the right.
//! - Body: the mono detail box, facts (label / value rows), the reason and
//!   context lines, a plan element, the options row (or the caller's
//!   `editor` in its place), extra ghost actions and an info note.
//! - Answered: one row with a check (allowed) or cancel icon, the bold
//!   spoken text and a secondary tail.
use super::{ApprovalCard, ApprovalKind};
use crate::components::button::{Button, ButtonKind, ButtonSize, ButtonVariant};
use crate::icons::{CarbonIcon, Icon};
use crate::t3::action::{ActionButton, ActionKind, Part};
use crate::t3::press::{self, inset_frame, rem};
use crate::{
    theme::color,
    tokens::{ColorRole, FontFamilyKey},
    typography::font,
};
use gpui_kit::{
    AnyElement, App, FontWeight, InteractiveElement as _, IntoElement, KeyDownEvent, ParentElement,
    SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _,
};
use std::rc::Rc;

/// An option's kind (ACP `PermissionOptionKind`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PermissionOptionKind {
    /// `allow_once`.
    AllowOnce,
    /// `allow_always`.
    AllowAlways,
    /// `reject_once`.
    RejectOnce,
    /// `reject_always`.
    RejectAlways,
}

impl PermissionOptionKind {
    /// Every kind.
    pub const ALL: [Self; 4] = [
        Self::AllowOnce,
        Self::AllowAlways,
        Self::RejectOnce,
        Self::RejectAlways,
    ];

    /// The protocol's name, e.g. `"allow_once"`.
    pub const fn name(self) -> &'static str {
        match self {
            Self::AllowOnce => "allow_once",
            Self::AllowAlways => "allow_always",
            Self::RejectOnce => "reject_once",
            Self::RejectAlways => "reject_always",
        }
    }

    /// The kind named `name` (the protocol's name).
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.name() == name)
    }

    /// Whether choosing it lets the asker go on.
    pub const fn allows(self) -> bool {
        matches!(self, Self::AllowOnce | Self::AllowAlways)
    }
}

/// One option the asker sent: its label, as sent, and its kind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApprovalOption {
    /// The button's text, as the asker sent it.
    pub label: SharedString,
    /// Its kind.
    pub kind: PermissionOptionKind,
}

impl ApprovalOption {
    /// An option named `label` of `kind`.
    pub fn new(label: impl Into<SharedString>, kind: PermissionOptionKind) -> Self {
        Self {
            label: label.into(),
            kind,
        }
    }
}

/// The option Return takes and the one primary button: the first
/// `allow_once`, else the first allow (so the default never grants more
/// than this once).
pub fn default_option(options: &[ApprovalOption]) -> Option<usize> {
    let at = |f: fn(PermissionOptionKind) -> bool| options.iter().position(|o| f(o.kind));
    at(|k| k == PermissionOptionKind::AllowOnce).or_else(|| at(PermissionOptionKind::allows))
}

/// The option Escape takes: the first `reject_once`, else the first reject.
pub fn escape_option(options: &[ApprovalOption]) -> Option<usize> {
    let at = |f: fn(PermissionOptionKind) -> bool| options.iter().position(|o| f(o.kind));
    at(|k| k == PermissionOptionKind::RejectOnce).or_else(|| at(|k| !k.allows()))
}

/// The tooltip of option `index`: its kind, then the key that takes it
/// (`tip: o.kind + (o === def ? ' · Return' : '')`).
pub fn option_tooltip(
    options: &[ApprovalOption],
    index: usize,
    return_key: &str,
    escape_key: &str,
) -> SharedString {
    let kind = options[index].kind.name();
    if default_option(options) == Some(index) {
        format!("{kind} · {return_key}").into()
    } else if escape_option(options) == Some(index) {
        format!("{kind} · {escape_key}").into()
    } else {
        kind.into()
    }
}

/// The head icon (`headIcon`: command → Terminal, edit → Edit, else Locked;
/// u0's own tools draw `home_repair_service`).
pub fn approval_head_icon(kind: ApprovalKind, tool: bool) -> Icon {
    if tool {
        return Icon::named("home_repair_service");
    }
    Icon::carbon(match kind {
        ApprovalKind::Command => CarbonIcon::Terminal,
        ApprovalKind::File => CarbonIcon::Edit,
        _ => CarbonIcon::Locked,
    })
}

/// An extra ghost action under the options ("No, and tell it…", "Revise…").
pub(super) struct Extra {
    pub(super) label: SharedString,
    pub(super) icon: Icon,
    pub(super) on_click: press::Handler,
}

/// The answered row's words.
pub(super) struct Answer {
    pub(super) allowed: bool,
    pub(super) spoken: SharedString,
    pub(super) tail: SharedString,
}

pub(super) type PickHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// A 16px icon in `role`.
fn glyph(icon: Icon, role: ColorRole) -> AnyElement {
    div()
        .flex_none()
        .child(icon.size_length(rem(16.)).color(color(role)))
        .into_any_element()
}

/// `.pmc-note { display: flex; align-items: flex-start; gap: 8px; font-size: 12px;
/// line-height: 16px; color: text-secondary }`, `.pmc-note svg { fill: icon-secondary }`.
fn info_note(text: SharedString) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .items_start()
        .gap(rem(8.))
        .text_size(rem(12.))
        .line_height(rem(16.))
        .text_color(color(ColorRole::TextSecondary))
        .child(glyph(
            Icon::carbon(CarbonIcon::Information),
            ColorRole::IconSecondary,
        ))
        .child(div().flex_1().min_w_0().child(text))
        .into_any_element()
}

impl ApprovalCard {
    /// The options variant. See the module docs.
    pub(super) fn render_options(self, window: &mut Window, cx: &mut App) -> AnyElement {
        let id = self.id.clone();
        let options = self.options.clone().unwrap_or_default();
        let pending = self.answer.is_none();
        let (focus, ring) = press::focus((id.clone(), "focus"), window, cx);
        let texts = self.texts.clone();
        // While the caller's editor is open, its keys are its own.
        let has_editor = self.editor.is_some();
        let primary = default_option(&options);
        let escape = escape_option(&options);
        let title = self
            .title
            .clone()
            .unwrap_or_else(|| self.kind.default_title().into());

        // `.pmc-head { display: flex; flex-wrap: wrap; align-items: center; gap: 4px 8px;
        // min-height: 20px; font-size: 14px; line-height: 20px }`.
        let head = div()
            .flex()
            .flex_row()
            .flex_wrap()
            .items_center()
            .gap_x(rem(8.))
            .gap_y(rem(4.))
            .min_h(rem(20.))
            .text_size(rem(14.))
            .line_height(rem(20.))
            // `.pmc-head svg { fill: icon-primary }`.
            .child(glyph(
                approval_head_icon(self.kind, self.tool),
                ColorRole::IconPrimary,
            ))
            // `.pmc-head b { flex: 1 1 220px; min-width: 0; font-weight: 600 }`.
            .child(
                div()
                    .flex_1()
                    .min_w(rem(220.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(title),
            )
            // `.pmc-who { margin-left: auto; font-size: 12px; line-height: 16px;
            // color: text-secondary }`.
            .when_some(self.provider.clone(), |this, who| {
                this.child(
                    div()
                        .ml_auto()
                        .min_w_0()
                        .text_size(rem(12.))
                        .line_height(rem(16.))
                        .text_color(color(ColorRole::TextSecondary))
                        .child(who),
                )
            });

        let mut body: Vec<AnyElement> = Vec::new();
        if !self.detail.is_empty() {
            // `.pmc-detail { padding: 8px 12px; background: field-01; font-family: mono;
            // font-size: 14px; line-height: 20px; white-space: pre-wrap }`.
            body.push(
                div()
                    .px(rem(12.))
                    .py(rem(8.))
                    .bg(color(ColorRole::Field01))
                    .font(font(FontFamilyKey::Mono, 400))
                    .text_size(rem(14.))
                    .line_height(rem(20.))
                    .child(self.detail.clone())
                    .into_any_element(),
            );
        }
        if !self.facts.is_empty() {
            // `.pmc-facts { display: grid; grid-template-columns: max-content minmax(0,1fr);
            // gap: 4px 16px; font-size: 14px; line-height: 20px }`, the label column 116px
            // wide; `dt { color: text-secondary }`.
            body.push(
                div()
                    .flex()
                    .flex_col()
                    .gap(rem(4.))
                    .text_size(rem(14.))
                    .line_height(rem(20.))
                    .children(self.facts.iter().map(|(label, value)| {
                        div()
                            .flex()
                            .flex_row()
                            .gap(rem(16.))
                            .child(
                                div()
                                    .flex_none()
                                    .w(rem(116.))
                                    .text_color(color(ColorRole::TextSecondary))
                                    .child(label.clone()),
                            )
                            .child(div().flex_1().min_w_0().child(value.clone()))
                    }))
                    .into_any_element(),
            );
        }
        if let Some(reason) = self.reason.clone() {
            // `.pmc-reason { font-size: 14px; line-height: 20px; color: text-secondary }`.
            body.push(
                div()
                    .text_size(rem(14.))
                    .line_height(rem(20.))
                    .text_color(color(ColorRole::TextSecondary))
                    .child(reason)
                    .into_any_element(),
            );
        }
        if let Some(context) = self.context.clone() {
            // `.pmc-ctx { font-size: 12px; line-height: 16px; color: text-secondary }`.
            body.push(
                div()
                    .text_size(rem(12.))
                    .line_height(rem(16.))
                    .text_color(color(ColorRole::TextSecondary))
                    .child(context)
                    .into_any_element(),
            );
        }
        if let Some(plan) = self.plan {
            body.push(
                div()
                    .id((id.clone(), "plan"))
                    .max_h(rem(320.))
                    .overflow_y_scroll()
                    .child(plan)
                    .into_any_element(),
            );
        }

        if let Some(answer) = self.answer.as_ref() {
            // `.pmc-done { display: flex; align-items: center; gap: 8px; min-height: 32px;
            // font-size: 14px; line-height: 20px; color: text-secondary }`,
            // `svg { fill: icon-secondary }`, `b { font-weight: 600; color: text-primary }`.
            let icon = if answer.allowed {
                Icon::carbon(CarbonIcon::CheckmarkOutline)
            } else {
                Icon::named("cancel")
            };
            body.push(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .items_center()
                    .gap(rem(8.))
                    .min_h(rem(32.))
                    .text_size(rem(14.))
                    .line_height(rem(20.))
                    .text_color(color(ColorRole::TextSecondary))
                    .child(glyph(icon, ColorRole::IconSecondary))
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(color(ColorRole::TextPrimary))
                            .child(answer.spoken.clone()),
                    )
                    .when(!answer.tail.is_empty(), |this| {
                        this.child(div().child(SharedString::from(format!("· {}", answer.tail))))
                    })
                    .into_any_element(),
            );
        } else if let Some(editor) = self.editor {
            body.push(editor);
        } else {
            // `.pmc-opts { display: flex; flex-wrap: wrap; align-items: flex-start; gap: 8px }`,
            // `.pmc-opt { display: flex; flex-direction: column; gap: 4px }`.
            let on_pick = self.on_pick.clone();
            body.push(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .items_start()
                    .gap(rem(8.))
                    .children(options.iter().enumerate().map(|(i, option)| {
                        let kind = if primary == Some(i) {
                            ButtonKind::Primary
                        } else {
                            ButtonKind::Secondary
                        };
                        let on_click = on_pick.clone().map(|f| {
                            Rc::new(move |window: &mut Window, cx: &mut App| f(i, window, cx))
                                as press::Handler
                        });
                        div()
                            .flex()
                            .flex_col()
                            .gap(rem(4.))
                            .child(
                                ActionButton::new(
                                    (id.clone(), format!("option-{i}")),
                                    option.label.clone(),
                                    ActionKind::Carbon(kind),
                                )
                                .part(Part::Text(option.label.clone()))
                                .tooltip(option_tooltip(
                                    &options,
                                    i,
                                    &texts.return_key,
                                    &texts.escape_key,
                                ))
                                .on_click(on_click),
                            )
                            // `.pmc-kind { font-family: mono; font-size: 12px; line-height: 16px;
                            // color: text-secondary }`.
                            .when(self.show_kinds, |this| {
                                this.child(
                                    div()
                                        .font(font(FontFamilyKey::Mono, 400))
                                        .text_size(rem(12.))
                                        .line_height(rem(16.))
                                        .text_color(color(ColorRole::TextSecondary))
                                        .child(option.kind.name()),
                                )
                            })
                    }))
                    .into_any_element(),
            );
            if !self.extras.is_empty() {
                // `.pmc-extra { display: flex; flex-wrap: wrap; gap: 8px }`: ghost sm buttons.
                body.push(
                    div()
                        .flex()
                        .flex_row()
                        .flex_wrap()
                        .gap(rem(8.))
                        .children(self.extras.into_iter().enumerate().map(|(i, extra)| {
                            let f = extra.on_click;
                            Button::new((id.clone(), format!("extra-{i}")), extra.label)
                                .variant(ButtonVariant::Ghost)
                                .size(ButtonSize::Sm)
                                .icon(extra.icon)
                                .on_click(move |_, window, cx| f(window, cx))
                        }))
                        .into_any_element(),
                );
            }
        }
        if pending && let Some(note) = self.note.clone() {
            body.push(info_note(note));
        }

        let on_pick = self.on_pick.clone();
        // `.pmc { gap: 12px; padding: 16px; background: layer-01; box-shadow: inset 0 0 0 1px
        // border-subtle-01 }`, `.pmc--pending { box-shadow: inset 0 0 0 1px border-strong-01 }`.
        super::card::frame()
            .id(id.clone())
            .shadow(inset_frame(color(if pending {
                ColorRole::BorderStrong01
            } else {
                ColorRole::BorderSubtle01
            })))
            .when(pending, |this| {
                let focus_handle = focus.clone();
                this.track_focus(&focus.tab_index(0).tab_stop(true))
                    .on_key_down(move |e: &KeyDownEvent, window, cx| {
                        if e.keystroke.modifiers.modified() || has_editor {
                            return;
                        }
                        let index = match e.keystroke.key.as_str() {
                            // Return on the card itself; a focused button takes its own.
                            "enter" if focus_handle.is_focused(window) => primary,
                            "escape" => escape,
                            _ => None,
                        };
                        if let (Some(i), Some(f)) = (index, on_pick.as_ref()) {
                            cx.stop_propagation();
                            f(i, window, cx);
                        }
                    })
            })
            .child(head)
            .children(body)
            // `.pmc :focus-visible { outline: 2px solid focus; outline-offset: -2px }`.
            .when(ring && pending, |this| this.child(press::ring()))
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use PermissionOptionKind::*;

    fn opts(kinds: &[PermissionOptionKind]) -> Vec<ApprovalOption> {
        kinds
            .iter()
            .map(|k| ApprovalOption::new(k.name(), *k))
            .collect()
    }

    #[test]
    fn return_takes_the_first_allow_once_else_the_first_allow() {
        // Claude's four.
        let four = opts(&[AllowOnce, AllowAlways, RejectOnce, RejectAlways]);
        assert_eq!(default_option(&four), Some(0));
        // allow_once after allow_always still wins.
        assert_eq!(default_option(&opts(&[AllowAlways, AllowOnce])), Some(1));
        // No allow_once: the first allow.
        assert_eq!(default_option(&opts(&[RejectOnce, AllowAlways])), Some(1));
        // Nothing allows: no default.
        assert_eq!(default_option(&opts(&[RejectOnce, RejectAlways])), None);
    }

    #[test]
    fn escape_takes_the_first_reject_once_else_the_first_reject() {
        let four = opts(&[AllowOnce, AllowAlways, RejectAlways, RejectOnce]);
        assert_eq!(escape_option(&four), Some(3));
        assert_eq!(escape_option(&opts(&[AllowOnce, RejectAlways])), Some(1));
        assert_eq!(escape_option(&opts(&[AllowOnce, AllowAlways])), None);
    }

    #[test]
    fn tooltips_name_the_kind_and_its_key() {
        // Codex's three.
        let three = opts(&[AllowOnce, AllowAlways, RejectOnce]);
        assert_eq!(
            option_tooltip(&three, 0, "Return", "Esc"),
            "allow_once · Return"
        );
        assert_eq!(option_tooltip(&three, 1, "Return", "Esc"), "allow_always");
        assert_eq!(
            option_tooltip(&three, 2, "Return", "Esc"),
            "reject_once · Esc"
        );
        assert_eq!(
            PermissionOptionKind::from_name("reject_always"),
            Some(RejectAlways)
        );
        assert_eq!(PermissionOptionKind::from_name("other"), None);
    }
}
