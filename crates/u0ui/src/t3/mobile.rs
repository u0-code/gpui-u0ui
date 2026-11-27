//! T3 · Mobile patterns (`src/t3/mobile.jsx`, `src/t3.css` "Mobile"),
//! redrawn with Carbon tokens: [`PhoneFrame`], [`MobileAppBar`],
//! [`MobileListSection`] / [`ListRow`], [`MobileSheet`], [`MobileTabDock`],
//! [`SplitButton`], [`FloatingActionButton`], [`SwipeRow`], [`ControlPill`],
//! [`MobileNotice`], [`LoadingStrip`] and [`FloatingWorkingControl`].
//!
//! The craft pass makes every mobile surface square (`gemini-MA`). Boards
//! that opt into the shared phone chrome (`[data-mb]` on a 390×844 root)
//! get a flat full-width tab bar, 56px list rows, a Carbon-proportioned
//! switch, a square field-01 search and symmetric nav-bar slots; the
//! components take that as `.phone(true)`.
use crate::{theme::color, tokens::ColorRole};
use gpui_kit::{AnyElement, App, BoxShadow, Hsla, IntoElement, Styled, Window, div, px};
use std::rc::Rc;

pub(super) type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

/// `box-shadow: 0 2px 6px var(--shadow)`.
pub(super) fn float_shadow(y: f32) -> Vec<BoxShadow> {
    vec![BoxShadow::new(px(0.), px(y), color(ColorRole::Shadow)).blur_radius(px(6.))]
}

/// A 1px inset frame (`outline: 1px solid …; outline-offset: -1px`).
pub(super) fn inset_frame(c: Hsla) -> AnyElement {
    div()
        .absolute()
        .top_0()
        .left_0()
        .right_0()
        .bottom_0()
        .border_1()
        .border_color(c)
        .into_any_element()
}

pub mod chrome;
pub mod controls;

pub use chrome::{
    BarAction, DockTab, Leading, ListRow, MobileAppBar, MobileListSection, MobileSheet,
    MobileTabDock, PhoneFrame, RowKind, home_indicator, in_phone_screen, screen_bottom_radius,
};
pub use controls::{
    ControlPill, FloatingActionButton, FloatingWorkingControl, LoadingStrip, MobileNotice,
    NoticeTone, Revealed, SplitButton, SplitKind, SwipeAction, SwipeRow, SwipeTone,
};
