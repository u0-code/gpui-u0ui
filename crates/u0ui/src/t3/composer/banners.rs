//! [`ComposerBanner`] and [`ComposerBannerStack`] (`.t3-banner`,
//! `.t3-bstack`, "Banners" in `t3.css`): notices above the composer.
//!
//! A banner is a 40px `layer-01` row with a 3px accent rail (info, warning,
//! error, success, or `interactive` for progress), its icon (a spinner for
//! progress), a semibold title and secondary text, an optional 4px progress
//! bar, actions and a dismiss ×. The title and text are selectable
//! (Base's `SelectableText`), so what a banner says can be copied. The stack puts 1px between banners; with
//! `collapsed` it shows the first and a "N more notices" link that unfolds
//! the rest ("Show fewer" folds them again).
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::t3::base::{Spin, warning_glyph};
use crate::t3::press::{self, Handler, rem};
use crate::{icons::CarbonIcon, theme::color, tokens::ColorRole};
use gpui_kit::base::SelectableText;
use gpui_kit::{
    AnyElement, App, ElementId, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, Window, div, prelude::FluentBuilder as _, px, relative,
};
use std::rc::Rc;

/// `ComposerBanner` `kind`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BannerKind {
    /// Information.
    #[default]
    Info,
    /// Warning.
    Warning,
    /// Error.
    Error,
    /// Success.
    Success,
    /// In progress.
    Progress,
}

impl BannerKind {
    /// Every kind.
    pub const ALL: [BannerKind; 5] = [
        Self::Info,
        Self::Warning,
        Self::Error,
        Self::Success,
        Self::Progress,
    ];

    /// The reference name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Success => "success",
            Self::Progress => "progress",
        }
    }

    /// `--t3-accent`: `support-info` by default; warning, error, success;
    /// progress `interactive`.
    pub fn accent(self) -> Hsla {
        color(match self {
            Self::Info => ColorRole::SupportInfo,
            Self::Warning => ColorRole::SupportWarning,
            Self::Error => ColorRole::SupportError,
            Self::Success => ColorRole::SupportSuccess,
            Self::Progress => ColorRole::Interactive,
        })
    }

    /// The default icon (progress shows a spinner instead).
    pub const fn icon(self) -> CarbonIcon {
        match self {
            Self::Info => CarbonIcon::InformationFilled,
            Self::Warning => CarbonIcon::WarningAltFilled,
            Self::Error => CarbonIcon::ErrorFilled,
            Self::Success => CarbonIcon::CheckmarkFilled,
            Self::Progress => CarbonIcon::InProgress,
        }
    }
}

/// One notice above the composer. See the module docs.
#[derive(IntoElement)]
pub struct ComposerBanner {
    id: ElementId,
    kind: BannerKind,
    icon: Option<CarbonIcon>,
    title: Option<SharedString>,
    body: Option<SharedString>,
    progress: Option<f32>,
    actions: Vec<AnyElement>,
    dismiss_label: SharedString,
    progress_label: SharedString,
    on_dismiss: Option<Handler>,
}

impl ComposerBanner {
    /// A `kind` banner.
    pub fn new(id: impl Into<ElementId>, kind: BannerKind) -> Self {
        Self {
            id: id.into(),
            kind,
            icon: None,
            title: None,
            body: None,
            progress: None,
            actions: Vec::new(),
            dismiss_label: "Dismiss".into(),
            progress_label: "In progress".into(),
            on_dismiss: None,
        }
    }

