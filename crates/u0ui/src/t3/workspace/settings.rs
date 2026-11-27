//! T3 · Settings: [`SettingsNav`], [`SettingsSection`], [`SettingsRow`],
//! [`RichText`], [`KeybindingPill`], [`KeybindingRow`] and [`ThemeCard`]
//! (`src/t3/workspace.jsx`, "Settings" in `src/t3.css` plus its later
//! overrides).
use super::parts::{child_id, icon_button, title_tooltip};
use crate::{
    components::{Badge, BadgeSize, BadgeVariant, Button, ButtonSize, ButtonVariant, Kbd},
    icons::{CarbonIcon, Icon},
    t3::{
        base::{visually_hidden, warning_glyph},
        craft::{mono, sans},
        press::{self, Handler, rem},
    },
    theme::color,
    tokens::{BORDER_RADIUS_16, ColorRole},
};
use gpui_kit::{
    AnyElement, App, CursorStyle, ElementId, Hsla, InteractiveElement as _, IntoElement,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _, px, relative,
};
use std::rc::Rc;

// ── RichText ────────────────────────────────────────────────────────────

/// A string or an element, rendered as-is (`RichText`): use it for parts
/// whose text holes may carry inline code or links, such as
/// [`SettingsRow::description`]. With no value it renders its children
/// (`value != null && value !== false ? value : children`).
#[derive(IntoElement, Default)]
pub struct RichText {
    value: Option<AnyElement>,
    children: Vec<AnyElement>,
}

impl RichText {
    /// An empty rich text (renders its children, if any).
    pub fn new() -> Self {
        Self::default()
    }

    /// The value (`value`): a string or any element.
    pub fn value(mut self, value: impl IntoElement) -> Self {
        self.value = Some(value.into_any_element());
        self
    }

    /// Whether there is nothing to render.
    pub fn is_empty(&self) -> bool {
        self.value.is_none() && self.children.is_empty()
    }
}

impl ParentElement for RichText {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl From<&'static str> for RichText {
    fn from(s: &'static str) -> Self {
        Self::new().value(SharedString::from(s))
    }
}

impl From<String> for RichText {
    fn from(s: String) -> Self {
        Self::new().value(SharedString::from(s))
    }
}

impl From<SharedString> for RichText {
    fn from(s: SharedString) -> Self {
        Self::new().value(s)
    }
}

impl From<AnyElement> for RichText {
    fn from(e: AnyElement) -> Self {
        Self {
            value: Some(e),
            children: Vec::new(),
        }
    }
}

impl RenderOnce for RichText {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        // A text run and inline elements flow together (`display: contents` in spirit).
        match self.value {
            Some(value) => div()
                .flex()
                .flex_row()
                .flex_wrap()
                .items_baseline()
                .child(value),
            None => div()
                .flex()
                .flex_row()
                .flex_wrap()
                .items_baseline()
                .children(self.children),
        }
    }
}

// ── SettingsNav ─────────────────────────────────────────────────────────

/// Severity tone for a [`SettingsNavItem`] badge.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SettingsBadgeTone {
    /// Informational blue tag (`tag-background-blue` / `tag-color-blue`), the default.
    #[default]
    Info,
    /// Neutral count tag (`tag-background-gray` / `tag-color-gray`).
    Neutral,
    /// High-contrast warning pill (`support-warning` with `#161616` dark ink).
    Warning,
    /// High-contrast error pill (`support-error` with `text-on-color`).
    Error,
}

impl SettingsBadgeTone {
    /// Every badge tone.
    pub const ALL: [Self; 4] = [Self::Info, Self::Neutral, Self::Warning, Self::Error];

    /// The tone name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Neutral => "neutral",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }

    /// `(background, text)` colors per Carbon contrast rules.
    pub fn colors(self) -> (Hsla, Hsla) {
        match self {
            Self::Info => (
                color(ColorRole::TagBackgroundBlue),
                color(ColorRole::TagColorBlue),
            ),
            Self::Neutral => (
                color(ColorRole::TagBackgroundGray),
                color(ColorRole::TagColorGray),
            ),
            Self::Warning => (color(ColorRole::SupportWarning), crate::t3::base::ink()),
            Self::Error => (
                color(ColorRole::SupportError),
                color(ColorRole::TextOnColor),
            ),
        }
    }
}

/// One settings page link (`{ id, label, icon, badge }`).
#[derive(Clone)]
pub struct SettingsNavItem {
    /// `id`: the value.
    pub id: SharedString,
    /// `label`.
    pub label: SharedString,
    /// `icon` (default `Settings`).
    pub icon: Icon,
    /// `badge`: a small status or count pill.
    pub badge: Option<SharedString>,
    /// `badge_tone`: severity tone of `badge` (default [`SettingsBadgeTone::Info`]).
    pub badge_tone: SettingsBadgeTone,
}

