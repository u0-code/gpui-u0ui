//! u0 · Settings › [`TelemetryPreview`] (`.g-tele`): what the current
//! telemetry level would send, as pretty-printed JSON (two-space indent, as
//! `JSON.stringify(payload, null, 2)`), mono, keys in `syntax-property` and
//! string values in `syntax-string`, under a `.g-codehead` with a copy
//! button. The body scrolls past 10rem.
use crate::{
    components::CopyButton,
    icons::CarbonIcon,
    t3::craft::{mono, rem},
    theme::{LayerLevel, color},
    tokens::{self, ColorRole},
    u0::panels::code_viewer::scroll_target,
    u0::panels::head::{code_head, grow, head_icon},
    u0::primitives::parts::{focus_ring, focus_visible, keyed_focus},
};
use gpui_kit::{
    App, ElementId, InteractiveElement as _, IntoElement, KeyDownEvent, ParentElement, RenderOnce,
    ScrollHandle, SharedString, StatefulInteractiveElement as _, Styled, Window, div, point,
    prelude::FluentBuilder as _, px,
};
use std::rc::Rc;

/// `.g-tele-pre { max-height: 10rem }`.
const MAX_HEIGHT: f32 = 160.;
/// `.g-tele-pre { font-size: 0.75rem; line-height: 1.4 }`.
const LINE: f32 = 12. * 1.4;
/// `.t3-copy.cds--copy-btn { width: var(--size-sm); height: var(--size-sm) }`.
const COPY: f32 = 32.;

/// A JSON value (`payload`), objects keeping their key order.
#[derive(Clone, Debug, PartialEq)]
pub enum Payload {
    /// `null`.
    Null,
    /// `true` / `false`.
    Bool(bool),
    /// A number.
    Number(f64),
    /// A string.
    String(SharedString),
    /// An array.
    Array(Vec<Payload>),
    /// An object, in insertion order.
    Object(Vec<(SharedString, Payload)>),
}

impl Payload {
    /// An object from `(key, value)` pairs.
    pub fn object<K: Into<SharedString>>(pairs: impl IntoIterator<Item = (K, Payload)>) -> Self {
        Self::Object(pairs.into_iter().map(|(k, v)| (k.into(), v)).collect())
    }

    /// A string value.
    pub fn str(s: impl Into<SharedString>) -> Self {
        Self::String(s.into())
    }

    /// `JSON.stringify(self, null, 2)`.
    pub fn to_pretty(&self) -> String {
        let mut out = String::new();
        self.write(&mut out, 0);
        out
    }

    fn write(&self, out: &mut String, depth: usize) {
        let pad = |out: &mut String, d: usize| out.push_str(&"  ".repeat(d));
        match self {
            Self::Null => out.push_str("null"),
            Self::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            Self::Number(n) => out.push_str(&number(*n)),
            Self::String(s) => quote(out, s),
            Self::Array(items) if items.is_empty() => out.push_str("[]"),
            Self::Object(pairs) if pairs.is_empty() => out.push_str("{}"),
            Self::Array(items) => {
                out.push_str("[\n");
                for (i, v) in items.iter().enumerate() {
                    pad(out, depth + 1);
                    v.write(out, depth + 1);
                    out.push_str(if i + 1 < items.len() { ",\n" } else { "\n" });
                }
                pad(out, depth);
                out.push(']');
            }
            Self::Object(pairs) => {
                out.push_str("{\n");
                for (i, (k, v)) in pairs.iter().enumerate() {
                    pad(out, depth + 1);
                    quote(out, k);
                    out.push_str(": ");
                    v.write(out, depth + 1);
                    out.push_str(if i + 1 < pairs.len() { ",\n" } else { "\n" });
                }
                pad(out, depth);
                out.push('}');
            }
        }
    }
}

/// A number as JSON writes it (`12`, `1.5`; non-finite → `null`).
fn number(n: f64) -> String {
    if !n.is_finite() {
        "null".into()
    } else if n.fract() == 0. && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}

/// A JSON string literal.
fn quote(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

/// A highlighted run's color.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Syn {
    /// `.g-syn-property`: an object key.
    Property,
    /// `.g-syn-string`: a run starting with a string or holding `: "`.
    String,
    /// No class: the pre's `syntax-variable`.
    Plain,
}

impl Syn {
    /// The token role.
    pub fn role(self) -> ColorRole {
        match self {
            Self::Property => ColorRole::SyntaxProperty,
            Self::String => ColorRole::SyntaxString,
            Self::Plain => ColorRole::SyntaxVariable,
        }
    }
}

