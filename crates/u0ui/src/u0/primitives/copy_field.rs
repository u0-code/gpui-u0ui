//! CopyField: a read-only value with a copy button and an optional open link
//! (`CopyField` in `src/ghost/primitives.jsx`). For webhook URLs, pairing
//! links, tunnel addresses and fingerprints.
//!
//! - **Behavior**: the copy button writes the value to the clipboard and turns
//!   into a checkmark for 1.5s (`useFlash`), its accessible name becoming
//!   "Copied". The open button calls the caller's handler (the reference's
//!   `href`). The full value shows as a tooltip (`title`) unless masked.
//! - **Appearance** (`.g-copy`): `layer-02` fill, 1px `border-strong-01`
//!   bottom rule, 12px start padding, the value truncated with an ellipsis,
//!   mono by default (`.g-mono`), ghost icon buttons (`.g-iconbtn`).
use super::parts::{self, IconButton, rem};
use crate::{
    components::form,
    density,
    icons::CarbonIcon,
    theme::{LayerLevel, color},
    tokens::{ColorRole, FontFamilyKey},
};
use gpui_kit::{
    App, ClickEvent, ClipboardItem, ElementId, Hsla, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled, Window,
    component::tooltip::Tooltip, div, prelude::FluentBuilder as _,
};
use std::rc::Rc;

/// How long the checkmark shows (`useFlash(ms = 1500)`).
pub const COPIED_MS: u64 = 1500;

/// `masked ? value.slice(0, 4) + '…' + value.slice(-4) : value`.
pub fn masked(value: &str) -> String {
    let chars: Vec<char> = value.chars().collect();
    let head: String = chars.iter().take(4).collect();
    let tail: String = chars[chars.len().saturating_sub(4)..].iter().collect();
    format!("{head}…{tail}")
}

/// Colors of `.g-copy` on `level`: `(fill, border_strong)`. The fill is
/// `.g-copy { background: var(--layer-02, var(--field-01)) }`, and
/// `--layer-02` is a theme token (no layer context redefines it), so only the
/// bottom rule follows the level.
pub fn box_look(level: LayerLevel) -> (Hsla, Hsla) {
    (color(ColorRole::Layer02), color(level.border_strong()))
}

type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// A read-only value with copy and open buttons. See the module docs.
#[derive(IntoElement)]
pub struct CopyField {
    id: ElementId,
    label: Option<SharedString>,
    value: SharedString,
    mono: bool,
    masked: bool,
    dashed: bool,
    level: LayerLevel,
    on_open: Option<ClickHandler>,
    open_label: SharedString,
    copy_label: Option<SharedString>,
    copied_label: SharedString,
}

impl CopyField {
    /// A copy field with a stable `id` showing `value`.
    pub fn new(id: impl Into<ElementId>, value: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: None,
            value: value.into(),
            mono: true,
            masked: false,
            dashed: false,
            level: LayerLevel::One,
            on_open: None,
            // The reference's `aria-label`s; override per locale.
            open_label: "Open".into(),
            copy_label: None,
            copied_label: "Copied".into(),
        }
    }

    /// The layer the field sits on: picks the bottom rule's contextual
    /// `border-strong` token (the fill is always `layer-02`).
    pub fn level(mut self, level: LayerLevel) -> Self {
        self.level = level;
        self
    }

    /// Draw a 1px dashed `border-strong` border around the field (for inactive
    /// or offline endpoints) instead of the solid bottom rule.
    pub fn dashed(mut self, dashed: bool) -> Self {
        self.dashed = dashed;
        self
    }

    /// The label above the field.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Mono type (default true; `.g-mono`).
    pub fn mono(mut self, mono: bool) -> Self {
        self.mono = mono;
        self
    }

    /// Show only the first and last four characters (and no tooltip).
    pub fn masked(mut self, masked: bool) -> Self {
        self.masked = masked;
        self
    }

    /// Show the open button (the reference's `href`), calling `f` on press.
    pub fn on_open(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_open = Some(Rc::new(f));
        self
    }

    /// Accessible names: open, copy (default `Copy <label>`), copied.
    pub fn button_labels(
        mut self,
        open: impl Into<SharedString>,
        copy: impl Into<SharedString>,
        copied: impl Into<SharedString>,
    ) -> Self {
        self.open_label = open.into();
        self.copy_label = Some(copy.into());
        self.copied_label = copied.into();
        self
    }
}

