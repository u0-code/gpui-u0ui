//! [`ApprovalCard`] (`.t3-approval`): the agent asks to run a command,
//! change or read a file, or use an app permission.
//!
//! - **Pending**: a 3px `support-warning` rail, the kind's label over the
//!   mono detail, optional context / reason / scope notes, then Always allow
//!   (ghost, on the card's text edge), Decline (a neutral outline) and
//!   Approve (the one filled button; `danger` when destructive).
//! - **Resolved**: one row (craft pass): approved = `support-success` rail,
//!   check and a green tag; declined = the user's choice, so a neutral
//!   `border-strong-01` rail, `icon-secondary` icon and a gray tag (app-r3).
//! - **Keyboard**: a pending card is a tab stop with a 2px `focus` ring;
//!   with `shortcuts` (and not destructive) ⇧⌘↵ approves and ⇧⌘⌫ declines
//!   while focus is inside the card, and the buttons show the keys.
use super::card;
use crate::components::badge::{Badge, BadgeSize, BadgeVariant};
use crate::components::button::{Button, ButtonKind, ButtonSize, ButtonVariant};
use crate::t3::action::{ActionButton, ActionKind, KeyCap, Part};
use crate::t3::press::{self, Handler, rem};
use crate::{
    icons::CarbonIcon,
    theme::color,
    tokens::{ColorRole, FontFamilyKey},
    typography::font,
};
use gpui_kit::{
    App, ElementId, Hsla, InteractiveElement as _, IntoElement, KeyDownEvent, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _,
};
use std::rc::Rc;

/// `ApprovalCard` `kind`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ApprovalKind {
    /// Run a command.
    #[default]
    Command,
    /// Change a file.
    File,
    /// Read a file.
    Read,
    /// An app permission.
    Permission,
    /// App access.
    Access,
}

impl ApprovalKind {
    /// Every kind.
    pub const ALL: [ApprovalKind; 5] = [
        Self::Command,
        Self::File,
        Self::Read,
        Self::Permission,
        Self::Access,
    ];

    /// The reference name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Command => "command",
            Self::File => "file",
            Self::Read => "read",
            Self::Permission => "permission",
            Self::Access => "access",
        }
    }

    /// The default head (`title` fallback).
    pub const fn default_title(self) -> &'static str {
        match self {
            Self::Command => "Command approval",
            Self::File => "File change approval",
            Self::Read => "File read approval",
            Self::Permission => "App permission approval",
            Self::Access => "App access approval",
        }
    }

    /// The default label over the detail.
    pub const fn default_label(self) -> &'static str {
        match self {
            Self::Command => "Command",
            Self::File => "File change",
            Self::Read => "File to read",
            Self::Permission => "Permission request",
            Self::Access => "App access request",
        }
    }
}

/// `ApprovalCard` `state`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ApprovalState {
    /// Waiting for the user.
    #[default]
    Pending,
    /// Approved.
    Approved,
    /// Declined.
    Declined,
}

impl ApprovalState {
    /// Every state.
    pub const ALL: [ApprovalState; 3] = [Self::Pending, Self::Approved, Self::Declined];

    /// The reference name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Declined => "declined",
        }
    }
}

/// The card's colors: (rail, head icon, head icon glyph).
pub fn approval_look(state: ApprovalState, destructive: bool) -> (Hsla, Hsla, CarbonIcon) {
    match state {
        // `.t3-approval--approved { box-shadow: inset 3px 0 0 support-success }`, `.t3-okicon`.
        ApprovalState::Approved => (
            color(ColorRole::SupportSuccess),
            color(ColorRole::SupportSuccess),
            CarbonIcon::CheckmarkFilled,
        ),
        // app-r3: `.t3-approval--declined { box-shadow: inset 3px 0 0 border-strong-01 }`,
        // `.t3-approval--declined .t3-card-head > svg.t3-err { fill: icon-secondary }`.
        ApprovalState::Declined => (
            color(ColorRole::BorderStrong01),
            color(ColorRole::IconSecondary),
            CarbonIcon::ErrorFilled,
        ),
        // `.t3-approval { box-shadow: inset 3px 0 0 support-warning }`; destructive: `.t3-err`.
        ApprovalState::Pending if destructive => (
            color(ColorRole::SupportWarning),
            color(ColorRole::SupportError),
            CarbonIcon::ErrorFilled,
        ),
        ApprovalState::Pending => (
            color(ColorRole::SupportWarning),
            color(ColorRole::SupportWarning),
            CarbonIcon::WarningAltFilled,
        ),
    }
}

