//! Pieces the settings rows and editors share: the ghost `sm` buttons they
//! end with.
use crate::{
    components::button::{Button, ButtonSize, ButtonVariant},
    icons::CarbonIcon,
};
use gpui_kit::{App, ElementId, SharedString, Window, prelude::FluentBuilder as _};
use std::rc::Rc;

/// A click handler without the event.
pub(crate) type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

/// `Button variant="ghost" size="sm"` with an optional icon.
pub(crate) fn ghost_button(
    id: impl Into<ElementId>,
    label: SharedString,
    glyph: Option<CarbonIcon>,
    on_click: Option<Handler>,
) -> Button {
    Button::new(id, label)
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::Sm)
        .when_some(glyph, |b, g| b.icon(g))
        .when_some(on_click, |b, f| b.on_click(move |_, w, cx| f(w, cx)))
}
