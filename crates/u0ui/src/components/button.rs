//! Button: shadcn `Button` → Carbon `Button` / `IconButton` / `Link`.
//!
//! The reference component: every later component copies this layout.
//!
//! - **Behavior** is Kit's: [`gpui_kit::base::Button`] owns focus (tab stop),
//!   Enter/Space and pointer activation, the disabled state and accessibility.
//! - **Appearance** is Carbon's, computed per frame from a small [`Look`]
//!   (pure function of kind × state × theme, unit-tested against `bundle.css`)
//!   and [`Geometry`] (pure function of size × density × icon placement).
//! - **Motion**: background, border and text colors animate between states with
//!   [`Motion::HOVER`] (`.cds--btn { transition: background 70ms cubic-bezier(0, 0, 0.38, 0.9) }`).
//!
//! ```ignore
//! Button::new("create", "Create project")
//!     .icon(CarbonIcon::Add)
//!     .on_click(|_, _, _| println!("clicked"))
//! Button::new("delete", "Delete").variant(ButtonVariant::Destructive).size(ButtonSize::Md)
//! Button::new("add", "Add item").icon(CarbonIcon::Add).icon_only(true)
//! ```
use crate::{
    density::{self, Density},
    icons::{Icon, IconSize},
    motion::{self, Motion},
    theme::color,
    tokens::{ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    AnyElement, App, ClickEvent, CursorStyle, ElementId, FocusHandle, Hsla, InteractiveElement,
    IntoElement, MouseButton, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement,
    Styled, Window, div, prelude::FluentBuilder as _, px,
};
use std::{rc::Rc, time::Duration};

/// The variants of `index.d.ts` `ButtonProps.variant`: shadcn names plus Carbon's.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ButtonVariant {
    /// shadcn `default` = Carbon `primary`.
    #[default]
    Default,
    /// Carbon `primary`.
    Primary,
    /// `secondary`.
    Secondary,
    /// shadcn `outline` = Carbon `tertiary`.
    Outline,
    /// Carbon `tertiary`.
    Tertiary,
    /// `ghost`.
    Ghost,
    /// shadcn `destructive` = Carbon `danger`.
    Destructive,
    /// Carbon `danger`.
    Danger,
    /// Carbon `danger-tertiary`.
    DangerTertiary,
    /// Carbon `danger-ghost`.
    DangerGhost,
    /// shadcn `link` = Carbon `Link`.
    Link,
}

impl ButtonVariant {
    /// Every variant, in `index.d.ts` order.
    pub const ALL: [ButtonVariant; 11] = [
        Self::Default,
        Self::Primary,
        Self::Secondary,
        Self::Outline,
        Self::Tertiary,
        Self::Ghost,
        Self::Destructive,
        Self::Danger,
        Self::DangerTertiary,
        Self::DangerGhost,
        Self::Link,
    ];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Primary => "primary",
            Self::Secondary => "secondary",
            Self::Outline => "outline",
            Self::Tertiary => "tertiary",
            Self::Ghost => "ghost",
            Self::Destructive => "destructive",
            Self::Danger => "danger",
            Self::DangerTertiary => "danger-tertiary",
            Self::DangerGhost => "danger-ghost",
            Self::Link => "link",
        }
    }

    /// The Carbon kind this variant draws (`KIND` in the reference `index.jsx`).
    pub const fn kind(self) -> ButtonKind {
        match self {
            Self::Default | Self::Primary => ButtonKind::Primary,
            Self::Secondary => ButtonKind::Secondary,
            Self::Outline | Self::Tertiary => ButtonKind::Tertiary,
            Self::Ghost => ButtonKind::Ghost,
            Self::Destructive | Self::Danger => ButtonKind::Danger,
            Self::DangerTertiary => ButtonKind::DangerTertiary,
            Self::DangerGhost => ButtonKind::DangerGhost,
            Self::Link => ButtonKind::Link,
        }
    }
}