    /// `icon`: replaces the kind's.
    pub fn icon(mut self, icon: CarbonIcon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// `title`.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// `children`: the secondary text.
    pub fn body(mut self, body: impl Into<SharedString>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// `progress` (0–100): a 4px bar under the text.
    pub fn progress(mut self, percent: f32) -> Self {
        self.progress = Some(percent);
        self
    }

    /// One of the `actions`.
    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.actions.push(action.into_any_element());
        self
    }

    /// The dismiss × and spinner labels (defaults "Dismiss", "In progress").
    pub fn labels(
        mut self,
        dismiss: impl Into<SharedString>,
        progress: impl Into<SharedString>,
    ) -> Self {
        self.dismiss_label = dismiss.into();
        self.progress_label = progress.into();
        self
    }

    /// `onDismiss`: adds the ×.
    pub fn on_dismiss(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for ComposerBanner {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let accent = self.kind.accent();
        let icon: AnyElement = match (self.kind, self.icon) {
            (BannerKind::Progress, None) => Spin::new((id.clone(), "spin"))
                .label(self.progress_label.clone())
                .into_any_element(),
            // `.t3-banner--warning .t3-banner-icon path[data-icon-path="inner-path"] { fill: #161616 }`.
            (BannerKind::Warning, None) => warning_glyph(16.).into_any_element(),
            (kind, icon) => {
                press::icon(icon.unwrap_or(kind.icon()), 16., accent).into_any_element()
            }
        };
        // `.t3-banner-body { display: flex; flex: 1; flex-wrap: wrap; align-items: center;
        // gap: 0 spacing-03; min-width: 0; padding: spacing-02 0 }`.
        let body = div()
            .flex()
            .flex_row()
            .flex_wrap()
            .flex_1()
            .items_center()
            .gap_x(rem(8.))
            .min_w_0()
            .py(rem(4.))
            // Each wraps by words, whatever an ancestor's white-space, and may
            // shrink below its unwrapped width: a line longer than the banner
            // wraps inside it instead of running past it and under its
            // actions.
            .when_some(self.title.clone(), |this, t| {
                this.child(
                    div()
                        .min_w_0()
                        .whitespace_normal()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(SelectableText::new((id.clone(), "title"), t)),
                )
            })
            .when_some(self.body.clone(), |this, b| {
                this.child(
                    div()
                        .min_w_0()
                        .whitespace_normal()
                        .text_color(color(ColorRole::TextSecondary))
                        .child(SelectableText::new((id.clone(), "body"), b)),
                )
            })
            // `.t3-banner-bar { flex-basis: 100%; height: 4px; margin-top: spacing-02; background: layer-accent-01 }`,
            // `i { background: interactive }`.
            .when_some(self.progress, |this, p| {
                this.child(
                    div()
                        .w_full()
                        .h(px(4.))
                        .mt(rem(4.))
                        .bg(color(ColorRole::LayerAccent01))
                        .child(
                            div()
                                .h_full()
                                .w(relative((p / 100.).clamp(0., 1.)))
                                .bg(color(ColorRole::Interactive)),
                        ),
                )
            });
        let dismiss = self.on_dismiss.clone();
        // `.t3-banner { display: flex; align-items: center; gap: spacing-04; min-height: size-md;
        // padding: spacing-02 spacing-02 spacing-02 spacing-05; background: layer-01;
        // box-shadow: inset 3px 0 0 var(--t3-accent); font-size: 0.875rem }`.
        div()
            .flex()
            .flex_row()
            .items_center()
            .w_full()
            .gap(rem(12.))
            .min_h(rem(40.))
            .pt(rem(4.))
            .pb(rem(4.))
            .pr(rem(4.))
            .pl(rem(16.))
            .bg(color(ColorRole::Layer01))
            .shadow(crate::t3::timeline::card::rail(accent))
            .text_size(rem(14.))
            .line_height(rem(18.))
            .text_color(color(ColorRole::TextPrimary))
            .child(div().flex_none().child(icon))
            .child(body)
            // `.t3-banner-actions { display: flex; gap: 1px }`.
            .when(!self.actions.is_empty(), |this| {
                this.child(
                    div()
                        .flex()
                        .flex_row()
                        .flex_none()
                        .gap(px(1.))
                        .children(self.actions),
                )
            })
            .when_some(dismiss, |this, f| {
                this.child(
                    Button::new((id.clone(), "dismiss"), self.dismiss_label.clone())
                        .icon(CarbonIcon::Close)
                        .icon_only(true)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .on_click(move |_, w, cx| f(w, cx)),
                )
            })
    }
}

/// The stack's fold labels: `"{n} more notices"` and "Show fewer".
#[derive(Clone)]
pub struct BannerStackTexts {
    /// Folded: `"{n} more notices"`.
    pub more: Rc<dyn Fn(usize) -> SharedString>,
    /// Unfolded.
    pub fewer: SharedString,
}

impl Default for BannerStackTexts {
    fn default() -> Self {
        Self {
            more: Rc::new(|n| format!("{n} more notices").into()),
            fewer: "Show fewer".into(),
        }
    }
}

/// Banners stacked 1px apart, optionally folded to the first. See the
/// module docs.
#[derive(IntoElement)]
pub struct ComposerBannerStack {
    id: ElementId,
    banners: Vec<AnyElement>,
    collapsed: bool,
    texts: BannerStackTexts,
}

impl ComposerBannerStack {
    /// An empty stack.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            banners: Vec::new(),
            collapsed: false,
            texts: BannerStackTexts::default(),
        }
    }

