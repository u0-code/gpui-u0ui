//! Badge: shadcn `Badge` → Carbon `Tag` (`.cds--tag*`), `DismissibleTag`
//! (`.cds--tag--filter`), `OperationalTag` (`.cds--tag--operational`) and
//! `SelectableTag` (`.cds--tag--selectable`).
//!
//! - **Appearance**: a 16px-radius pill in `label-01`, 18/24/32px tall
//!   (`.cds--tag { --layout-size-height-sm: 1.125rem; -md: 1.5rem; -lg: 2rem }`),
//!   at least 2rem and at most 13rem wide (the label ellipsizes), 8px side
//!   padding (12px at lg). Colors come from the `tag-*` token quadruple of the
//!   type ([`TagType`]); see [`Look::of`] for every state.
//! - **Dismissible** (`on_dismiss`): the end padding goes to 0 and a round
//!   close button as tall as the tag follows the label (2px gap, 5px at sm),
//!   hover `tag-hover-<type>`, focus a 1px inset `focus` ring.
//! - **Operational / selectable** tags are buttons: focusable, Enter/Space or
//!   click; focus draws Carbon's `outline: 2px solid focus; outline-offset: 1px`.
//! - **Motion**: background 70ms (`.cds--tag--interactive { transition:
//!   background-color 70ms cubic-bezier(0, 0, 0.38, 0.9) }` = [`Motion::HOVER`];
//!   the close button uses the productive standard curve).
//! - **Behavior**: Base `Button` (focus, keyboard and pointer activation) for
//!   every interactive part. Kit's `Tag` is a styled div with its own palette
//!   and radius, with nothing to reuse.
//!
//! Tags do not follow density: Carbon has no compact rule for `.cds--tag`.
use crate::{
    icons::{CarbonIcon, Icon, IconSize},
    motion::{self, Curve, Motion, Speed},
    theme::{LayerLevel, color},
    tokens::{BORDER_RADIUS_16, ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    App, ClickEvent, CursorStyle, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder as _, px, rems,
};
use std::rc::Rc;

/// Carbon tag types (`.cds--tag--<type>`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TagType {
    /// `red`.
    Red,
    /// `magenta`.
    Magenta,
    /// `purple`.
    Purple,
    /// `blue`.
    Blue,
    /// `cyan`.
    Cyan,
    /// `teal`.
    Teal,
    /// `green`.
    Green,
    /// `gray`, Carbon's default.
    #[default]
    Gray,
    /// `cool-gray`.
    CoolGray,
    /// `warm-gray`.
    WarmGray,
    /// `high-contrast`: `background-inverse` / `text-inverse`.
    HighContrast,
    /// `outline`: `background` with a 1px inset `background-inverse` ring.
    Outline,
}

impl TagType {
    /// The ten colored types (with a `tag-*` token set).
    pub const COLORS: [TagType; 10] = [
        Self::Red,
        Self::Magenta,
        Self::Purple,
        Self::Blue,
        Self::Cyan,
        Self::Teal,
        Self::Green,
        Self::Gray,
        Self::CoolGray,
        Self::WarmGray,
    ];

    /// The `tag-background`, `tag-color`, `tag-hover` and `tag-border` roles
    /// of a colored type; high-contrast and outline have none.
    pub const fn roles(self) -> Option<(ColorRole, ColorRole, ColorRole, ColorRole)> {
        use ColorRole::*;
        Some(match self {
            Self::Red => (TagBackgroundRed, TagColorRed, TagHoverRed, TagBorderRed),
            Self::Magenta => (
                TagBackgroundMagenta,
                TagColorMagenta,
                TagHoverMagenta,
                TagBorderMagenta,
            ),
            Self::Purple => (
                TagBackgroundPurple,
                TagColorPurple,
                TagHoverPurple,
                TagBorderPurple,
            ),
            Self::Blue => (TagBackgroundBlue, TagColorBlue, TagHoverBlue, TagBorderBlue),
            Self::Cyan => (TagBackgroundCyan, TagColorCyan, TagHoverCyan, TagBorderCyan),
            Self::Teal => (TagBackgroundTeal, TagColorTeal, TagHoverTeal, TagBorderTeal),
            Self::Green => (
                TagBackgroundGreen,
                TagColorGreen,
                TagHoverGreen,
                TagBorderGreen,
            ),
            Self::Gray => (TagBackgroundGray, TagColorGray, TagHoverGray, TagBorderGray),
            Self::CoolGray => (
                TagBackgroundCoolGray,
                TagColorCoolGray,
                TagHoverCoolGray,
                TagBorderCoolGray,
            ),
            Self::WarmGray => (
                TagBackgroundWarmGray,
                TagColorWarmGray,
                TagHoverWarmGray,
                TagBorderWarmGray,
            ),
            Self::HighContrast | Self::Outline => return None,
        })
    }
}

