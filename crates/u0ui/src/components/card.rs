//! Card: shadcn `Card` → Carbon `Tile` (`.cds--tile*`), `ClickableTile`
//! (`href` / `on_click`), `SelectableTile` (`selectable`) and, beyond the
//! reference API, `ExpandableTile` (`expandable`).
//!
//! - **Appearance**: `.cds--tile { padding: 1rem; background: var(--layer);
//!   min-block-size: 4rem; min-inline-size: 8rem }`, square, no shadow;
//!   `variant="outline"` paints `background` with a 1px inset `border-tile-01`
//!   (`.cu-card--outline`). Inside, `.cu-card` stacks the parts 16px apart:
//!   [`CardHeader`] (title `heading-03`, description `body-01` in
//!   `text-secondary`, an optional action spanning both rows), [`CardContent`]
//!   (`body-01`) and [`CardFooter`] (a row pushed to the bottom).
//! - **Clickable**: hover `layer-hover`, a 2px inset `focus` outline.
//! - **Selectable**: 48px end padding for the checkbox icon 16px from the top
//!   and end (`.cds--tile__checkmark--persistent`): `Checkbox` in
//!   `icon-secondary`, selected `CheckboxCheckedFilled` in `icon-primary` with a
//!   1px `layer-selected-inverse` border; disabled `text-disabled`.
//! - **Expandable**: the whole tile toggles; the chevron (48px square, bottom
//!   end) turns 180°, the below-the-fold content reveals by height and fades.
//! - **Motion**: background `transition: 150ms cubic-bezier(0.2, 0, 0.38, 0.9)`
//!   ([`HOVER`]); checkmark and chevron 110ms; the fold opens in 110ms
//!   (`.cds--tile--is-expanded { transition: max-height 110ms }`) and closes in
//!   150ms (`.cds--tile--expandable { transition: max-height 150ms }`), spatial
//!   (reduced motion keeps only the fade).
//! - **Behavior**: Base `Button` (focus, Enter/Space, pointer) for the
//!   interactive tiles; Base `Collapsible` for the fold. Kit has no tile.
use super::form::{controlled, rem};
use crate::{
    icons::{CarbonIcon, Icon, IconSize},
    motion::{self, Curve, Motion, Speed},
    theme::{LayerLevel, color},
    tokens::{ColorRole, SPACING_02, SPACING_03, SPACING_05, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    AnyElement, App, ClickEvent, CursorStyle, ElementId, Hsla, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement as _, StyleRefinement,
    Styled, Window, base::StyledExt as _, div, prelude::FluentBuilder as _, px,
};
use std::rc::Rc;

/// Tile background transition: `.cds--tile--clickable, .cds--tile--selectable
/// { transition: 150ms cubic-bezier(0.2, 0, 0.38, 0.9) }`.
pub const HOVER: Motion = Motion::new(Curve::ProductiveStandard, Speed::Moderate01);
/// Checkmark and chevron: `transition: 110ms cubic-bezier(0.2, 0, 0.38, 0.9)`.
pub const DETAIL: Motion = Motion::new(Curve::ProductiveStandard, Speed::Fast02);
/// The fold opening: `.cds--tile--is-expanded { transition: max-height 110ms … }`.
pub const EXPAND: Motion = Motion::new(Curve::ProductiveStandard, Speed::Fast02);
/// The fold closing: `.cds--tile--expandable { transition: max-height 150ms … }`.
pub const COLLAPSE: Motion = Motion::new(Curve::ProductiveStandard, Speed::Moderate01);

/// `.cds--tile { padding: var(--layout-density-padding-inline-local) }` (1rem, normal density).
pub const PADDING: f32 = 16.;
/// `.cds--tile { min-block-size: 4rem }`.
pub const MIN_HEIGHT: f32 = 64.;
/// `.cds--tile { min-inline-size: 8rem }`.
pub const MIN_WIDTH: f32 = 128.;
/// `.cds--tile--selectable { padding-inline-end: calc(pad * 2 + 1rem) }` and
/// the expandable chevron box (`.cds--tile__chevron { block-size: calc(pad * 2 + 1rem) }`).
pub const AFFORDANCE: f32 = PADDING * 2. + 16.;

/// `index.d.ts` `CardProps.variant`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CardVariant {
    /// `layer` fill.
    #[default]
    Default,
    /// `background` fill with a 1px `border-tile-01` outline (`.cu-card--outline`).
    Outline,
}