/// Carbon button kinds (`.cds--btn--<kind>`), plus `Link` (`.cds--link`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ButtonKind {
    /// `.cds--btn--primary`
    Primary,
    /// `.cds--btn--secondary`
    Secondary,
    /// `.cds--btn--tertiary`
    Tertiary,
    /// `.cds--btn--ghost`
    Ghost,
    /// `.cds--btn--danger`
    Danger,
    /// `.cds--btn--danger--tertiary`
    DangerTertiary,
    /// `.cds--btn--danger--ghost`
    DangerGhost,
    /// `.cds--link`
    Link,
}

impl ButtonKind {
    /// The seven Carbon button kinds (without `Link`).
    pub const BUTTONS: [ButtonKind; 7] = [
        Self::Primary,
        Self::Secondary,
        Self::Tertiary,
        Self::Ghost,
        Self::Danger,
        Self::DangerTertiary,
        Self::DangerGhost,
    ];

    /// Filled kinds get the 1px `focus-inset` line inside the focus ring.
    const fn is_ghost(self) -> bool {
        matches!(self, Self::Ghost | Self::DangerGhost)
    }
}

/// `index.d.ts` `ButtonProps.size`; heights from `.cds--layout--size-*`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ButtonSize {
    /// `sm`: 32px (`size-sm`).
    Sm,
    /// `md`: 40px (`size-md`).
    Md,
    /// `lg`: 48px (`size-lg`), the default.
    #[default]
    Lg,
    /// `xl`: 64px (`size-xl`).
    Xl,
    /// `2xl`: 80px (`size-2xl`).
    Xxl,
}

impl ButtonSize {
    /// Every size.
    pub const ALL: [ButtonSize; 5] = [Self::Sm, Self::Md, Self::Lg, Self::Xl, Self::Xxl];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
            Self::Xl => "xl",
            Self::Xxl => "2xl",
        }
    }

    /// Carbon height in px (`--layout-size-height-*`).
    pub const fn carbon_height(self) -> f32 {
        use crate::tokens::{SIZE_2XL, SIZE_LG, SIZE_MD, SIZE_SM, SIZE_XL};
        match self {
            Self::Sm => SIZE_SM.px_value(),
            Self::Md => SIZE_MD.px_value(),
            Self::Lg => SIZE_LG.px_value(),
            Self::Xl => SIZE_XL.px_value(),
            Self::Xxl => SIZE_2XL.px_value(),
        }
    }

    /// Height under `density`: compact shrinks every size but sm/xl/2xl to
    /// `density-field-md` (`[data-density="compact"] .cds--btn:not(.cds--btn--sm):not(.cds--btn--xl):not(.cds--btn--2xl)`).
    pub const fn height(self, density: Density) -> f32 {
        match (density, self) {
            (Density::Compact, Self::Md | Self::Lg) => density_field_md(),
            _ => self.carbon_height(),
        }
    }
}

const fn density_field_md() -> f32 {
    crate::tokens::DENSITY_FIELD_MD.px_value()
}

/// The interaction state a look is computed for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ButtonState {
    /// Pointer over the button.
    pub hovered: bool,
    /// Pointer pressed on the button (`:active`).
    pub pressed: bool,
    /// Keyboard focus is on the button (`:focus`).
    pub focused: bool,
    /// Disabled (or loading).
    pub disabled: bool,
}

/// The colors a button paints in one state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Look {
    /// Fill.
    pub background: Hsla,
    /// 1px border (`border-width: 1px` on every kind).
    pub border: Hsla,
    /// Label color.
    pub text: Hsla,
    /// Icon color.
    pub icon: Hsla,
    /// Inner 1px focus line (`box-shadow: inset 0 0 0 1px focus`), if focused.
    pub focus_ring: Option<Hsla>,
    /// Innermost 1px line (`inset 0 0 0 2px background`; the `focus-inset` token).
    pub focus_inset: Option<Hsla>,
    /// Underline the label (Link hover/focus).
    pub underline: bool,
}

