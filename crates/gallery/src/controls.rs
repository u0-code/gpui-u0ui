//! Configuration-panel controls, built from u0ui's own components: a
//! [`Switch`] per flag, a content switcher ([`ToggleGroup`]) for a short
//! choice whose labels fit their equal-width segments (measured, so none is
//! clipped) and a [`Select`] otherwise, each under a `label-01` label.
//! The panel is a `layer-01` surface, so fields paint layer two.
use gpui_kit::{
    App, ElementId, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, StyledText,
    TextRun, Window, div,
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

/// A content-switcher segment's inline padding (`.cds--content-switcher-btn
/// { padding-inline: 1rem }`), both sides.
const SEGMENT_PADDING: f32 = 32.;

/// Whether labels `widths` px wide fit a content switcher `panel` px wide
/// without clipping. Carbon's segments share the width equally, so the
/// widest label plus its padding must fit `panel / n`; at most five.
pub fn fits_switcher(widths: &[f32], panel: f32) -> bool {
    let n = widths.len();
    n > 0
        && n <= 5
        && widths
            .iter()
            .all(|w| w + SEGMENT_PADDING <= panel / n as f32)
}

/// One of `options`: a content switcher ([`ToggleGroup`], size sm) when every
/// label fits its equal-width segment, a [`Select`] otherwise. The labels
/// are measured with the switcher's own font when the control renders.
pub fn choices(
    id: impl Into<ElementId>,
    options: &[&'static str],
    selected: usize,
    on_select: impl Fn(usize, &mut Window, &mut App) + 'static,
) -> gpui_kit::AnyElement {
    Choices {
        id: id.into(),
        options: options.to_vec(),
        selected,
        on_select: Rc::new(on_select),
    }
    .into_any_element()
}

type SelectHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

#[derive(IntoElement)]
struct Choices {
    id: ElementId,
    options: Vec<&'static str>,
    selected: usize,
    on_select: SelectHandler,
}

/// The width of `label` in px at a 16px rem, in `body-compact-01` (14px IBM
/// Plex Sans, 0.16px tracking), the switcher's label style.
fn label_width(label: &str, window: &Window) -> f32 {
    let rem = window.rem_size();
    let size = gpui_kit::rems(14. / 16.).to_pixels(rem);
    let run = TextRun {
        len: label.len(),
        font: u0ui::typography::font(u0ui::tokens::FontFamilyKey::Sans, 400),
        color: Hsla::default(),
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    let line = window
        .text_system()
        .shape_line(label.to_string().into(), size, &[run], None);
    let scale = f32::from(rem) / 16.;
    f32::from(line.width) / scale + 0.16 * label.chars().count() as f32
}

impl RenderOnce for Choices {
    fn render(self, window: &mut Window, _: &mut App) -> impl IntoElement {
        let Choices {
            id,
            options,
            selected,
            on_select,
        } = self;
        let current: SharedString = options.get(selected).copied().unwrap_or_default().into();
        let index_of = {
            let options = options.clone();
            move |value: &str| options.iter().position(|o| *o == value)
        };
        let widths: Vec<f32> = options.iter().map(|o| label_width(o, window)).collect();
        if fits_switcher(&widths, PANEL_CONTENT) {
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
        // Three 96px segments in the 288px panel: a 63px label fits ("horizontal").
        assert!(fits_switcher(&[33., 52., 63.], PANEL_CONTENT));
        assert!(fits_switcher(&[16., 16., 16., 16.], PANEL_CONTENT));
        // "broken image" / "10 (default)" (~80px) would be clipped: Select.
        assert!(!fits_switcher(&[30., 36., 82.], PANEL_CONTENT));
        assert!(!fits_switcher(&[20.; 6], PANEL_CONTENT));
        assert!(!fits_switcher(&[], PANEL_CONTENT));
    }
}