impl SettingsNavItem {
    /// An item.
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: Icon::carbon(CarbonIcon::Settings),
            badge: None,
            badge_tone: SettingsBadgeTone::Info,
        }
    }

    /// The glyph.
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = icon.into();
        self
    }

    /// A badge (`badge`).
    pub fn badge(mut self, badge: impl Into<SharedString>) -> Self {
        self.badge = Some(badge.into());
        self
    }

    /// Set the severity tone of [`Self::badge`].
    pub fn badge_tone(mut self, tone: SettingsBadgeTone) -> Self {
        self.badge_tone = tone;
        self
    }

    /// Convenience: set a [`SettingsBadgeTone::Warning`] badge.
    pub fn warn_badge(mut self, badge: impl Into<SharedString>) -> Self {
        self.badge = Some(badge.into());
        self.badge_tone = SettingsBadgeTone::Warning;
        self
    }

    /// Convenience: set a [`SettingsBadgeTone::Error`] badge.
    pub fn error_badge(mut self, badge: impl Into<SharedString>) -> Self {
        self.badge = Some(badge.into());
        self.badge_tone = SettingsBadgeTone::Error;
        self
    }
}

/// A group of links (`{ label, items }`; an empty label draws no header).
#[derive(Clone)]
pub struct SettingsNavSection {
    /// `label`.
    pub label: SharedString,
    /// `items`.
    pub items: Vec<SettingsNavItem>,
}

impl SettingsNavSection {
    /// A section.
    pub fn new(
        label: impl Into<SharedString>,
        items: impl IntoIterator<Item = SettingsNavItem>,
    ) -> Self {
        Self {
            label: label.into(),
            items: items.into_iter().collect(),
        }
    }
}

/// The settings side navigation (`SettingsNav`, `.t3-snav`): grouped links
/// with a glyph and optional badge; the current one is `layer-selected-01`,
/// semibold, with a 3px `border-interactive` start bar. 15rem wide.
/// Controlled with [`Self::value`], else it keeps its own (first item).
#[derive(IntoElement)]
pub struct SettingsNav {
    id: ElementId,
    sections: Vec<SettingsNavSection>,
    value: Option<SharedString>,
    label: SharedString,
    on_value_change: Option<super::parts::TextHandler>,
}

impl SettingsNav {
    /// A nav over `sections`.
    pub fn new(
        id: impl Into<ElementId>,
        sections: impl IntoIterator<Item = SettingsNavSection>,
    ) -> Self {
        Self {
            id: id.into(),
            sections: sections.into_iter().collect(),
            value: None,
            label: "Settings".into(),
            on_value_change: None,
        }
    }