/// `index.d.ts` `BadgeProps.variant`: shadcn names plus every Carbon type.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BadgeVariant {
    /// shadcn `default` = Carbon `gray`.
    #[default]
    Default,
    /// shadcn `secondary` = Carbon `cool-gray`.
    Secondary,
    /// shadcn `destructive` = Carbon `red`.
    Destructive,
    /// `outline`.
    Outline,
    /// `red`.
    Red,
    /// `magenta`.
    Magenta,
    /// `purple`.
    Purple,
    /// `blue`.
    Blue,
    /// `cyan`.
    Cyan,
    /// `teal`.
    Teal,
    /// `green`.
    Green,
    /// `gray`.
    Gray,
    /// `cool-gray`.
    CoolGray,
    /// `warm-gray`.
    WarmGray,
    /// `high-contrast`.
    HighContrast,
}

impl BadgeVariant {
    /// Every variant, in `index.d.ts` order.
    pub const ALL: [BadgeVariant; 15] = [
        Self::Default,
        Self::Secondary,
        Self::Destructive,
        Self::Outline,
        Self::Red,
        Self::Magenta,
        Self::Purple,
        Self::Blue,
        Self::Cyan,
        Self::Teal,
        Self::Green,
        Self::Gray,
        Self::CoolGray,
        Self::WarmGray,
        Self::HighContrast,
    ];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Secondary => "secondary",
            Self::Destructive => "destructive",
            Self::Outline => "outline",
            Self::Red => "red",
            Self::Magenta => "magenta",
            Self::Purple => "purple",
            Self::Blue => "blue",
            Self::Cyan => "cyan",
            Self::Teal => "teal",
            Self::Green => "green",
            Self::Gray => "gray",
            Self::CoolGray => "cool-gray",
            Self::WarmGray => "warm-gray",
            Self::HighContrast => "high-contrast",
        }
    }

    /// The Carbon tag type (`TAG` in the reference `index.jsx`:
    /// `{ default: 'gray', secondary: 'cool-gray', destructive: 'red', outline: 'outline' }`).
    pub const fn tag_type(self) -> TagType {
        match self {
            Self::Default | Self::Gray => TagType::Gray,
            Self::Secondary | Self::CoolGray => TagType::CoolGray,
            Self::Destructive | Self::Red => TagType::Red,
            Self::Outline => TagType::Outline,
            Self::Magenta => TagType::Magenta,
            Self::Purple => TagType::Purple,
            Self::Blue => TagType::Blue,
            Self::Cyan => TagType::Cyan,
            Self::Teal => TagType::Teal,
            Self::Green => TagType::Green,
            Self::WarmGray => TagType::WarmGray,
            Self::HighContrast => TagType::HighContrast,
        }
    }
}

/// `index.d.ts` `Size` for tags.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BadgeSize {
    /// `sm`: 18px.
    Sm,
    /// `md`: 24px, the default.
    #[default]
    Md,
    /// `lg`: 32px.
    Lg,
}

impl BadgeSize {
    /// Every size.
    pub const ALL: [BadgeSize; 3] = [Self::Sm, Self::Md, Self::Lg];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
        }
    }
}