impl Look {
    /// The Carbon look of `kind` in `state` on the active theme. Each value
    /// records the `bundle.css` rule it comes from.
    pub fn of(kind: ButtonKind, state: ButtonState, icon_only: bool) -> Look {
        use ColorRole::*;
        let clear = Hsla::transparent_black();
        let c = color;
        let mut look = match kind {
            // `.cds--btn--primary` etc.: filled, transparent border, text-on-color.
            ButtonKind::Primary | ButtonKind::Secondary | ButtonKind::Danger => {
                let (rest, hover, active) = match kind {
                    ButtonKind::Primary => (ButtonPrimary, ButtonPrimaryHover, ButtonPrimaryActive),
                    ButtonKind::Secondary => {
                        (ButtonSecondary, ButtonSecondaryHover, ButtonSecondaryActive)
                    }
                    _ => (ButtonDangerPrimary, ButtonDangerHover, ButtonDangerActive),
                };
                let background = if state.pressed {
                    c(active)
                } else if state.hovered {
                    c(hover)
                } else {
                    c(rest)
                };
                Look::plain(background, clear, c(TextOnColor))
            }
            // `.cds--btn--tertiary`: outlined in button-tertiary; hover fills
            // button-tertiary-hover; focus fills button-tertiary; active fills
            // button-tertiary-active with a transparent border; text-inverse on fills.
            ButtonKind::Tertiary => {
                if state.pressed {
                    Look::plain(c(ButtonTertiaryActive), clear, c(TextInverse))
                } else if state.focused {
                    Look::plain(c(ButtonTertiary), c(ButtonTertiary), c(TextInverse))
                } else if state.hovered {
                    Look::plain(c(ButtonTertiaryHover), c(ButtonTertiary), c(TextInverse))
                } else {
                    Look::plain(clear, c(ButtonTertiary), c(ButtonTertiary))
                }
            }
            // `.cds--btn--danger--tertiary`.
            ButtonKind::DangerTertiary => {
                if state.pressed {
                    Look::plain(c(ButtonDangerActive), c(ButtonDangerActive), c(TextOnColor))
                } else if state.focused {
                    Look::plain(
                        c(ButtonDangerPrimary),
                        c(ButtonDangerSecondary),
                        c(TextOnColor),
                    )
                } else if state.hovered {
                    Look::plain(c(ButtonDangerHover), c(ButtonDangerHover), c(TextOnColor))
                } else {
                    Look::plain(clear, c(ButtonDangerSecondary), c(ButtonDangerSecondary))
                }
            }
            // `.cds--btn--ghost`: link-primary text, background-hover/active
            // washes, link-primary-hover text on hover/press; the icon is
            // icon-primary (`.cds--btn--ghost:not([disabled]) svg`).
            ButtonKind::Ghost => {
                let mut look = if state.pressed {
                    Look::plain(c(BackgroundActive), clear, c(LinkPrimaryHover))
                } else if state.hovered {
                    Look::plain(c(BackgroundHover), clear, c(LinkPrimaryHover))
                } else {
                    Look::plain(clear, clear, c(LinkPrimary))
                };
                look.icon = c(IconPrimary);
                look
            }
            // `.cds--btn--danger--ghost`.
            ButtonKind::DangerGhost => {
                if state.pressed {
                    Look::plain(c(ButtonDangerActive), clear, c(TextOnColor))
                } else if state.hovered {
                    Look::plain(c(ButtonDangerHover), clear, c(TextOnColor))
                } else {
                    Look::plain(clear, clear, c(ButtonDangerSecondary))
                }
            }
            // `.cds--link`: link-primary; hover link-primary-hover + underline;
            // focus 1px focus outline + underline.
            ButtonKind::Link => {
                let text = if state.hovered && !state.pressed {
                    c(LinkPrimaryHover)
                } else {
                    c(LinkPrimary)
                };
                let mut look = Look::plain(clear, clear, text);
                look.underline = state.hovered || state.focused || state.pressed;
                if state.focused || state.pressed {
                    look.border = c(Focus);
                }
                look
            }
        };

        if state.disabled {
            // `.cds--btn:disabled`: button-disabled fill and border,
            // text-on-color-disabled; tertiary/ghost/link variants override.
            look = match kind {
                ButtonKind::Tertiary | ButtonKind::DangerTertiary => {
                    Look::plain(clear, c(ButtonDisabled), c(TextDisabled))
                }
                ButtonKind::Ghost | ButtonKind::DangerGhost | ButtonKind::Link => {
                    let mut look = Look::plain(clear, clear, c(TextDisabled));
                    if icon_only {
                        // `.cds--btn--ghost.cds--btn--icon-only[disabled] .cds--btn__icon`.
                        look.icon = c(IconOnColorDisabled);
                    }
                    look
                }
                _ => Look::plain(c(ButtonDisabled), c(ButtonDisabled), c(TextOnColorDisabled)),
            };
            return look;
        }

        if state.focused && kind != ButtonKind::Link {
            // `.cds--btn--<kind>:focus { border-color: focus; box-shadow: inset 0 0 0 1px
            // focus, inset 0 0 0 2px background }`; icon-only ghost drops the second line.
            look.border = c(Focus);
            look.focus_ring = Some(c(Focus));
            if !(icon_only && kind == ButtonKind::Ghost) {
                look.focus_inset = Some(c(FocusInset));
            }
        }
        look
    }