    /// The current item id (`value`, controlled).
    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// The navigation's accessible name (default `Settings`).
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    /// An item chosen (`onValueChange`).
    pub fn on_value_change(
        mut self,
        f: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_value_change = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for SettingsNav {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let first = self
            .sections
            .first()
            .and_then(|s| s.items.first())
            .map(|i| i.id.clone())
            .unwrap_or_default();
        let (current, store) = crate::components::form::controlled(
            (id.clone(), "value"),
            self.value.clone(),
            first,
            window,
            cx,
        );
        let mut n = 0;
        let groups = self
            .sections
            .iter()
            .map(|section| {
                let items = section
                    .items
                    .iter()
                    .map(|item| {
                        let i = n;
                        n += 1;
                        let on = item.id == current;
                        let item_id = child_id(&id, "item", i);
                        let (hovered, hover) = press::hover((item_id.clone(), "hover"), window, cx);
                        let (focus, ring) = press::focus((item_id.clone(), "focus"), window, cx);
                        // `.t3-snav-item { color: text-secondary }`, `:hover { background:
                        // layer-hover-01; color: text-primary }`, `--on { background:
                        // layer-selected-01; color: text-primary; font-weight: 600 }`.
                        let ink = color(if on || hovered {
                            ColorRole::TextPrimary
                        } else {
                            ColorRole::TextSecondary
                        });
                        let bg = if on {
                            Some(color(ColorRole::LayerSelected01))
                        } else if hovered {
                            Some(color(ColorRole::LayerHover01))
                        } else {
                            None
                        };
                        let value = item.id.clone();
                        let store = store.clone();
                        let f = self.on_value_change.clone();
                        // `.t3-snav-item { display: flex; align-items: center; gap: spacing-04;
                        // width: 100%; height: size-sm; padding: 0 spacing-05; font-size: 0.875rem }`.
                        sans(
                            gpui_kit::base::Button::new(item_id)
                                .track_focus(&focus)
                                .accessibility_label(item.label.clone()),
                            14.,
                            if on { 600 } else { 400 },
                        )
                        .relative()
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_start()
                        .gap(rem(12.))
                        .w_full()
                        .h(rem(32.))
                        .px(rem(16.))
                        .text_color(ink)
                        .when_some(bg, |this, bg| this.bg(bg))
                        .cursor(CursorStyle::PointingHand)
                        .on_hover(press::track_hover(hover))
                        .on_click(move |_, window, cx| {
                            store.update(cx, |v, cx| {
                                *v = value.clone();
                                cx.notify();
                            });
                            if let Some(f) = f.as_ref() {
                                f(value.clone(), window, cx);
                            }
                        })
                        // `svg { fill: currentColor; flex: none }`.
                        .child(
                            item.icon
                                .clone()
                                .size_length(rem(16.))
                                .color(ink)
                                .flex_none(),
                        )
                        .child(div().flex_1().text_left().child(item.label.clone()))
                        .when_some(item.badge.clone(), |this, badge| {
                            let tag = match item.badge_tone {
                                SettingsBadgeTone::Info => Badge::new(badge)
                                    .variant(BadgeVariant::Blue)
                                    .size(BadgeSize::Sm),
                                SettingsBadgeTone::Neutral => Badge::new(badge)
                                    .variant(BadgeVariant::Gray)
                                    .size(BadgeSize::Sm),
                                tone => {
                                    let (bg, fg) = tone.colors();
                                    Badge::new(badge).size(BadgeSize::Sm).colors(bg, fg)
                                }
                            };
                            this.child(tag)
                        })
                        // `box-shadow: inset 3px 0 0 border-interactive`.
                        .when(on, |this| {
                            this.child(
                                div()
                                    .absolute()
                                    .top_0()
                                    .bottom_0()
                                    .left_0()
                                    .w(px(3.))
                                    .bg(color(ColorRole::BorderInteractive)),
                            )
                        })
                        .when(ring, |this| this.child(press::ring()))
                        .into_any_element()
                    })
                    .collect::<Vec<_>>();
                div()
                    .flex()
                    .flex_col()
                    .when(!section.label.is_empty(), |this| {
                        // `.t3-snav-h { padding: 0 spacing-05 spacing-02; font-size: 0.75rem;
                        // color: text-secondary }`.
                        this.child(
                            sans(div(), 12., 400)
                                .px(rem(16.))
                                .pb(rem(4.))
                                .text_color(color(ColorRole::TextSecondary))
                                .child(section.label.clone()),
                        )
                    })
                    .children(items)
            })
            .collect::<Vec<_>>();
        // `.t3-snav { display: flex; flex-direction: column; gap: spacing-05; width: 15rem;
        // padding: spacing-05 0 }`.
        div()
            .id(id)
            .flex()
            .flex_col()
            .gap(rem(16.))
            .w(rem(240.))
            .py(rem(16.))
            .children(groups)
    }
}

// ── SettingsSection / SettingsRow ───────────────────────────────────────

/// A titled block of settings rows (`SettingsSection`, `.t3-ssec`): a 16px
/// semibold title, an optional description and action, then the rows on
/// `layer-01`.
#[derive(IntoElement)]
pub struct SettingsSection {
    title: SharedString,
    description: Option<SharedString>,
    action: Option<AnyElement>,
    children: Vec<AnyElement>,
}

impl SettingsSection {
    /// A section titled `title`.
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            description: None,
            action: None,
            children: Vec::new(),
        }
    }

    /// The description (`description`).
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// An action at the header's end (`action`).
    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.action = Some(action.into_any_element());
        self
    }
}

impl ParentElement for SettingsSection {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for SettingsSection {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let rows = self.children.into_iter().enumerate().map(|(i, row)| {
            // `.t3-srow + .t3-srow { box-shadow: inset 0 1px 0 var(--border-subtle-00) }`.
            div().relative().child(row).when(i > 0, |this| {
                this.child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .right_0()
                        .h(px(1.))
                        .bg(color(ColorRole::BorderSubtle00)),
                )
            })
        });
        // `.t3-ssec { max-width: none }`; `.t3-ssec-head { display: flex;
        // justify-content: space-between; gap: spacing-05; padding-bottom: spacing-05;
        // align-items: flex-end }`.
        div()
            .flex()
            .flex_col()
            .w_full()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_between()
                    .items_end()
                    .gap(rem(16.))
                    .pb(rem(16.))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .min_w_0()
                            // `h3 { font-size: 1rem; font-weight: 600; line-height: 1.5rem }`.
                            .child(
                                sans(div(), 16., 600)
                                    .line_height(rem(24.))
                                    .text_color(color(ColorRole::TextPrimary))
                                    .child(self.title),
                            )
                            // `p { margin-top: spacing-02; font-size: 0.875rem;
                            // color: text-secondary; line-height: 1.25rem }`.
                            .when_some(self.description, |this, d| {
                                this.child(
                                    sans(div(), 14., 400)
                                        .mt(rem(4.))
                                        .line_height(rem(20.))
                                        .text_color(color(ColorRole::TextSecondary))
                                        .child(d),
                                )
                            }),
                    )
                    .children(self.action),
            )
            // `.t3-ssec-rows { background: layer-01 }`.
            .child(
                div()
                    .flex()
                    .flex_col()
                    .bg(color(ColorRole::Layer01))
                    .children(rows),
            )
    }
}

