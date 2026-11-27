//! The [`ThemeEditorLayout::Customize`] layout: the body of u0's "Customize
//! <theme>" drawer (board `SetAppearance`, `.ap-edit`, `.ap-check`,
//! `.ap-tokens`).
//!
//! - Left: the primary tokens as rows (a 32px swatch, the name and hex on a
//!   `field-02` row; the one being edited ringed 2px `focus`), then the
//!   preview card.
//! - Right: the inline [`ColorPicker`] for the selected token (plane, hue,
//!   hex, palette) and the Contrast checks ([`contrast_checks`]).
//! - Below: "Tokens", every token as a swatch, its name and a hex field, in
//!   two columns.
//!
//! The values are the caller's: each edit, from the picker or a hex field,
//! calls `on_change(index, value)` and the caller passes the new tokens back.
use super::{MIN_CONTRAST, ThemeEditor, ThemeToken, contrast_ratio, lookup, parse_hex, to_hex};
use crate::{
    icons::{CarbonIcon, Icon},
    t3::craft::{mono, rem, sans},
    t3::press::{hover, inset_frame, track_hover},
    theme::color,
    tokens::ColorRole,
    u0::fields::{field_fill, line_input},
    u0::primitives::{color_picker::ColorPicker, parts},
};
use gpui_kit::{
    AnyElement, App, AppContext as _, Context, ElementId, Entity, Focusable as _, FontWeight, Hsla,
    InteractiveElement as _, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement as _, Styled, Subscription, Window,
    component::input::{InputEvent, InputState},
    div,
    prelude::FluentBuilder as _,
    white,
};
use std::rc::Rc;

/// `on_change(token_index, value)`.
pub(super) type ChangeHandler = Rc<dyn Fn(usize, Hsla, &mut Window, &mut App)>;

/// The text drawn on `interactive` (the preview's button and the third
/// contrast check): the `text-on-color` token when the theme has one, else
/// whichever of white and the theme's `background` reads better on it (the
/// board's dark text on a light `interactive`).
pub fn on_color(tokens: &[ThemeToken]) -> Hsla {
    let find = |name: &str| tokens.iter().find(|t| t.name == name).map(|t| t.value);
    if let Some(c) = find("text-on-color") {
        return c;
    }
    let accent = find("interactive").unwrap_or_else(|| color(ColorRole::Interactive));
    let dark = find("background").unwrap_or_else(|| color(ColorRole::Background));
    if contrast_ratio(dark, accent) > contrast_ratio(white(), accent) {
        dark
    } else {
        white()
    }
}

/// The three Contrast checks' ratios: `text-primary` on `background`,
/// `text-secondary` on `layer-01`, and [`on_color`] on `interactive`. Each
/// passes at [`MIN_CONTRAST`].
pub fn contrast_checks(tokens: &[ThemeToken]) -> [f32; 3] {
    let values: Vec<Hsla> = tokens.iter().map(|t| t.value).collect();
    let get = |name: &str, role: ColorRole| lookup(tokens, &values, name, color(role));
    [
        contrast_ratio(
            get("text-primary", ColorRole::TextPrimary),
            get("background", ColorRole::Background),
        ),
        contrast_ratio(
            get("text-secondary", ColorRole::TextSecondary),
            get("layer-01", ColorRole::Layer01),
        ),
        contrast_ratio(on_color(tokens), get("interactive", ColorRole::Interactive)),
    ]
}

/// A ratio as the board writes it: `15.6:1`.
pub fn ratio_text(ratio: f32) -> String {
    format!("{ratio:.1}:1")
}

/// One token's hex field: Kit's text state, committed on blur or Return.
struct HexField {
    input: Entity<InputState>,
    /// The value the field last showed.
    shown: Hsla,
    index: usize,
    on_change: Option<ChangeHandler>,
    _subscription: Subscription,
}

