//! IBM Plex fonts and Carbon type styles.
//!
//! [`crate::init`] registers the bundled IBM Plex Sans (Light 300, Regular 400,
//! SemiBold 600, Italic 400), IBM Plex Mono (Regular 400, Italic 400 for
//! `.g-syn-comment`, SemiBold 600) and IBM
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
//!
//! **Inline code.** GPUI text runs cannot change size, so the 13px inline
//! code of 14px prose uses a derived face, [`INLINE_CODE_FAMILY`] (see
//! [`inline_code_font`] and `inline_face.rs`).
use crate::tokens::{FontFamilyKey, TypeStyle};
use gpui_kit::{App, Div, Font, FontFallbacks, FontFeatures, FontStyle, FontWeight, Styled, div};
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
        // IBM/plex v6.4.0 `IBM-Plex-Mono/fonts/complete/ttf`, the release the
        // other Plex files come from.
        "IBMPlexMono-Italic.ttf",
        include_bytes!("../assets/fonts/IBMPlexMono-Italic.ttf"),
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

mod inline_face;

/// The family of the inline-code face: IBM Plex Mono Regular whose glyphs
/// draw at 13/14 of the run's size, so a run of it in a 14px paragraph is
/// 13px code (`.g-prose code, .t3-amsg-body code { font-size: 0.8125rem }`).
pub const INLINE_CODE_FAMILY: &str = "u0 Inline Mono";

/// The prose size the inline-code face is scaled for (`.g-prose,
/// .t3-amsg-body { font-size: 0.875rem }`) and the code size it draws at.
pub const INLINE_CODE_SIZES: (f32, f32) = (14., 13.);

/// The face's em in font units: Plex Mono's 1000 × 14 / 13.
const INLINE_CODE_EM: u16 = 1077;

/// The face's U+202F advance in font units: the code's 4px inline padding
/// (`padding: 1px 4px`) at 14px.
const INLINE_CODE_PAD: u16 = 308;

/// The inline-code face's bytes, derived from the bundled Plex Mono.
pub fn inline_code_face() -> Option<Vec<u8>> {
    let (_, mono) = FONT_FILES
        .iter()
        .find(|(name, _)| *name == "IBMPlexMono-Regular.ttf")?;
    inline_face::derive(mono, INLINE_CODE_EM, INLINE_CODE_FAMILY, INLINE_CODE_PAD)
}

/// The inline-code face with the mono CJK fallbacks. In a paragraph at
/// `INLINE_CODE_SIZES.0` it draws `INLINE_CODE_SIZES.1`; elsewhere it scales
/// with the paragraph (GPUI has no per-run size).
pub fn inline_code_font() -> Font {
    let mut f = font(FontFamilyKey::Mono, 400);
    f.family = INLINE_CODE_FAMILY.into();
    f
}