/// The card's strings (the reference's English defaults).
#[derive(Clone)]
pub struct ApprovalTexts {
    /// The approved tag.
    pub approved: SharedString,
    /// The declined tag.
    pub declined: SharedString,
    /// Decline.
    pub decline: SharedString,
    /// Approve (`approveLabel` overrides it).
    pub approve: SharedString,
    /// Always allow (`alwaysLabel` overrides it).
    pub always: SharedString,
}

impl Default for ApprovalTexts {
    fn default() -> Self {
        Self {
            approved: "Approved".into(),
            declined: "Declined".into(),
            decline: "Decline".into(),
            approve: "Approve".into(),
            always: "Always allow this session".into(),
        }
    }
}

/// ⇧⌘↵ (approve) and ⇧⌘⌫ (decline): `shift` plus `cmd` or `ctrl`.
pub fn approval_shortcut(key: &str, shift: bool, command: bool) -> Option<bool> {
    if !shift || !command {
        return None;
    }
    match key {
        "enter" => Some(true),
        "backspace" => Some(false),
        _ => None,
    }
}

/// An approval request. See the module docs.
#[derive(IntoElement)]
pub struct ApprovalCard {
    id: ElementId,
    kind: ApprovalKind,
    title: Option<SharedString>,
    detail: SharedString,
    reason: Option<SharedString>,
    provider: Option<SharedString>,
    context: Option<SharedString>,
    always_hint: Option<SharedString>,
    always_label: Option<Option<SharedString>>,
    approve_label: Option<SharedString>,
    state: ApprovalState,
    shortcuts: bool,
    destructive: bool,
    texts: ApprovalTexts,
    on_approve: Option<Handler>,
    on_decline: Option<Handler>,
    on_always: Option<Handler>,
}