/// What kind of Carbon tag is drawn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TagMode {
    /// `Tag`: read-only.
    #[default]
    Static,
    /// `DismissibleTag` (`.cds--tag--filter`): a close button after the label.
    Dismissible,
    /// `OperationalTag` (`.cds--tag--operational`): a button with a border.
    Operational,
    /// `SelectableTag` (`.cds--tag--selectable`), with its selected state.
    Selectable {
        /// `.cds--tag--selectable-selected`.
        selected: bool,
    },
}

/// Pointer and focus state for a look.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct TagState {
    /// Pointer over the tag (operational/selectable hover).
    pub hovered: bool,
    /// Pointer over the close button.
    pub close_hovered: bool,
    /// `.cds--tag--disabled`.
    pub disabled: bool,
}

/// The colors a tag paints in one state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Look {
    /// Fill.
    pub background: Hsla,
    /// Label and icon color.
    pub text: Hsla,
    /// 1px border or inset ring, if any.
    pub border: Option<Hsla>,
    /// Close button fill (transparent unless hovered).
    pub close_background: Hsla,
    /// Close icon color.
    pub close_icon: Hsla,
    /// The close button's 1px inset focus ring color.
    pub close_focus: Hsla,
}

impl Look {
    /// The Carbon look of a tag of `ty` drawn as `mode` in `state`, on `level`
    /// (selectable and disabled tags paint the contextual `layer`).
    pub fn of(ty: TagType, mode: TagMode, state: TagState, level: LayerLevel) -> Look {
        use ColorRole::*;
        let c = color;
        let clear = Hsla::transparent_black();
        let layer = c(level.layer());
        if let TagMode::Selectable { selected } = mode {
            // `.cds--tag--selectable { border: 1px solid border-inverse; background: layer;
            // color: text-primary }`, hover `layer-hover`; selected
            // `layer-selected-inverse` / `text-inverse`; disabled `border-disabled`.
            let (background, text, border) = if state.disabled {
                (layer, c(TextDisabled), c(BorderDisabled))
            } else if selected {
                (c(LayerSelectedInverse), c(TextInverse), c(BorderInverse))
            } else if state.hovered {
                (c(level.layer_hover()), c(TextPrimary), c(BorderInverse))
            } else {
                (layer, c(TextPrimary), c(BorderInverse))
            };
            return Look {
                background,
                text,
                border: Some(border),
                close_background: clear,
                close_icon: text,
                close_focus: c(Focus),
            };
        }
        // `:not(.cds--tag--operational)` keeps high-contrast and outline from
        // applying to operational tags: those fall back to gray.
        let ty = if mode == TagMode::Operational && ty.roles().is_none() {
            TagType::Gray
        } else {
            ty
        };
        if state.disabled {
            // `.cds--tag--disabled:not(.cds--tag--operational), .cds--tag--filter.cds--tag--disabled
            // { background: layer; color: text-disabled }`; operational adds
            // `border: 1px solid border-disabled`; the close icon `icon-disabled`.
            return Look {
                background: layer,
                text: c(TextDisabled),
                border: (mode == TagMode::Operational).then(|| c(BorderDisabled)),
                close_background: clear,
                close_icon: c(IconDisabled),
                close_focus: c(Focus),
            };
        }
        let (background, text, hover, border, close_focus) = match ty.roles() {
            Some((bg, fg, hover, border)) => (c(bg), c(fg), c(hover), Some(c(border)), c(Focus)),
            // `.cds--tag--high-contrast`: background-inverse / text-inverse;
            // close hover background-inverse-hover, close focus focus-inverse.
            None if ty == TagType::HighContrast => (
                c(BackgroundInverse),
                c(TextInverse),
                c(BackgroundInverseHover),
                None,
                c(FocusInverse),
            ),
            // `.cds--tag--outline`: background / text-primary, `box-shadow:
            // inset 0 0 0 1px background-inverse`; close hover `layer-hover`.
            None => (
                c(Background),
                c(TextPrimary),
                c(level.layer_hover()),
                Some(c(BackgroundInverse)),
                c(Focus),
            ),
        };
        let operational = mode == TagMode::Operational;
        Look {
            // `.cds--tag--<type>.cds--tag--operational:hover { background: tag-hover-<type> }`.
            background: if operational && state.hovered {
                hover
            } else {
                background
            },
            text,
            // Static/dismissible colored tags have `border: 0`; operational
            // `border: 1px solid tag-border-<type>`; outline keeps its ring.
            border: if operational || ty == TagType::Outline {
                border
            } else {
                None
            },
            // `.cds--tag--<type> .cds--tag__close-icon:hover { background: tag-hover-<type> }`.
            close_background: if state.close_hovered { hover } else { clear },
            close_icon: text,
            close_focus,
        }
    }
}

