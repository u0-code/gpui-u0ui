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
            // washes, link-primary-hover text on hover/press.
            //
            // The icon of a text + icon ghost follows the text: its paths take
            // `.cds--btn--ghost .cds--btn__icon path { fill: currentColor }`,
            // which beats the `svg` fill they would inherit. An icon-only ghost
            // draws its svg without `.cds--btn__icon`, so its paths inherit
            // `.cds--btn--ghost:not([disabled]) svg { fill: icon-primary }`
            // (reference render: text + icon #0f62fe, icon-only #161616).
            ButtonKind::Ghost => {
                let mut look = if state.pressed {
                    Look::plain(c(BackgroundActive), clear, c(LinkPrimaryHover))
                } else if state.hovered {
                    Look::plain(c(BackgroundHover), clear, c(LinkPrimaryHover))
                } else {
                    Look::plain(clear, clear, c(LinkPrimary))
                };
                if icon_only {
                    look.icon = c(IconPrimary);
                }
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
            look = match kind {
                // `t3.css` global craft pass (gemini-A, "disabled buttons stay
                // legible"): `.cds--btn:disabled:not(.cds--btn--ghost)
                // :not(.cds--btn--icon-only) { background: layer-01; color:
                // text-secondary; box-shadow: inset 0 0 0 1px border-subtle-01;
                // border-color: transparent }`. It outranks the per-kind
                // `:disabled` rules, `.cds--btn--danger--ghost` included (that
                // class is not `.cds--btn--ghost`); the inset line is drawn as
                // the 1px border. Primary, secondary and danger are taken back
                // by the later r2-gal rule below.
                ButtonKind::Tertiary | ButtonKind::DangerTertiary | ButtonKind::DangerGhost
                    if !icon_only =>
                {
                    Look::plain(c(Layer01), c(BorderSubtle01), c(TextSecondary))
                }
                // `.cds--btn--tertiary:disabled`, `.cds--btn--danger--tertiary:disabled`.
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
                // `.cds--btn:disabled`: button-disabled fill and border,
                // text-on-color-disabled. For text buttons this is the r2-gal
                // rule that follows the craft pass in `t3.css`: "disabled filled
                // buttons (primary, secondary, danger) use Carbon's disabled fill
                // so they never read as a tertiary or ghost" (`.cds--btn.cds--btn--
                // primary:disabled:not(.cds--btn--icon-only), … { background:
                // button-disabled; color: text-on-color-disabled; box-shadow:
                // none }`).
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
    /// `padding-inline-end`: `3 × 16 + 16 − 1` = 63px when the button carries
    /// an icon, 16px when it is text-only (see [`TEXT_ONLY_END_PADDING`]),
    /// ghost 15px.
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

/// `padding-inline-end` of a text-only, non-ghost button: `spacing-05`
/// (`t3.css` global craft pass: `.cds--btn:not(.cds--btn--icon-only)
/// :not(.cds--btn--ghost):not(:has(.cds--btn__icon)) { padding-inline-end:
/// var(--spacing-05) }`). Text-only buttons don't reserve Carbon's 63px icon slot.
pub const TEXT_ONLY_END_PADDING: f32 = PAD;

/// One turn of the loading spinner (`cu-spin 0.69s linear infinite`, as
/// `.t3-spin svg`); under reduced motion it pulses instead (see
/// [`crate::icons::spin_now`]).
pub const LOADING_SPIN_PERIOD: Duration = Duration::from_millis(690);

impl Geometry {
    /// Geometry for `size` under `density` of a button that carries an icon
    /// (Carbon's 63px end slot). Use [`Self::for_content`] for the geometry a
    /// [`Button`] actually draws.
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

    /// Geometry of a button with or without an icon: a text-only button that
    /// is neither ghost nor icon-only ends at [`TEXT_ONLY_END_PADDING`]. The
    /// craft-pass rule excludes `.cds--btn--ghost` only, so a text-only
    /// danger ghost takes it too.
    pub fn for_content(
        size: ButtonSize,
        density: Density,
        kind: ButtonKind,
        icon_only: bool,
        has_icon: bool,
    ) -> Self {
        let mut geometry = Self::of(size, density, kind, icon_only);
        if !icon_only && !has_icon && !matches!(kind, ButtonKind::Ghost | ButtonKind::Link) {
            geometry.padding_right = TEXT_ONLY_END_PADDING;
        }
        geometry
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
    icon_color: Option<Hsla>,
    icon_only: bool,
    disabled: bool,
    loading: bool,
    selected: bool,
    full_width: bool,
    wrap_label: bool,
    icon_size: IconSize,
    tooltip: bool,
    tooltip_align: super::floating::Align,
    tab_index: isize,
    focus_handle: Option<FocusHandle>,
    on_click: Option<ClickHandler>,
    trailing: Vec<AnyElement>,
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
            icon_color: None,
            icon_only: false,
            disabled: false,
            loading: false,
            selected: false,
            full_width: false,
            wrap_label: false,
            icon_size: IconSize::S16,
            tooltip: true,
            tooltip_align: super::floating::Align::Top,
            tab_index: 0,
            focus_handle: None,
            on_click: None,
            trailing: Vec::new(),
        }
    }

    /// Does nothing: text-only, non-ghost buttons end at
    /// [`TEXT_ONLY_END_PADDING`] by default. Kept one cycle for downstream
    /// callers; drop the call.
    #[doc(hidden)]
    #[deprecated(note = "no-op: text-only buttons end at TEXT_ONLY_END_PADDING by default")]
    pub fn balanced(self, _balanced: bool) -> Self {
        self
    }

    /// Content after the label inside the button, such as a shortcut key
    /// cap (`<Button>Approve<Kbd>⇧⌘↵</Kbd></Button>`).
    pub fn trailing(mut self, element: impl IntoElement) -> Self {
        self.trailing.push(element.into_any_element());
        self
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

    /// Paint the icon in `ink` while the button is enabled, for a context
    /// rule such as `.g-ptree-row .cds--btn--ghost svg { fill:
    /// var(--icon-secondary) }`. A disabled button keeps its disabled ink.
    pub fn icon_color(mut self, ink: Hsla) -> Self {
        self.icon_color = Some(ink);
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

    /// The icon's size (default 16px). Carbon sizes an `IconButton`'s icon
    /// by the child the caller passes, such as `<Search size={20} />` in
    /// the UI shell header.
    pub fn icon_size(mut self, size: IconSize) -> Self {
        self.icon_size = size;
        self
    }

    /// Let a label that does not fit wrap onto more lines, the button growing
    /// from its size's height (`.cds--btn` sets `min-block-size` and no
    /// `white-space: nowrap`; a narrow `flex: 1` button in Carbon wraps).
    /// Off by default: one line, as wide as the label.
    pub fn wrap_label(mut self, wrap: bool) -> Self {
        self.wrap_label = wrap;
        self
    }

    /// Show the label as a Carbon tooltip when icon-only (default true; turn
    /// it off when the caller wraps the button in its own `Tooltip`).
    pub fn tooltip(mut self, tooltip: bool) -> Self {
        self.tooltip = tooltip;
        self
    }

    /// Where the icon-only tooltip sits (Carbon `IconButton` `align`, default
    /// `top`; header actions use `bottom`).
    pub fn tooltip_align(mut self, align: super::floating::Align) -> Self {
        self.tooltip_align = align;
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

    /// The focus handle the button will track: the caller's
    /// ([`Self::track_focus`]) or the one kept under its id.
    pub fn focus_handle(&self, window: &mut Window, cx: &mut App) -> FocusHandle {
        self.focus_handle.clone().unwrap_or_else(|| {
            window
                .use_keyed_state((self.id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
                .read(cx)
                .clone()
        })
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
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let kind = self.kind();
        let density = self.density.unwrap_or_else(density::current);
        let geometry = Geometry::for_content(
            self.size,
            density,
            kind,
            self.icon_only,
            self.icon.is_some() || self.loading,
        );
        let id = self.id.clone();
        let pointer =
            window.use_keyed_state((id.clone(), "pointer"), cx, |_, _| Pointer::default());
        let focus = self.focus_handle(window, cx);
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
        if let Some(ink) = self.icon_color.filter(|_| !self.disabled) {
            look.icon = ink;
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
                crate::icons::spin_now(
                    Icon::named("progress_activity")
                        .size(IconSize::S16)
                        .color(icon_color),
                    LOADING_SPIN_PERIOD,
                    window,
                    cx,
                )
                .into_any_element(),
            )
        } else {
            self.icon.map(|icon| {
                icon.size(self.icon_size)
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

        let wrap_label = self.wrap_label && geometry.width.is_none();
        button = if wrap_label {
            // `min-block-size` with the block padding on both sides: one line
            // is exactly `height`, each further line adds its line height.
            button
                .min_h(rem(geometry.height))
                .pb(rem(geometry.padding_top))
                .whitespace_normal()
        } else {
            button.h(rem(geometry.height))
        };
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
                .map(|this| {
                    if wrap_label {
                        this.min_w_0()
                            .child(div().flex_1().min_w_0().child(label.clone()))
                    } else {
                        this.child(label.clone())
                    }
                })
                .children(std::mem::take(&mut self.trailing));
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
        let button = button.children(focus_layers).into_any_element();
        if self.icon_only && self.tooltip {
            // `.cds--icon-tooltip`: the label as a Carbon tooltip (above by
            // default). Carbon `IconButton` defaults `closeOnActivation` to
            // true: a press hides it.
            return super::tooltip::Tooltip::new((id, "tooltip"), button)
                .content(label)
                .close_on_activation(true)
                .align(self.tooltip_align)
                .into_any_element();
        }
        button
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    #[test]
    fn wrapping_and_icon_size_are_opt_in() {
        let b = Button::new("b", "Apply filters");
        assert!(!b.wrap_label);
        assert_eq!(b.icon_size, IconSize::S16);
        let b = b.wrap_label(true).icon_size(IconSize::S20);
        assert!(b.wrap_label);
        assert_eq!(b.icon_size, IconSize::S20);
        // A wrapping `sm` button: one line is exactly the size's height (the
        // block padding on both sides), two lines add one line height:
        // Carbon's 50px "Apply filters" in a 94px `flex: 1` slot.
        let g = Geometry::for_content(
            ButtonSize::Sm,
            Density::Compact,
            ButtonKind::Primary,
            false,
            false,
        );
        assert_eq!(g.height, 32.);
        assert_eq!(2. * g.padding_top + LINE + 2., g.height);
        assert_eq!(2. * g.padding_top + 2. * LINE + 2., 50.);
    }

    #[test]
    fn the_loading_spinner_pulses_under_reduced_motion() {
        use crate::t3::base::spin_frame;
        assert_eq!(LOADING_SPIN_PERIOD, Duration::from_millis(690));
        // Turning: a quarter turn a quarter period in, fully opaque.
        let f = spin_frame(Duration::from_micros(172_500), LOADING_SPIN_PERIOD, false);
        assert!((f.turn - 0.25).abs() < 1e-3 && f.opacity == 1.);
        // Reduced: no rotation, `g-pulse 1.4s` 1 → 0.35 → 1.
        let f = spin_frame(Duration::from_millis(700), LOADING_SPIN_PERIOD, true);
        assert_eq!(f.turn, 0.);
        assert!((f.opacity - 0.35).abs() < 1e-4);
        let f = spin_frame(Duration::from_millis(1400), LOADING_SPIN_PERIOD, true);
        assert!((f.opacity - 1.).abs() < 1e-4);
    }

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
        // `t3.css` r2-gal: filled kinds keep Carbon's disabled fill.
        assert_eq!(hex(disabled.background), "#c6c6c6");
        assert_eq!(hex(disabled.text), "#8d8d8d");
        assert!(disabled.focus_ring.is_none());
    }

    #[test]
    fn disabled_outlined_kinds_follow_the_craft_pass() {
        let disabled = ButtonState {
            disabled: true,
            ..Default::default()
        };
        crate::theme::tests_set(CarbonTheme::G100);
        for kind in [
            ButtonKind::Tertiary,
            ButtonKind::DangerTertiary,
            ButtonKind::DangerGhost,
        ] {
            let look = Look::of(kind, disabled, false);
            // G100 layer-01 / text-secondary / border-subtle-01.
            assert_eq!(hex(look.background), "#262626", "{kind:?}");
            assert_eq!(hex(look.text), "#c6c6c6", "{kind:?}");
            assert_eq!(hex(look.border), "#525252", "{kind:?}");
        }
        // r2-gal: primary, secondary and danger keep button-disabled.
        for kind in [
            ButtonKind::Primary,
            ButtonKind::Secondary,
            ButtonKind::Danger,
        ] {
            let look = Look::of(kind, disabled, false);
            assert_eq!(
                look.background,
                color(ColorRole::ButtonDisabled),
                "{kind:?}"
            );
            assert_eq!(look.text, color(ColorRole::TextOnColorDisabled), "{kind:?}");
        }
        crate::theme::tests_set(CarbonTheme::White);
        let ghost = Look::of(ButtonKind::Ghost, disabled, false);
        assert_eq!(ghost.background.a, 0.);
        assert_eq!(hex(ghost.text), "#161616");
        assert!(ghost.text.a < 1.);
        // Icon-only filled buttons keep `.cds--btn:disabled`.
        let icon = Look::of(ButtonKind::Primary, disabled, true);
        assert_eq!(hex(icon.background), "#c6c6c6");
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

    /// Reference render (ScriptRow, ThemeEditor, Button): a text + icon ghost
    /// paints its icon in the text color, link-primary at rest and
    /// link-primary-hover on hover; an icon-only ghost keeps icon-primary.
    #[test]
    fn ghost_icon_follows_text_unless_icon_only() {
        let hovered = ButtonState {
            hovered: true,
            ..Default::default()
        };
        for (theme, link, link_hover, icon) in [
            (CarbonTheme::White, "#0f62fe", "#0043ce", "#161616"),
            (CarbonTheme::G100, "#78a9ff", "#a6c8ff", "#f4f4f4"),
        ] {
            crate::theme::tests_set(theme);
            let rest = Look::of(ButtonKind::Ghost, ButtonState::default(), false);
            assert_eq!(hex(rest.icon), link, "{theme:?}");
            assert_eq!(
                hex(Look::of(ButtonKind::Ghost, hovered, false).icon),
                link_hover
            );
            let only = Look::of(ButtonKind::Ghost, ButtonState::default(), true);
            assert_eq!(hex(only.icon), icon, "{theme:?}");
            assert_eq!(hex(Look::of(ButtonKind::Ghost, hovered, true).icon), icon);
        }
        // Every other kind is unchanged: its icon is its text color.
        crate::theme::tests_set(CarbonTheme::White);
        for kind in ButtonKind::BUTTONS.into_iter().chain([ButtonKind::Link]) {
            if kind == ButtonKind::Ghost {
                continue;
            }
            let look = Look::of(kind, ButtonState::default(), false);
            assert_eq!(look.icon, look.text, "{kind:?}");
        }
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
        // `t3.css` craft pass: text-only, non-ghost buttons end at spacing-05.
        let text_only = Geometry::for_content(
            ButtonSize::Lg,
            Density::Comfortable,
            ButtonKind::Primary,
            false,
            false,
        );
        assert_eq!(text_only.padding_right, 16.);
        let with_icon = Geometry::for_content(
            ButtonSize::Lg,
            Density::Comfortable,
            ButtonKind::Secondary,
            false,
            true,
        );
        assert_eq!(with_icon.padding_right, 63.);
        let ghost_text = Geometry::for_content(
            ButtonSize::Lg,
            Density::Comfortable,
            ButtonKind::Ghost,
            false,
            false,
        );
        assert_eq!(ghost_text.padding_right, 15.);
        let danger_ghost_text = Geometry::for_content(
            ButtonSize::Lg,
            Density::Comfortable,
            ButtonKind::DangerGhost,
            false,
            false,
        );
        assert_eq!(danger_ghost_text.padding_right, 16.);
        let icon = Geometry::of(
            ButtonSize::Md,
            Density::Comfortable,
            ButtonKind::Ghost,
            true,
        );
        assert_eq!((icon.height, icon.width), (40., Some(40.)));
    }
}