impl HexField {
    fn new(index: usize, value: Hsla, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| InputState::new(window, cx).default_value(to_hex(value)));
        let subscription = cx.subscribe_in(&input, window, |this: &mut Self, _, e, window, cx| {
            if matches!(e, InputEvent::Blur | InputEvent::PressEnter { .. }) {
                this.commit(window, cx)
            }
        });
        Self {
            input,
            shown: value,
            index,
            on_change: None,
            _subscription: subscription,
        }
    }

    /// A valid `#rrggbb` (or `#rgb`) is the token's new value; anything else
    /// puts the current one back.
    fn commit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let text = self.input.read(cx).value().to_string();
        let typed = if text.starts_with('#') {
            text
        } else {
            format!("#{text}")
        };
        match parse_hex(&typed) {
            Some(value) if to_hex(value) != to_hex(self.shown) => {
                self.shown = value;
                let hex = to_hex(value);
                self.input.update(cx, |s, cx| s.set_value(hex, window, cx));
                if let Some(f) = self.on_change.clone() {
                    f(self.index, value, window, cx);
                }
            }
            _ => {
                let hex = to_hex(self.shown);
                self.input.update(cx, |s, cx| s.set_value(hex, window, cx));
            }
        }
        cx.notify();
    }
}

/// A swatch `size` px square with its hairline (`.ap-tok-sw { box-shadow:
/// inset 0 0 0 1px border-strong-01 }`; the rows' `.g-sw-chip` hairline is
/// `border-subtle-01`).
fn swatch(value: Hsla, size: f32, frame: ColorRole) -> AnyElement {
    div()
        .flex_none()
        .size(rem(size))
        .bg(value)
        .shadow(inset_frame(color(frame)))
        .into_any_element()
}

