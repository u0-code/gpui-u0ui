//! Inline code in prose, shared by the Ghost `MarkdownProse` and the T3
//! `AssistantMessage` [`Prose`](crate::t3::timeline::Prose).
//!
//! Every rule for it, in cascade order (`components/bundle.css`):
//!
//! 1. `.t3-amsg-body code { font-family: var(--font-mono); font-size:
//!    0.8125rem; padding: 0 var(--spacing-02); background: var(--layer-01) }`
//! 2. `.g-prose code { padding: 1px 4px; background: var(--layer-accent-01);
//!    font-family: var(--font-mono); font-size: 0.8125em }`
//! 3. gemini-A, `.g-prose code, .t3-amsg-body code { padding: 1px 4px;
//!    background: var(--layer-01); box-shadow: inset 0 0 0 1px
//!    var(--border-subtle-00); font-size: 0.8125rem }`
//! 4. r2-desk ("inline code is a quiet tint, not a boxed chip"), `.g-prose
//!    code, .t3-amsg-body code { box-shadow: none; padding: 1px 4px;
//!    background: color-mix(in srgb, var(--layer-accent-01) 60%,
//!    transparent) }`
//!
//! So both draw IBM Plex Mono at 13px on a borderless `layer-accent-01` 60%
//! tint, padded 1px 4px.
//!
//! GPUI text runs can neither pad, outline nor change size, so:
//! - the 13px comes from [`inline_code_font`], a Plex Mono face whose glyphs
//!   draw at 13/14 of the paragraph size (in 13px table cells the plain
//!   13px Plex Mono is used instead);
//! - the 4px padding is a narrow no-break space (U+202F, whose advance that
//!   face sets to the padding) on each side, so it never wraps away;
//! - the tint is painted behind the text by [`with_code_boxes`], as tall as
//!   the 13px code line plus the 1px top and bottom padding, centred on the
//!   paragraph line, instead of a run background that would fill the whole
//!   line height.
use crate::{
    theme::color,
    tokens::{ColorRole, FontFamilyKey},
    typography::{INLINE_CODE_SIZES, font, inline_code_font},
};
use gpui_kit::{
    Bounds, Div, Hsla, ParentElement, Pixels, Styled, StyledText, TextRun, canvas, div, fill,
    point, px, size,
};
use std::ops::Range;

/// The padding on each side of the code: one U+202F.
pub const PAD: &str = "\u{202F}";

/// r2-desk: `background: color-mix(in srgb, var(--layer-accent-01) 60%,
/// transparent)`.
pub fn inline_code_fill() -> Hsla {
    color(ColorRole::LayerAccent01).opacity(0.6)
}

/// The drawn code size in a paragraph of `body` px: 13px in 14px prose (the
/// derived face), the paragraph size itself below that.
pub fn code_size(body: f32) -> f32 {
    let (prose, code) = INLINE_CODE_SIZES;
    if body < prose {
        body
    } else {
        body * code / prose
    }
}

/// The tint's height: the code's content box (Plex Mono ascent + descent,
/// 1.3em) plus `padding: 1px` above and below.
pub fn box_height(body: f32) -> f32 {
    code_size(body) * 1.3 + 2.
}

/// Append `code` to a paragraph of `body` px in `ink`: the padded text and
/// its runs. Returns the byte range to hand [`with_code_boxes`].
pub fn push_inline_code(
    text: &mut String,
    runs: &mut Vec<TextRun>,
    code: &str,
    ink: Hsla,
    body: f32,
) -> Range<usize> {
    let start = text.len();
    let face = inline_code_font();
    let code_font = if body < INLINE_CODE_SIZES.0 {
        font(FontFamilyKey::Mono, 400)
    } else {
        face.clone()
    };
    for (s, f) in [(PAD, &face), (code, &code_font), (PAD, &face)] {
        if s.is_empty() {
            continue;
        }
        text.push_str(s);
        runs.push(TextRun {
            len: s.len(),
            font: f.clone(),
            color: ink,
            background_color: None,
            underline: None,
            strikethrough: None,
        });
    }
    start..text.len()
}

/// `styled` (a paragraph of `body` px) with the inline-code tint painted
/// behind each of the `code` byte ranges, line by line where one wraps.
pub fn with_code_boxes(styled: StyledText, code: Vec<Range<usize>>, body: f32) -> Div {
    let layout = styled.layout().clone();
    let tint = inline_code_fill();
    let h = px(box_height(body));
    div()
        .relative()
        .min_w_0()
        .child(
            canvas(
                |_, _, _| (),
                move |bounds: Bounds<Pixels>, _, window, _| {
                    let inset = (layout.line_height() - h) / 2.;
                    for r in &code {
                        let (Some(a), Some(b)) = (
                            layout.position_for_index(r.start),
                            layout.position_for_index(r.end),
                        ) else {
                            continue;
                        };
                        let mut boxes = Vec::new();
                        if a.y == b.y {
                            boxes.push((a.x, b.x, a.y));
                        } else {
                            boxes.push((a.x, bounds.right(), a.y));
                            boxes.push((bounds.left(), b.x, b.y));
                        }
                        for (x0, x1, y) in boxes {
                            if x1 > x0 {
                                let rect = Bounds::new(point(x0, y + inset), size(x1 - x0, h));
                                window.paint_quad(fill(rect, tint));
                            }
                        }
                    }
                },
            )
            .absolute()
            .top_0()
            .left_0()
            .size_full(),
        )
        .child(styled)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typography::INLINE_CODE_FAMILY;

    #[test]
    fn inline_code_is_the_r2_desk_tint() {
        crate::theme::tests_set(crate::CarbonTheme::White);
        let tint = inline_code_fill();
        let base = color(ColorRole::LayerAccent01);
        assert!((tint.a - 0.6).abs() < 1e-3);
        assert_eq!((tint.h, tint.s, tint.l), (base.h, base.s, base.l));
    }

    #[test]
    fn code_is_13px_padded_with_no_break_spaces() {
        assert_eq!(code_size(14.), 13.);
        assert_eq!(code_size(13.), 13.);
        assert!((box_height(14.) - 18.9).abs() < 1e-3);
        let (mut text, mut runs) = ("a ".to_string(), Vec::new());
        let r = push_inline_code(&mut text, &mut runs, "g90", Hsla::default(), 14.);
        assert_eq!(&text[r], "\u{202F}g90\u{202F}");
        assert_eq!(runs.len(), 3);
        assert!(
            runs.iter()
                .all(|r| r.font.family.as_ref() == INLINE_CODE_FAMILY)
        );
        assert!(runs.iter().all(|r| r.background_color.is_none()));
        // In a 13px table cell the code is plain Plex Mono.
        let mut runs = Vec::new();
        push_inline_code(&mut String::new(), &mut runs, "x", Hsla::default(), 13.);
        assert_eq!(runs[1].font.family.as_ref(), "IBM Plex Mono");
    }
}