impl CardVariant {
    /// Both variants.
    pub const ALL: [CardVariant; 2] = [Self::Default, Self::Outline];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Outline => "outline",
        }
    }
}

/// Which Carbon tile is drawn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TileKind {
    /// `Tile`: static.
    #[default]
    Static,
    /// `ClickableTile`.
    Clickable,
    /// `SelectableTile`.
    Selectable,
    /// `ExpandableTile`.
    Expandable,
}

/// The state a look is computed for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct TileState {
    /// Pointer over the tile.
    pub hovered: bool,
    /// Keyboard focus on the tile.
    pub focused: bool,
    /// Selected (selectable tiles).
    pub selected: bool,
    /// Disabled (selectable tiles).
    pub disabled: bool,
}

/// The colors a tile paints in one state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Look {
    /// Fill.
    pub background: Hsla,
    /// 1px border/outline, if any.
    pub border: Option<Hsla>,
    /// Text color (`text-primary`, disabled `text-disabled`).
    pub text: Hsla,
    /// Checkmark / chevron icon color.
    pub icon: Hsla,
    /// 2px inset focus outline, if focused.
    pub focus: Option<Hsla>,
}

impl Look {
    /// The Carbon look of a `kind` tile of `variant` in `state` on `level`.
    pub fn of(kind: TileKind, variant: CardVariant, state: TileState, level: LayerLevel) -> Look {
        use ColorRole::*;
        let c = color;
        let interactive = kind != TileKind::Static && !state.disabled;
        let rest = match variant {
            // `.cds--tile { background-color: var(--layer) }`.
            CardVariant::Default => c(level.layer()),
            // `.cu-card--outline { background: var(--background) }`.
            CardVariant::Outline => c(Background),
        };
        let background = if interactive && state.hovered {
            // `.cds--tile--clickable:hover, .cds--tile--selectable:hover,
            // .cds--tile--expandable:hover { background: var(--layer-hover) }`.
            c(level.layer_hover())
        } else if state.disabled {
            // `.cds--tile--disabled.cds--tile--selectable { background-color: var(--layer) }`.
            c(level.layer())
        } else {
            rest
        };
        let border = if kind == TileKind::Selectable && state.selected {
            // `.cds--tile--is-selected { border: 1px solid layer-selected-inverse }`;
            // disabled `border-disabled`.
            Some(c(if state.disabled {
                BorderDisabled
            } else {
                LayerSelectedInverse
            }))
        } else if variant == CardVariant::Outline {
            // `.cu-card--outline { outline: 1px solid var(--border-tile-01); outline-offset: -1px }`.
            Some(c(BorderTile01))
        } else {
            None
        };
        let icon = match kind {
            // `.cds--tile--disabled .cds--tile__checkmark svg { fill: icon-disabled }`.
            TileKind::Selectable if state.disabled => c(IconDisabled),
            // `.cds--tile--is-selected .cds--tile__checkmark svg { fill: icon-primary }`.
            TileKind::Selectable if state.selected => c(IconPrimary),
            // `.cds--tile__checkmark svg { fill: var(--icon-secondary) }`.
            TileKind::Selectable => c(IconSecondary),
            // `.cds--tile__chevron svg { fill: var(--icon-primary) }`.
            _ => c(IconPrimary),
        };
        Look {
            background,
            border,
            text: c(if state.disabled {
                TextDisabled
            } else {
                TextPrimary
            }),
            icon,
            // `.cds--tile--clickable:focus { outline: 2px solid focus; outline-offset: -2px }`.
            focus: (interactive && state.focused).then(|| c(Focus)),
        }
    }
}

/// How a tile draws its 2px inset focus ring.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FocusRing {
    /// The tile's own 1px border takes the focus color (the outline covers it).
    pub border_takes_focus: bool,
    /// The ring's width inside the border, on a layer over the padding box.
    pub inner_width: f32,
}

/// The focus ring of a `kind` tile. The outline (2px, offset -2px) starts at
/// the border edge; a selectable tile has a real 1px border
/// (`.cds--tile--selectable { border: 1px solid transparent }`), which GPUI
/// paints over the children, so the border itself turns `focus` and the
/// layer inside adds the second pixel. No other border shows outside it.
pub fn focus_ring(kind: TileKind) -> FocusRing {
    if kind == TileKind::Selectable {
        FocusRing {
            border_takes_focus: true,
            inner_width: 1.,
        }
    } else {
        FocusRing {
            border_takes_focus: false,
            inner_width: 2.,
        }
    }
}