    fn plain(background: Hsla, border: Hsla, text: Hsla) -> Look {
        Look {
            background,
            border,
            text,
            icon: text,
            focus_ring: None,
            focus_inset: None,
            underline: false,
        }
    }
}

/// Box geometry in px at 100% zoom (converted to rems when drawn).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Geometry {
    /// Border-box height (`min-block-size: --layout-size-height-local`).
    pub height: f32,
    /// Fixed width (icon-only squares), else `None` (content width).
    pub width: Option<f32>,
    /// `padding-block-start`: `min((height − 1lh) / 2 − 1px, (48 − 18) / 2 − 1)`.
    pub padding_top: f32,
    /// `padding-inline-start`: `spacing-05 − 1px` = 15px.
    pub padding_left: f32,
    /// `padding-inline-end`: `3 × 16 + 16 − 1` = 63px, ghost 15px.
    pub padding_right: f32,
    /// Absolute icon offset from the padding box top (`inset-block-start` + margin),
    /// `None` when the icon flows inline (ghost, icon-only).
    pub icon_top: Option<f32>,
    /// Absolute icon offset from the padding box right (`inset-inline-end: 16px`).
    pub icon_right: f32,
}

/// body-compact-01 line box: 14px × 1.28572 = 18px.
const LINE: f32 = 18.;
/// `--layout-density-padding-inline-local`: spacing-05.
const PAD: f32 = 16.;
/// `--temp-padding-block-max`: (size-lg − 1lh) / 2 − 1px.
const PAD_BLOCK_MAX: f32 = (48. - LINE) / 2. - 1.;