/// The reference's line split: keys `/("[^"]+")(:)/` become
/// [`Syn::Property`] runs; every other run is [`Syn::String`] when it starts
/// with `"` (after spaces) or holds `: "`, [`Syn::Plain`] otherwise.
pub fn line_runs(line: &str) -> Vec<(&str, Syn)> {
    let b = line.as_bytes();
    let mut out = Vec::new();
    let (mut start, mut i) = (0, 0);
    let plain = |s: &'_ str| {
        if s.trim_start().starts_with('"') || s.contains(": \"") {
            Syn::String
        } else {
            Syn::Plain
        }
    };
    while i < b.len() {
        if b[i] == b'"'
            && let Some(len) = line[i + 1..].find('"')
            && len > 0
            && b.get(i + 1 + len + 1) == Some(&b':')
        {
            let end = i + len + 2;
            if start < i {
                out.push((&line[start..i], plain(&line[start..i])));
            }
            out.push((&line[i..end], Syn::Property));
            start = end;
            i = end + 1;
            continue;
        }
        i += 1;
    }
    if start < line.len() {
        out.push((&line[start..], plain(&line[start..])));
    }
    out
}

/// The telemetry level (`level: number | string`).
#[derive(Clone, Debug, PartialEq)]
pub enum TelemetryLevel {
    /// A numbered level.
    Number(u32),
    /// A named level (`"Crash reports"`).
    Named(SharedString),
}

impl From<u32> for TelemetryLevel {
    fn from(n: u32) -> Self {
        Self::Number(n)
    }
}

impl From<&'static str> for TelemetryLevel {
    fn from(s: &'static str) -> Self {
        Self::Named(s.into())
    }
}

/// The preview's user-facing strings (the reference's English defaults).
#[derive(Clone)]
pub struct TelemetryPreviewTexts {
    /// The head for a numbered level: `"What is sent at level {level}"`.
    pub at_level: Rc<dyn Fn(u32) -> SharedString>,
    /// The head for a named level: `"What is sent at {level}"`.
    pub at_named: Rc<dyn Fn(&str) -> SharedString>,
    /// The copy button's label: `"Copy JSON"`.
    pub copy: SharedString,
    /// The copy feedback: `"Copied!"`.
    pub copied: SharedString,
}

impl Default for TelemetryPreviewTexts {
    fn default() -> Self {
        Self {
            at_level: Rc::new(|n| format!("What is sent at level {n}").into()),
            at_named: Rc::new(|s| format!("What is sent at {s}").into()),
            copy: "Copy JSON".into(),
            copied: "Copied!".into(),
        }
    }
}

impl TelemetryPreviewTexts {
    /// The head line for `level`.
    pub fn head(&self, level: &TelemetryLevel) -> SharedString {
        match level {
            TelemetryLevel::Number(n) => (self.at_level)(*n),
            TelemetryLevel::Named(s) => (self.at_named)(s),
        }
    }
}

/// The telemetry preview (`TelemetryPreview`). See the module docs.
#[derive(IntoElement)]
pub struct TelemetryPreview {
    id: ElementId,
    level: TelemetryLevel,
    payload: Payload,
    texts: TelemetryPreviewTexts,
}

impl TelemetryPreview {
    /// The preview of `payload` at `level`.
    pub fn new(
        id: impl Into<ElementId>,
        level: impl Into<TelemetryLevel>,
        payload: Payload,
    ) -> Self {
        Self {
            id: id.into(),
            level: level.into(),
            payload,
            texts: TelemetryPreviewTexts::default(),
        }
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: TelemetryPreviewTexts) -> Self {
        self.texts = texts;
        self
    }
}

