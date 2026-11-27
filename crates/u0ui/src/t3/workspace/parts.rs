//! Small pieces shared by the workspace cards: the reference's ghost
//! `Cb.IconButton`, `title` tooltips and the `.t3-card-sub` caption.
use crate::{
    components::{Button, ButtonSize, ButtonVariant},
    icons::Icon,
    t3::craft::sans,
    theme::color,
    tokens::ColorRole,
};
use gpui_kit::{App, Div, ElementId, ParentElement, SharedString, Styled, Window, div};
use std::rc::Rc;

/// A handler given an index.
pub(crate) type IndexHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;
/// A handler given a string (a path, an id, a URL).
pub(crate) type TextHandler = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;

/// `<Cb.IconButton kind="ghost" size="sm" label=…>`: a 32px ghost icon
/// button whose label is its tooltip.
pub(crate) fn icon_button(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    icon: impl Into<Icon>,
) -> Button {
    Button::new(id, label)
        .icon(icon)
        .icon_only(true)
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::Sm)
}

/// A native `title` tooltip (Kit's plain tooltip).
pub(crate) fn title_tooltip(
    text: SharedString,
) -> impl Fn(&mut Window, &mut App) -> gpui_kit::AnyView {
    move |window, cx| gpui_kit::component::tooltip::Tooltip::new(text.clone()).build(window, cx)
}

/// `.t3-card-sub { font-size: 0.75rem; color: var(--text-secondary) }`.
pub(crate) fn card_sub(text: impl Into<SharedString>) -> Div {
    sans(div(), 12., 400)
        .text_color(color(ColorRole::TextSecondary))
        .child(text.into())
}

/// An element id scoped under `id` with an index (`(id, "row-3")`).
pub(crate) fn child_id(id: &ElementId, part: &str, i: usize) -> ElementId {
    ElementId::from((id.clone(), SharedString::from(format!("{part}-{i}"))))
}