impl RenderOnce for CopyField {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let flash = parts::flash_state((id.clone(), "flash"), window, cx);
        let done = parts::flash_on(&flash, COPIED_MS, cx);
        let field_h = density::current().field().px_value();
        let shown: SharedString = if self.masked {
            masked(&self.value).into()
        } else {
            self.value.clone()
        };
        let copy_label = if done {
            self.copied_label.clone()
        } else {
            self.copy_label.clone().unwrap_or_else(|| {
                format!("Copy {}", self.label.as_deref().unwrap_or("value")).into()
            })
        };
        let title = (!self.masked).then(|| self.value.clone());
        let value = div()
            .id((id.clone(), "value"))
            .flex_1()
            .min_w_0()
            .overflow_hidden()
            .whitespace_nowrap()
            .text_ellipsis()
            .text_color(color(ColorRole::TextPrimary))
            .map(|d| {
                if self.mono {
                    // `.g-mono { font-family: var(--font-mono); font-size: 0.75rem }`.
                    d.font(crate::typography::font(FontFamilyKey::Mono, 400))
                        .text_size(rem(12.))
                        .line_height(rem(16.))
                } else {
                    d.text_size(rem(14.)).line_height(rem(18.))
                }
            })
            .when_some(title, |d, t| {
                d.tooltip(move |window, cx| Tooltip::new(t.clone()).build(window, cx))
            })
            .child(shown);
        let text = self.value.clone();
        let copy = IconButton::new(
            (id.clone(), "copy"),
            if done {
                CarbonIcon::Checkmark
            } else {
                CarbonIcon::Copy
            },
            copy_label,
        )
        .on_click(move |_, _, cx| {
            cx.write_to_clipboard(ClipboardItem::new_string(text.to_string()));
            parts::flash(&flash, COPIED_MS, cx);
        });
        let open = self.on_open.clone().map(|f| {
            IconButton::new(
                (id.clone(), "open"),
                CarbonIcon::Launch,
                self.open_label.clone(),
            )
            .on_click(move |e, w, cx| f(e, w, cx))
        });
        let (fill, border) = box_look(self.level);
        let body = div()
            .relative()
            .flex()
            .items_center()
            .min_h(rem(field_h))
            .pl(rem(12.))
            .bg(fill)
            .when(self.dashed, |d| {
                d.border_1().border_dashed().border_color(border)
            })
            .child(value)
            .children(open)
            .child(copy)
            .when(!self.dashed, |d| d.child(form::bottom_rule(border)));
        parts::field()
            .when_some(self.label.clone(), |f, l| f.child(parts::label(l)))
            .child(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CarbonTheme, theme::color_in};

    #[test]
    fn masking_keeps_four_each_side() {
        assert_eq!(masked("sk-live-123456"), "sk-l…3456");
        assert_eq!(masked("abc"), "abc…abc");
    }

    #[test]
    fn layer_levels_select_contextual_tokens() {
        crate::theme::tests_set(CarbonTheme::White);
        let c = |r| color_in(CarbonTheme::White, r);
        // White: `layer-02` is #ffffff (the page's `field-01` is #f4f4f4).
        assert_eq!(
            box_look(LayerLevel::One),
            (c(ColorRole::Layer02), c(ColorRole::BorderStrong01))
        );
        assert_eq!(
            box_look(LayerLevel::Two),
            (c(ColorRole::Layer02), c(ColorRole::BorderStrong02))
        );
        assert_eq!(c(ColorRole::Layer02), gpui_kit::rgb(0xffffff).into());
        assert_eq!(
            color_in(CarbonTheme::G100, ColorRole::Layer02),
            gpui_kit::rgb(0x393939).into()
        );
    }
}