/// Texts of a [`SettingsRow`] (hard-coded in the reference).
#[derive(Clone)]
pub struct SettingsRowTexts {
    /// The overridden tag (`Overridden`).
    pub overridden: SharedString,
    /// The modified dot's title (`Changed from default`).
    pub changed: SharedString,
    /// The managed lock (`Managed`).
    pub managed: SharedString,
    /// The lock's title (`Managed by your organization`).
    pub managed_title: SharedString,
    /// `Global value: {v}`.
    pub inherited: Rc<dyn Fn(&str) -> SharedString>,
    /// The overridden reset button (`Reset`).
    pub reset: SharedString,
    /// The modified reset icon button (`Reset to default`).
    pub reset_default: SharedString,
}

impl Default for SettingsRowTexts {
    fn default() -> Self {
        Self {
            overridden: "Overridden".into(),
            changed: "Changed from default".into(),
            managed: "Managed".into(),
            managed_title: "Managed by your organization".into(),
            inherited: Rc::new(|v| format!("Global value: {v}").into()),
            reset: "Reset".into(),
            reset_default: "Reset to default".into(),
        }
    }
}

/// One setting (`SettingsRow`, `.t3-srow`): title (with the Overridden tag,
/// a changed-from-default dot, a Managed lock), description, scope and
/// inherited value on the left; Reset and the control on the right. A
/// managed row's control fades to 60% and ignores the pointer.
#[derive(IntoElement)]
pub struct SettingsRow {
    id: ElementId,
    title: SharedString,
    description: RichText,
    control: Option<AnyElement>,
    scope: Option<SharedString>,
    managed: bool,
    modified: bool,
    overridden: bool,
    inherited_value: Option<SharedString>,
    on_reset: Option<Handler>,
    texts: SettingsRowTexts,
}

impl SettingsRow {
    /// A row titled `title`.
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: RichText::new(),
            control: None,
            scope: None,
            managed: false,
            modified: false,
            overridden: false,
            inherited_value: None,
            on_reset: None,
            texts: SettingsRowTexts::default(),
        }
    }

    /// The description: text, or any element via [`RichText`].
    pub fn description(mut self, description: impl Into<RichText>) -> Self {
        self.description = description.into();
        self
    }

    /// The control at the end (`control`).
    pub fn control(mut self, control: impl IntoElement) -> Self {
        self.control = Some(control.into_any_element());
        self
    }

    /// The scope line (`scope`, in `link-primary` with a laptop glyph).
    pub fn scope(mut self, scope: impl Into<SharedString>) -> Self {
        self.scope = Some(scope.into());
        self
    }

    /// Managed by the organization (`managed`).
    pub fn managed(mut self, managed: bool) -> Self {
        self.managed = managed;
        self
    }

    /// Changed from its default (`modified`): a blue dot and, with
    /// [`Self::on_reset`], a Reset to default icon button.
    pub fn modified(mut self, modified: bool) -> Self {
        self.modified = modified;
        self
    }

    /// Overridden for this scope (`overridden`): the tag and, with
    /// [`Self::on_reset`], a Reset button.
    pub fn overridden(mut self, overridden: bool) -> Self {
        self.overridden = overridden;
        self
    }

    /// The inherited global value shown when overridden (`inheritedValue`).
    pub fn inherited_value(mut self, value: impl Into<SharedString>) -> Self {
        self.inherited_value = Some(value.into());
        self
    }

    /// Reset pressed (`onReset`).
    pub fn on_reset(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_reset = Some(Rc::new(f));
        self
    }

    /// Replace the default texts.
    pub fn texts(mut self, texts: SettingsRowTexts) -> Self {
        self.texts = texts;
        self
    }
}

/// The 12px text style of the row's secondary lines (`.t3-srow-desc {
/// font-size: 0.75rem; line-height: 1.33333; color: text-secondary }`).
fn desc(el: gpui_kit::Div) -> gpui_kit::Div {
    sans(el, 12., 400)
        .line_height(relative(1.33333))
        .text_color(color(ColorRole::TextSecondary))
}

