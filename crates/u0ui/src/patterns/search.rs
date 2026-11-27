//! `Search` / `.cds--search`: Carbon's search field, the filter box of data
//! table toolbars and filter panels.
//!
//! - **Behavior** is Kit's: [`KitInput`] over the caller's [`InputState`]
//!   (editing, IME, selection, clipboard). The clear button empties it.
//! - **Appearance**: `field` fill, 1px `border-strong` bottom rule, 16px
//!   magnifier centred in a height-square at the start, the clear `×` in a
//!   height-square at the end (shown only when there is text), 2px inset
//!   `focus` outline. The label is visually hidden (`.cds--search
//!   .cds--label`), so it only names the field.
//!
//! ```ignore
//! PatternSearch::new("filter", &state)
//!     .label("Search")
//!     .placeholder("Filter resources")
//!     .close_label("Clear search")
//!     .size(FieldSize::Lg)
//! ```
use crate::{
    components::{
        form::{self, FieldSize, rem},
        input::FIELD_MOTION,
    },
    density::{self, Density},
    icons::{CarbonIcon, Icon, IconSize},
    motion,
    theme::{LayerLevel, color},
    tokens::{ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    App, CursorStyle, ElementId, Entity, Focusable as _, Hsla, InteractiveElement as _,
    IntoElement, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled,
    Window,
    component::{
        Sizable as _, Size,
        input::{Input as KitInput, InputState},
    },
    div,
    prelude::FluentBuilder as _,
};

/// The search field's measurements, in px at 100% zoom.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SearchGeometry {
    /// `.cds--search-input { block-size: var(--layout-size-height-local) }`;
    /// compact density sets it to `density-field-md`
    /// (`[data-density="compact"] .cds--search-input`).
    pub height: f32,
    /// `.cds--search-input { padding: 0 var(--layout-size-height-local) }`:
    /// text clears the magnifier and the clear button on both sides.
    pub padding_inline: f32,
    /// `.cds--search-magnifier-icon { inset-inline-start: calc((height - 1rem) / 2) }`.
    pub magnifier_inset: f32,
    /// `.cds--search-magnifier-icon { inline-size: 1rem }`.
    pub magnifier: f32,
    /// `.cds--search-close { inline-size: var(--layout-size-height-local) }`.
    pub close: f32,
}

impl SearchGeometry {
    /// The geometry of `size` under `density`.
    pub fn of(size: FieldSize, density: Density) -> Self {
        let height = size.height(density);
        Self {
            height,
            padding_inline: height,
            magnifier_inset: (height - 16.) / 2.,
            magnifier: 16.,
            close: height,
        }
    }
}

/// The search field's colors in one state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SearchLook {
    /// `.cds--search-input { background: var(--field) }`.
    pub background: Hsla,
    /// `border-block-end: 1px solid var(--border-strong)`; disabled transparent.
    pub rule: Hsla,
    /// `color: var(--text-primary)`; disabled `text-disabled`.
    pub text: Hsla,
    /// `::placeholder { color: var(--text-placeholder) }`.
    pub placeholder: Hsla,
    /// `.cds--search-magnifier-icon { fill: var(--icon-secondary) }`;
    /// disabled `icon-on-color-disabled` (`.cds--search--disabled svg`).
    pub magnifier: Hsla,
    /// `.cds--search-close { fill: var(--icon-primary) }`.
    pub close_icon: Hsla,
    /// `.cds--search-close:hover { background-color: var(--field-hover) }`.
    pub close_hover: Hsla,
    /// `.cds--search-close:active { background-color: var(--background-selected) }`.
    pub close_active: Hsla,
}

impl SearchLook {
    /// The look on `level`, `disabled` or not.
    pub fn of(level: LayerLevel, disabled: bool) -> Self {
        Self {
            background: color(level.field()),
            rule: if disabled {
                Hsla::transparent_black()
            } else {
                color(level.border_strong())
            },
            text: color(if disabled {
                ColorRole::TextDisabled
            } else {
                ColorRole::TextPrimary
            }),
            placeholder: color(ColorRole::TextPlaceholder),
            magnifier: color(if disabled {
                ColorRole::IconOnColorDisabled
            } else {
                ColorRole::IconSecondary
            }),
            close_icon: color(ColorRole::IconPrimary),
            close_hover: color(level.field_hover()),
            close_active: color(ColorRole::BackgroundSelected),
        }
    }
}

/// Whether the clear button shows: only with text, and never when disabled
/// (`.cds--search-close--hidden { opacity: 0; visibility: hidden }`).
pub fn shows_clear(empty: bool, disabled: bool) -> bool {
    !empty && !disabled
}

/// A Carbon search field. See the module docs.
#[derive(IntoElement)]
pub struct PatternSearch {
    id: ElementId,
    state: Entity<InputState>,
    label: Option<SharedString>,
    placeholder: Option<SharedString>,
    close_label: SharedString,
    size: FieldSize,
    density: Option<Density>,
    disabled: bool,
    level: LayerLevel,
}

impl PatternSearch {
    /// A search field with a stable `id` over the caller's `state`.
    pub fn new(id: impl Into<ElementId>, state: &Entity<InputState>) -> Self {
        Self {
            id: id.into(),
            state: state.clone(),
            label: None,
            placeholder: None,
            close_label: SharedString::default(),
            size: FieldSize::Md,
            density: None,
            disabled: false,
            level: LayerLevel::One,
        }
    }

    /// The accessible name (`labelText`, visually hidden).
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// `placeholder`, in `text-placeholder`.
    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    /// The clear button's accessible name (`closeButtonLabelText`).
    pub fn close_label(mut self, label: impl Into<SharedString>) -> Self {
        self.close_label = label.into();
        self
    }

