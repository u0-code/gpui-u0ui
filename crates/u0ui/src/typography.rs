//! IBM Plex fonts and Carbon type styles.
//!
//! [`crate::init`] registers the bundled IBM Plex Sans (Light 300, Regular 400,
//! SemiBold 600, Italic 400), IBM Plex Mono (Regular 400, SemiBold 600) and IBM
//! Plex Serif (Light 300, Regular 400; `.cu-typeset blockquote`), so nothing
//! depends on what the machine has installed. Chinese/Japanese text falls back
//! through `tokens.json`'s stack: PingFang SC → Hiragino Sans GB → Noto Sans CJK
//! SC (mono: PingFang SC → Noto Sans Mono CJK SC).
//!
//! Apply a type style to any styled element:
//!
//! ```ignore
//! use u0ui::prelude::*;
//! div().type_style(TypeStyle::BodyCompact01).child("Create project")
//! ```
//!
//! This sets family (with fallbacks), size (rems, so zoom reflows), line height
//! (a multiplier of the size) and weight. **Letter spacing** (`0.16px` on
//! body-compact-01, `0.32px` on label-01) is not applied: GPUI's text system has
//! no tracking control. The value is kept in [`crate::tokens::TypeSpec`] for when
//! it does; at these sizes the difference is under half a pixel per glyph.
use crate::tokens::{FontFamilyKey, TypeStyle};
use gpui_kit::{App, Font, FontFallbacks, FontFeatures, FontStyle, FontWeight, Styled};
use std::borrow::Cow;

/// The bundled font files (IBM Plex, SIL OFL 1.1; see `assets/fonts/OFL.txt`).
pub const FONT_FILES: &[(&str, &[u8])] = &[
    (
        "IBMPlexSans-Light.ttf",
        include_bytes!("../assets/fonts/IBMPlexSans-Light.ttf"),
    ),
    (
        "IBMPlexSans-Regular.ttf",
        include_bytes!("../assets/fonts/IBMPlexSans-Regular.ttf"),
    ),
    (
        "IBMPlexSans-Italic.ttf",
        include_bytes!("../assets/fonts/IBMPlexSans-Italic.ttf"),
    ),
    (
        "IBMPlexSans-SemiBold.ttf",
        include_bytes!("../assets/fonts/IBMPlexSans-SemiBold.ttf"),
    ),
    (
        "IBMPlexMono-Regular.ttf",
        include_bytes!("../assets/fonts/IBMPlexMono-Regular.ttf"),
    ),
    (
        "IBMPlexMono-SemiBold.ttf",
        include_bytes!("../assets/fonts/IBMPlexMono-SemiBold.ttf"),
    ),
    (
        "IBMPlexSerif-Light.ttf",
        include_bytes!("../assets/fonts/IBMPlexSerif-Light.ttf"),
    ),
    (
        "IBMPlexSerif-Regular.ttf",
        include_bytes!("../assets/fonts/IBMPlexSerif-Regular.ttf"),
    ),
];

/// Register the bundled fonts with GPUI's text system (called by [`crate::init`]).
pub fn register_fonts(cx: &mut App) {
    let fonts = FONT_FILES
        .iter()
        .map(|(_, bytes)| Cow::Borrowed(*bytes))
        .collect();
    if let Err(error) = cx.text_system().add_fonts(fonts) {
        eprintln!("u0ui: failed to register IBM Plex fonts: {error}");
    }
}

/// A GPUI font for a Carbon family at a CSS weight, with the CJK fallback stack.
pub fn font(family: FontFamilyKey, weight: u16) -> Font {
    let stack = family.stack();
    Font {
        family: stack[0].into(),
        features: FontFeatures::default(),
        fallbacks: Some(FontFallbacks::from_fonts(
            stack[1..].iter().map(|s| s.to_string()).collect(),
        )),
        weight: FontWeight(weight as f32),
        style: FontStyle::Normal,
    }
}

/// Apply Carbon type styles to any [`Styled`] element.
pub trait TypeStyled: Styled + Sized {
    /// Set family (with CJK fallbacks), size, line height and weight from a
    /// Carbon type style. See the module docs about letter spacing.
    fn type_style(self, style: TypeStyle) -> Self {
        let spec = style.spec();
        self.font(font(spec.family, spec.weight))
            .text_size(spec.font_size())
            .line_height(spec.line_height())
    }
}

impl<T: Styled> TypeStyled for T {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fonts_are_bundled_and_stacks_fall_back_to_cjk() {
        assert_eq!(FONT_FILES.len(), 8);
        for (name, bytes) in FONT_FILES {
            assert!(bytes.len() > 100_000, "{name}");
        }
        let sans = font(FontFamilyKey::Sans, 600);
        assert_eq!(sans.family.as_ref(), "IBM Plex Sans");
        assert_eq!(sans.weight, FontWeight(600.));
        assert!(sans.fallbacks.is_some());
        // `.cu-typeset blockquote { font-family: var(--font-serif); font-weight: 300 }`.
        let serif = font(FontFamilyKey::Serif, 300);
        assert_eq!(serif.family.as_ref(), "IBM Plex Serif");
        assert!(
            FONT_FILES
                .iter()
                .any(|(n, _)| *n == "IBMPlexSerif-Light.ttf")
        );
    }
}