/// The checkbox glyph of a selectable tile (`SelectableTile`:
/// `selected ? CheckboxCheckedFilled : Checkbox`).
pub fn checkmark_icon(selected: bool) -> CarbonIcon {
    if selected {
        CarbonIcon::CheckboxCheckedFilled
    } else {
        CarbonIcon::Checkbox
    }
}

type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;
type BoolHandler = Rc<dyn Fn(bool, &mut Window, &mut App)>;

/// A Carbon tile. See the module docs.
#[derive(IntoElement)]
pub struct Card {
    id: ElementId,
    variant: CardVariant,
    level: LayerLevel,
    href: Option<SharedString>,
    on_click: Option<ClickHandler>,
    selectable: bool,
    selected: Option<bool>,
    default_selected: bool,
    on_selected_change: Option<BoolHandler>,
    below: Option<AnyElement>,
    expanded: Option<bool>,
    default_expanded: bool,
    on_expanded_change: Option<BoolHandler>,
    disabled: bool,
    label: SharedString,
    style: StyleRefinement,
    children: Vec<AnyElement>,
}

impl Styled for Card {
    /// Sizing and placement of the tile (the reference's `style={{ width: 300 }}`).
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl Card {
    /// A static tile with a stable `id`.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            variant: CardVariant::Default,
            level: LayerLevel::One,
            href: None,
            on_click: None,
            selectable: false,
            selected: None,
            default_selected: false,
            on_selected_change: None,
            below: None,
            expanded: None,
            default_expanded: false,
            on_expanded_change: None,
            disabled: false,
            label: SharedString::default(),
            style: StyleRefinement::default(),
            children: Vec::new(),
        }
    }

    /// `default` or `outline`.
    pub fn variant(mut self, variant: CardVariant) -> Self {
        self.variant = variant;
        self
    }

    /// The layer the tile sits on (it paints `layer` at this level).
    pub fn layer(mut self, level: LayerLevel) -> Self {
        self.level = level;
        self
    }

    /// A link target: the tile becomes a `ClickableTile` that opens `href`.
    pub fn href(mut self, href: impl Into<SharedString>) -> Self {
        self.href = Some(href.into());
        self
    }

    /// A click handler: the tile becomes a `ClickableTile`.
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    /// Make the tile a `SelectableTile`.
    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    /// Controlled selection.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = Some(selected);
        self
    }

    /// Initial selection when uncontrolled.
    pub fn default_selected(mut self, selected: bool) -> Self {
        self.default_selected = selected;
        self
    }

    /// Called with the new selection.
    pub fn on_selected_change(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_selected_change = Some(Rc::new(f));
        self
    }

    /// Make the tile an `ExpandableTile` whose below-the-fold content is `below`.
    pub fn expandable(mut self, below: impl IntoElement) -> Self {
        self.below = Some(below.into_any_element());
        self
    }

    /// Controlled expansion.
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = Some(expanded);
        self
    }

    /// Initial expansion when uncontrolled.
    pub fn default_expanded(mut self, expanded: bool) -> Self {
        self.default_expanded = expanded;
        self
    }

    /// Called with the new expansion.
    pub fn on_expanded_change(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_expanded_change = Some(Rc::new(f));
        self
    }

    /// Disable a selectable tile.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Accessible name of an interactive tile.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    /// The tile kind these props draw.
    pub fn kind(&self) -> TileKind {
        if self.href.is_some() || self.on_click.is_some() {
            TileKind::Clickable
        } else if self.selectable {
            TileKind::Selectable
        } else if self.below.is_some() {
            TileKind::Expandable
        } else {
            TileKind::Static
        }
    }
}