/// Box geometry in px.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Geometry {
    /// `min-block-size: var(--layout-size-height-local)`.
    pub height: f32,
    /// `padding-inline-start`.
    pub padding_left: f32,
    /// `padding-inline-end` (0 for the filter tag: the close button ends it).
    pub padding_right: f32,
    /// Close button `margin-inline-start`.
    pub close_gap: f32,
}

/// `.cds--tag { min-inline-size: 2rem }`.
pub const MIN_WIDTH: f32 = 32.;
/// `.cds--tag { max-inline-size: 13rem }`.
pub const MAX_WIDTH: f32 = 208.;
/// `.cds--tag__custom-icon { inline-size: 1rem; margin-inline-end: 0.25rem }`.
pub const ICON_GAP: f32 = 4.;

impl Geometry {
    /// Geometry for `size`, dismissible or not, with or without a leading icon.
    pub fn of(size: BadgeSize, dismissible: bool, icon: bool) -> Self {
        // `.cds--tag { --layout-size-height-sm: 1.125rem; -md: 1.5rem; -lg: 2rem }`.
        let height = match size {
            BadgeSize::Sm => 18.,
            BadgeSize::Md => 24.,
            BadgeSize::Lg => 32.,
        };
        let lg = size == BadgeSize::Lg;
        // `.cds--tag { padding-inline: 0.5rem }`; `.cds--tag.cds--tag--lg { padding-inline-start:
        // 0.75rem }` (and end, unless filter); `:has(.cds--tag__custom-icon)`
        // start 0.25rem (lg 0.5rem).
        let padding_left = match (icon, lg) {
            (true, true) => 8.,
            (true, false) => 4.,
            (false, true) => 12.,
            (false, false) => 8.,
        };
        Self {
            height,
            padding_left,
            // `.cds--tag--filter { padding-inline-end: 0 }`.
            padding_right: if dismissible {
                0.
            } else if lg {
                12.
            } else {
                8.
            },
            // `.cds--tag__close-icon { margin: 0 0 0 0.125rem }`; sm 0.3125rem.
            close_gap: if size == BadgeSize::Sm { 5. } else { 2. },
        }
    }
}

type Handler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;
type SelectHandler = Rc<dyn Fn(bool, &mut Window, &mut App)>;

/// A Carbon tag. See the module docs.
#[derive(IntoElement)]
pub struct Badge {
    id: Option<ElementId>,
    label: SharedString,
    variant: BadgeVariant,
    size: BadgeSize,
    icon: Option<Icon>,
    disabled: bool,
    level: LayerLevel,
    on_dismiss: Option<Handler>,
    dismiss_label: SharedString,
    on_click: Option<Handler>,
    selectable: Option<(bool, Option<SelectHandler>)>,
    colors: Option<(Hsla, Hsla)>,
    shrink: bool,
}