impl RenderOnce for SettingsRow {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let texts = self.texts.clone();
        let mut title = sans(div(), 14., 400)
            .flex()
            .flex_row()
            .flex_wrap()
            .items_center()
            .gap(rem(8.))
            .text_color(color(ColorRole::TextPrimary))
            .child(self.title.clone());
        if self.overridden {
            // `.g-over-tag { padding: 0 spacing-02; background: tag-background-blue;
            // color: tag-color-blue; font-size: 0.6875rem; line-height: 1rem; border-radius: 16 }`.
            title = title.child(
                sans(div(), 11., 400)
                    .line_height(rem(16.))
                    .px(rem(4.))
                    .rounded(BORDER_RADIUS_16)
                    .bg(color(ColorRole::TagBackgroundBlue))
                    .text_color(color(ColorRole::TagColorBlue))
                    .child(texts.overridden.clone()),
            );
        }
        if self.modified {
            // `.t3-srow-dot { width: 6px; height: 6px; border-radius: 50%; background: interactive }`.
            title = title.child(
                div()
                    .id((id.clone(), "dot"))
                    .flex_none()
                    .size(px(6.))
                    .rounded_full()
                    .bg(color(ColorRole::Interactive))
                    .tooltip(title_tooltip(texts.changed.clone()))
                    .child(visually_hidden(texts.changed.clone())),
            );
        }
        if self.managed {
            // `.t3-srow-lock { display: inline-flex; align-items: center; gap: 2px;
            // font-size: 0.6875rem; color: text-secondary }`.
            let ink = color(ColorRole::TextSecondary);
            title = title.child(
                sans(div(), 11., 400)
                    .id((id.clone(), "lock"))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(2.))
                    .text_color(ink)
                    .tooltip(title_tooltip(texts.managed_title.clone()))
                    .child(press::icon(CarbonIcon::Locked, 12., ink))
                    .child(texts.managed.clone()),
            );
        }
        let scope = self.scope.clone().map(|scope| {
            // `.t3-srow-scope { display: inline-flex; align-items: center; gap: spacing-02;
            // font-size: 0.75rem; color: link-primary }`.
            let ink = color(ColorRole::LinkPrimary);
            sans(div(), 12., 400)
                .flex()
                .flex_row()
                .items_center()
                .gap(rem(4.))
                .text_color(ink)
                .child(press::icon(CarbonIcon::Laptop, 12., ink))
                .child(scope)
        });
        let inherited = self
            .inherited_value
            .clone()
            .filter(|_| self.overridden)
            .map(|v| desc(div()).child((texts.inherited)(&v)));
        let reset = self.on_reset.clone().and_then(|f| {
            if self.overridden {
                Some(
                    Button::new((id.clone(), "reset"), texts.reset.clone())
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .on_click(move |_, w, cx| f(w, cx))
                        .into_any_element(),
                )
            } else if self.modified {
                Some(
                    icon_button(
                        (id.clone(), "reset-default"),
                        texts.reset_default.clone(),
                        CarbonIcon::Undo,
                    )
                    .on_click(move |_, w, cx| f(w, cx))
                    .into_any_element(),
                )
            } else {
                None
            }
        });
        // `.t3-srow-ctl { display: flex; flex: none; align-items: center; gap: spacing-04 }`;
        // `.t3-srow--managed .t3-srow-ctl { opacity: .6; pointer-events: none }`.
        let control = div()
            .relative()
            .flex()
            .flex_row()
            .flex_none()
            .items_center()
            .gap(rem(12.))
            .children(reset)
            .children(self.control)
            .when(self.managed, |this| {
                this.opacity(0.6).child(
                    div()
                        .id((id.clone(), "managed-block"))
                        .absolute()
                        .top_0()
                        .left_0()
                        .right_0()
                        .bottom_0()
                        .occlude(),
                )
            });
        // `.t3-srow { display: flex; align-items: center; gap: spacing-07; min-height: 4rem;
        // padding: spacing-05 1.25rem; border-bottom: 0 }`.
        div()
            .id(id)
            .flex()
            .flex_row()
            .items_center()
            .gap(rem(32.))
            .min_h(rem(64.))
            .py(rem(16.))
            .px(rem(20.))
            .when(self.overridden, |this| {
                // `.t3-srow--over { box-shadow: inset 2px 0 0 var(--interactive) }`.
                this.relative().child(
                    div()
                        .absolute()
                        .top_0()
                        .bottom_0()
                        .left_0()
                        .w(px(2.))
                        .bg(color(ColorRole::Interactive)),
                )
            })
            .child(
                // `.t3-srow-text { display: flex; flex: 1; min-width: 0; flex-direction: column;
                // gap: spacing-02 }`.
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .gap(rem(4.))
                    .child(title)
                    .when(!self.description.is_empty(), |this| {
                        this.child(desc(div()).child(self.description))
                    })
                    .children(scope)
                    .children(inherited),
            )
            .child(control)
    }
}

