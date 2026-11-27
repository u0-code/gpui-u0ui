//! [`EffortGlyph`] and [`ComposerControl`] (`.t3-effort`, `.t3-cctl`,
//! "Effort / controls" in `t3.css`) and [`ReadOnlyComposerBar`]
//! (`.t3-readonly`).
use crate::t3::press::{self, Handler, rem};
use crate::{icons::CarbonIcon, theme::color, tokens::ColorRole};
use gpui_kit::{
    AnyElement, App, CursorStyle, ElementId, Hsla, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _, px,
};
use std::rc::Rc;

/// `EffortGlyph` `level` (`LEVELS` in `composer.jsx`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum EffortLevel {
    Minimal,
    Low,
    #[default]
    Medium,
    High,
    Max,
}

impl EffortLevel {
    /// Every level, lowest first.
    pub const ALL: [EffortLevel; 5] = [
        Self::Minimal,
        Self::Low,
        Self::Medium,
        Self::High,
        Self::Max,
    ];

    /// The reference name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Minimal => "minimal",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Max => "max",
        }
    }

    /// Lit bars: `LEVELS.indexOf(level) + 1`.
    pub const fn bars(self) -> usize {
        self as usize + 1
    }
}

/// Bar `i` (1-based) height in px: `style={{ height: 4 + i * 2 }}`.
pub const fn effort_bar_height(i: usize) -> f32 {
    4. + i as f32 * 2.
}

/// A bar's fill: `.t3-effort-bar--on { background: icon-primary }`, else
/// `color-mix(in srgb, var(--icon-primary) 26%, transparent)` (gemini-B).
pub fn effort_bar_color(on: bool) -> Hsla {
    let ink = color(ColorRole::IconPrimary);
    if on { ink } else { ink.opacity(0.26) }
}

/// Five rising bars for a reasoning-effort level, with a bolt when `fast`.
#[derive(IntoElement)]
pub struct EffortGlyph {
    level: EffortLevel,
    fast: bool,
}

impl EffortGlyph {
    /// The glyph for `level`.
    pub fn new(level: EffortLevel) -> Self {
        Self { level, fast: false }
    }

    /// `fast`: a 12px `FlashFilled` in `support-warning` after the bars.
    pub fn fast(mut self, fast: bool) -> Self {
        self.fast = fast;
        self
    }
}

impl RenderOnce for EffortGlyph {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let lit = self.level.bars();
        // `.t3-effort { display: inline-flex; align-items: flex-end; gap: 1px; height: 14px }`.
        div()
            .flex()
            .flex_row()
            .flex_none()
            .items_end()
            .gap(px(1.))
            .h(rem(14.))
            .children((1..=5).map(|i| {
                // `.t3-effort-bar { width: 3px }`.
                div()
                    .w(px(3.))
                    .h(rem(effort_bar_height(i)))
                    .bg(effort_bar_color(i <= lit))
            }))
            .when(self.fast, |this| {
                // `.t3-effort-fast { fill: support-warning; margin-left: 2px; align-self: center }`.
                this.child(div().ml(px(2.)).self_center().child(press::icon(
                    CarbonIcon::FlashFilled,
                    12.,
                    color(ColorRole::SupportWarning),
                )))
            })
    }
}

/// `.t3-cctl` colors (background, ink), at rest or hovered / `active`.
pub fn composer_control_colors(hot: bool) -> (Hsla, Hsla) {
    if hot {
        // `.t3-cctl:hover, .t3-cctl--on { background: background-hover; color: text-primary }`.
        (
            color(ColorRole::BackgroundHover),
            color(ColorRole::TextPrimary),
        )
    } else {
        // `background: transparent; color: text-secondary`.
        (
            gpui_kit::transparent_black(),
            color(ColorRole::TextSecondary),
        )
    }
}

/// A composer footer control (`ComposerControl`, `.t3-cctl`): an optional
/// 16px icon, its content and a 12px chevron; `active` while its menu is open.
#[derive(IntoElement)]
pub struct ComposerControl {
    id: ElementId,
    icon: Option<CarbonIcon>,
    label: SharedString,
    children: Vec<AnyElement>,
    active: bool,
    on_click: Option<Handler>,
}