impl Badge {
    /// A read-only tag showing `label`.
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            id: None,
            label: label.into(),
            variant: BadgeVariant::Default,
            size: BadgeSize::Md,
            icon: None,
            disabled: false,
            level: LayerLevel::One,
            on_dismiss: None,
            dismiss_label: SharedString::default(),
            on_click: None,
            selectable: None,
            colors: None,
            shrink: false,
        }
    }

    /// Let the tag shrink in a crowded flex row down to its 2rem minimum,
    /// its label ending in an ellipsis (`.cds--tag` keeps CSS's
    /// `flex-shrink: 1` with `min-inline-size: 2rem`). Off by default: in
    /// GPUI a text sibling cannot shrink below its full width, so a tag that
    /// shrinks by default would give way before the text does.
    pub fn shrink(mut self, shrink: bool) -> Self {
        self.shrink = shrink;
        self
    }

    /// Paint a read-only tag in a pattern library's own pair (fill, text),
    /// e.g. `.t3-provcard .cds--tag--warm-gray { background:
    /// var(--tag-background-yellow, #fcf4d6); color: var(--text-primary) }`.
    pub fn colors(mut self, background: Hsla, text: Hsla) -> Self {
        self.colors = Some((background, text));
        self
    }

    /// A stable id (needed when several interactive tags share a label).
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// The variant (shadcn or Carbon type name).
    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }

    /// The size (default `md`).
    pub fn size(mut self, size: BadgeSize) -> Self {
        self.size = size;
        self
    }

    /// A 16px icon before the label (`.cds--tag__custom-icon`).
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Disabled (`.cds--tag--disabled`).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// The layer the tag sits on (selectable/disabled tags paint `layer`).
    pub fn layer(mut self, level: LayerLevel) -> Self {
        self.level = level;
        self
    }

    /// Make the tag dismissible (`DismissibleTag`); `handler` runs when the
    /// close button is pressed; it is expected to remove the tag. A focused
    /// close button leaves the Tab starting point where it was
    /// ([`crate::u0::primitives::parts::vacate_focus`]).
    pub fn on_dismiss(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_dismiss = Some(Rc::new(handler));
        self
    }

    /// The close button's accessible name (Carbon's `title`, e.g. "Dismiss").
    pub fn dismiss_label(mut self, label: impl Into<SharedString>) -> Self {
        self.dismiss_label = label.into();
        self
    }

    /// Make the tag an `OperationalTag`: a bordered button running `handler`.
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    /// Make the tag a `SelectableTag` showing `selected`; `on_change` gets the
    /// toggled value.
    pub fn selectable(
        mut self,
        selected: bool,
        on_change: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.selectable = Some((selected, Some(Rc::new(on_change))));
        self
    }

    /// The tag mode these props draw.
    pub fn mode(&self) -> TagMode {
        if let Some((selected, _)) = self.selectable {
            TagMode::Selectable { selected }
        } else if self.on_click.is_some() {
            TagMode::Operational
        } else if self.on_dismiss.is_some() {
            TagMode::Dismissible
        } else {
            TagMode::Static
        }
    }
}

#[derive(Default)]
struct Pointer {
    hovered: bool,
    close_hovered: bool,
}

fn rem(px_value: f32) -> gpui_kit::Rems {
    rems(px_value / crate::tokens::REM_PX)
}

/// `.cds--tag--operational:focus` / `.cds--tag--selectable:focus { outline: 2px solid focus }`.
pub const FOCUS_WIDTH: f32 = 2.;
/// `… { outline-offset: 1px }`: the gap between the tag and its ring.
pub const FOCUS_OFFSET: f32 = 1.;

/// The inset (px, negative: outside) of the focus ring layer on a tag with a
/// `border` px border. An absolute child is placed from the padding box, so
/// the border is added back to start the 1px gap at the tag's outer edge.
pub fn focus_ring_inset(border: f32) -> f32 {
    -(border + FOCUS_OFFSET + FOCUS_WIDTH)
}

/// Carbon's `outline: 2px solid focus; outline-offset: 1px` around a pill
/// with a `border` px border.
fn focus_outline(radius: f32, border: f32) -> impl IntoElement {
    let inset = px(focus_ring_inset(border));
    div()
        .absolute()
        .top(inset)
        .left(inset)
        .right(inset)
        .bottom(inset)
        .rounded(px(radius + border + FOCUS_OFFSET + FOCUS_WIDTH))
        .border(px(FOCUS_WIDTH))
        .border_color(color(ColorRole::Focus))
}