impl ParentElement for Card {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Card {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let kind = self.kind();
        let id = self.id.clone();
        let hovered = window.use_keyed_state((id.clone(), "hover"), cx, |_, _| false);
        let focus = window
            .use_keyed_state((id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let (selected, selected_store) = controlled(
            (id.clone(), "selected"),
            self.selected,
            self.default_selected,
            window,
            cx,
        );
        let (expanded, expanded_store) = controlled(
            (id.clone(), "expanded"),
            self.expanded,
            self.default_expanded,
            window,
            cx,
        );
        let state = TileState {
            hovered: *hovered.read(cx),
            focused: focus.is_focused(window),
            selected: kind == TileKind::Selectable && selected,
            disabled: self.disabled && kind == TileKind::Selectable,
        };
        let look = Look::of(kind, self.variant, state, self.level);
        let background =
            motion::animate_color((id.clone(), "bg"), look.background, HOVER, window, cx);
        let border = look.border.map(|target| {
            motion::animate_color((id.clone(), "border"), target, DETAIL, window, cx)
        });
        let icon_color = motion::animate_color((id.clone(), "icon"), look.icon, DETAIL, window, cx);

        // `.cu-card { display: flex; flex-direction: column; gap: var(--spacing-05) }`.
        let body = div()
            .flex()
            .flex_col()
            .flex_1()
            .gap(SPACING_05)
            .children(self.children);
        let mut tile = div()
            .id((id.clone(), "tile"))
            .relative()
            .flex()
            .flex_col()
            .min_h(rem(MIN_HEIGHT))
            .min_w(rem(MIN_WIDTH))
            .p(rem(PADDING))
            .bg(background)
            .text_color(look.text)
            .type_style(TypeStyle::BodyCompact01)
            .refine_style(&self.style)
            .map(|this| match (kind, border) {
                // `.cds--tile--selectable { border: 1px solid transparent }`: a real border.
                (TileKind::Selectable, border) => this
                    .border_1()
                    .border_color(border.unwrap_or_else(Hsla::transparent_black)),
                // `outline: 1px solid …; outline-offset: -1px` takes no layout space.
                (_, Some(border)) => this.child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .border_1()
                        .border_color(border),
                ),
                (_, None) => this,
            })
            .child(body);

        if kind == TileKind::Selectable {
            tile = tile.pr(rem(AFFORDANCE)).child(
                // `.cds--tile__checkmark { inset-block-start: pad; inset-inline-end: pad; block-size: 1rem }`.
                div()
                    .absolute()
                    .top(rem(PADDING))
                    .right(rem(PADDING))
                    .size(rem(16.))
                    .child(
                        Icon::carbon(checkmark_icon(selected))
                            .size(IconSize::S16)
                            .color(icon_color),
                    ),
            );
        }
        if kind == TileKind::Expandable {
            let rotation = motion::chevron_rotation((id.clone(), "chevron"), expanded, window, cx);
            let progress = super::overlay::presence(
                (id.clone(), "fold"),
                expanded,
                EXPAND,
                COLLAPSE,
                window,
                cx,
            );
            let reduced = motion::reduced_motion(cx);
            if let Some(below) = self.below
                && (expanded || progress > 0.)
            {
                tile = tile.child(
                    gpui_kit::base::Collapsible::new()
                        .open(true)
                        .reveal(
                            (id.clone(), "fold-reveal"),
                            if reduced {
                                if expanded { 1. } else { 0. }
                            } else {
                                progress
                            },
                        )
                        .content(div().pt(SPACING_05).opacity(progress).child(below)),
                );
            }
            // Room for the chevron box under the content.
            tile = tile.pb(rem(AFFORDANCE)).child(
                div()
                    .absolute()
                    .bottom_0()
                    .right_0()
                    .size(rem(AFFORDANCE))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        Icon::carbon(CarbonIcon::ChevronDown)
                            .size(IconSize::S16)
                            .color(icon_color)
                            .rotate(gpui_kit::radians(rotation)),
                    ),
            );
        }
        if let Some(focus_color) = look.focus {
            // `.cds--tile:focus { outline: 2px solid focus; outline-offset: -2px }`
            // runs from the border edge inward, over a selectable tile's border.
            let ring = focus_ring(kind);
            tile = tile
                .when(ring.border_takes_focus, |this| {
                    this.border_color(focus_color)
                })
                .child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .border(px(ring.inner_width))
                        .border_color(focus_color),
                );
        }
        if kind == TileKind::Static {
            return tile.into_any_element();
        }