impl RenderOnce for TelemetryPreview {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id;
        let json = self.payload.to_pretty();
        let scroll = window
            .use_keyed_state((id.clone(), "scroll"), cx, |_, _| ScrollHandle::new())
            .read(cx)
            .clone();
        let head = code_head()
            // `<I.Json size={14} />`.
            .child(head_icon(CarbonIcon::Json))
            .child(
                div()
                    .min_w_0()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .child(self.texts.head(&self.level)),
            )
            .child(grow())
            .child(
                CopyButton::new((id.clone(), "copy"), json.clone())
                    .label(self.texts.copy.clone())
                    .feedback(self.texts.copied.clone())
                    .size(COPY)
                    .quiet(true)
                    .layer(LayerLevel::One),
            );
        let lines = json.lines().map(|line| {
            div().flex().whitespace_nowrap().min_h(rem(LINE)).children(
                line_runs(line).into_iter().map(|(text, syn)| {
                    div()
                        .flex_none()
                        .text_color(color(syn.role()))
                        .child(text.to_string())
                }),
            )
        });
        // `.g-tele-pre { margin: 0; max-height: 10rem; overflow: auto; padding:
        // var(--spacing-03) var(--spacing-04); background: var(--background);
        // font-family: var(--font-mono); font-size: 0.75rem; line-height: 1.4;
        // color: var(--syntax-variable) }`.
        // Chromium makes an overflowing scroller a tab stop of its own (the
        // reference's Tab walk: copy, then the JSON); ↑/↓, PageUp/PageDown,
        // Home/End scroll it. The ring is the library's 2px inset `focus`.
        let focus = keyed_focus((id.clone(), "pre-focus"), window, cx);
        let ring = focus_visible(&focus, window);
        let scrolls = f32::from(scroll.max_offset().y) > 0.;
        let key_scroll = scroll.clone();
        let pre = mono(div(), 12., 400)
            .id((id.clone(), "pre"))
            .track_focus(&focus.clone().tab_index(0).tab_stop(scrolls))
            .on_key_down(move |e: &KeyDownEvent, window, cx| {
                let s = &key_scroll;
                if let Some(y) = scroll_target(
                    e.keystroke.key.as_str(),
                    f32::from(s.offset().y),
                    f32::from(s.bounds().size.height),
                    f32::from(s.max_offset().y),
                ) {
                    s.set_offset(point(s.offset().x, px(y)));
                    window.refresh();
                    cx.stop_propagation();
                }
            })
            .max_h(rem(MAX_HEIGHT))
            .overflow_scroll()
            .track_scroll(&scroll)
            .py(tokens::SPACING_03)
            .px(tokens::SPACING_04)
            .bg(color(ColorRole::Background))
            .text_color(color(ColorRole::SyntaxVariable))
            .line_height(rem(LINE))
            .child(div().flex().flex_col().children(lines));
        // `.g-tele { background: var(--layer-01) }`.
        div()
            .id(id)
            .flex()
            .flex_col()
            .w_full()
            .min_w_0()
            .bg(color(ColorRole::Layer01))
            .child(head)
            .child(
                div()
                    .relative()
                    .child(pre)
                    .when(ring, |d| d.child(focus_ring())),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The reference payload.
    fn reference() -> Payload {
        Payload::object([
            ("app", Payload::str("1.4.2")),
            ("os", Payload::str("macOS 15.2")),
            (
                "crash",
                Payload::object([
                    ("signal", Payload::str("SIGABRT")),
                    ("frames", Payload::Number(12.)),
                ]),
            ),
            (
                "features",
                Payload::object([
                    ("palette", Payload::Number(14.)),
                    ("sideChat", Payload::Number(3.)),
                ]),
            ),
        ])
    }

    #[test]
    fn pretty_prints_like_json_stringify() {
        assert_eq!(
            reference().to_pretty(),
            "{\n  \"app\": \"1.4.2\",\n  \"os\": \"macOS 15.2\",\n  \"crash\": {\n    \"signal\": \"SIGABRT\",\n    \"frames\": 12\n  },\n  \"features\": {\n    \"palette\": 14,\n    \"sideChat\": 3\n  }\n}"
        );
        assert_eq!(
            Payload::Array(vec![
                Payload::Null,
                Payload::Bool(true),
                Payload::Number(1.5)
            ])
            .to_pretty(),
            "[\n  null,\n  true,\n  1.5\n]"
        );
        assert_eq!(Payload::object::<&str>([]).to_pretty(), "{}");
        assert_eq!(Payload::str("a\"b").to_pretty(), "\"a\\\"b\"");
    }

    #[test]
    fn lines_split_keys_and_strings() {
        assert_eq!(
            line_runs("  \"os\": \"macOS 15.2\","),
            [
                ("  ", Syn::Plain),
                ("\"os\"", Syn::Property),
                (": \"macOS 15.2\",", Syn::String)
            ]
        );
        assert_eq!(
            line_runs("    \"frames\": 12"),
            [
                ("    ", Syn::Plain),
                ("\"frames\"", Syn::Property),
                (": 12", Syn::Plain)
            ]
        );
        assert_eq!(line_runs("  \"a\","), [("  \"a\",", Syn::String)]);
        assert_eq!(line_runs("}"), [("}", Syn::Plain)]);
    }

    #[test]
    fn head_names_the_level() {
        let t = TelemetryPreviewTexts::default();
        assert_eq!(t.head(&2.into()).as_ref(), "What is sent at level 2");
        assert_eq!(
            t.head(&"Crash reports".into()).as_ref(),
            "What is sent at Crash reports"
        );
        assert_eq!(Syn::Property.role(), ColorRole::SyntaxProperty);
    }
}
