//! In-thread separators and banners: [`TimelineDivider`] (`.t3-divider`) and
//! [`ThreadErrorBanner`] (`.t3-tbanner`, "Thread banners" in `t3.css`).
use super::card;
use crate::t3::press::{self, rem};
use crate::{icons::CarbonIcon, theme::color, tokens::ColorRole};
use gpui_kit::{
    AnyElement, App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, Window, div, prelude::FluentBuilder as _, px,
};

/// `TimelineDivider` `kind`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DividerKind {
    /// Switched model.
    #[default]
    Model,
    /// New session.
    Session,
    /// Hand-off between agents.
    Handoff,
    /// "New since your last visit" (red).
    Now,
    /// Context compacted.
    Compact,
    /// Worktree event.
    Worktree,
    /// Unread.
    Unread,
}

impl DividerKind {
    /// Every kind.
    pub const ALL: [DividerKind; 7] = [
        Self::Model,
        Self::Session,
        Self::Handoff,
        Self::Now,
        Self::Compact,
        Self::Worktree,
        Self::Unread,
    ];

    /// The reference name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Model => "model",
            Self::Session => "session",
            Self::Handoff => "handoff",
            Self::Now => "now",
            Self::Compact => "compact",
            Self::Worktree => "worktree",
            Self::Unread => "unread",
        }
    }

    /// The kind for a reference name; unknown names fall back to `model`.
    pub fn from_name(name: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|k| k.name() == name)
            .unwrap_or_default()
    }

    /// The 14px glyph.
    pub const fn icon(self) -> CarbonIcon {
        match self {
            Self::Model => CarbonIcon::Model,
            Self::Session => CarbonIcon::Renew,
            Self::Handoff => CarbonIcon::ArrowRight,
            Self::Now | Self::Unread => CarbonIcon::Time,
            Self::Compact => CarbonIcon::Subtract,
            Self::Worktree => CarbonIcon::Branch,
        }
    }

    /// (ink, rule): `.t3-divider { color: text-secondary }` with
    /// `border-subtle-01` rules; `.t3-divider--now` is `support-error`.
    pub fn colors(self) -> (Hsla, Hsla) {
        if self == Self::Now {
            let red = color(ColorRole::SupportError);
            (red, red)
        } else {
            (
                color(ColorRole::TextSecondary),
                color(ColorRole::BorderSubtle01),
            )
        }
    }
}

/// A system separator in the timeline: rule, icon, text, optional time, rule.
#[derive(IntoElement)]
pub struct TimelineDivider {
    kind: DividerKind,
    text: SharedString,
    time: Option<SharedString>,
}

impl TimelineDivider {
    /// A `kind` divider saying `text`.
    pub fn new(kind: DividerKind, text: impl Into<SharedString>) -> Self {
        Self {
            kind,
            text: text.into(),
            time: None,
        }
    }

    /// `time`, in `text-helper` after the text.
    pub fn time(mut self, time: impl Into<SharedString>) -> Self {
        self.time = Some(time.into());
        self
    }
}

impl RenderOnce for TimelineDivider {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let (ink, rule) = self.kind.colors();
        let line = || div().flex_1().h(px(1.)).bg(rule);
        // `.t3-divider { display: flex; align-items: center; gap: spacing-03; font-size: 0.75rem;
        // color: text-secondary }`, `::before, ::after { flex: 1; height: 1px }`.
        div()
            .flex()
            .flex_row()
            .items_center()
            .w_full()
            .gap(rem(8.))
            .text_size(rem(12.))
            .line_height(rem(16.))
            .text_color(ink)
            .child(line())
            .child(press::icon(self.kind.icon(), 14., ink))
            .child(div().flex_none().child(self.text))
            // `.t3-divider-time { color: text-helper }`.
            .when_some(self.time, |this, t| {
                this.child(
                    div()
                        .flex_none()
                        .text_color(color(ColorRole::TextHelper))
                        .child(t),
                )
            })
            .child(line())
    }
}

/// `ThreadErrorBanner` `kind`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ThreadBannerKind {
    /// `notification-background-error`, `support-error` rail and icon.
    #[default]
    Error,
    /// `notification-background-warning`, `support-warning` rail and icon.
    Warning,
}

