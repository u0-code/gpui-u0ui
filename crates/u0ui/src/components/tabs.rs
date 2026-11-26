//! Tabs: shadcn `Tabs` (by `value`) → Carbon `Tabs` / `TabList` / `Tab` /
//! `TabPanels` (`.cds--tabs`, `.cds--tab-content`).
//!
//! - **Behavior** is Kit's: [`gpui_kit::base::Tabs`] / [`gpui_kit::base::Tab`]
//!   implement the ARIA tabs pattern (one tab stop, ←/→ and Home/End move
//!   focus, Enter/Space activate, disabled tabs skipped). Activation is
//!   automatic by default, as Carbon's `Tabs` (`activation="automatic"`):
//!   ←/→/Home/End also select the tab they move to; [`TabsActivation::Manual`]
//!   keeps Kit's focus-only arrows. Controlled or not.
//! - **Appearance** ([`TabLook`]): `line` tabs are `size-md` tall with a 2px
//!   bottom indicator (`border-subtle`, hover `border-strong`, selected
//!   `border-interactive`), `text-secondary` labels that turn `text-primary`
//!   on hover and heading-compact-01 when selected, 1px apart. `contained`
//!   tabs are `size-lg` on `layer-accent` with a 1px `border-strong` divider,
//!   `layer-accent-hover` on hover, and the selected tab on `layer` with a 2px
//!   `border-interactive` top line; `full_width` splits the row equally.
//!   Disabled: `text-disabled` (contained: `button-disabled` fill,
//!   `text-on-color-disabled`). Keyboard focus: a 2px inset `focus` outline.
//!   The panel is padded 1rem (`.cds--tab-content`), on `layer` when contained.
//! - **Motion**: `transition: color 70ms, border 70ms, background-color 70ms`
//!   (productive standard) on every tab.
use super::overlay::rem;
use crate::{
    icons::{Icon, IconSize},
    motion::{self, Curve, Motion, Speed},
    theme::color,
    tokens::{ColorRole, SIZE_LG, SIZE_MD, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    AnyElement, App, CursorStyle, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _, px,
};
use std::rc::Rc;

/// `.cds--tabs__nav-item { transition: background-color 70ms cubic-bezier(0.2, 0, 0.38, 0.9) }`.
pub const TAB_MOTION: Motion = Motion::new(Curve::ProductiveStandard, Speed::Fast01);

/// `index.d.ts` `TabsProps.variant`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TabsVariant {
    /// `line` (default).
    #[default]
    Line,
    /// `contained`.
    Contained,
}

impl TabsVariant {
    /// Both variants.
    pub const ALL: [TabsVariant; 2] = [Self::Line, Self::Contained];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Line => "line",
            Self::Contained => "contained",
        }
    }

    /// Tab height in px: `--layout-size-height-local` is md for line tabs and
    /// lg for contained ones.
    pub const fn height(self) -> f32 {
        match self {
            Self::Line => SIZE_MD.px_value(),
            Self::Contained => SIZE_LG.px_value(),
        }
    }
}

/// Carbon `Tabs` `activation`: whether moving focus with the arrow keys
/// selects the tab.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TabsActivation {
    /// Arrows, Home and End select the tab they move to (Carbon's default).
    #[default]
    Automatic,
    /// Arrows only move focus; Enter or Space selects.
    Manual,
}

impl TabsActivation {
    /// Both modes.
    pub const ALL: [TabsActivation; 2] = [Self::Automatic, Self::Manual];

    /// Carbon's name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Automatic => "automatic",
            Self::Manual => "manual",
        }
    }
}

/// The tab an arrow/Home/End key moves to from `from` in automatic
/// activation, skipping disabled tabs and wrapping (Kit's focus rule).
pub fn automatic_target(key: &str, from: usize, disabled: &[bool]) -> Option<usize> {
    let n = disabled.len();
    let enabled = |i: &usize| !disabled[*i];
    match key {
        "home" => (0..n).find(enabled),
        "end" => (0..n).rev().find(enabled),
        "right" | "left" => {
            let step = if key == "right" { 1 } else { n - 1 };
            (1..n).map(|k| (from + k * step) % n).find(enabled)
        }
        _ => None,
    }
}

/// The state a tab is drawn in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TabState {
    /// The selected tab.
    pub selected: bool,
    /// Pointer over it.
    pub hovered: bool,
    /// Disabled.
    pub disabled: bool,
}

/// The colors of one tab.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TabLook {
    /// Tab fill.
    pub background: Hsla,
    /// Label color.
    pub text: Hsla,
    /// The 2px indicator: bottom (line) or top (contained, selected only).
    pub indicator: Hsla,
    /// Contained tabs' 1px left divider (`box-shadow: -1px 0 0 0 border-strong`).
    pub divider: Hsla,
}