impl RenderOnce for Badge {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let mode = self.mode();
        let ty = self.variant.tag_type();
        let id = self
            .id
            .clone()
            .unwrap_or_else(|| ElementId::Name(format!("badge-{}", self.label).into()));
        let pointer =
            window.use_keyed_state((id.clone(), "pointer"), cx, |_, _| Pointer::default());
        let (hovered, close_hovered) = {
            let p = pointer.read(cx);
            (
                p.hovered && !self.disabled,
                p.close_hovered && !self.disabled,
            )
        };
        let look = Look::of(
            ty,
            mode,
            TagState {
                hovered,
                close_hovered,
                disabled: self.disabled,
            },
            self.level,
        );
        let mut look = look;
        if let (Some((bg, fg)), TagMode::Static) = (self.colors, mode) {
            look.background = bg;
            look.text = fg;
        }
        let geometry = Geometry::of(self.size, mode == TagMode::Dismissible, self.icon.is_some());
        let background = motion::animate_color(
            (id.clone(), "bg"),
            look.background,
            Motion::HOVER,
            window,
            cx,
        );
        let close_background = motion::animate_color(
            (id.clone(), "close-bg"),
            look.close_background,
            Motion::new(Curve::ProductiveStandard, Speed::Fast01),
            window,
            cx,
        );
        let radius = BORDER_RADIUS_16.px_value();

        let label = div()
            .min_w_0()
            .overflow_hidden()
            .text_ellipsis()
            .whitespace_nowrap()
            .child(self.label.clone());
        let icon = self.icon.clone().map(|icon| {
            div()
                .flex_none()
                .mr(rem(ICON_GAP))
                .child(icon.size(IconSize::S16).color(look.text))
        });

        let set_hover = {
            let pointer = pointer.clone();
            move |hovered: &bool, _: &mut Window, cx: &mut App| {
                pointer.update(cx, |p, cx| {
                    if p.hovered != *hovered {
                        p.hovered = *hovered;
                        cx.notify();
                    }
                })
            }
        };

        let shrink = self.shrink;
        let shell = |el: gpui_kit::Stateful<gpui_kit::Div>| {
            el.relative()
                .flex()
                .flex_row()
                .flex_none()
                .when(shrink, |this| this.flex_shrink(1.))
                .items_center()
                .justify_center()
                .h(rem(geometry.height))
                .min_w(rem(MIN_WIDTH))
                .max_w(rem(MAX_WIDTH))
                .pl(rem(geometry.padding_left))
                .pr(rem(geometry.padding_right))
                .rounded(rem(radius))
                .bg(background)
                .text_color(look.text)
                .type_style(TypeStyle::Label01)
                .when_some(look.border, |this, border| {
                    this.border_1().border_color(border)
                })
        };