impl ThreadBannerKind {
    /// Both kinds.
    pub const ALL: [ThreadBannerKind; 2] = [Self::Error, Self::Warning];

    /// The reference name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
        }
    }

    /// (fill, rail / icon, glyph).
    pub fn look(self) -> (Hsla, Hsla, CarbonIcon) {
        match self {
            Self::Error => (
                color(ColorRole::NotificationBackgroundError),
                color(ColorRole::SupportError),
                CarbonIcon::ErrorFilled,
            ),
            Self::Warning => (
                color(ColorRole::NotificationBackgroundWarning),
                color(ColorRole::SupportWarning),
                CarbonIcon::WarningAltFilled,
            ),
        }
    }
}

/// An error or warning in the thread with recovery actions.
#[derive(IntoElement)]
pub struct ThreadErrorBanner {
    kind: ThreadBannerKind,
    title: SharedString,
    body: Option<SharedString>,
    actions: Vec<AnyElement>,
}

impl ThreadErrorBanner {
    /// A `kind` banner titled `title`.
    pub fn new(kind: ThreadBannerKind, title: impl Into<SharedString>) -> Self {
        Self {
            kind,
            title: title.into(),
            body: None,
            actions: Vec::new(),
        }
    }

    /// `children`: the secondary line.
    pub fn body(mut self, body: impl Into<SharedString>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// One of the `actions` (buttons), in order.
    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.actions.push(action.into_any_element());
        self
    }
}

impl RenderOnce for ThreadErrorBanner {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let (fill, accent, icon) = self.kind.look();
        // `.t3-tbanner { display: flex; flex-wrap: wrap; align-items: center; gap: spacing-04;
        // padding: spacing-04 spacing-05; background: …; box-shadow: inset 3px 0 0 …; font-size: 0.875rem }`.
        div()
            .flex()
            .flex_row()
            .flex_wrap()
            .items_center()
            .w_full()
            .gap(rem(12.))
            .py(rem(12.))
            .px(rem(16.))
            .bg(fill)
            .shadow(card::rail(accent))
            .text_size(rem(14.))
            .line_height(rem(18.))
            .text_color(color(ColorRole::TextPrimary))
            .child(if icon == CarbonIcon::WarningAltFilled {
                // `.t3-tbanner--warning > svg path[data-icon-path="inner-path"] { fill: #161616 }`.
                crate::t3::base::warning_glyph(20.).into_any_element()
            } else {
                press::icon(icon, 20., accent).into_any_element()
            })
            // `.t3-tbanner-body { display: flex; flex: 1 1 16rem; flex-direction: column; gap: 2px }`,
            // `b { font-weight: 600 }` (app-r3), `span { color: text-secondary }`.
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_grow(1.)
                    .flex_shrink(1.)
                    .flex_basis(rem(256.))
                    .gap(px(2.))
                    .child(div().font_weight(FontWeight::SEMIBOLD).child(self.title))
                    .when_some(self.body, |this, b| {
                        this.child(div().text_color(color(ColorRole::TextSecondary)).child(b))
                    }),
            )
            .child(card::actions().children(self.actions))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    #[test]
    fn unknown_divider_kinds_fall_back_to_model() {
        assert_eq!(DividerKind::from_name("handoff"), DividerKind::Handoff);
        assert_eq!(DividerKind::from_name("bogus"), DividerKind::Model);
        assert_eq!(DividerKind::Unread.icon(), CarbonIcon::Time);
    }

    #[test]
    fn the_now_line_is_red() {
        crate::theme::tests_set(CarbonTheme::White);
        let (ink, rule) = DividerKind::Now.colors();
        assert_eq!(ink, color(ColorRole::SupportError));
        assert_eq!(rule, ink);
        let (ink, rule) = DividerKind::Model.colors();
        assert_eq!(ink, color(ColorRole::TextSecondary));
        assert_eq!(rule, color(ColorRole::BorderSubtle01));
    }

    #[test]
    fn banners_tint_by_kind() {
        crate::theme::tests_set(CarbonTheme::White);
        let (fill, accent, _) = ThreadBannerKind::Warning.look();
        assert_eq!(fill, color(ColorRole::NotificationBackgroundWarning));
        assert_eq!(accent, color(ColorRole::SupportWarning));
    }
}