impl ComposerControl {
    /// A control named `label`: its text, or its accessible name when
    /// children replace the text.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            icon: None,
            label: label.into(),
            children: Vec::new(),
            active: false,
            on_click: None,
        }
    }

    /// `icon`: a 16px glyph before the content.
    pub fn icon(mut self, icon: CarbonIcon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// `active` (`.t3-cctl--on`).
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// `onClick`.
    pub fn on_click(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
}

impl ParentElement for ComposerControl {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for ComposerControl {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let (hovered, hover) = press::hover((id.clone(), "hover"), window, cx);
        let (focus, ring) = press::focus((id.clone(), "focus"), window, cx);
        let (bg, ink) = composer_control_colors(hovered || self.active);
        let on_click = self.on_click.clone();
        let content: Vec<AnyElement> = if self.children.is_empty() {
            vec![self.label.clone().into_any_element()]
        } else {
            self.children
        };
        gpui_kit::base::Button::new(id)
            .track_focus(&focus)
            .accessibility_label(self.label.clone())
            .relative()
            .flex()
            .flex_row()
            .flex_none()
            .items_center()
            // `gap: spacing-02; height: size-sm; padding: 0 spacing-03; font-size: 0.75rem`.
            .gap(rem(4.))
            .h(rem(32.))
            .px(rem(8.))
            .bg(bg)
            .text_color(ink)
            .text_size(rem(12.))
            .line_height(rem(16.))
            .whitespace_nowrap()
            .cursor(CursorStyle::PointingHand)
            .on_hover(press::track_hover(hover))
            .when_some(on_click, |this, f| {
                this.on_click(move |_, window, cx| f(window, cx))
            })
            // `.t3-cctl svg { fill: currentColor }`.
            .when_some(self.icon, |this, icon| {
                this.child(press::icon(icon, 16., ink))
            })
            .children(content)
            // `.t3-cctl-chev { opacity: .7 }`.
            .child(press::icon(CarbonIcon::ChevronDown, 12., ink.opacity(0.7)))
            .when(ring, |this| this.child(press::ring()))
    }
}

/// The bar that replaces the composer on a read-only thread
/// (`ReadOnlyComposerBar`, `.t3-readonly`): a lock, the lead, an optional
/// status and an action at the end.
#[derive(IntoElement)]
pub struct ReadOnlyComposerBar {
    lead: SharedString,
    status: Option<SharedString>,
    action: Option<AnyElement>,
}

impl ReadOnlyComposerBar {
    /// The bar with `lead` (the reference default: "This thread is read-only").
    pub fn new(lead: impl Into<SharedString>) -> Self {
        Self {
            lead: lead.into(),
            status: None,
            action: None,
        }
    }

    /// `status`: a 12px note after the lead.
    pub fn status(mut self, status: impl Into<SharedString>) -> Self {
        self.status = Some(status.into());
        self
    }

    /// `action`: usually a ghost `sm` button.
    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.action = Some(action.into_any_element());
        self
    }
}

impl RenderOnce for ReadOnlyComposerBar {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        // `.t3-readonly { display: flex; align-items: center; gap: spacing-03; height: size-lg;
        // padding: 0 spacing-05; background: layer-01; color: text-secondary; font-size: 0.875rem }`.
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(rem(8.))
            .h(rem(48.))
            .px(rem(16.))
            .w_full()
            .min_w_0()
            .bg(color(ColorRole::Layer01))
            .text_color(color(ColorRole::TextSecondary))
            .text_size(rem(14.))
            .line_height(rem(18.))
            // `.t3-readonly > svg { fill: icon-secondary }`.
            .child(press::icon(
                CarbonIcon::Locked,
                16.,
                color(ColorRole::IconSecondary),
            ))
            // `.t3-readonly-lead { color: text-primary }`.
            .child(
                div()
                    .min_w_0()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .text_color(color(ColorRole::TextPrimary))
                    .child(self.lead),
            )
            // `.t3-readonly-status { font-size: 0.75rem }`.
            .when_some(self.status, |this, status| {
                this.child(
                    div()
                        .flex_none()
                        .text_size(rem(12.))
                        .line_height(rem(16.))
                        .child(status),
                )
            })
            .child(div().flex_1())
            .children(self.action)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    fn hex(c: Hsla) -> String {
        let c = c.to_rgb();
        let b = |v: f32| (v * 255.).round() as u8;
        format!("#{:02x}{:02x}{:02x}", b(c.r), b(c.g), b(c.b))
    }

    #[test]
    fn effort_bars_rise_and_dim() {
        crate::theme::tests_set(CarbonTheme::White);
        // `height: 4 + i * 2` for i = 1..=5.
        assert_eq!(
            (1..=5).map(effort_bar_height).collect::<Vec<_>>(),
            [6., 8., 10., 12., 14.]
        );
        assert_eq!(EffortLevel::Minimal.bars(), 1);
        assert_eq!(EffortLevel::Medium.bars(), 3);
        assert_eq!(EffortLevel::Max.bars(), 5);
        assert_eq!(hex(effort_bar_color(true)), "#161616");
        assert!((effort_bar_color(false).a - 0.26).abs() < 1e-3);
    }

    #[test]
    fn composer_control_hover_and_on_share_a_look() {
        crate::theme::tests_set(CarbonTheme::White);
        let (bg, ink) = composer_control_colors(true);
        assert_eq!(bg, color(ColorRole::BackgroundHover));
        assert_eq!(hex(ink), "#161616");
        let (rest, ink) = composer_control_colors(false);
        assert_eq!(rest.a, 0.);
        assert_eq!(hex(ink), "#525252");
    }
}
