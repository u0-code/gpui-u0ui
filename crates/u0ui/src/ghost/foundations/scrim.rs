//! Scrim: the veil behind dialogs and the command palette (`Scrim` in
//! `src/ghost/foundations.jsx`).
//!
//! - `scrim` (Carbon `overlay`) behind dialogs, confirmations and sheets;
//!   `scrim-palette`, lighter, behind ⌘K so the context stays visible.
//! - `.g-scrim`: covers its positioned parent (`position: absolute; inset:
//!   0; z-index: 20`), centers its content horizontally with 32px padding;
//!   `align="center"` centers it vertically, `align="top"` puts it 12vh from
//!   the top (the palette).
//! - A press on the scrim itself (not the content) calls `on_dismiss`.
use crate::{
    theme::color,
    tokens::{self, ColorRole},
};
use gpui_kit::{
    AnyElement, App, ElementId, Hsla, InteractiveElement as _, IntoElement, MouseButton,
    ParentElement, RenderOnce, Styled, Window, div, px,
};
use std::rc::Rc;

/// Which veil (`variant`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ScrimVariant {
    /// `dialog`: `scrim`.
    #[default]
    Dialog,
    /// `palette`: `scrim-palette`.
    Palette,
}

impl ScrimVariant {
    /// Both variants.
    pub const ALL: [ScrimVariant; 2] = [Self::Dialog, Self::Palette];

    /// The reference name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Dialog => "dialog",
            Self::Palette => "palette",
        }
    }

    /// The fill (`.g-scrim { background: var(--scrim) }`, `--palette`).
    pub fn color(self) -> Hsla {
        color(match self {
            Self::Dialog => ColorRole::Scrim,
            Self::Palette => ColorRole::ScrimPalette,
        })
    }
}

/// Where the content sits (`align`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ScrimAlign {
    /// `center`.
    #[default]
    Center,
    /// `top`: 12vh from the top.
    Top,
}

impl ScrimAlign {
    /// Both alignments.
    pub const ALL: [ScrimAlign; 2] = [Self::Center, Self::Top];

    /// The reference name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Center => "center",
            Self::Top => "top",
        }
    }
}

/// `padding-top: 12vh` for `align="top"`, from the viewport height.
pub fn top_offset(viewport_height: f32) -> f32 {
    viewport_height * 0.12
}

type Dismiss = Rc<dyn Fn(&mut Window, &mut App)>;

/// A scrim. See the module docs.
#[derive(IntoElement)]
pub struct Scrim {
    id: ElementId,
    variant: ScrimVariant,
    align: ScrimAlign,
    children: Vec<AnyElement>,
    on_dismiss: Option<Dismiss>,
}

impl Scrim {
    /// A dialog scrim, content centered.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            variant: ScrimVariant::Dialog,
            align: ScrimAlign::Center,
            children: Vec::new(),
            on_dismiss: None,
        }
    }

    /// The veil (`variant`).
    pub fn variant(mut self, variant: ScrimVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Content placement (`align`).
    pub fn align(mut self, align: ScrimAlign) -> Self {
        self.align = align;
        self
    }

    /// Called when the scrim itself is pressed.
    pub fn on_dismiss(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(f));
        self
    }
}

impl ParentElement for Scrim {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Scrim {
    fn render(self, window: &mut Window, _: &mut App) -> impl IntoElement {
        let viewport = f32::from(window.viewport_size().height);
        let dismiss = self.on_dismiss.clone();
        let content = div()
            .id((self.id.clone(), "content"))
            .flex()
            .flex_col()
            // Presses inside the content never reach the scrim
            // (`e.target === e.currentTarget`).
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .children(self.children);
        let scrim = div()
            .id(self.id.clone())
            .absolute()
            .inset_0()
            .flex()
            .justify_center()
            .p(tokens::SPACING_07)
            .bg(self.variant.color())
            .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                if let Some(f) = &dismiss {
                    f(window, cx);
                }
            })
            .child(content);
        match self.align {
            ScrimAlign::Center => scrim.items_center(),
            ScrimAlign::Top => scrim.items_start().pt(px(top_offset(viewport))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CarbonTheme, theme::color_in};

    #[test]
    fn variants_use_their_tokens() {
        crate::theme::tests_set(CarbonTheme::White);
        assert_eq!(
            ScrimVariant::Dialog.color(),
            color_in(CarbonTheme::White, ColorRole::Scrim)
        );
        assert_eq!(
            ScrimVariant::Palette.color(),
            color_in(CarbonTheme::White, ColorRole::ScrimPalette)
        );
        // The palette scrim is lighter.
        assert!(ScrimVariant::Palette.color().a < ScrimVariant::Dialog.color().a);
        assert_eq!(top_offset(900.), 108.);
    }
}
