//! CommandBox: the command centre in a window's top bar, as VS Code's
//! and Zed's (`.cmdbox*`, the u0 design's `CommandBox` board). It looks
//! like a search field and is a button: a press opens the app's command
//! palette under it, which owns the typing.
//!
//! - **Appearance**: 28px high, `field-01` fill inside a 1px
//!   `border-subtle-01` line, `padding: 0 4px 0 8px`, `gap: 8px`; the 16px
//!   magnifier in `icon-secondary`, the words in `label-01` (12/16)
//!   `text-secondary`, one line cut with an ellipsis, the key as a [`Kbd`]
//!   at the end. Hover `field-hover-01`. While its palette is open
//!   ([`CommandBox::open`]), and with keyboard focus, the line is the 2px
//!   `focus` ring. The words and key are its tooltip, whole.
//! - **Narrow**: below 160px wide (`@container (max-width: 160px)`) only
//!   the magnifier is left, in `icon-primary`: a 32×32 ghost button
//!   (`background-hover` on hover), centred where the box was.
//! - **Width**: it fills what its parent gives it; in a bar the parent is
//!   `flex: 0 1 400px; min-width: 32px` between two `flex: 1 1 0` sides.
//!
//! ```ignore
//! CommandBox::new("command-box", "Search threads, projects, commands and settings")
//!     .shortcut("⌘K")
//!     .open(palette_open)
//!     .on_click(|window, cx| open_palette(window, cx))
//! ```
use crate::{
    components::{form, kbd::Kbd},
    icons::{CarbonIcon, Icon},
    t3::craft::{rem, sans},
    theme::color,
    tokens::{self, ColorRole},
    u0::primitives::parts::{focus_ring, focus_visible, keyed_focus},
};
use gpui_kit::{
    App, Bounds, CursorStyle, ElementId, Hsla, IntoElement, ParentElement, Pixels, RenderOnce,
    SharedString, StatefulInteractiveElement as _, Styled, Window, canvas, div,
    prelude::FluentBuilder as _,
};
use std::rc::Rc;

/// `.cmdbox-b { height: 28px }`.
pub const HEIGHT: f32 = 28.;
/// `@container (max-width: 160px) { .cmdbox-b { height: 32px } }`.
pub const ICON_HEIGHT: f32 = 32.;
/// `@container (max-width: 160px)`: at this width and below, the magnifier
/// alone.
pub const NARROW: f32 = 160.;

/// Whether a box `width` wide shows only its magnifier.
pub fn narrow(width: f32) -> bool {
    width <= NARROW
}

/// The box's colors in one state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CommandBoxLook {
    /// `field-01`, `field-hover-01` on hover; narrow: none, `background-hover`.
    pub background: Hsla,
    /// The inset line: `border-subtle-01`; open, `focus`; narrow, none.
    pub line: Hsla,
    /// The line's width: 1px, 2px while open.
    pub line_width: f32,
    /// `text-secondary`.
    pub text: Hsla,
    /// `icon-secondary`; narrow, `icon-primary`.
    pub magnifier: Hsla,
}

impl CommandBoxLook {
    /// The look `narrow` or not, `open` or not, `hovered` or not.
    pub fn of(narrow: bool, open: bool, hovered: bool) -> Self {
        let background = match (narrow, open || !hovered) {
            (false, true) => color(ColorRole::Field01),
            (false, false) => color(ColorRole::FieldHover01),
            (true, _) if hovered => color(ColorRole::BackgroundHover),
            (true, _) => Hsla::transparent_black(),
        };
        let (line, line_width) = match (narrow, open) {
            (true, _) => (Hsla::transparent_black(), 0.),
            (false, true) => (color(ColorRole::Focus), 2.),
            (false, false) => (color(ColorRole::BorderSubtle01), 1.),
        };
        Self {
            background,
            line,
            line_width,
            text: color(ColorRole::TextSecondary),
            magnifier: color(if narrow {
                ColorRole::IconPrimary
            } else {
                ColorRole::IconSecondary
            }),
        }
    }
}

/// The tooltip (`title`): the words, then the key in brackets.
pub fn tip(label: &str, shortcut: Option<&str>) -> SharedString {
    match shortcut {
        Some(key) if !key.is_empty() => format!("{label} ({key})").into(),
        _ => label.to_owned().into(),
    }
}

/// The command box. See the module docs.
#[derive(IntoElement)]
pub struct CommandBox {
    id: ElementId,
    label: SharedString,
    shortcut: Option<SharedString>,
    open: bool,
    on_click: Option<super::super::settings::row::Handler>,
}

