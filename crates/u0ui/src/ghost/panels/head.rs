//! Parts the right-column viewers share (`src/ghost.css` "Right column"):
//! the `.g-codehead` bar above code, images, PDFs and prose code blocks, the
//! ghost `sm` icon button (`Cb.IconButton kind="ghost" size="sm"`), the
//! `.g-dim` run and `.g-sep` rule.
use crate::{
    components::{Button, ButtonSize, ButtonVariant},
    density,
    icons::{CarbonIcon, Icon},
    t3::craft::{mono, rem, sans},
    theme::color,
    tokens::{self, ColorRole},
};
use gpui_kit::{
    App, Div, ElementId, ParentElement, SharedString, Styled, Window, div,
    prelude::FluentBuilder as _,
};
use std::rc::Rc;

/// A click handler without the event.
pub type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

/// `.g-codehead { display: flex; align-items: center; gap: var(--spacing-03);
/// height: var(--density-row-md); padding: 0 var(--spacing-02) 0
/// var(--spacing-04); background: var(--layer-accent-01); font-size:
/// 0.75rem; color: var(--text-secondary) }`.
pub fn code_head() -> Div {
    sans(div(), 12., 400)
        .flex()
        .flex_none()
        .items_center()
        .gap(tokens::SPACING_03)
        .h(density::current().row())
        .pl(tokens::SPACING_04)
        .pr(tokens::SPACING_02)
        .bg(color(ColorRole::LayerAccent01))
        .text_color(color(ColorRole::TextSecondary))
        .overflow_hidden()
}

/// The head's 14px glyph (`.g-codehead svg { fill: currentColor; flex: none }`).
pub fn head_icon(name: CarbonIcon) -> Div {
    div().flex().flex_none().child(
        Icon::carbon(name)
            .size_length(rem(14.))
            .color(color(ColorRole::TextSecondary)),
    )
}

/// `.g-mono` in the head: IBM Plex Mono 12px, one line.
pub fn head_name(name: impl Into<SharedString>, role: ColorRole) -> Div {
    mono(div(), 12., 400)
        .min_w_0()
        .overflow_hidden()
        .whitespace_nowrap()
        .text_ellipsis()
        .text_color(color(role))
        .child(name.into())
}

/// `.g-dim { color: var(--text-secondary) }` at `size` px.
pub fn dim(text: impl Into<SharedString>, size: f32) -> Div {
    sans(div(), size, 400)
        .flex_none()
        .whitespace_nowrap()
        .text_color(color(ColorRole::TextSecondary))
        .child(text.into())
}

/// `.t3-grow { flex: 1 }`.
pub fn grow() -> Div {
    div().flex_1().min_w_0()
}

/// `.g-sep { width: 1px; height: 1rem; margin: 0 var(--spacing-02);
/// background: var(--border-subtle-01) }`.
pub fn sep() -> Div {
    div()
        .flex_none()
        .w(gpui_kit::px(1.))
        .h(rem(16.))
        .mx(tokens::SPACING_02)
        .bg(color(ColorRole::BorderSubtle01))
}

/// `Cb.IconButton kind="ghost" size="sm" label=…`: a 32px ghost icon button
/// with the label as its tooltip and accessible name.
pub fn ghost_icon(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    glyph: CarbonIcon,
    disabled: bool,
    on_click: Option<Handler>,
) -> Button {
    Button::new(id, label)
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::Sm)
        .icon(glyph)
        .icon_only(true)
        .disabled(disabled)
        .when_some(on_click, |b, f| b.on_click(move |_, w, cx| f(w, cx)))
}

/// `<Button variant="ghost" size="sm">label</Button>`.
pub fn ghost_text(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    on_click: Option<Handler>,
) -> Button {
    Button::new(id, label)
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::Sm)
        .when_some(on_click, |b, f| b.on_click(move |_, w, cx| f(w, cx)))
}