impl ApprovalCard {
    /// A `kind` request showing `detail` (mono).
    pub fn new(
        id: impl Into<ElementId>,
        kind: ApprovalKind,
        detail: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            kind,
            title: None,
            detail: detail.into(),
            reason: None,
            provider: None,
            context: None,
            always_hint: None,
            always_label: None,
            approve_label: None,
            state: ApprovalState::Pending,
            shortcuts: false,
            destructive: false,
            texts: ApprovalTexts::default(),
            on_approve: None,
            on_decline: None,
            on_always: None,
        }
    }

    /// `title` (default per kind).
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// `reason`: why the agent asks.
    pub fn reason(mut self, reason: impl Into<SharedString>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// `provider`: who asks, after the head.
    pub fn provider(mut self, provider: impl Into<SharedString>) -> Self {
        self.provider = Some(provider.into());
        self
    }

    /// `context`: where it runs (12px note).
    pub fn context(mut self, context: impl Into<SharedString>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// `alwaysHint`: what "Always allow" covers (12px helper note).
    pub fn always_hint(mut self, hint: impl Into<SharedString>) -> Self {
        self.always_hint = Some(hint.into());
        self
    }

    /// `alwaysLabel`: `None` hides the button (`alwaysLabel={false}`).
    pub fn always_label(mut self, label: Option<SharedString>) -> Self {
        self.always_label = Some(label);
        self
    }

    /// `approveLabel`.
    pub fn approve_label(mut self, label: impl Into<SharedString>) -> Self {
        self.approve_label = Some(label.into());
        self
    }

    /// `state`.
    pub fn state(mut self, state: ApprovalState) -> Self {
        self.state = state;
        self
    }

    /// `shortcuts`: ⇧⌘↵ / ⇧⌘⌫ while the card has focus.
    pub fn shortcuts(mut self, shortcuts: bool) -> Self {
        self.shortcuts = shortcuts;
        self
    }

    /// `destructive`: red icon, danger Approve, no shortcuts.
    pub fn destructive(mut self, destructive: bool) -> Self {
        self.destructive = destructive;
        self
    }

    /// Replace the strings.
    pub fn texts(mut self, texts: ApprovalTexts) -> Self {
        self.texts = texts;
        self
    }

    /// `onApprove`.
    pub fn on_approve(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_approve = Some(Rc::new(f));
        self
    }

    /// `onDecline`.
    pub fn on_decline(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_decline = Some(Rc::new(f));
        self
    }

    /// `onAlways`.
    pub fn on_always(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_always = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for ApprovalCard {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let pending = self.state == ApprovalState::Pending;
        let shortcuts = self.shortcuts && !self.destructive && pending;
        let (rail, icon_ink, icon) = approval_look(self.state, self.destructive);
        let (focus, ring) = press::focus((id.clone(), "focus"), window, cx);
        let head_text = self
            .title
            .clone()
            .unwrap_or_else(|| self.kind.default_title().into());
        let texts = self.texts.clone();

        let head = card::head()
            .child(if icon == CarbonIcon::WarningAltFilled {
                // `.t3-warnicon`: the "!" is #161616 in every theme.
                crate::t3::base::warning_glyph(16.).into_any_element()
            } else {
                card::head_icon(icon, icon_ink)
            })
            .child(card::head_title(head_text.clone()))
            .when_some(self.provider.clone(), |this, p| this.child(card::sub(p)))
            .child(card::grow())
            // r3-desk hides the pending "Waiting for approval" sub; resolved cards show a tag.
            .when(!pending, |this| {
                let approved = self.state == ApprovalState::Approved;
                this.child(
                    Badge::new(if approved {
                        texts.approved.clone()
                    } else {
                        texts.declined.clone()
                    })
                    .size(BadgeSize::Sm)
                    // app-r3: a declined tag is gray, not red.
                    .variant(if approved {
                        BadgeVariant::Green
                    } else {
                        BadgeVariant::Gray
                    }),
                )
            });

        let mut body = Vec::new();
        if pending {
            body.push(card::label(self.kind.default_label()).into_any_element());
            // `.t3-approval-text { padding: spacing-04; max-height: 10rem; overflow: auto;
            // background: layer-02; font-family: mono; font-size: 0.75rem; line-height: 1.33333;
            // white-space: pre-wrap }`.
            body.push(
                div()
                    .id((id.clone(), "detail"))
                    .p(rem(12.))
                    .max_h(rem(160.))
                    .overflow_y_scroll()
                    .bg(color(ColorRole::Layer02))
                    .font(font(FontFamilyKey::Mono, 400))
                    .text_size(rem(12.))
                    .line_height(rem(16.))
                    .child(self.detail.clone())
                    .into_any_element(),
            );
            if let Some(context) = self.context.clone() {
                // `.t3-approval-ctx { font-size: 0.75rem; line-height: 1rem; color: text-secondary }`.
                body.push(
                    card::note(context)
                        .text_size(rem(12.))
                        .line_height(rem(16.))
                        .into_any_element(),
                );
            }
            if let Some(reason) = self.reason.clone() {
                body.push(card::note(reason).into_any_element());
            }
            if let Some(hint) = self.always_hint.clone() {
                // `.t3-approval-scope { font-size: 0.75rem; line-height: 1rem; color: text-helper }`.
                body.push(
                    card::note(hint)
                        .text_size(rem(12.))
                        .line_height(rem(16.))
                        .text_color(color(ColorRole::TextHelper))
                        .into_any_element(),
                );
            }
            let always = match self.always_label.clone() {
                Some(None) => None,
                Some(Some(label)) => Some(label),
                None => Some(texts.always.clone()),
            };
            let on_always = self.on_always.clone();
            let decline = ActionButton::new(
                (id.clone(), "decline"),
                texts.decline.clone(),
                ActionKind::NeutralOutline,
            )
            .part(Part::Text(texts.decline.clone()))
            .when(shortcuts, |b| {
                b.part(Part::Key("⇧⌘⌫".into(), KeyCap::Framed))
            })
            .when(shortcuts, |b| b.tooltip(format!("{} (⇧⌘⌫)", texts.decline)))
            .on_click(self.on_decline.clone());
            let approve_label = self
                .approve_label
                .clone()
                .unwrap_or_else(|| texts.approve.clone());
            let approve = ActionButton::new(
                (id.clone(), "approve"),
                approve_label.clone(),
                ActionKind::Carbon(if self.destructive {
                    ButtonKind::Danger
                } else {
                    ButtonKind::Primary
                }),
            )
            .part(Part::Text(approve_label.clone()))
            .when(shortcuts, |b| {
                b.part(Part::Key("⇧⌘↵".into(), KeyCap::Framed))
            })
            .when(shortcuts, |b| b.tooltip(format!("{approve_label} (⇧⌘↵)")))
            .on_click(self.on_approve.clone());
            body.push(
                card::actions()
                    .when_some(always, |this, label| {
                        // r3-desk: the ghost Always allow sits on the card's text edge.
                        this.child(
                            div().ml(rem(-16.)).child(
                                Button::new((id.clone(), "always"), label)
                                    .variant(ButtonVariant::Ghost)
                                    .size(ButtonSize::Sm)
                                    .when_some(on_always, |b, f| {
                                        b.on_click(move |_, w, cx| f(w, cx))
                                    }),
                            ),
                        )
                    })
                    .child(card::grow())
                    .child(decline)
                    .child(approve)
                    .into_any_element(),
            );
        }

        let on_approve = self.on_approve.clone();
        let on_decline = self.on_decline.clone();
        card::framed(Some(rail))
            .id(id.clone())
            // A resolved approval collapses to one row: `gap: 0; padding-block: spacing-04`.
            .when(!pending, |this| this.gap_0().py(rem(12.)))
            .when(pending, |this| {
                this.track_focus(&focus.tab_index(0).tab_stop(true))
                    .on_key_down(move |e: &KeyDownEvent, window, cx| {
                        if !shortcuts {
                            return;
                        }
                        let m = &e.keystroke.modifiers;
                        match approval_shortcut(&e.keystroke.key, m.shift, m.platform || m.control)
                        {
                            Some(true) => {
                                cx.stop_propagation();
                                if let Some(f) = on_approve.as_ref() {
                                    f(window, cx)
                                }
                            }
                            Some(false) => {
                                cx.stop_propagation();
                                if let Some(f) = on_decline.as_ref() {
                                    f(window, cx)
                                }
                            }
                            None => {}
                        }
                    })
            })
            .child(head)
            .children(body)
            // `.t3-approval:focus-visible { outline: 2px solid focus; outline-offset: -2px }`.
            .when(ring && pending, |this| this.child(press::ring()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    #[test]
    fn rails_follow_the_state() {
        crate::theme::tests_set(CarbonTheme::White);
        let (rail, icon, glyph) = approval_look(ApprovalState::Pending, false);
        assert_eq!(rail, color(ColorRole::SupportWarning));
        assert_eq!(icon, color(ColorRole::SupportWarning));
        assert_eq!(glyph, CarbonIcon::WarningAltFilled);
        let (rail, icon, _) = approval_look(ApprovalState::Declined, false);
        // app-r3: declined is neutral.
        assert_eq!(rail, color(ColorRole::BorderStrong01));
        assert_eq!(icon, color(ColorRole::IconSecondary));
        let (_, icon, glyph) = approval_look(ApprovalState::Pending, true);
        assert_eq!(icon, color(ColorRole::SupportError));
        assert_eq!(glyph, CarbonIcon::ErrorFilled);
    }

    #[test]
    fn shortcuts_need_shift_and_command() {
        assert_eq!(approval_shortcut("enter", true, true), Some(true));
        assert_eq!(approval_shortcut("backspace", true, true), Some(false));
        assert_eq!(approval_shortcut("enter", false, true), None);
        assert_eq!(approval_shortcut("enter", true, false), None);
        assert_eq!(approval_shortcut("a", true, true), None);
        assert_eq!(ApprovalKind::Read.default_label(), "File to read");
    }
}