impl CommandBox {
    /// A box with a stable `id`, reading `label` (also its accessible name).
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            shortcut: None,
            open: false,
            on_click: None,
        }
    }

    /// The key that opens the palette, shown at the end (`⌘K`).
    pub fn shortcut(mut self, shortcut: impl Into<SharedString>) -> Self {
        self.shortcut = Some(shortcut.into()).filter(|s| !s.is_empty());
        self
    }

    /// Whether its palette is open under it (`aria-expanded`).
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// What a press (or Enter / Space with focus) does: open the palette.
    pub fn on_click(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for CommandBox {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id;
        let focus = keyed_focus((id.clone(), "focus"), window, cx);
        let ring = focus_visible(&focus, window);
        let hovered = window.use_keyed_state((id.clone(), "hover"), cx, |_, _| false);
        // The container query: the width it was given last, measured as it
        // is laid out; a change across `NARROW` draws it again.
        let thin = window.use_keyed_state((id.clone(), "narrow"), cx, |_, _| false);
        let is_narrow = *thin.read(cx);
        let look = CommandBoxLook::of(is_narrow, self.open, *hovered.read(cx));
        let tip = tip(&self.label, self.shortcut.as_deref());
        let on_click = self.on_click;

        let measure = canvas(
            move |bounds: Bounds<Pixels>, _, cx| {
                let now = narrow(f32::from(bounds.size.width));
                thin.update(cx, |was, cx| {
                    if *was != now {
                        *was = now;
                        cx.notify();
                    }
                });
            },
            |_, _, _, _| {},
        )
        .absolute()
        .size_full();

        let button = sans(gpui_kit::base::Button::new(id), 12., 400)
            .track_focus(&focus)
            .aria_label(tip.clone())
            .relative()
            .flex()
            .items_center()
            .h(rem(if is_narrow { ICON_HEIGHT } else { HEIGHT }))
            .bg(look.background)
            .text_color(look.text)
            .cursor(CursorStyle::PointingHand)
            .tooltip(move |window, cx| {
                gpui_kit::component::tooltip::Tooltip::new(tip.clone()).build(window, cx)
            })
            .on_hover(move |h, _, cx| {
                hovered.update(cx, |v, cx| {
                    if *v != *h {
                        *v = *h;
                        cx.notify();
                    }
                })
            })
            .when_some(on_click, |b, f| b.on_click(move |_, w, cx| f(w, cx)))
            .child(
                Icon::carbon(CarbonIcon::Search)
                    .size_length(rem(16.))
                    .color(look.magnifier),
            );
        let button = if is_narrow {
            button.flex_none().w(rem(ICON_HEIGHT)).justify_center()
        } else {
            button
                .w_full()
                .gap(tokens::SPACING_03)
                .pl(tokens::SPACING_03)
                .pr(tokens::SPACING_02)
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .overflow_hidden()
                        .text_ellipsis()
                        .whitespace_nowrap()
                        // `letter-spacing: .32px; line-height: 16px`: label-01.
                        .line_height(rem(16.))
                        .child(self.label),
                )
                .children(self.shortcut.map(Kbd::new))
                .when(look.line_width > 0., |b| {
                    b.child(form::inset_outline(look.line_width, look.line))
                })
        };
        div()
            .relative()
            .flex()
            .items_center()
            .justify_center()
            .w_full()
            .min_w(rem(ICON_HEIGHT))
            .child(measure)
            .child(button.when(ring, |b| b.child(focus_ring())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_box_reads_as_a_search_field() {
        let rest = CommandBoxLook::of(false, false, false);
        assert_eq!(rest.background, color(ColorRole::Field01));
        assert_eq!(
            (rest.line, rest.line_width),
            (color(ColorRole::BorderSubtle01), 1.)
        );
        assert_eq!(rest.text, color(ColorRole::TextSecondary));
        assert_eq!(rest.magnifier, color(ColorRole::IconSecondary));
        let hover = CommandBoxLook::of(false, false, true);
        assert_eq!(hover.background, color(ColorRole::FieldHover01));
    }

    #[test]
    fn an_open_box_has_the_focus_ring_on_its_field() {
        let open = CommandBoxLook::of(false, true, true);
        assert_eq!(open.background, color(ColorRole::Field01));
        assert_eq!((open.line, open.line_width), (color(ColorRole::Focus), 2.));
    }

    #[test]
    fn a_narrow_box_is_a_ghost_icon_button() {
        assert!(narrow(160.) && narrow(32.) && !narrow(161.));
        let rest = CommandBoxLook::of(true, false, false);
        assert_eq!(rest.background, Hsla::transparent_black());
        assert_eq!(rest.line_width, 0.);
        assert_eq!(rest.magnifier, color(ColorRole::IconPrimary));
        let hover = CommandBoxLook::of(true, false, true);
        assert_eq!(hover.background, color(ColorRole::BackgroundHover));
    }

    #[test]
    fn the_tip_names_the_key_when_there_is_one() {
        assert_eq!(tip("Search", Some("⌘K")).as_ref(), "Search (⌘K)");
        assert_eq!(tip("Search", Some("")).as_ref(), "Search");
        assert_eq!(tip("Search", None).as_ref(), "Search");
    }

    #[test]
    fn the_height_is_the_small_field() {
        assert_eq!(
            tokens::DENSITY_FIELD_SM,
            tokens::TokenLength::Rem(HEIGHT / tokens::REM_PX)
        );
    }
}
