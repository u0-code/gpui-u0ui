//! Configuration-panel controls, built from u0ui's own components: a
//! [`Switch`] per flag, a content switcher ([`ToggleGroup`]) for a short
//! choice and a [`Select`] for a long one, each under a `label-01` label.
//! The panel is a `layer-01` surface, so fields paint layer two.
use gpui_kit::{
    App, ElementId, Hsla, IntoElement, ParentElement, SharedString, Styled, StyledText, TextRun,
    Window, div,
};
use std::rc::Rc;
use u0ui::components::switch::{Switch, SwitchSize};
use u0ui::components::toggle::ToggleSize;
use u0ui::components::{Select, ToggleGroup, ToggleGroupItem};
use u0ui::prelude::*;

/// The panel's content width (`20rem` less its `1rem` padding), in px.
const PANEL_CONTENT: f32 = 288.;

/// A config-panel switch: u0ui's small [`Switch`] whose state text is the
/// option name (`.cds--toggle__appearance--sm`: 32×16 track).
pub fn toggle(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    on: bool,
    on_change: impl Fn(bool, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let label = label.into();
    Switch::new(id)
        .size(SwitchSize::Sm)
        .checked(on)
        .label_on(label.clone())
        .label_off(label)
        .on_checked_change(on_change)
}

/// A labeled control in the config panel (`label-01` in `text-secondary`, 8px gap).
pub fn field(label: impl Into<SharedString>, control: impl IntoElement) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_03)
        .child(
            div()
                .type_style(TypeStyle::Label01)
                .text_color(color(ColorRole::TextSecondary))
                .child(label.into()),
        )
        .child(control)
}

/// Whether `options` fit one content-switcher row in the panel: about 7px a
/// character plus each segment's 2×16px padding.
pub fn fits_switcher(options: &[&str]) -> bool {
    let text: usize = options.iter().map(|o| o.chars().count()).sum();
    options.len() <= 5 && text as f32 * 7.5 + options.len() as f32 * 32. <= PANEL_CONTENT
}

/// One of `options`: a content switcher ([`ToggleGroup`], size sm) when they
/// fit one row, a [`Select`] otherwise.
pub fn choices(
    id: impl Into<ElementId>,
    options: &[&'static str],
    selected: usize,
    on_select: impl Fn(usize, &mut Window, &mut App) + 'static,
) -> gpui_kit::AnyElement {
    let id: ElementId = id.into();
    let options: Vec<&'static str> = options.to_vec();
    let on_select = Rc::new(on_select);
    let current: SharedString = options.get(selected).copied().unwrap_or_default().into();
    let index_of = {
        let options = options.clone();
        move |value: &str| options.iter().position(|o| *o == value)
    };
    if fits_switcher(&options) {
        let mut group = ToggleGroup::new(id)
            .size(ToggleSize::Sm)
            .value([current])
            .on_value_change(move |value, window, cx| {
                if let Some(i) = value.first().and_then(|v| index_of(v)) {
                    on_select(i, window, cx);
                }
            });
        for option in &options {
            group = group.item(ToggleGroupItem::new(*option).child(*option));
        }
        group.into_any_element()
    } else {
        Select::new(id)
            .items(options.iter().copied())
            .value(Some(current))
            .size(u0ui::components::FieldSize::Sm)
            .layer(LayerLevel::Two)
            .on_value_change(move |value, window, cx| {
                if let Some(i) = value.and_then(|v| index_of(v)) {
                    on_select(i, window, cx);
                }
            })
            .into_any_element()
    }
}

/// A section heading inside a page (`heading-compact-01`).
pub fn section(title: impl Into<SharedString>) -> impl IntoElement {
    div()
        .type_style(TypeStyle::HeadingCompact01)
        .text_color(color(ColorRole::TextPrimary))
        .child(title.into())
}

/// A titled page section: a [`section`] heading over `content`, 16px apart.
pub fn block(title: impl Into<SharedString>, content: impl IntoElement) -> gpui_kit::Div {
    div()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_05)
        .child(section(title))
        .child(content)
}

/// Split `text` at its backticks: `(span, is_code)` pairs, empty spans
/// dropped. An unpaired backtick stays literal.
pub fn code_spans(text: &str) -> Vec<(&str, bool)> {
    let ticks = text.matches('`').count();
    // An unpaired last backtick does not open a span.
    let paired = ticks - ticks % 2;
    let mut spans = Vec::new();
    let (mut code, mut start, mut seen) = (false, 0, 0);
    for (i, ch) in text.char_indices() {
        if ch == '`' && seen < paired {
            spans.push((&text[start..i], code));
            code = !code;
            start = i + 1;
            seen += 1;
        }
    }
    spans.push((&text[start..], code));
    spans.retain(|(span, _)| !span.is_empty());
    spans
}

/// Prose whose `` `code` `` spans render as inline code (IBM Plex Mono on
/// `layer-01`, `.cu-typeset code`) instead of literal backticks, in `color`;
/// the font size comes from the parent's type style.
pub fn prose(text: &str, color: Hsla) -> StyledText {
    let mut out = String::new();
    let mut runs = Vec::new();
    for (span, code) in code_spans(text) {
        out.push_str(span);
        runs.push(TextRun {
            len: span.len(),
            font: u0ui::typography::font(
                if code {
                    tokens::FontFamilyKey::Mono
                } else {
                    tokens::FontFamilyKey::Sans
                },
                400,
            ),
            color,
            background_color: code.then(|| theme::color(ColorRole::Layer01)),
            underline: None,
            strikethrough: None,
        });
    }
    StyledText::new(out).with_runs(runs)
}

/// A control callback that updates the page entity with `f` and re-renders:
/// `controls::toggle(.., controls::setter(&entity, |this, on| this.flag = on))`.
pub fn setter<T: 'static, V: 'static>(
    entity: &gpui_kit::Entity<T>,
    f: impl Fn(&mut T, V) + 'static,
) -> impl Fn(V, &mut Window, &mut App) + 'static {
    let entity = entity.clone();
    move |value, _, cx| {
        entity.update(cx, |this, cx| {
            f(this, value);
            cx.notify();
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backticks_become_code_spans() {
        assert_eq!(
            code_spans("Native `<select>` dropdown."),
            vec![
                ("Native ", false),
                ("<select>", true),
                (" dropdown.", false)
            ]
        );
        assert_eq!(
            code_spans("`a` and `b`"),
            vec![("a", true), (" and ", false), ("b", true)]
        );
        assert_eq!(code_spans("plain"), vec![("plain", false)]);
        // An unpaired backtick stays literal.
        assert_eq!(
            code_spans("x `a` y ` z"),
            vec![("x ", false), ("a", true), (" y ` z", false)]
        );
    }

    #[test]
    fn short_choices_switch_long_ones_select() {
        assert!(fits_switcher(&["sm", "md", "lg"]));
        assert!(fits_switcher(&["16", "20", "24", "32"]));
        assert!(!fits_switcher(&[
            "primary",
            "secondary",
            "tertiary",
            "ghost",
            "danger"
        ]));
    }
}