/// Register the bundled fonts with GPUI's text system (called by [`crate::init`]).
pub fn register_fonts(cx: &mut App) {
    let mut fonts: Vec<Cow<'static, [u8]>> = FONT_FILES
        .iter()
        .map(|(_, bytes)| Cow::Borrowed(*bytes))
        .collect();
    fonts.extend(inline_code_face().map(Cow::Owned));
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

/// The width of the longest word of `text` at `size_px` in `family` /
/// `weight`: the min-content width CSS gives wrapping text. GPUI wraps text
/// only at its box's width and breaks a word that does not fit, so a box
/// that may shrink holds this as its least width to wrap between words only.
pub fn longest_word_width(
    text: &str,
    size_px: f32,
    family: FontFamilyKey,
    weight: u16,
    window: &gpui_kit::Window,
) -> gpui_kit::Pixels {
    let size = gpui_kit::rems(size_px / crate::tokens::REM_PX).to_pixels(window.rem_size());
    text.split_whitespace()
        .map(|word| {
            // With the space after it: GPUI's line wrapper keeps the space
            // on the word's line, and one that does not fit becomes a line
            // of its own.
            let word = format!("{word} ");
            let run = gpui_kit::TextRun {
                len: word.len(),
                font: font(family, weight),
                color: gpui_kit::black(),
                background_color: None,
                underline: None,
                strikethrough: None,
            };
            window
                .text_system()
                .shape_line(gpui_kit::SharedString::from(word), size, &[run], None)
                .width
        })
        .fold(gpui_kit::px(0.), |a, b| a.max(b))
        // Whole pixels and one to spare: the line wrapper breaks a word
        // that only just fits its measured width.
        .ceil()
        + gpui_kit::px(1.)
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

/// The type style of a bare `<p>`: Carbon's global rule
/// (`bundle.css`: `p { font-size: var(--body-02-font-size); line-height:
/// var(--body-02-line-height); letter-spacing: 0 }`), 16px/24px. It wins over
/// an inherited `font-size`, so a `<p>` inside 14px text is still 16px unless
/// a rule of its own (`.cu-pempty p`, …) sets the size.
pub const PARAGRAPH: TypeStyle = TypeStyle::Body02;

/// A block for a reference's bare `<p>` (one without a size rule of its
/// own): [`PARAGRAPH`], 16px/24px. Callers set the color.
pub fn paragraph() -> Div {
    div().type_style(PARAGRAPH)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_paragraph_is_body_02_16_on_24() {
        let spec = PARAGRAPH.spec();
        assert_eq!(spec.font_size_rem * 16., 16.);
        assert_eq!(spec.weight, 400);
        assert_eq!(spec.letter_spacing_px, 0.);
        assert!(
            matches!(spec.line_height, crate::tokens::LineHeight::Multiplier(m) if (m * 16. - 24.).abs() < 0.01)
        );
    }

    #[test]
    fn fonts_are_bundled_and_stacks_fall_back_to_cjk() {
        assert_eq!(FONT_FILES.len(), 9);
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
        // `.g-syn-comment { font-style: italic }` in Plex Mono, and italic
        // prose in Plex Sans, each resolve to a real italic face: the `head`
        // table's macStyle has the italic bit.
        for name in ["IBMPlexMono-Italic.ttf", "IBMPlexSans-Italic.ttf"] {
            let (_, bytes) = FONT_FILES.iter().find(|(n, _)| *n == name).unwrap();
            assert!(mac_style(bytes) & 0b10 != 0, "{name} is italic");
        }
        let (_, regular) = FONT_FILES
            .iter()
            .find(|(n, _)| *n == "IBMPlexMono-Regular.ttf")
            .unwrap();
        assert_eq!(mac_style(regular) & 0b10, 0);
    }

    /// `head.macStyle` (offset 44) of a TrueType font.
    fn mac_style(font: &[u8]) -> u16 {
        let be16 = |o: usize| u16::from_be_bytes([font[o], font[o + 1]]);
        let be32 = |o: usize| u32::from_be_bytes([font[o], font[o + 1], font[o + 2], font[o + 3]]);
        let tables = be16(4) as usize;
        let head = (0..tables)
            .map(|i| 12 + 16 * i)
            .find(|&rec| &font[rec..rec + 4] == b"head")
            .map(|rec| be32(rec + 8) as usize)
            .expect("head table");
        be16(head + 44)
    }

    #[test]
    fn inline_code_face_draws_13px_in_14px_prose() {
        let (body, code) = INLINE_CODE_SIZES;
        assert!((1000. * body / code - INLINE_CODE_EM as f32).abs() < 0.5);
        // 4px of padding at the 14px run size.
        assert!((INLINE_CODE_PAD as f32 / INLINE_CODE_EM as f32 * body - 4.).abs() < 0.01);
        assert!(inline_code_face().is_some_and(|f| f.len() > 100_000));
        let f = inline_code_font();
        assert_eq!(f.family.as_ref(), INLINE_CODE_FAMILY);
        assert_eq!(f.weight, FontWeight(400.));
    }
}