impl Geometry {
    /// Geometry for `size` under `density`.
    pub fn of(size: ButtonSize, density: Density, kind: ButtonKind, icon_only: bool) -> Self {
        let height = size.height(density);
        if icon_only {
            // `.cds--btn--icon-only { padding: 0; inline-size: block-size }`.
            return Self {
                height,
                width: Some(height),
                padding_top: 0.,
                padding_left: 0.,
                padding_right: 0.,
                icon_top: None,
                icon_right: 0.,
            };
        }
        if kind == ButtonKind::Link {
            return Self {
                height: LINE,
                width: None,
                padding_top: 0.,
                padding_left: 0.,
                padding_right: 0.,
                icon_top: None,
                icon_right: 0.,
            };
        }
        let padding_top = ((height - LINE) / 2. - 1.).min(PAD_BLOCK_MAX);
        // `.cds--btn--sm/md:not(.cds--btn--icon-only) .cds--btn__icon { margin-block-start: 0 }`.
        let icon_margin = if matches!(size, ButtonSize::Sm | ButtonSize::Md) {
            0.
        } else {
            1.
        };
        Self {
            height,
            width: None,
            padding_top,
            padding_left: PAD - 1.,
            padding_right: if kind.is_ghost() {
                PAD - 1.
            } else {
                PAD * 3. + 16. - 1.
            },
            icon_top: (!kind.is_ghost())
                .then(|| ((height - 16.) / 2. - 1.).min(PAD_BLOCK_MAX) + icon_margin),
            icon_right: PAD,
        }
    }
}

type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// A Carbon button. See the module docs.
#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    label: SharedString,
    variant: ButtonVariant,
    size: ButtonSize,
    density: Option<Density>,
    icon: Option<Icon>,
    icon_only: bool,
    disabled: bool,
    loading: bool,
    selected: bool,
    full_width: bool,
    tab_index: isize,
    focus_handle: Option<FocusHandle>,
    on_click: Option<ClickHandler>,
}

impl Button {
    /// A button with a stable `id` and a `label` (the visible text, or the
    /// accessible name and tooltip when [`Self::icon_only`]).
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            variant: ButtonVariant::Default,
            size: ButtonSize::Lg,
            density: None,
            icon: None,
            icon_only: false,
            disabled: false,
            loading: false,
            selected: false,
            full_width: false,
            tab_index: 0,
            focus_handle: None,
            on_click: None,
        }
    }

    /// The variant (shadcn or Carbon name).
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// The size (default `lg`).
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    /// Override the app density for this button.
    pub fn density(mut self, density: Density) -> Self {
        self.density = Some(density);
        self
    }

    /// An icon, drawn at the right edge (`.cds--btn__icon`).
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Draw only the icon in a square button; the label becomes the tooltip and
    /// accessible name. A `default` variant icon-only button is ghost
    /// (`index.jsx`: `kind = variant === 'default' ? 'ghost' : KIND[variant]`).
    pub fn icon_only(mut self, icon_only: bool) -> Self {
        self.icon_only = icon_only;
        self
    }

    /// Disable activation (`cursor: not-allowed`, disabled colors).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Show a spinner in the icon slot and ignore activation.
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    /// Selected icon button (`.cds--btn--icon-only.cds--btn--selected`:
    /// `background-selected`).
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Fill the parent's width instead of hugging the label (`max-inline-size:
    /// none`, as in `.cds--modal-footer .cds--btn`).
    pub fn full_width(mut self, full_width: bool) -> Self {
        self.full_width = full_width;
        self
    }

    /// Focus traversal index (default 0).
    pub fn tab_index(mut self, tab_index: isize) -> Self {
        self.tab_index = tab_index;
        self
    }

    /// Use a caller-owned focus handle (to focus the button programmatically).
    pub fn track_focus(mut self, handle: &FocusHandle) -> Self {
        self.focus_handle = Some(handle.clone());
        self
    }

    /// Activation handler (pointer, Enter, Space).
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    /// Whether the button is disabled.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// The Carbon kind actually drawn.
    pub fn kind(&self) -> ButtonKind {
        if self.icon_only && self.variant == ButtonVariant::Default {
            ButtonKind::Ghost
        } else {
            self.variant.kind()
        }
    }
}

/// Pointer state kept across frames for one button.
#[derive(Default)]
struct Pointer {
    hovered: bool,
    pressed: bool,
}

fn rem(px_value: f32) -> gpui_kit::Rems {
    gpui_kit::rems(px_value / crate::tokens::REM_PX)
}