// ── Keybindings ─────────────────────────────────────────────────────────

/// The keys of a binding (`keys.split('+')`).
pub fn split_keys(keys: &str) -> Vec<&str> {
    keys.split('+').collect()
}

/// A key combination as key caps (`KeybindingPill`, `.t3-kbpill`: 2px apart).
#[derive(IntoElement, Clone, Debug)]
pub struct KeybindingPill {
    keys: SharedString,
}

impl KeybindingPill {
    /// `keys` joined by `+` (`⌘+⇧+Y`).
    pub fn new(keys: impl Into<SharedString>) -> Self {
        Self { keys: keys.into() }
    }
}

impl RenderOnce for KeybindingPill {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div().flex().flex_row().flex_none().gap(px(2.)).children(
            split_keys(&self.keys)
                .into_iter()
                .map(|k| Kbd::new(SharedString::from(k.to_string()))),
        )
    }
}

/// Texts of a [`KeybindingRow`].
#[derive(Clone, Debug)]
pub struct KeybindingTexts {
    /// The custom badge (`Custom`).
    pub custom: SharedString,
    /// The edit button (`Edit keybinding`).
    pub edit: SharedString,
}

impl Default for KeybindingTexts {
    fn default() -> Self {
        Self {
            custom: "Custom".into(),
            edit: "Edit keybinding".into(),
        }
    }
}

/// `.t3-kbrow { grid-template-columns: 1fr 9rem 12rem 1.5rem 2rem }`: the
/// fixed columns after the command, in px.
pub const KEYBINDING_COLUMNS: [f32; 4] = [144., 192., 24., 32.];

/// One keybinding (`KeybindingRow`, `.t3-kbrow`): command (with a Custom
/// badge), keys, the `when` clause in mono, a warning glyph with its
/// tooltip, and Edit, in a five-column grid over a bottom hairline.
#[derive(IntoElement)]
pub struct KeybindingRow {
    id: ElementId,
    command: SharedString,
    keys: SharedString,
    when: Option<SharedString>,
    warning: Option<SharedString>,
    custom: bool,
    texts: KeybindingTexts,
    on_edit: Option<Handler>,
}

impl KeybindingRow {
    /// The row for `command` bound to `keys`.
    pub fn new(
        id: impl Into<ElementId>,
        command: impl Into<SharedString>,
        keys: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            command: command.into(),
            keys: keys.into(),
            when: None,
            warning: None,
            custom: false,
            texts: KeybindingTexts::default(),
            on_edit: None,
        }
    }

    /// The `when` clause.
    pub fn when_clause(mut self, when: impl Into<SharedString>) -> Self {
        self.when = Some(when.into());
        self
    }

    /// A warning (`warning`): the glyph's tooltip.
    pub fn warning(mut self, warning: impl Into<SharedString>) -> Self {
        self.warning = Some(warning.into());
        self
    }

    /// A user binding (`custom`).
    pub fn custom(mut self, custom: bool) -> Self {
        self.custom = custom;
        self
    }

    /// Replace the default texts.
    pub fn texts(mut self, texts: KeybindingTexts) -> Self {
        self.texts = texts;
        self
    }

    /// Edit pressed.
    pub fn on_edit(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_edit = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for KeybindingRow {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let [keys_w, when_w, warn_w, edit_w] = KEYBINDING_COLUMNS;
        let on_edit = self.on_edit.clone();
        // `.t3-kbrow { display: grid; align-items: center; gap: spacing-04;
        // min-height: size-md; padding: 0 spacing-02 0 spacing-05;
        // border-bottom: 1px solid border-subtle-01; font-size: 0.875rem }`.
        sans(div(), 14., 400)
            .id(id.clone())
            .flex()
            .flex_row()
            .items_center()
            .gap(rem(12.))
            .min_h(rem(40.))
            .pl(rem(16.))
            .pr(rem(4.))
            .border_b_1()
            .border_color(color(ColorRole::BorderSubtle01))
            .text_color(color(ColorRole::TextPrimary))
            .child(
                // `.t3-kbrow-cmd { display: flex; gap: spacing-03; align-items: center }`.
                div()
                    .flex()
                    .flex_row()
                    .flex_1()
                    .min_w_0()
                    .items_center()
                    .gap(rem(8.))
                    .child(self.command.clone())
                    .when(self.custom, |this| {
                        this.child(
                            Badge::new(self.texts.custom.clone())
                                .variant(BadgeVariant::Blue)
                                .size(BadgeSize::Sm),
                        )
                    }),
            )
            .child(
                div()
                    .flex_none()
                    .w(rem(keys_w))
                    .child(KeybindingPill::new(self.keys.clone())),
            )
            .child(
                // `.t3-kbrow-when { overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
                // font-family: mono; font-size: 0.75rem; color: text-secondary }`.
                mono(div(), 12., 400)
                    .flex_none()
                    .w(rem(when_w))
                    .overflow_hidden()
                    .text_ellipsis()
                    .whitespace_nowrap()
                    .text_color(color(ColorRole::TextSecondary))
                    .children(self.when.clone()),
            )
            .child(
                div()
                    .id((id.clone(), "warn"))
                    .flex()
                    .flex_none()
                    .w(rem(warn_w))
                    .when_some(self.warning.clone(), |this, w| {
                        this.tooltip(title_tooltip(w.clone()))
                            .child(warning_glyph(16.))
                            .child(visually_hidden(w))
                    }),
            )
            .child(
                div().flex_none().w(rem(edit_w)).child(
                    icon_button(
                        (id.clone(), "edit"),
                        self.texts.edit.clone(),
                        CarbonIcon::Edit,
                    )
                    .when_some(on_edit, |b, f| b.on_click(move |_, w, cx| f(w, cx))),
                ),
            )
    }
}