    /// `collapsedCount`: start folded to the first banner.
    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }

    /// Replace the strings.
    pub fn texts(mut self, texts: BannerStackTexts) -> Self {
        self.texts = texts;
        self
    }
}

impl ParentElement for ComposerBannerStack {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.banners.extend(elements);
    }
}

impl RenderOnce for ComposerBannerStack {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let collapsed = self.collapsed;
        let open_state = window.use_keyed_state((id.clone(), "open"), cx, move |_, _| !collapsed);
        let open = *open_state.read(cx);
        let count = self.banners.len();
        let foldable = collapsed && count > 1;
        let (focus, ring) = press::focus((id.clone(), "more-focus"), window, cx);
        let link = color(ColorRole::LinkPrimary);
        let mut banners = self.banners;
        if !open {
            banners.truncate(1);
        }
        // `.t3-bstack { display: flex; flex-direction: column; gap: 1px }`.
        div()
            .flex()
            .flex_col()
            .w_full()
            .gap(px(1.))
            .children(banners)
            .when(foldable, |this| {
                // `.t3-bstack-more { display: flex; align-items: center; gap: spacing-02; align-self: flex-start;
                // padding: spacing-02 spacing-03; color: link-primary; font-size: 0.75rem }`.
                this.child(
                    gpui_kit::base::Button::new((id.clone(), "more"))
                        .track_focus(&focus)
                        .relative()
                        .flex()
                        .flex_row()
                        .items_center()
                        .self_start()
                        .gap(rem(4.))
                        .py(rem(4.))
                        .px(rem(8.))
                        .text_size(rem(12.))
                        .line_height(rem(16.))
                        .text_color(link)
                        .cursor_pointer()
                        .on_click(move |_, _, cx| {
                            open_state.update(cx, |o, cx| {
                                *o = !*o;
                                cx.notify();
                            })
                        })
                        .child(if open {
                            self.texts.fewer.clone()
                        } else {
                            (self.texts.more)(count - 1)
                        })
                        .child(press::icon(CarbonIcon::ChevronDown, 12., link))
                        .when(ring, |this| this.child(press::ring())),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    #[test]
    fn accents_follow_the_kind() {
        crate::theme::tests_set(CarbonTheme::White);
        assert_eq!(BannerKind::Info.accent(), color(ColorRole::SupportInfo));
        assert_eq!(BannerKind::Progress.accent(), color(ColorRole::Interactive));
        assert_eq!(BannerKind::Success.icon(), CarbonIcon::CheckmarkFilled);
        assert_eq!((BannerStackTexts::default().more)(3), "3 more notices");
    }
}