impl RenderOnce for Button {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let kind = self.kind();
        let density = self.density.unwrap_or_else(density::current);
        let geometry = Geometry::of(self.size, density, kind, self.icon_only);
        let id = self.id.clone();
        let pointer =
            window.use_keyed_state((id.clone(), "pointer"), cx, |_, _| Pointer::default());
        let focus = self.focus_handle.clone().unwrap_or_else(|| {
            window
                .use_keyed_state((id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
                .read(cx)
                .clone()
        });
        let inert = self.disabled || self.loading;
        let state = {
            let p = pointer.read(cx);
            ButtonState {
                hovered: p.hovered && !inert,
                pressed: p.pressed && !inert,
                focused: focus.is_focused(window),
                disabled: self.disabled,
            }
        };
        let mut look = Look::of(kind, state, self.icon_only);
        if self.selected && self.icon_only && !self.disabled && !state.hovered && !state.pressed {
            look.background = color(ColorRole::BackgroundSelected);
        }
        let animate = |channel: &'static str, target: Hsla, window: &mut Window, cx: &mut App| {
            motion::animate_color((id.clone(), channel), target, Motion::HOVER, window, cx)
        };
        let background = animate("bg", look.background, window, cx);
        let border = animate("border", look.border, window, cx);
        let text = animate("text", look.text, window, cx);
        let icon_color = animate("icon", look.icon, window, cx);

        let icon = if self.loading {
            Some(
                crate::icons::spin(
                    Icon::named("progress_activity")
                        .size(IconSize::S16)
                        .color(icon_color),
                    (id.clone(), "spinner"),
                    Duration::from_millis(690),
                    cx,
                )
                .into_any_element(),
            )
        } else {
            self.icon.map(|icon| {
                icon.size(IconSize::S16)
                    .color(icon_color)
                    .into_any_element()
            })
        };

        let on_click = self.on_click.clone();
        let label = self.label.clone();
        let mut button = gpui_kit::base::Button::new(id.clone())
            .track_focus(&focus)
            .tab_index(self.tab_index)
            .disabled(inert)
            .accessibility_label(label.clone())
            .relative()
            .flex()
            .flex_row()
            .flex_shrink_0()
            .whitespace_nowrap()
            .type_style(TypeStyle::BodyCompact01)
            .text_color(text)
            .bg(background)
            .border_1()
            .border_color(border)
            .cursor(if self.disabled {
                CursorStyle::OperationNotAllowed
            } else if self.loading {
                CursorStyle::Arrow
            } else {
                CursorStyle::PointingHand
            })
            .on_hover({
                let pointer = pointer.clone();
                move |hovered, _, cx| {
                    pointer.update(cx, |p, cx| {
                        if p.hovered != *hovered {
                            p.hovered = *hovered;
                            if !hovered {
                                p.pressed = false;
                            }
                            cx.notify();
                        }
                    })
                }
            })
            .on_mouse_down(MouseButton::Left, {
                let pointer = pointer.clone();
                move |_, _, cx| {
                    pointer.update(cx, |p, cx| {
                        p.pressed = true;
                        cx.notify();
                    })
                }
            })
            .on_mouse_up(MouseButton::Left, {
                let pointer = pointer.clone();
                move |_, _, cx| {
                    pointer.update(cx, |p, cx| {
                        p.pressed = false;
                        cx.notify();
                    })
                }
            })
            .on_mouse_up_out(MouseButton::Left, {
                let pointer = pointer.clone();
                move |_, _, cx| {
                    pointer.update(cx, |p, cx| {
                        if p.pressed {
                            p.pressed = false;
                            cx.notify();
                        }
                    })
                }
            })
            .when_some(on_click, |this, handler| {
                this.on_click(move |event, window, cx| handler(event, window, cx))
            });

        if kind == ButtonKind::Link {
            // `.cds--link`: inline text, icon after it, no box.
            button = button
                .items_center()
                .gap(rem(8.))
                .border_color(Hsla::transparent_black())
                .when(look.underline, |this| this.underline())
                .child(label.clone())
                .when_some(icon, |this, icon| this.child(icon));
            // Focus: 1px `focus` outline around the text (`.cds--link:focus`).
            return button
                .when(state.focused || state.pressed, |this| {
                    this.border_color(border)
                })
                .into_any_element();
        }

        button = button.h(rem(geometry.height));
        if let Some(width) = geometry.width {
            button = button
                .w(rem(width))
                .items_center()
                .justify_center()
                .when_some(icon, |this, icon| this.child(icon));
        } else {
            button = button
                .map(|this| {
                    if self.full_width {
                        this.w_full()
                    } else {
                        this.max_w(rem(320.))
                    }
                })
                // Base centers its content; Carbon's label starts at the padding box.
                .justify_start()
                .items_start()
                .pt(rem(geometry.padding_top))
                .pl(rem(geometry.padding_left))
                .pr(rem(geometry.padding_right))
                .child(label.clone());
            button = match (icon, geometry.icon_top) {
                // `.cds--btn .cds--btn__icon { position: absolute; inset-inline-end: 16px }`.
                (Some(icon), Some(top)) => button.child(
                    div()
                        .absolute()
                        .top(rem(top))
                        .right(rem(geometry.icon_right))
                        .size(rem(16.))
                        .child(icon),
                ),
                // `.cds--btn--ghost .cds--btn__icon { position: static; align-self: center; margin-inline-start: 0.5rem }`.
                (Some(icon), None) => button.child(
                    div()
                        .self_center()
                        .ml(rem(8.))
                        .mt(rem(-geometry.padding_top))
                        .size(rem(16.))
                        .child(icon),
                ),
                (None, _) => button,
            };
        }
        if self.icon_only {
            button = button.tooltip({
                let label = label.clone();
                move |window, cx| {
                    gpui_kit::component::tooltip::Tooltip::new(label.clone()).build(window, cx)
                }
            });
        }

        // Focus: inset 1px `focus`, then 1px `focus-inset` (box-shadow has no
        // inset form in GPUI, so two absolutely positioned outlines).
        let focus_layers: Vec<AnyElement> = [
            look.focus_ring.map(|c| (0., c)),
            look.focus_inset.map(|c| (1., c)),
        ]
        .into_iter()
        .flatten()
        .map(|(inset, c)| {
            div()
                .absolute()
                .top(px(inset))
                .left(px(inset))
                .right(px(inset))
                .bottom(px(inset))
                .border_1()
                .border_color(c)
                .into_any_element()
        })
        .collect();
        button.children(focus_layers).into_any_element()
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
    fn variants_map_to_carbon_kinds() {
        assert_eq!(ButtonVariant::Default.kind(), ButtonKind::Primary);
        assert_eq!(ButtonVariant::Outline.kind(), ButtonKind::Tertiary);
        assert_eq!(ButtonVariant::Destructive.kind(), ButtonKind::Danger);
        assert_eq!(ButtonVariant::Link.kind(), ButtonKind::Link);
        let icon = Button::new("x", "Add").icon_only(true);
        assert_eq!(icon.kind(), ButtonKind::Ghost);
    }

    #[test]
    fn primary_state_colors_follow_bundle_css() {
        crate::theme::tests_set(CarbonTheme::White);
        let rest = Look::of(ButtonKind::Primary, ButtonState::default(), false);
        assert_eq!(hex(rest.background), "#0f62fe");
        assert_eq!(hex(rest.text), "#ffffff");
        assert_eq!(rest.border.a, 0.);
        let hover = Look::of(
            ButtonKind::Primary,
            ButtonState {
                hovered: true,
                ..Default::default()
            },
            false,
        );
        assert_eq!(hex(hover.background), "#0050e6");
        let active = Look::of(
            ButtonKind::Primary,
            ButtonState {
                hovered: true,
                pressed: true,
                ..Default::default()
            },
            false,
        );
        assert_eq!(hex(active.background), "#002d9c");
        let focus = Look::of(
            ButtonKind::Primary,
            ButtonState {
                focused: true,
                ..Default::default()
            },
            false,
        );
        assert_eq!(hex(focus.border), "#0f62fe");
        assert_eq!(focus.focus_ring.map(hex).as_deref(), Some("#0f62fe"));
        assert_eq!(focus.focus_inset.map(hex).as_deref(), Some("#ffffff"));
        let disabled = Look::of(
            ButtonKind::Primary,
            ButtonState {
                disabled: true,
                focused: true,
                ..Default::default()
            },
            false,
        );
        assert_eq!(hex(disabled.background), "#c6c6c6");
        assert_eq!(hex(disabled.text), "#8d8d8d");
        assert!(disabled.focus_ring.is_none());
    }

    #[test]
    fn tertiary_and_ghost_states() {
        crate::theme::tests_set(CarbonTheme::White);
        let rest = Look::of(ButtonKind::Tertiary, ButtonState::default(), false);
        assert_eq!(rest.background.a, 0.);
        assert_eq!(hex(rest.border), "#0f62fe");
        assert_eq!(hex(rest.text), "#0f62fe");
        let hover = Look::of(
            ButtonKind::Tertiary,
            ButtonState {
                hovered: true,
                ..Default::default()
            },
            false,
        );
        assert_eq!(hex(hover.text), "#ffffff");
        let ghost = Look::of(ButtonKind::Ghost, ButtonState::default(), false);
        assert_eq!(hex(ghost.text), "#0f62fe");
        assert_eq!(hex(ghost.icon), "#161616");
        let focused_icon_ghost = Look::of(
            ButtonKind::Ghost,
            ButtonState {
                focused: true,
                ..Default::default()
            },
            true,
        );
        assert!(focused_icon_ghost.focus_ring.is_some());
        assert!(focused_icon_ghost.focus_inset.is_none());
    }

    #[test]
    fn geometry_follows_bundle_css() {
        let lg = Geometry::of(
            ButtonSize::Lg,
            Density::Comfortable,
            ButtonKind::Primary,
            false,
        );
        assert_eq!(lg.height, 48.);
        assert_eq!(
            (lg.padding_top, lg.padding_left, lg.padding_right),
            (14., 15., 63.)
        );
        assert_eq!(lg.icon_top, Some(15.));
        let sm = Geometry::of(
            ButtonSize::Sm,
            Density::Comfortable,
            ButtonKind::Primary,
            false,
        );
        assert_eq!(
            (sm.height, sm.padding_top, sm.icon_top),
            (32., 6., Some(7.))
        );
        let xl = Geometry::of(
            ButtonSize::Xl,
            Density::Comfortable,
            ButtonKind::Primary,
            false,
        );
        assert_eq!((xl.height, xl.padding_top), (64., 14.));
        let ghost = Geometry::of(
            ButtonSize::Lg,
            Density::Comfortable,
            ButtonKind::Ghost,
            false,
        );
        assert_eq!((ghost.padding_right, ghost.icon_top), (15., None));
        let compact = Geometry::of(ButtonSize::Lg, Density::Compact, ButtonKind::Primary, false);
        assert_eq!((compact.height, compact.padding_top), (32., 6.));
        let compact_xl = Geometry::of(ButtonSize::Xl, Density::Compact, ButtonKind::Primary, false);
        assert_eq!(compact_xl.height, 64.);
        let icon = Geometry::of(
            ButtonSize::Md,
            Density::Comfortable,
            ButtonKind::Ghost,
            true,
        );
        assert_eq!((icon.height, icon.width), (40., Some(40.)));
    }
}