impl TabLook {
    /// The look of a tab of `variant` in `state` on the active theme.
    pub fn of(variant: TabsVariant, state: TabState) -> Self {
        use ColorRole::*;
        let clear = Hsla::transparent_black();
        match variant {
            TabsVariant::Line => {
                let (text, indicator) = if state.disabled {
                    // `.cds--tabs__nav-item--disabled`.
                    (color(TextDisabled), color(BorderSubtle01))
                } else if state.selected {
                    // `.cds--tabs__nav-item--selected { border-block-end: 2px solid border-interactive }`.
                    (color(TextPrimary), color(BorderInteractive))
                } else if state.hovered {
                    // `:hover { border-block-end: 2px solid border-strong; color: text-primary }`.
                    (color(TextPrimary), color(BorderStrong01))
                } else {
                    // `.cds--tabs__nav-link { border-block-end: 2px solid border-subtle; color: text-secondary }`.
                    (color(TextSecondary), color(BorderSubtle01))
                };
                Self {
                    background: clear,
                    text,
                    indicator,
                    divider: clear,
                }
            }
            TabsVariant::Contained => {
                let (background, text, indicator) = if state.disabled {
                    (color(ButtonDisabled), color(TextOnColorDisabled), clear)
                } else if state.selected {
                    // `--selected { background-color: var(--layer); box-shadow: inset 0 2px 0 0 border-interactive }`.
                    (color(Layer01), color(TextPrimary), color(BorderInteractive))
                } else if state.hovered {
                    (color(LayerAccentHover01), color(TextPrimary), clear)
                } else {
                    (color(LayerAccent01), color(TextSecondary), clear)
                };
                Self {
                    background,
                    text,
                    indicator,
                    divider: color(BorderStrong01),
                }
            }
        }
    }
}

/// One tab (`TabsTrigger value`).
pub struct TabsTrigger {
    value: SharedString,
    label: SharedString,
    disabled: bool,
    icon: Option<Icon>,
}

impl TabsTrigger {
    /// A tab for `value` showing `label`.
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
            icon: None,
        }
    }

    /// Disable the tab.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// A trailing icon (`renderIcon`).
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

type ValueChange = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;

/// Carbon tabs. See the module docs.
#[derive(IntoElement)]
pub struct Tabs {
    id: ElementId,
    variant: TabsVariant,
    full_width: bool,
    value: Option<SharedString>,
    default_value: Option<SharedString>,
    on_value_change: Option<ValueChange>,
    triggers: Vec<TabsTrigger>,
    contents: Vec<(SharedString, AnyElement)>,
    label: SharedString,
    activation: TabsActivation,
}

impl Tabs {
    /// Tabs with a stable `id`.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            variant: TabsVariant::Line,
            full_width: false,
            value: None,
            default_value: None,
            on_value_change: None,
            triggers: Vec::new(),
            contents: Vec::new(),
            label: SharedString::default(),
            activation: TabsActivation::Automatic,
        }
    }

    /// `line` or `contained`.
    pub fn variant(mut self, variant: TabsVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Contained tabs share the row equally.
    pub fn full_width(mut self, full_width: bool) -> Self {
        self.full_width = full_width;
        self
    }

    /// Controlled selected value.
    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Initially selected value when uncontrolled (default: the first tab).
    pub fn default_value(mut self, value: impl Into<SharedString>) -> Self {
        self.default_value = Some(value.into());
        self
    }

    /// Called when a tab is activated.
    pub fn on_value_change(
        mut self,
        f: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_value_change = Some(Rc::new(f));
        self
    }

    /// Append a tab.
    pub fn trigger(mut self, trigger: TabsTrigger) -> Self {
        self.triggers.push(trigger);
        self
    }

    /// The panel shown for `value` (`TabsContent value`).
    pub fn content(mut self, value: impl Into<SharedString>, content: impl IntoElement) -> Self {
        self.contents
            .push((value.into(), content.into_any_element()));
        self
    }

    /// Automatic (default) or manual activation.
    pub fn activation(mut self, activation: TabsActivation) -> Self {
        self.activation = activation;
        self
    }

    /// The tab list's accessible name.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }
}

impl RenderOnce for Tabs {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let first = self.triggers.first().map(|t| t.value.clone());
        let initial = self.default_value.clone().or(first).unwrap_or_default();
        let internal = window.use_keyed_state((id.clone(), "value"), cx, |_, _| initial);
        let selected = self
            .value
            .clone()
            .unwrap_or_else(|| internal.read(cx).clone());
        let variant = self.variant;
        let height = variant.height();