        match mode {
            TagMode::Operational | TagMode::Selectable { .. } => {
                let focus = window
                    .use_keyed_state((id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
                    .read(cx)
                    .clone();
                let focused = focus.is_focused(window) && !self.disabled;
                let on_click = self.on_click.clone();
                let selectable = self.selectable.clone();
                let button = gpui_kit::base::Button::new(id.clone())
                    .track_focus(&focus)
                    .disabled(self.disabled)
                    .accessibility_label(self.label.clone())
                    .on_hover(set_hover)
                    .on_click(move |event, window, cx| {
                        if let Some(handler) = &on_click {
                            handler(event, window, cx);
                        }
                        if let Some((selected, Some(on_change))) = &selectable {
                            on_change(!*selected, window, cx);
                        }
                    });
                shell(div().id((id.clone(), "shell")))
                    .cursor(if self.disabled {
                        CursorStyle::OperationNotAllowed
                    } else {
                        CursorStyle::PointingHand
                    })
                    .children(icon)
                    .child(label)
                    .when(focused, |this| {
                        this.child(focus_outline(
                            radius,
                            if look.border.is_some() { 1. } else { 0. },
                        ))
                    })
                    .child(
                        // The Base button covers the pill: it owns focus,
                        // keyboard and pointer activation.
                        button.absolute().top_0().left_0().size_full(),
                    )
                    .into_any_element()
            }
            TagMode::Dismissible => {
                let on_dismiss = self.on_dismiss.clone().expect("dismissible");
                let close_focus = window
                    .use_keyed_state((id.clone(), "close-focus"), cx, |_, cx| cx.focus_handle())
                    .read(cx)
                    .clone();
                let close_focused = close_focus.is_focused(window) && !self.disabled;
                let close = gpui_kit::base::Button::new((id.clone(), "close"))
                    .track_focus(&close_focus)
                    .disabled(self.disabled)
                    .accessibility_label(self.dismiss_label.clone())
                    .relative()
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .ml(rem(geometry.close_gap))
                    .size(rem(geometry.height))
                    .rounded_full()
                    .bg(close_background)
                    .cursor(if self.disabled {
                        CursorStyle::OperationNotAllowed
                    } else {
                        CursorStyle::PointingHand
                    })
                    .on_hover({
                        let pointer = pointer.clone();
                        move |hovered, _, cx| {
                            pointer.update(cx, |p, cx| {
                                if p.close_hovered != *hovered {
                                    p.close_hovered = *hovered;
                                    cx.notify();
                                }
                            })
                        }
                    })
                    .on_click({
                        let close_focus = close_focus.clone();
                        move |event, window, cx| {
                            // The caller removes the tag: keep the Tab starting
                            // point where its focused close button was.
                            crate::u0::primitives::parts::vacate_focus(&close_focus, window, cx);
                            on_dismiss(event, window, cx)
                        }
                    })
                    .child(
                        Icon::carbon(CarbonIcon::Close)
                            .size(IconSize::S16)
                            .color(look.close_icon),
                    )
                    // `.cds--tag__close-icon:focus { box-shadow: inset 0 0 0 1px focus }`.
                    .when(close_focused, |this| {
                        this.child(
                            div()
                                .absolute()
                                .top_0()
                                .left_0()
                                .size_full()
                                .rounded_full()
                                .border_1()
                                .border_color(look.close_focus),
                        )
                    });
                shell(div().id((id.clone(), "shell")))
                    .children(icon)
                    .child(label)
                    .child(close)
                    .into_any_element()
            }
            TagMode::Static => shell(div().id((id.clone(), "shell")))
                .children(icon)
                .child(label)
                .into_any_element(),
        }
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
    fn variants_map_to_tag_types() {
        // `index.jsx`: TAG = { default: 'gray', secondary: 'cool-gray', destructive: 'red', outline: 'outline' }.
        assert_eq!(BadgeVariant::Default.tag_type(), TagType::Gray);
        assert_eq!(BadgeVariant::Secondary.tag_type(), TagType::CoolGray);
        assert_eq!(BadgeVariant::Destructive.tag_type(), TagType::Red);
        assert_eq!(BadgeVariant::Outline.tag_type(), TagType::Outline);
        assert_eq!(BadgeVariant::WarmGray.name(), "warm-gray");
    }

    #[test]
    fn static_colors_follow_bundle_css() {
        crate::theme::tests_set(CarbonTheme::White);
        let look = |ty| Look::of(ty, TagMode::Static, TagState::default(), LayerLevel::One);
        // `.cds--tag--blue { background: tag-background-blue #d0e2ff; color: tag-color-blue #0043ce }`.
        let blue = look(TagType::Blue);
        assert_eq!(
            (hex(blue.background), hex(blue.text)),
            ("#d0e2ff".into(), "#0043ce".into())
        );
        assert!(
            blue.border.is_none(),
            "`.cds--tag:not(.cds--tag--selectable) {{ border: 0 }}`"
        );
        // `.cds--tag--high-contrast { background: background-inverse #393939; color: text-inverse }`.
        let hc = look(TagType::HighContrast);
        assert_eq!(hex(hc.background), "#393939");
        assert_eq!(hex(hc.text), "#ffffff");
        // `.cds--tag--outline { background: background; box-shadow: inset 0 0 0 1px background-inverse }`.
        let outline = look(TagType::Outline);
        assert_eq!(hex(outline.background), "#ffffff");
        assert_eq!(outline.border.map(hex).as_deref(), Some("#393939"));
    }

    #[test]
    fn interactive_states() {
        crate::theme::tests_set(CarbonTheme::White);
        let hovered = TagState {
            hovered: true,
            ..Default::default()
        };
        // `.cds--tag--red.cds--tag--operational { border: tag-border-red #ff8389 }`, hover #ffc2c5.
        let op = Look::of(TagType::Red, TagMode::Operational, hovered, LayerLevel::One);
        assert_eq!(hex(op.background), "#ffc2c5");
        assert_eq!(op.border.map(hex).as_deref(), Some("#ff8389"));
        // High contrast does not apply to operational tags: they fall back to gray.
        let hc = Look::of(
            TagType::HighContrast,
            TagMode::Operational,
            TagState::default(),
            LayerLevel::One,
        );
        assert_eq!(hex(hc.background), "#e0e0e0");
        // Close hover: `.cds--tag--gray .cds--tag__close-icon:hover { tag-hover-gray #d1d1d1 }`.
        let close = Look::of(
            TagType::Gray,
            TagMode::Dismissible,
            TagState {
                close_hovered: true,
                ..Default::default()
            },
            LayerLevel::One,
        );
        assert_eq!(hex(close.close_background), "#d1d1d1");
        // Selectable: border-inverse ring on layer; selected layer-selected-inverse.
        let sel = Look::of(
            TagType::Gray,
            TagMode::Selectable { selected: true },
            TagState::default(),
            LayerLevel::One,
        );
        assert_eq!(hex(sel.background), "#161616");
        assert_eq!(hex(sel.text), "#ffffff");
        let unsel = Look::of(
            TagType::Gray,
            TagMode::Selectable { selected: false },
            TagState::default(),
            LayerLevel::One,
        );
        assert_eq!(hex(unsel.background), "#f4f4f4");
        assert_eq!(unsel.border.map(hex).as_deref(), Some("#161616"));
        // Disabled: layer background, text-disabled.
        let disabled = Look::of(
            TagType::Blue,
            TagMode::Static,
            TagState {
                disabled: true,
                ..Default::default()
            },
            LayerLevel::One,
        );
        assert_eq!(hex(disabled.background), "#f4f4f4");
    }

    #[test]
    fn geometry_follows_bundle_css() {
        let md = Geometry::of(BadgeSize::Md, false, false);
        assert_eq!(
            (md.height, md.padding_left, md.padding_right),
            (24., 8., 8.)
        );
        let lg = Geometry::of(BadgeSize::Lg, false, false);
        assert_eq!(
            (lg.height, lg.padding_left, lg.padding_right),
            (32., 12., 12.)
        );
        let sm_filter = Geometry::of(BadgeSize::Sm, true, false);
        assert_eq!(
            (
                sm_filter.height,
                sm_filter.padding_right,
                sm_filter.close_gap
            ),
            (18., 0., 5.)
        );
        let lg_filter = Geometry::of(BadgeSize::Lg, true, true);
        assert_eq!(
            (
                lg_filter.padding_left,
                lg_filter.padding_right,
                lg_filter.close_gap
            ),
            (8., 0., 2.)
        );
        assert_eq!(Geometry::of(BadgeSize::Md, false, true).padding_left, 4.);
        assert_eq!((MIN_WIDTH, MAX_WIDTH), (32., 208.));
    }

    #[test]
    fn focus_ring_leaves_a_1px_gap_outside_the_border() {
        // Operational and selectable tags have a 1px border: the ring's inner
        // edge sits 1px outside it (2px from the padding box), 2px wide.
        assert_eq!(focus_ring_inset(1.), -4.);
        assert_eq!(focus_ring_inset(0.), -3.);
        assert_eq!((FOCUS_WIDTH, FOCUS_OFFSET), (2., 1.));
    }
}
