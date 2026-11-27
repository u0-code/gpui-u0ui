//! HighlightedText: fuzzy-match highlighting (`HighlightedText`, `fuzzyMatch`
//! in `src/ghost/primitives.jsx`). Matched characters turn 600 weight and get
//! an underline, so a match never depends on color alone (`.g-hl`).
//!
//! Used by ⌘K, ⌘P, @ mentions and filter menus.
//!
//! ```ignore
//! HighlightedText::new("SidebarThreadRow.tsx").query("stro")
//! HighlightedText::new("Open settings").indices(vec![0, 1, 5, 6, 7])
//! let hits = fuzzy_match("packages/web/src/components", "pwsc"); // Some([0, 9, 13, 15])
//! ```
use crate::{theme::color, tokens::ColorRole};
use gpui_kit::{
    App, FontWeight, HighlightStyle, IntoElement, RenderOnce, SharedString, StyledText,
    UnderlineStyle, Window, px,
};
use std::ops::Range;

/// The character indices of `text` that match `query` as an in-order,
/// case-insensitive subsequence (`fuzzyMatch`): `Some(vec![])` for an empty
/// query, `None` when `query` is not a subsequence of `text`.
///
/// Indices count Unicode scalar values (`char`s), not bytes; the first
/// occurrence of each query character wins, as in the reference loop.
pub fn fuzzy_match(text: &str, query: &str) -> Option<Vec<usize>> {
    // `if (!query) return [];`
    if query.is_empty() {
        return Some(Vec::new());
    }
    let query: Vec<char> = query.chars().flat_map(char::to_lowercase).collect();
    let mut hits = Vec::with_capacity(query.len());
    let mut j = 0;
    for (i, ch) in text.chars().enumerate() {
        if j == query.len() {
            break;
        }
        // `t[i] === q[j]` on the lower-cased strings. A character whose lower
        // case is several chars (`İ`) matches on its first one.
        if ch.to_lowercase().next() == Some(query[j]) {
            hits.push(i);
            j += 1;
        }
    }
    // `return j === q.length ? idx : null;`
    (j == query.len()).then_some(hits)
}

/// The `.g-hl` highlight: `background: none; color: var(--text-primary);
/// font-weight: 600; text-decoration: underline; text-decoration-color:
/// var(--border-interactive)` (the 2px underline offset has no GPUI
/// equivalent; the underline sits at the font's position).
pub fn highlight_style() -> HighlightStyle {
    HighlightStyle {
        color: Some(color(ColorRole::TextPrimary)),
        font_weight: Some(FontWeight::SEMIBOLD),
        underline: Some(UnderlineStyle {
            thickness: px(1.),
            color: Some(color(ColorRole::BorderInteractive)),
            wavy: false,
        }),
        ..Default::default()
    }
}

/// Byte ranges of `text` covering the character `indices`, merged into runs
/// (the reference wraps each run of consecutive hits in one `<mark>`), each
/// with [`highlight_style`]. Out-of-range indices are ignored.
pub fn highlight_runs(text: &str, indices: &[usize]) -> Vec<(Range<usize>, HighlightStyle)> {
    let style = highlight_style();
    match_ranges(text, indices)
        .into_iter()
        .map(|range| (range, style))
        .collect()
}

/// The merged byte ranges of the character `indices` in `text`.
fn match_ranges(text: &str, indices: &[usize]) -> Vec<Range<usize>> {
    let mut hit = indices.to_vec();
    hit.sort_unstable();
    hit.dedup();
    let mut ranges: Vec<Range<usize>> = Vec::new();
    let mut want = hit.into_iter().peekable();
    for (i, (byte, ch)) in text.char_indices().enumerate() {
        while want.peek().is_some_and(|&w| w < i) {
            want.next();
        }
        if want.peek() != Some(&i) {
            continue;
        }
        want.next();
        let end = byte + ch.len_utf8();
        match ranges.last_mut() {
            Some(last) if last.end == byte => last.end = end,
            _ => ranges.push(byte..end),
        }
    }
    ranges
}