        let disabled = self.disabled && kind == TileKind::Selectable;
        let on_click = self.on_click.clone();
        let href = self.href.clone();
        let on_selected = self.on_selected_change.clone();
        let on_expanded = self.on_expanded_change.clone();
        let controlled_selection = self.selected.is_some();
        let controlled_expansion = self.expanded.is_some();
        let activator = gpui_kit::base::Button::new((id.clone(), "activate"))
            .track_focus(&focus)
            .disabled(disabled)
            .accessibility_label(self.label.clone())
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .cursor(if disabled {
                CursorStyle::OperationNotAllowed
            } else {
                CursorStyle::PointingHand
            })
            .on_hover(move |h, _, cx| {
                hovered.update(cx, |v, cx| {
                    if *v != *h {
                        *v = *h;
                        cx.notify();
                    }
                })
            })
            .on_click(move |event, window, cx| match kind {
                TileKind::Clickable => {
                    if let Some(handler) = &on_click {
                        handler(event, window, cx);
                    }
                    if let Some(href) = &href {
                        cx.open_url(href);
                    }
                }
                TileKind::Selectable => {
                    if !controlled_selection {
                        selected_store.update(cx, |s, cx| {
                            *s = !selected;
                            cx.notify();
                        });
                    }
                    if let Some(f) = &on_selected {
                        f(!selected, window, cx);
                    }
                }
                TileKind::Expandable => {
                    if !controlled_expansion {
                        expanded_store.update(cx, |s, cx| {
                            *s = !expanded;
                            cx.notify();
                        });
                    }
                    if let Some(f) = &on_expanded {
                        f(!expanded, window, cx);
                    }
                }
                TileKind::Static => {}
            });
        // The Base button covers the tile so the whole surface is the target
        // (Carbon renders the tile itself as the `<a>`/`<button>`).
        tile.child(activator).into_any_element()
    }
}

/// `CardHeader`: title and description in the first column, an optional
/// [`CardHeader::action`] in the second spanning both rows
/// (`.cu-card-header { display: grid; grid-template-columns: 1fr auto; gap:
/// var(--spacing-02) var(--spacing-05) }`).
#[derive(IntoElement, Default)]
pub struct CardHeader {
    children: Vec<AnyElement>,
    action: Option<AnyElement>,
}

impl CardHeader {
    /// An empty header.
    pub fn new() -> Self {
        Self::default()
    }

    /// `CardAction`: the trailing element (`.cu-card-action { grid-column: 2;
    /// grid-row: 1 / span 2 }`).
    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.action = Some(action.into_any_element());
        self
    }
}

impl ParentElement for CardHeader {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for CardHeader {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .items_start()
            .gap(SPACING_05)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .gap(SPACING_02)
                    .children(self.children),
            )
            .when_some(self.action, |this, action| {
                this.child(div().flex_none().child(action))
            })
    }
}

/// `CardTitle`: `.cu-card-title { font-size: 1.25rem; line-height: 1.4;
/// font-weight: 400; color: var(--text-primary) }` (heading-03).
#[derive(IntoElement)]
pub struct CardTitle(SharedString);

impl CardTitle {
    /// A title.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self(text.into())
    }
}

impl RenderOnce for CardTitle {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .type_style(TypeStyle::Heading03)
            .text_color(color(ColorRole::TextPrimary))
            .child(self.0)
    }
}

/// `CardDescription`: `.cu-card-desc { font-size: 0.875rem; line-height:
/// 1.42857; color: var(--text-secondary) }` (body-01).
#[derive(IntoElement)]
pub struct CardDescription(SharedString);

impl CardDescription {
    /// A description.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self(text.into())
    }
}

impl RenderOnce for CardDescription {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .type_style(TypeStyle::Body01)
            .text_color(color(ColorRole::TextSecondary))
            .child(self.0)
    }
}

/// `CardAction`: a wrapper for the header's trailing element (pass it to
/// [`CardHeader::action`]).
#[derive(IntoElement, Default)]
pub struct CardAction {
    children: Vec<AnyElement>,
}

impl CardAction {
    /// An empty action slot.
    pub fn new() -> Self {
        Self::default()
    }
}

impl ParentElement for CardAction {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for CardAction {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div().flex().flex_row().children(self.children)
    }
}

/// `CardContent`: `.cu-card-content { font-size: 0.875rem; line-height: 1.42857 }` (body-01).
#[derive(IntoElement, Default)]
pub struct CardContent {
    children: Vec<AnyElement>,
}

impl CardContent {
    /// An empty content block.
    pub fn new() -> Self {
        Self::default()
    }
}