// ── ThemeCard ───────────────────────────────────────────────────────────

/// A theme in the library (`ThemeCard`, `.t3-theme`): a 6.5rem wireframe in
/// the theme's colors (`colors`: background, sidebar, body, secondary,
/// accent), then name, `mode · author` and color dots; selected draws a 2px
/// `border-interactive` frame and a check. 13rem wide.
#[derive(IntoElement)]
pub struct ThemeCard {
    id: ElementId,
    name: SharedString,
    colors: Vec<Hsla>,
    selected: bool,
    mode: SharedString,
    author: Option<SharedString>,
    on_click: Option<Handler>,
}

impl ThemeCard {
    /// A card for the theme `name`.
    pub fn new(id: impl Into<ElementId>, name: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            colors: Vec::new(),
            selected: false,
            mode: "Light".into(),
            author: None,
            on_click: None,
        }
    }

    /// The five preview colors (`colors`).
    pub fn colors(mut self, colors: impl IntoIterator<Item = Hsla>) -> Self {
        self.colors = colors.into_iter().collect();
        self
    }

    /// Selected (`selected`, `aria-pressed`).
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// The mode label (`mode`, default `Light`).
    pub fn mode(mut self, mode: impl Into<SharedString>) -> Self {
        self.mode = mode.into();
        self
    }

    /// The author (`author`).
    pub fn author(mut self, author: impl Into<SharedString>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Pressed.
    pub fn on_click(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
}

/// `mode · author` (`{mode}{author ? ` · ${author}` : ''}`).
pub fn theme_caption(mode: &str, author: Option<&str>) -> String {
    match author {
        Some(a) => format!("{mode} \u{b7} {a}"),
        None => mode.to_string(),
    }
}

impl RenderOnce for ThemeCard {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let (hovered, hover) = press::hover((id.clone(), "hover"), window, cx);
        let (focus, ring) = press::focus((id.clone(), "focus"), window, cx);
        let c = |i: usize| {
            self.colors
                .get(i)
                .copied()
                .unwrap_or(gpui_kit::transparent_black())
        };
        // `.t3-theme-wire { position: relative; height: 6.5rem; margin: 1px 1px 0 }`;
        // `i { left: 0; top: 0; bottom: 0; width: 28% }`; `span { left: 34%; right: 8%;
        // height: 8px; top: 20% }`; `span + span { top: 36%; right: 30% }`;
        // `b { right: 8%; bottom: 14%; width: 22%; height: 14px }`.
        let wire = div()
            .relative()
            .h(rem(104.))
            .mt(px(1.))
            .mx(px(1.))
            .bg(c(0))
            .child(
                div()
                    .absolute()
                    .left_0()
                    .top_0()
                    .bottom_0()
                    .w(relative(0.28))
                    .bg(c(1)),
            )
            .child(
                div()
                    .absolute()
                    .left(relative(0.34))
                    .right(relative(0.08))
                    .top(relative(0.2))
                    .h(px(8.))
                    .bg(c(2)),
            )
            .child(
                div()
                    .absolute()
                    .left(relative(0.34))
                    .right(relative(0.3))
                    .top(relative(0.36))
                    .h(px(8.))
                    .bg(c(3)),
            )
            .child(
                div()
                    .absolute()
                    .right(relative(0.08))
                    .bottom(relative(0.14))
                    .w(relative(0.22))
                    .h(px(14.))
                    .bg(c(4)),
            );
        // `.t3-theme-dots { display: flex; gap: 3px; margin-top: spacing-02 }`;
        // `i { width: 12px; height: 12px; border-radius: 50%; outline: 1px solid border-subtle-01 }`.
        let dots = div().flex().flex_row().gap(px(3.)).mt(rem(4.)).children(
            self.colors.iter().skip(1).map(|c| {
                div().size(px(12.)).rounded_full().bg(*c).shadow(vec![
                    gpui_kit::BoxShadow::new(px(0.), px(0.), color(ColorRole::BorderSubtle01))
                        .spread_radius(px(1.)),
                ])
            }),
        );
        let caption = theme_caption(&self.mode, self.author.as_deref());
        let meta = sans(div(), 14., 400)
            // `.t3-theme-meta { position: relative; display: flex; flex-direction: column;
            // gap: 2px; padding: spacing-04; font-size: 0.875rem }`.
            .relative()
            .flex()
            .flex_col()
            .gap(px(2.))
            .p(rem(12.))
            .child(sans(div(), 14., 600).child(self.name.clone()))
            .child(super::parts::card_sub(caption))
            .child(dots)
            .when(self.selected, |this| {
                // `.t3-theme-check { position: absolute; right: spacing-04; top: spacing-04;
                // fill: interactive }`.
                this.child(
                    press::icon(
                        CarbonIcon::CheckmarkFilled,
                        16.,
                        color(ColorRole::Interactive),
                    )
                    .absolute()
                    .right(rem(12.))
                    .top(rem(12.)),
                )
            });
        let frame = |width: f32, offset: f32, c: Hsla| {
            div()
                .absolute()
                .top(px(-offset))
                .left(px(-offset))
                .right(px(-offset))
                .bottom(px(-offset))
                .border(px(width))
                .border_color(c)
        };
        let on_click = self.on_click.clone();
        // `.t3-theme { width: 13rem; background: layer-01; outline: 1px solid border-subtle-01;
        // outline-offset: -1px }`, `:hover { background: layer-hover-01 }`, `--on { outline:
        // 2px solid border-interactive; outline-offset: -2px }`, `:focus-visible { outline: 2px
        // solid focus }` (keeping the offset).
        let outline = if ring {
            let offset = if self.selected { 2. } else { 1. };
            frame(2., 2. - offset, color(ColorRole::Focus))
        } else if self.selected {
            frame(2., 0., color(ColorRole::BorderInteractive))
        } else {
            frame(1., 0., color(ColorRole::BorderSubtle01))
        };
        gpui_kit::base::Button::new(id)
            .track_focus(&focus)
            .accessibility_label(self.name.clone())
            .relative()
            .flex()
            .flex_col()
            .items_stretch()
            .w(rem(208.))
            .flex_none()
            .bg(color(if hovered {
                ColorRole::LayerHover01
            } else {
                ColorRole::Layer01
            }))
            .text_color(color(ColorRole::TextPrimary))
            .cursor(CursorStyle::PointingHand)
            .on_hover(press::track_hover(hover))
            .when_some(on_click, |this, f| this.on_click(move |_, w, cx| f(w, cx)))
            .child(wire)
            .child(meta)
            .child(outline)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_split_on_plus() {
        assert_eq!(
            split_keys("\u{2318}+\u{21e7}+Y"),
            vec!["\u{2318}", "\u{21e7}", "Y"]
        );
        assert_eq!(split_keys("\u{2325}+S"), vec!["\u{2325}", "S"]);
    }

    #[test]
    fn theme_caption_joins_mode_and_author() {
        assert_eq!(theme_caption("Dark", Some("Carbon")), "Dark \u{b7} Carbon");
        assert_eq!(theme_caption("Light", None), "Light");
    }

    #[test]
    fn keybinding_columns_follow_the_grid() {
        // `grid-template-columns: 1fr 9rem 12rem 1.5rem 2rem`.
        assert_eq!(KEYBINDING_COLUMNS, [144., 192., 24., 32.]);
    }

    #[test]
    fn nav_badge_tones_follow_carbon_contrast_rules() {
        for theme in crate::CarbonTheme::ALL {
            crate::theme::tests_set(theme);
            assert_eq!(
                SettingsBadgeTone::Warning.colors(),
                (color(ColorRole::SupportWarning), crate::t3::base::ink()),
            );
            assert_eq!(
                SettingsBadgeTone::Error.colors(),
                (
                    color(ColorRole::SupportError),
                    color(ColorRole::TextOnColor)
                ),
            );
            assert_eq!(
                SettingsBadgeTone::Neutral.colors(),
                (
                    color(ColorRole::TagBackgroundGray),
                    color(ColorRole::TagColorGray),
                ),
            );
        }
        let warn = SettingsNavItem::new("agents", "Agents").warn_badge("2");
        assert_eq!(warn.badge.as_deref(), Some("2"));
        assert_eq!(warn.badge_tone, SettingsBadgeTone::Warning);
        let err = SettingsNavItem::new("diag", "Diagnostics").error_badge("!");
        assert_eq!(err.badge_tone, SettingsBadgeTone::Error);
    }
}