    /// `size` (sm 32, md 40, lg 48).
    pub fn size(mut self, size: FieldSize) -> Self {
        self.size = size;
        self
    }

    /// Override the active density.
    pub fn density(mut self, density: Density) -> Self {
        self.density = Some(density);
        self
    }

    /// `disabled`.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// The layer the field sits on (`field-01` on layer one, …).
    pub fn layer(mut self, level: LayerLevel) -> Self {
        self.level = level;
        self
    }
}

impl RenderOnce for PatternSearch {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id;
        let g = SearchGeometry::of(self.size, self.density.unwrap_or_else(density::current));
        let look = SearchLook::of(self.level, self.disabled);
        let state = self.state.clone();
        crate::components::form::select_all_on_tab_in(&state, window, cx);
        let focus = state.read(cx).focus_handle(cx);
        let focused = focus.is_focused(window) && !self.disabled;
        let empty = state.read(cx).value().is_empty();

        // `transition: background-color 110ms, outline 110ms` on the input;
        // Carbon fields share the 70ms field motion in u0ui.
        let outline = motion::animate_color(
            (id.clone(), "outline"),
            if focused {
                color(ColorRole::Focus)
            } else {
                Hsla::transparent_black()
            },
            FIELD_MOTION,
            window,
            cx,
        );

        let editor = KitInput::new(&state)
            .appearance(false)
            .with_size(Size::XSmall)
            .disabled(self.disabled)
            .when_some(self.label.clone(), |this, label| this.aria_label(label))
            .p_0()
            .h_full()
            .flex_1()
            .min_w_0()
            .text_size(rem(14.))
            .line_height(rem(18.))
            .text_color(look.text);

        let placeholder = self.placeholder.filter(|_| empty).map(|text| {
            div()
                .absolute()
                .left(rem(g.padding_inline))
                .right(rem(g.padding_inline))
                .top_0()
                .bottom_0()
                .flex()
                .items_center()
                .overflow_hidden()
                .whitespace_nowrap()
                .text_color(look.placeholder)
                .child(text)
        });

        let magnifier = div()
            .absolute()
            .top_0()
            .bottom_0()
            .left(rem(g.magnifier_inset))
            .flex()
            .items_center()
            .child(
                Icon::carbon(CarbonIcon::Search)
                    .size(IconSize::S16)
                    .color(look.magnifier),
            );

        let clear = shows_clear(empty, self.disabled).then(|| {
            let close_focus = window
                .use_keyed_state((id.clone(), "close-focus"), cx, |_, cx| cx.focus_handle())
                .read(cx)
                .clone();
            let close_focused = close_focus.is_focused(window);
            let state = state.clone();
            gpui_kit::base::Button::new((id.clone(), "close"))
                .track_focus(&close_focus)
                .aria_label(self.close_label.clone())
                .absolute()
                .top_0()
                .right_0()
                .size(rem(g.close))
                .flex()
                .items_center()
                .justify_center()
                .cursor(CursorStyle::PointingHand)
                .hover(move |s| s.bg(look.close_hover))
                .active(move |s| s.bg(look.close_active))
                .child(
                    Icon::carbon(CarbonIcon::Close)
                        .size(IconSize::S16)
                        .color(look.close_icon),
                )
                // `.cds--search-close:focus { outline: 2px solid var(--focus); outline-offset: -2px }`.
                .when(close_focused, |this| {
                    this.child(form::inset_outline(2., color(ColorRole::Focus)))
                })
                .on_click(move |_, window, cx| {
                    state.update(cx, |s, cx| {
                        s.set_value("", window, cx);
                        s.focus(window, cx);
                    })
                })
        });

        div()
            .id(id.clone())
            .relative()
            .flex()
            .flex_1()
            .items_center()
            .w_full()
            .min_w_0()
            .h(rem(g.height))
            .pl(rem(g.padding_inline))
            .pr(rem(g.padding_inline))
            .bg(look.background)
            .type_style(TypeStyle::BodyCompact01)
            .when(self.disabled, |this| {
                this.cursor(CursorStyle::OperationNotAllowed)
            })
            .child(form::bottom_rule(look.rule))
            .children(placeholder)
            .child(editor)
            .child(magnifier)
            .children(clear)
            .child(form::inset_outline(2., outline))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry_follows_the_layout_height() {
        let lg = SearchGeometry::of(FieldSize::Lg, Density::Comfortable);
        assert_eq!((lg.height, lg.padding_inline, lg.close), (48., 48., 48.));
        assert_eq!(lg.magnifier_inset, 16.);
        let sm = SearchGeometry::of(FieldSize::Sm, Density::Comfortable);
        assert_eq!((sm.height, sm.magnifier_inset), (32., 8.));
        // Compact: every size takes `density-field-md`.
        let compact = SearchGeometry::of(FieldSize::Lg, Density::Compact);
        assert_eq!(compact.height, 32.);
    }

    #[test]
    fn look_matches_the_css() {
        let look = SearchLook::of(LayerLevel::One, false);
        assert_eq!(look.background, color(ColorRole::Field01));
        assert_eq!(look.rule, color(ColorRole::BorderStrong01));
        assert_eq!(look.magnifier, color(ColorRole::IconSecondary));
        assert_eq!(look.close_hover, color(ColorRole::FieldHover01));
        let off = SearchLook::of(LayerLevel::One, true);
        assert_eq!(off.rule, Hsla::transparent_black());
        assert_eq!(off.text, color(ColorRole::TextDisabled));
        assert_eq!(off.magnifier, color(ColorRole::IconOnColorDisabled));
    }

    #[test]
    fn clear_shows_only_with_text() {
        assert!(shows_clear(false, false));
        assert!(!shows_clear(true, false));
        assert!(!shows_clear(false, true));
    }
}