impl ParentElement for CardContent {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for CardContent {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .type_style(TypeStyle::Body01)
            .flex()
            .flex_col()
            .children(self.children)
    }
}

/// `CardFooter`: `.cu-card-footer { display: flex; gap: var(--spacing-03);
/// align-items: center; margin-top: auto }`.
#[derive(IntoElement, Default)]
pub struct CardFooter {
    children: Vec<AnyElement>,
}

impl CardFooter {
    /// An empty footer.
    pub fn new() -> Self {
        Self::default()
    }
}

impl ParentElement for CardFooter {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for CardFooter {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .mt_auto()
            .flex()
            .flex_row()
            .items_center()
            .gap(SPACING_03)
            .children(self.children)
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
    fn tile_states_follow_bundle_css() {
        crate::theme::tests_set(CarbonTheme::White);
        let st = |hovered, focused, selected, disabled| TileState {
            hovered,
            focused,
            selected,
            disabled,
        };
        let rest = Look::of(
            TileKind::Static,
            CardVariant::Default,
            st(true, true, false, false),
            LayerLevel::One,
        );
        // `.cds--tile { background: layer }`: static tiles ignore hover and focus.
        assert_eq!(hex(rest.background), "#f4f4f4");
        assert!(rest.focus.is_none());
        let hover = Look::of(
            TileKind::Clickable,
            CardVariant::Default,
            st(true, true, false, false),
            LayerLevel::One,
        );
        // `.cds--tile--clickable:hover { background: layer-hover #e8e8e8 }`, focus 2px `focus`.
        assert_eq!(hex(hover.background), "#e8e8e8");
        assert_eq!(hover.focus.map(hex).as_deref(), Some("#0f62fe"));
        let selected = Look::of(
            TileKind::Selectable,
            CardVariant::Default,
            st(false, false, true, false),
            LayerLevel::One,
        );
        // `.cds--tile--is-selected { border: 1px solid layer-selected-inverse #161616 }`.
        assert_eq!(selected.border.map(hex).as_deref(), Some("#161616"));
        assert_eq!(hex(selected.icon), "#161616");
        let unselected = Look::of(
            TileKind::Selectable,
            CardVariant::Default,
            st(false, false, false, false),
            LayerLevel::One,
        );
        assert_eq!(hex(unselected.icon), "#525252");
        let outline = Look::of(
            TileKind::Static,
            CardVariant::Outline,
            TileState::default(),
            LayerLevel::One,
        );
        // `.cu-card--outline { background: background; outline: 1px solid border-tile-01 #c6c6c6 }`.
        assert_eq!(hex(outline.background), "#ffffff");
        assert_eq!(outline.border.map(hex).as_deref(), Some("#c6c6c6"));
        let disabled = Look::of(
            TileKind::Selectable,
            CardVariant::Default,
            st(true, false, true, true),
            LayerLevel::One,
        );
        assert_eq!(hex(disabled.background), "#f4f4f4");
        assert_eq!(disabled.border.map(hex).as_deref(), Some("#c6c6c6"));
    }

    #[test]
    fn geometry_and_motion() {
        assert_eq!(
            (PADDING, MIN_HEIGHT, MIN_WIDTH, AFFORDANCE),
            (16., 64., 128., 48.)
        );
        assert_eq!((HOVER.speed.millis(), DETAIL.speed.millis()), (150, 110));
        assert_eq!((EXPAND.speed.millis(), COLLAPSE.speed.millis()), (110, 150));
        assert_eq!(checkmark_icon(true), CarbonIcon::CheckboxCheckedFilled);
        let card = Card::new("c").selectable(true);
        assert_eq!(card.kind(), TileKind::Selectable);
        assert_eq!(Card::new("c").href("#").kind(), TileKind::Clickable);
        assert_eq!(
            Card::new("c").expandable(div()).kind(),
            TileKind::Expandable
        );
    }

    #[test]
    fn focus_ring_covers_the_selectable_border() {
        // Border (1px) + inner layer (1px) = the 2px outline from the edge.
        let sel = focus_ring(TileKind::Selectable);
        assert!(sel.border_takes_focus);
        assert_eq!(sel.inner_width + 1., 2.);
        let click = focus_ring(TileKind::Clickable);
        assert!(!click.border_takes_focus);
        assert_eq!(click.inner_width, 2.);
    }
}
