//! Truncate: middle truncation for paths and IDs, end truncation for titles
//! (`Truncate` in `src/ghost/foundations.jsx`).
//!
//! - `Path`: the file name always shows in full; the directories give way
//!   from their end (`.g-trunc-head` shrinks to 1.5em with an ellipsis,
//!   `.g-trunc-tail` keeps up to 70% of the width).
//! - `Middle` (branches, IDs, hosts): split at the middle, the start shrinks.
//! - `End` (titles, messages): an ellipsis at the end.
//!
//! Layout does the truncating (never a character count), so zoom and width
//! changes stay right; the full text is the tooltip (`title`) and the
//! accessible name.
use gpui_kit::{
    App, ElementId, InteractiveElement as _, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement as _, Styled, Window, component::tooltip::Tooltip, div,
    prelude::FluentBuilder as _, relative, rems,
};

/// How to truncate (`mode`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TruncateMode {
    /// `end` (default): titles.
    #[default]
    End,
    /// `middle`: branch names, IDs, hosts.
    Middle,
    /// `path`: keep the file name whole.
    Path,
}

impl TruncateMode {
    /// Every mode.
    pub const ALL: [TruncateMode; 3] = [Self::End, Self::Middle, Self::Path];

    /// The reference name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::End => "end",
            Self::Middle => "middle",
            Self::Path => "path",
        }
    }
}

/// Maximum trailing characters kept whole in [`TruncateMode::Middle`] so the
/// tail fits inside narrow chips (≤70% width) without a second ellipsis while
/// `head` shrinks cleanly up to it.
const MIDDLE_TAIL_MAX: usize = 12;

/// The (head, tail) split: `path` after the last `/`, `middle` keeping up to
/// [`MIDDLE_TAIL_MAX`] characters (or `ceil(len / 2)` for shorter text) in the
/// tail so the suffix stays whole; `End` puts everything in the head.
pub fn split(text: &str, mode: TruncateMode) -> (String, String) {
    match mode {
        TruncateMode::End => (text.to_string(), String::new()),
        TruncateMode::Path => match text.rfind('/') {
            Some(i) => (text[..=i].to_string(), text[i + 1..].to_string()),
            None => (String::new(), text.to_string()),
        },
        TruncateMode::Middle => {
            let total = text.chars().count();
            let tail_n = total.div_ceil(2).min(MIDDLE_TAIL_MAX);
            let head_n = total.saturating_sub(tail_n);
            let i = text
                .char_indices()
                .nth(head_n)
                .map_or(text.len(), |(i, _)| i);
            (text[..i].to_string(), text[i..].to_string())
        }
    }
}

/// Truncated text. See the module docs.
#[derive(IntoElement)]
pub struct Truncate {
    id: ElementId,
    text: SharedString,
    mode: TruncateMode,
    title: bool,
}

impl Truncate {
    /// End-truncated `text` with its tooltip.
    pub fn new(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            mode: TruncateMode::End,
            title: true,
        }
    }

    /// The mode.
    pub fn mode(mut self, mode: TruncateMode) -> Self {
        self.mode = mode;
        self
    }

    /// Show the full text as a tooltip (default true).
    pub fn title(mut self, title: bool) -> Self {
        self.title = title;
        self
    }
}

/// Whether `(head, tail)` should render as two flex spans (`head` with a
/// `1.5em` minimum width, `tail` capped at `70%`). When either half is empty
/// (a bare file name in [`TruncateMode::Path`]) or a [`TruncateMode::Middle`]
/// head is shorter than `1.5em` (`< 4` characters, such as `"main"`), a single
/// ellipsis span avoids inserting a blank gap inside short text.
fn uses_mid_split(head: &str, tail: &str, mode: TruncateMode) -> bool {
    !head.is_empty()
        && !tail.is_empty()
        && !(mode == TruncateMode::Middle && head.chars().count() < 4)
}

impl RenderOnce for Truncate {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let full = self.text.clone();
        let tip = self.title.then(|| full.clone());
        let base = div()
            .id(self.id.clone())
            .min_w_0()
            .overflow_hidden()
            .whitespace_nowrap()
            .when_some(tip, |d, t| {
                d.tooltip(move |window, cx| Tooltip::new(t.clone()).build(window, cx))
            });
        if self.mode == TruncateMode::End {
            // `.g-trunc-end { display: block; overflow: hidden; text-overflow: ellipsis }`.
            return base.text_ellipsis().child(full);
        }
        let (head, tail) = split(&self.text, self.mode);
        if !uses_mid_split(&head, &tail, self.mode) {
            return base.text_ellipsis().child(full);
        }
        // `.g-trunc-mid { display: flex }`; head `flex: 0 1 auto; min-width:
        // 1.5em`; tail `flex: none; max-width: 70%`.
        base.flex()
            .child(
                div()
                    .flex_shrink_1()
                    .min_w(rems(1.5 * 0.875))
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .child(head),
            )
            .child(
                div()
                    .flex_none()
                    .max_w(relative(0.7))
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .child(tail),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_follow_the_reference() {
        assert_eq!(
            split(
                "packages/web/src/components/sidebar/SidebarThreadRow.tsx",
                TruncateMode::Path
            ),
            (
                "packages/web/src/components/sidebar/".to_string(),
                "SidebarThreadRow.tsx".to_string()
            )
        );
        assert_eq!(
            split("feature/dark", TruncateMode::Middle),
            ("featur".to_string(), "e/dark".to_string())
        );
        assert_eq!(
            split(
                "feat/onboarding-banner-for-long-languages-ENG-1142",
                TruncateMode::Middle
            ),
            (
                "feat/onboarding-banner-for-long-langua".to_string(),
                "ges-ENG-1142".to_string()
            )
        );
        assert_eq!(split("README.md", TruncateMode::Path).0, "");
        assert_eq!(split("设置通用", TruncateMode::Middle).1, "通用");
        let (h, t) = split("README.md", TruncateMode::Path);
        assert!(!uses_mid_split(&h, &t, TruncateMode::Path));
        let (h, t) = split("main", TruncateMode::Middle);
        assert!(!uses_mid_split(&h, &t, TruncateMode::Middle));
        let (h, t) = split("feature/dark", TruncateMode::Middle);
        assert!(uses_mid_split(&h, &t, TruncateMode::Middle));
    }
}
