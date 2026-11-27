//! Pieces the settings rows share (`src/ghost.css` "admin"): the row frame
//! `.g-task, .g-wfrow, .g-slrow, .g-agent, .g-acct, .g-script, …`, its
//! title/subtitle column `.g-slrow-main`, and the ghost `sm` buttons the rows
//! end with.
use crate::{
    components::button::{Button, ButtonSize, ButtonVariant},
    icons::CarbonIcon,
    t3::craft::{rem, sans},
    theme::color,
    tokens::{self, ColorRole},
};
use gpui_kit::{
    App, Div, ElementId, IntoElement, ParentElement, SharedString, Styled, Window, div,
    prelude::FluentBuilder as _,
};
use std::rc::Rc;

/// A click handler without the event.
pub(crate) type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

/// The row frame (`.g-slrow` shares `.g-task`'s rule): Ghost · Tasks' frame.
pub(crate) use crate::ghost::tasks::row::frame;

/// `.g-slrow-main`: the title (`b`, weight 400, one ellipsised line) over an
/// optional 12px subtitle (`.g-slrow-main > .g-dim { font-size: 0.75rem }`,
/// `text-secondary`), `gap: 2px`, taking the free width.
pub(crate) fn main(title: impl IntoElement, subtitle: Option<Div>) -> Div {
    div()
        .flex()
        .flex_col()
        .flex_1()
        .min_w_0()
        .gap(rem(2.))
        .child(
            div()
                .flex()
                .items_center()
                .gap(tokens::SPACING_03)
                .overflow_hidden()
                .whitespace_nowrap()
                .text_ellipsis()
                .child(title),
        )
        .when_some(subtitle, |this, s| this.child(s))
}

/// A 12px `text-secondary` subtitle line (`.g-dim` in `.g-slrow-main`).
pub(crate) fn subtitle(text: impl Into<SharedString>) -> Div {
    sans(div(), 12., 400)
        .text_color(color(ColorRole::TextSecondary))
        .overflow_hidden()
        .whitespace_nowrap()
        .text_ellipsis()
        .child(text.into())
}

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

/// `Cb.IconButton kind="ghost" size="sm" label=…`: icon-only, the label as
/// its tooltip and accessible name.
pub(crate) fn icon_button(
    id: impl Into<ElementId>,
    label: SharedString,
    glyph: CarbonIcon,
    on_click: Option<Handler>,
) -> Button {
    ghost_button(id, label, Some(glyph), on_click).icon_only(true)
}