impl ThemeEditor {
    /// The Customize layout. See the module docs.
    pub(super) fn render_customize(self, window: &mut Window, cx: &mut App) -> AnyElement {
        let id = self.id.clone();
        let tokens = self.tokens.clone();
        let texts = self.texts.clone();
        let on_change = self.on_change.clone();
        let values: Vec<Hsla> = tokens.iter().map(|t| t.value).collect();
        let primary: Vec<usize> = self
            .primary
            .iter()
            .copied()
            .filter(|i| *i < tokens.len())
            .collect();
        let selected_state = window.use_keyed_state((id.clone(), "selected"), cx, {
            let first = primary.first().copied().unwrap_or(0);
            move |_, _| first
        });
        let selected = (*selected_state.read(cx)).min(tokens.len().saturating_sub(1));

        // ── Left: the primary tokens, then the preview ──
        let rows: Vec<AnyElement> = primary
            .iter()
            .map(|&i| {
                let t = &tokens[i];
                let row_id: ElementId = (id.clone(), SharedString::from(format!("row-{i}"))).into();
                let (over, store) = hover((row_id.clone(), "hover"), window, cx);
                let focus = parts::keyed_focus((row_id.clone(), "focus"), window, cx);
                let ring = parts::focus_visible(&focus, window);
                let on = i == selected;
                let select = selected_state.clone();
                gpui_kit::base::Button::new(row_id)
                    .track_focus(&focus)
                    .accessibility_label(t.name.clone())
                    .relative()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_start()
                    .gap(rem(8.))
                    .p(rem(4.))
                    .w_full()
                    .min_w_0()
                    // `.ap-edit .g-sw { background: field-02 }`, hover `field-hover-02`.
                    .bg(color(if over {
                        ColorRole::FieldHover02
                    } else {
                        ColorRole::Field02
                    }))
                    .on_hover(track_hover(store))
                    .on_click(move |_, _, cx| {
                        select.update(cx, |s, cx| {
                            *s = i;
                            cx.notify();
                        })
                    })
                    .child(swatch(t.value, 32., ColorRole::BorderSubtle01))
                    .child(
                        // `.g-sw-text span { font-size: 12px }`, mono.
                        div()
                            .flex()
                            .flex_col()
                            .min_w_0()
                            .child(
                                mono(div(), 12., 400)
                                    .overflow_hidden()
                                    .whitespace_nowrap()
                                    .text_ellipsis()
                                    .text_color(color(ColorRole::TextPrimary))
                                    .child(t.name.clone()),
                            )
                            .child(
                                mono(div(), 12., 400)
                                    .text_color(color(ColorRole::TextSecondary))
                                    .child(to_hex(t.value)),
                            ),
                    )
                    // `.ap-edit .g-sw:nth-child(5) { outline: 2px solid focus; outline-offset: -2px }`:
                    // the token the picker edits (and keyboard focus).
                    .when(on || ring, |this| this.child(parts::focus_ring()))
                    .into_any_element()
            })
            .collect();
        let left = div()
            .flex()
            .flex_col()
            .gap(rem(8.))
            .flex_1()
            .min_w_0()
            .children(rows)
            .child(div().mt(rem(8.)).child(super::preview(
                &tokens,
                &values,
                &texts,
                on_color(&tokens),
            )));

        // ── Right: the picker and the contrast checks ──
        let picked = tokens.get(selected).map(|t| (t.name.clone(), t.value));
        let picker = picked.map(|(name, value)| {
            let on_change = on_change.clone();
            ColorPicker::new((id.clone(), "picker"))
                .label(name)
                .value(to_hex(value))
                .labels(texts.picker.clone())
                .full_width(true)
                .on_value_change(move |hex, window, cx| {
                    if let (Some(v), Some(f)) = (parse_hex(&hex), on_change.as_ref()) {
                        f(selected, v, window, cx)
                    }
                })
        });
        let checks = contrast_checks(&tokens);
        let labels = [
            texts.contrast_text.clone(),
            texts.contrast_secondary.clone(),
            texts.contrast_button.clone(),
        ];
        // `.ap-check { display: flex; flex-direction: column; gap: 8px; padding-top: 12px;
        // box-shadow: inset 0 1px 0 border-subtle-01 }`, `b { font-size: 12px; font-weight: 600 }`.
        let contrast = div()
            .flex()
            .flex_col()
            .gap(rem(8.))
            .pt(rem(12.))
            .border_t_1()
            .border_color(color(ColorRole::BorderSubtle01))
            .child(sans(div(), 12., 600).child(texts.contrast.clone()))
            .children(labels.into_iter().zip(checks).map(|(label, ratio)| {
                let pass = ratio >= MIN_CONTRAST;
                // `.ap-check span { display: flex; align-items: center; gap: 6px;
                // font-size: 12px; line-height: 16px; color: text-secondary }`,
                // `svg { fill: support-success }`, `i { margin-left: auto; color: text-primary }`.
                sans(div(), 12., 400)
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(rem(6.))
                    .text_color(color(ColorRole::TextSecondary))
                    .child(
                        Icon::carbon(if pass {
                            CarbonIcon::CheckmarkFilled
                        } else {
                            CarbonIcon::ErrorFilled
                        })
                        .size_length(rem(16.))
                        .color(color(if pass {
                            ColorRole::SupportSuccess
                        } else {
                            ColorRole::SupportError
                        })),
                    )
                    .child(div().min_w_0().child(label))
                    .child(
                        div()
                            .ml_auto()
                            .flex_none()
                            .text_color(color(ColorRole::TextPrimary))
                            .child(ratio_text(ratio)),
                    )
            }));
        // `.ap-right { display: flex; flex-direction: column; gap: 16px; min-width: 0 }`.
        let right = div()
            .flex()
            .flex_col()
            .gap(rem(16.))
            .flex_1()
            .min_w_0()
            .children(picker)
            .child(contrast);

        // ── Tokens: every token with a hex field ──
        let fields = window.use_keyed_state((id.clone(), "hex-fields"), cx, |_, _| {
            Vec::<Entity<HexField>>::new()
        });
        let have = fields.read(cx).len();
        if have != tokens.len() {
            let mut made: Vec<_> = fields.read(cx).iter().take(tokens.len()).cloned().collect();
            for (i, t) in tokens.iter().enumerate().skip(made.len()) {
                let v = t.value;
                made.push(cx.new(|cx| HexField::new(i, v, window, cx)));
            }
            fields.update(cx, |f, _| *f = made);
        }
        let fields: Vec<_> = fields.read(cx).clone();
        let cells: Vec<AnyElement> = tokens
            .iter()
            .zip(&fields)
            .map(|(t, field)| {
                // Follow the caller's value while the field is not being typed in.
                let focused = {
                    let input = field.read(cx).input.clone();
                    input.read(cx).focus_handle(cx).is_focused(window)
                };
                let on_change = on_change.clone();
                let value = t.value;
                field.update(cx, |f, cx| {
                    f.on_change = on_change;
                    if !focused && to_hex(f.shown) != to_hex(value) {
                        f.shown = value;
                        let hex = to_hex(value);
                        f.input.update(cx, |s, cx| s.set_value(hex, window, cx));
                    }
                });
                let input = field.read(cx).input.clone();
                // `.ap-tok { display: flex; align-items: center; gap: 8px; height: 32px }`.
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(rem(8.))
                    .h(rem(32.))
                    .flex_1()
                    .min_w_0()
                    .child(swatch(t.value, 20., ColorRole::BorderStrong01))
                    // `.ap-tok code { flex: 1; min-width: 0; font-family: mono; font-size: 12px;
                    // line-height: 16px; text-overflow: ellipsis }`.
                    .child(
                        mono(div(), 12., 400)
                            .flex_1()
                            .min_w_0()
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .text_ellipsis()
                            .text_color(color(ColorRole::TextPrimary))
                            .child(t.name.clone()),
                    )
                    // `.ap-tok input { width: 88px; height: 28px; padding: 0 8px; background:
                    // field-02; border-bottom: 1px solid border-strong-01; font-family: mono;
                    // font-size: 12px }`.
                    .child(
                        line_input(&input, true, field_fill(true), None, window, cx)
                            .flex_none()
                            .w(rem(88.))
                            .h(rem(28.))
                            .px(rem(8.)),
                    )
                    .into_any_element()
            })
            .collect();
        // `.ap-tokens-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 4px 16px }`.
        let mut grid = Vec::new();
        let mut cells = cells.into_iter();
        while let Some(first) = cells.next() {
            grid.push(
                div().flex().flex_row().gap(rem(16.)).child(first).child(
                    cells
                        .next()
                        .unwrap_or_else(|| div().flex_1().min_w_0().into_any_element()),
                ),
            );
        }
        // `.ap-tokens { display: flex; flex-direction: column; gap: 8px; padding-top: 12px;
        // box-shadow: inset 0 1px 0 border-subtle-01 }`.
        let all = div()
            .flex()
            .flex_col()
            .gap(rem(8.))
            .pt(rem(12.))
            .border_t_1()
            .border_color(color(ColorRole::BorderSubtle01))
            .child(
                // `.ap-tokens-h { gap: 2px }`, `b { 14/20 600 }`, `span { 12/16 text-secondary }`.
                div()
                    .flex()
                    .flex_col()
                    .gap(rem(2.))
                    .child(
                        sans(div(), 14., 600)
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(texts.tokens_title.clone()),
                    )
                    .child(
                        sans(div(), 12., 400)
                            .text_color(color(ColorRole::TextSecondary))
                            .child(texts.tokens_description.clone()),
                    ),
            )
            .child(div().flex().flex_col().gap(rem(4.)).children(grid));

        // `.ap-dbody { display: flex; flex-direction: column; gap: 16px }`,
        // `.ap-edit { display: grid; grid-template-columns: 1fr 1fr; gap: 24px }`.
        sans(div(), 14., 400)
            .id(id)
            .flex()
            .flex_col()
            .gap(rem(16.))
            .w_full()
            .text_color(color(ColorRole::TextPrimary))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_start()
                    .gap(rem(24.))
                    .w_full()
                    .child(left)
                    .child(right),
            )
            .child(all)
            .into_any_element()
    }
}