/// Text with its fuzzy-matched characters highlighted. See the module docs.
///
/// The text inherits the surrounding text style (family, size, color); only
/// the matched characters change.
#[derive(IntoElement)]
pub struct HighlightedText {
    text: SharedString,
    query: Option<SharedString>,
    indices: Option<Vec<usize>>,
}

impl HighlightedText {
    /// Plain `text` (nothing highlighted until [`Self::query`] or
    /// [`Self::indices`]).
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            query: None,
            indices: None,
        }
    }

    /// Highlight the [`fuzzy_match`] of `query` (nothing when it does not match).
    pub fn query(mut self, query: impl Into<SharedString>) -> Self {
        self.query = Some(query.into());
        self
    }

    /// Highlight these character indices instead of matching a query
    /// (`indices` wins over `query`, as in the reference).
    pub fn indices(mut self, indices: Vec<usize>) -> Self {
        self.indices = Some(indices);
        self
    }

    /// The character indices drawn highlighted.
    pub fn hits(&self) -> Vec<usize> {
        // `new Set(indices || fuzzyMatch(text, query) || [])`.
        self.indices.clone().unwrap_or_else(|| {
            self.query
                .as_ref()
                .and_then(|q| fuzzy_match(&self.text, q))
                .unwrap_or_default()
        })
    }
}

impl RenderOnce for HighlightedText {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let runs = highlight_runs(&self.text, &self.hits());
        StyledText::new(self.text).with_highlights(runs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CarbonTheme, theme::color_in};

    #[test]
    fn fuzzy_match_follows_the_reference() {
        // The three preview rows.
        assert_eq!(
            fuzzy_match("SidebarThreadRow.tsx", "stro"),
            Some(vec![0, 7, 9, 14])
        );
        assert_eq!(
            fuzzy_match("Open settings", "opset"),
            Some(vec![0, 1, 5, 6, 7])
        );
        assert_eq!(
            fuzzy_match("packages/web/src/components", "pwsc"),
            Some(vec![0, 9, 13, 15])
        );
        assert_eq!(fuzzy_match("abc", ""), Some(vec![]));
        assert_eq!(fuzzy_match("abc", "abd"), None);
        assert_eq!(fuzzy_match("abc", "abcd"), None);
        // Case-insensitive both ways; indices are chars, not bytes.
        assert_eq!(fuzzy_match("ÄbC", "äc"), Some(vec![0, 2]));
        assert_eq!(fuzzy_match("设置 · 通用", "通"), Some(vec![5]));
    }

    #[test]
    fn runs_merge_consecutive_hits_into_byte_ranges() {
        assert_eq!(
            match_ranges("Open settings", &[0, 1, 5, 6, 7]),
            vec![0..2, 5..8]
        );
        // Unsorted, duplicated and out-of-range indices are tolerated.
        assert_eq!(match_ranges("abc", &[2, 0, 0, 9]), vec![0..1, 2..3]);
        // Multi-byte characters keep char boundaries.
        assert_eq!(match_ranges("设置通用", &[1, 2]), vec![3..9]);
        assert!(match_ranges("abc", &[]).is_empty());
    }

    #[test]
    fn highlight_is_weight_and_underline_not_only_color() {
        // `.g-hl { color: text-primary; font-weight: 600; text-decoration:
        // underline; text-decoration-color: border-interactive }`.
        crate::theme::tests_set(CarbonTheme::G100);
        {
            let style = highlight_style();
            assert_eq!(style.font_weight, Some(FontWeight::SEMIBOLD));
            assert_eq!(
                style.color,
                Some(color_in(CarbonTheme::G100, ColorRole::TextPrimary))
            );
            let underline = style.underline.expect("underlined");
            assert_eq!(
                underline.color,
                Some(color_in(CarbonTheme::G100, ColorRole::BorderInteractive))
            );
            assert!(style.background_color.is_none());
        }
    }

    #[test]
    fn indices_win_over_query() {
        let text = HighlightedText::new("Open settings")
            .query("opset")
            .indices(vec![3]);
        assert_eq!(text.hits(), vec![3]);
        let text = HighlightedText::new("Open settings").query("zz");
        assert!(text.hits().is_empty());
    }
}