        let mut list = gpui_kit::base::Tabs::new((id.clone(), "list"))
            .flex()
            .flex_row()
            .when(
                self.full_width && variant == TabsVariant::Contained,
                |this| this.w_full(),
            );
        let count = self.triggers.len();
        let selected_index = self.triggers.iter().position(|t| t.value == selected);
        // Automatic activation: select where the arrow keys take focus. Kit
        // moves focus in its own key handler; this runs in the capture phase
        // with the same rule, from the selected tab (which has focus).
        let automatic = (self.activation == TabsActivation::Automatic).then(|| {
            let values: Vec<SharedString> = self.triggers.iter().map(|t| t.value.clone()).collect();
            let disabled: Vec<bool> = self.triggers.iter().map(|t| t.disabled).collect();
            let internal = internal.clone();
            let on_change = self.on_value_change.clone();
            move |event: &gpui_kit::KeyDownEvent, window: &mut Window, cx: &mut App| {
                if event.keystroke.modifiers.modified() {
                    return;
                }
                let Some(from) = selected_index else {
                    return;
                };
                let rtl = gpui_kit::base::LayoutDirection::global(cx).is_rtl();
                let key = match (event.keystroke.key.as_str(), rtl) {
                    ("left", true) => "right",
                    ("right", true) => "left",
                    (key, _) => key,
                };
                let Some(to) = automatic_target(key, from, &disabled) else {
                    return;
                };
                if to == from {
                    return;
                }
                let value = values[to].clone();
                internal.update(cx, |v, cx| {
                    *v = value.clone();
                    cx.notify();
                });
                if let Some(f) = &on_change {
                    f(value, window, cx);
                }
            }
        });
        for (index, trigger) in self.triggers.into_iter().enumerate() {
            let tab_id = ElementId::NamedChild(
                std::sync::Arc::new(id.clone()),
                SharedString::from(format!("tab-{}", trigger.value)),
            );
            let hover = window.use_keyed_state((tab_id.clone(), "hover"), cx, |_, _| false);
            let state = TabState {
                selected: trigger.value == selected,
                hovered: *hover.read(cx),
                disabled: trigger.disabled,
            };
            let look = TabLook::of(variant, state);
            let animate =
                |channel: &'static str, target: Hsla, window: &mut Window, cx: &mut App| {
                    motion::animate_color((tab_id.clone(), channel), target, TAB_MOTION, window, cx)
                };
            let background = animate("bg", look.background, window, cx);
            let text = animate("text", look.text, window, cx);
            let indicator = animate("indicator", look.indicator, window, cx);
            let value = trigger.value.clone();
            let on_change = self.on_value_change.clone();
            let internal = internal.clone();
            // The divider is hidden on the first tab and next to the selected one.
            let previous_selected = index > 0 && state.selected;
            let tab = gpui_kit::base::Tab::new(tab_id.clone())
                .selected(state.selected)
                .disabled(trigger.disabled)
                .set_position(index + 1, count)
                .accessibility_label(trigger.label.clone())
                .relative()
                .h(rem(height))
                .flex()
                .flex_row()
                .items_center()
                .gap(rem(8.))
                .px(rem(16.))
                // A transparent 2px border reserves the focus outline's room.
                .border_2()
                .border_color(Hsla::transparent_black())
                .focus_visible(|s| s.border_color(color(ColorRole::Focus)))
                .bg(background)
                .text_color(text)
                .whitespace_nowrap()
                .map(|this| {
                    if state.selected {
                        this.type_style(TypeStyle::HeadingCompact01)
                    } else {
                        this.type_style(TypeStyle::BodyCompact01)
                    }
                })
                .cursor(if trigger.disabled {
                    CursorStyle::OperationNotAllowed
                } else {
                    CursorStyle::PointingHand
                })
                .when(variant == TabsVariant::Line && index > 0, |this| {
                    this.ml(px(1.))
                })
                .when(
                    self.full_width && variant == TabsVariant::Contained,
                    |this| this.flex_1().min_w_0(),
                )
                .on_hover(move |hovered, _, cx| {
                    hover.update(cx, |h, cx| {
                        *h = *hovered;
                        cx.notify();
                    })
                })
                .on_click(move |_, window, cx| {
                    internal.update(cx, |v, cx| {
                        *v = value.clone();
                        cx.notify();
                    });
                    if let Some(f) = &on_change {
                        f(value.clone(), window, cx);
                    }
                })
                .child(div().flex_1().min_w_0().child(trigger.label.clone()))
                .when_some(trigger.icon, |this, icon| {
                    this.child(icon.size(IconSize::S16).color(text))
                })
                .child(match variant {
                    // The bottom indicator covers the reserved border.
                    TabsVariant::Line => div()
                        .absolute()
                        .left(px(-2.))
                        .right(px(-2.))
                        .bottom(px(-2.))
                        .h(px(2.))
                        .bg(indicator),
                    TabsVariant::Contained => div()
                        .absolute()
                        .left(px(-2.))
                        .right(px(-2.))
                        .top(px(-2.))
                        .h(px(2.))
                        .bg(indicator),
                })
                .when(
                    variant == TabsVariant::Contained && index > 0 && !previous_selected,
                    |this| {
                        this.child(
                            div()
                                .absolute()
                                .left(px(-3.))
                                .top(px(-2.))
                                .bottom(px(-2.))
                                .w(px(1.))
                                .bg(look.divider),
                        )
                    },
                );
            list = list.child(tab);
        }

        let panel = self
            .contents
            .into_iter()
            .find(|(value, _)| *value == selected)
            .map(|(_, content)| content);
        div()
            .flex()
            .flex_col()
            .w_full()
            .child(
                div()
                    .when_some(automatic, |this, on_key| this.capture_key_down(on_key))
                    .child(list),
            )
            .when_some(panel, |this, panel| {
                this.child(
                    div()
                        .p(rem(16.))
                        .type_style(TypeStyle::Body01)
                        .text_color(color(ColorRole::TextPrimary))
                        .when(variant == TabsVariant::Contained, |this| {
                            this.bg(color(ColorRole::Layer01))
                        })
                        .child(panel),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn automatic_activation_moves_like_kit_focus() {
        let disabled = [false, true, false, false];
        assert_eq!(TabsActivation::default(), TabsActivation::Automatic);
        assert_eq!(automatic_target("right", 0, &disabled), Some(2));
        assert_eq!(automatic_target("right", 3, &disabled), Some(0));
        assert_eq!(automatic_target("left", 0, &disabled), Some(3));
        assert_eq!(automatic_target("left", 2, &disabled), Some(0));
        assert_eq!(automatic_target("home", 3, &disabled), Some(0));
        assert_eq!(automatic_target("end", 0, &disabled), Some(3));
        assert_eq!(automatic_target("enter", 0, &disabled), None);
    }
    use crate::tokens::CarbonTheme;

    fn hex(c: Hsla) -> String {
        let c = c.to_rgb();
        let b = |v: f32| (v * 255.).round() as u8;
        format!("#{:02x}{:02x}{:02x}", b(c.r), b(c.g), b(c.b))
    }

    #[test]
    fn line_tab_states_follow_bundle_css() {
        crate::theme::tests_set(CarbonTheme::White);
        let rest = TabLook::of(TabsVariant::Line, TabState::default());
        assert_eq!(
            (hex(rest.text), hex(rest.indicator)),
            ("#525252".into(), "#c6c6c6".into())
        );
        let hover = TabLook::of(
            TabsVariant::Line,
            TabState {
                hovered: true,
                ..Default::default()
            },
        );
        assert_eq!(
            (hex(hover.text), hex(hover.indicator)),
            ("#161616".into(), "#8d8d8d".into())
        );
        let selected = TabLook::of(
            TabsVariant::Line,
            TabState {
                selected: true,
                ..Default::default()
            },
        );
        assert_eq!(hex(selected.indicator), "#0f62fe");
        assert_eq!(TabsVariant::Line.height(), 40.);
        assert_eq!(TabsVariant::Contained.height(), 48.);
    }

    #[test]
    fn contained_tab_states_follow_bundle_css() {
        crate::theme::tests_set(CarbonTheme::White);
        let rest = TabLook::of(TabsVariant::Contained, TabState::default());
        assert_eq!(hex(rest.background), "#e0e0e0");
        assert_eq!(hex(rest.divider), "#8d8d8d");
        let selected = TabLook::of(
            TabsVariant::Contained,
            TabState {
                selected: true,
                ..Default::default()
            },
        );
        assert_eq!(
            (hex(selected.background), hex(selected.indicator)),
            ("#f4f4f4".into(), "#0f62fe".into())
        );
        let disabled = TabLook::of(
            TabsVariant::Contained,
            TabState {
                disabled: true,
                ..Default::default()
            },
        );
        assert_eq!(
            (hex(disabled.background), hex(disabled.text)),
            ("#c6c6c6".into(), "#8d8d8d".into())
        );
    }
}
