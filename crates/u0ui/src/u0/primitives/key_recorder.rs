//! KeyRecorder: record a shortcut by pressing it (`KeyRecorder` in
//! `src/ghost/primitives.jsx`).
//!
//! - **Behavior**: pressing the button starts recording. Held modifiers show
//!   live; the first non-modifier key completes the shortcut (`⌘+⇧+K`,
//!   modifiers as ⌃⌥⇧⌘ in that order). Escape without ⌘ cancels; Tab still
//!   moves focus; losing focus stops recording. The clear button empties it.
//! - **Appearance** (`.g-keyrec`): a `field-01` field at least 9rem wide with
//!   a `border-strong-01` rule; recording adds a 2px `focus` outline on
//!   `highlight`, a conflict a 2px `support-error` outline and an error line
//!   naming the command that already uses it. Keys draw as [`Kbd`]s.
use super::parts::{self, IconButton, rem};
use crate::{
    components::{form, kbd::Kbd},
    density,
    icons::CarbonIcon,
    theme::color,
    tokens::ColorRole,
};
use gpui_kit::{
    App, CursorStyle, ElementId, Hsla, InteractiveElement, IntoElement, KeyDownEvent, Modifiers,
    ModifiersChangedEvent, ParentElement, RenderOnce, SharedString, Styled, Window, div,
    prelude::FluentBuilder as _, px,
};
use std::rc::Rc;

/// The modifier glyphs held, in the reference order ⌃ ⌥ ⇧ ⌘.
pub fn modifier_glyphs(m: &Modifiers) -> Vec<&'static str> {
    [
        (m.control, "⌃"),
        (m.alt, "⌥"),
        (m.shift, "⇧"),
        (m.platform, "⌘"),
    ]
    .into_iter()
    .filter_map(|(on, g)| on.then_some(g))
    .collect()
}

/// A GPUI key name as the reference shows it (`KEYNAME`, `+` → `Plus`, else
/// upper case); `None` for a modifier key.
pub fn key_name(key: &str) -> Option<String> {
    Some(
        match key {
            "shift" | "control" | "alt" | "platform" | "cmd" | "function" | "capslock" => {
                return None;
            }
            "up" => "↑",
            "down" => "↓",
            "left" => "←",
            "right" => "→",
            "enter" => "↵",
            "escape" => "Esc",
            "backspace" => "⌫",
            "space" => "Space",
            "+" => "Plus",
            other => return Some(other.to_uppercase()),
        }
        .to_string(),
    )
}

/// `[...mods, k].join('+')`.
pub fn combo(m: &Modifiers, key: Option<&str>) -> String {
    let mut parts: Vec<String> = modifier_glyphs(m).into_iter().map(String::from).collect();
    if let Some(k) = key {
        parts.push(k.to_string());
    }
    parts.join("+")
}

/// `.g-keyrec` (fill, outline): `--rec` focus on highlight; `--conflict`
/// `support-error` (later in the cascade, so it wins while recording too).
pub fn look(recording: bool, conflict: bool) -> (Hsla, Hsla) {
    let fill = color(if recording {
        ColorRole::Highlight
    } else {
        ColorRole::Field01
    });
    let outline = if conflict {
        color(ColorRole::SupportError)
    } else if recording {
        color(ColorRole::Focus)
    } else {
        Hsla::transparent_black()
    };
    (fill, outline)
}

#[derive(Default)]
struct Rec {
    recording: bool,
    live: String,
}

type Handler = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;

/// A shortcut recorder. See the module docs.
#[derive(IntoElement)]
pub struct KeyRecorder {
    id: ElementId,
    label: Option<SharedString>,
    value: Option<SharedString>,
    default_value: SharedString,
    conflict: Option<SharedString>,
    texts: [SharedString; 5],
    on_value_change: Option<Handler>,
}

impl KeyRecorder {
    /// A recorder with a stable `id` (uncontrolled, not set).
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            value: None,
            default_value: SharedString::default(),
            conflict: None,
            // The reference's texts; override per locale with [`Self::texts`].
            texts: [
                "Press a shortcut…".into(),
                "Not set".into(),
                "Esc to cancel".into(),
                "Click, then press the keys".into(),
                "Clear shortcut".into(),
            ],
            on_value_change: None,
        }
    }

    /// The label above.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Control the value (`⌘+⇧+K`; empty = not set).
    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// The initial value when uncontrolled.
    pub fn default_value(mut self, value: impl Into<SharedString>) -> Self {
        self.default_value = value.into();
        self
    }

    /// The command that already uses this shortcut.
    pub fn conflict(mut self, command: impl Into<SharedString>) -> Self {
        self.conflict = Some(command.into());
        self
    }

    /// Texts: recording hint, not-set hint, recording helper, idle helper,
    /// clear button label.
    pub fn texts(mut self, texts: [SharedString; 5]) -> Self {
        self.texts = texts;
        self
    }

    /// Called with each recorded (or cleared) shortcut.
    pub fn on_value_change(
        mut self,
        f: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_value_change = Some(Rc::new(f));
        self
    }
}

fn keys(combo: &str) -> Vec<Kbd> {
    combo
        .split('+')
        .filter(|k| !k.is_empty())
        .map(|k| Kbd::new(k.to_string()))
        .collect()
}

impl RenderOnce for KeyRecorder {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let own = window.use_keyed_state((id.clone(), "value"), cx, {
            let d = self.default_value.clone();
            move |_, _| d
        });
        let rec = window.use_keyed_state((id.clone(), "rec"), cx, |_, _| Rec::default());
        let focus = parts::keyed_focus((id.clone(), "focus"), window, cx);
        // `onBlur={() => { setRec(false); setLive('') }}`.
        if rec.read(cx).recording && !focus.is_focused(window) {
            rec.update(cx, |r, _| *r = Rec::default());
        }
        let value = self.value.clone().unwrap_or_else(|| own.read(cx).clone());
        let (recording, live) = {
            let r = rec.read(cx);
            (r.recording, r.live.clone())
        };
        let set: Handler = {
            let own = own.clone();
            let f = self.on_value_change.clone();
            let controlled = self.value.is_some();
            Rc::new(move |v, window, cx| {
                if !controlled {
                    own.update(cx, |s, cx| {
                        *s = v.clone();
                        cx.notify();
                    });
                }
                if let Some(f) = &f {
                    f(v, window, cx);
                }
            })
        };
        let [rec_hint, unset_hint, rec_help, idle_help, clear_label] = self.texts.clone();
        let (fill, outline) = look(recording, self.conflict.is_some());
        let ring = parts::focus_visible(&focus, window) && !recording;
        let field_h = density::current().field().px_value();
        let hint = |t: SharedString| {
            // `.g-keyrec-hint { font-size: 0.875rem; color: var(--text-placeholder) }`.
            div()
                .text_size(rem(14.))
                .text_color(color(ColorRole::TextPlaceholder))
                .child(t)
                .into_any_element()
        };
        let content: Vec<gpui_kit::AnyElement> = if recording {
            if live.is_empty() {
                vec![hint(rec_hint)]
            } else {
                keys(&live)
                    .into_iter()
                    .map(|k| k.into_any_element())
                    .collect()
            }
        } else if value.is_empty() {
            vec![hint(unset_hint)]
        } else {
            keys(&value)
                .into_iter()
                .map(|k| k.into_any_element())
                .collect()
        };

        let on_key = {
            let rec = rec.clone();
            let set = set.clone();
            move |e: &KeyDownEvent, window: &mut Window, cx: &mut App| {
                if !rec.read(cx).recording || e.keystroke.key == "tab" {
                    return;
                }
                cx.stop_propagation();
                let m = e.keystroke.modifiers;
                if e.keystroke.key == "escape" && !m.platform {
                    rec.update(cx, |r, cx| {
                        *r = Rec::default();
                        cx.notify();
                    });
                    return;
                }
                let k = key_name(&e.keystroke.key);
                let c = combo(&m, k.as_deref());
                if k.is_some() {
                    rec.update(cx, |r, cx| {
                        *r = Rec::default();
                        cx.notify();
                    });
                    set(c.into(), window, cx);
                } else {
                    rec.update(cx, |r, cx| {
                        r.live = c;
                        cx.notify();
                    });
                }
            }
        };
        let on_mods = {
            let rec = rec.clone();
            move |e: &ModifiersChangedEvent, _: &mut Window, cx: &mut App| {
                if rec.read(cx).recording {
                    rec.update(cx, |r, cx| {
                        r.live = combo(&e.modifiers, None);
                        cx.notify();
                    });
                }
            }
        };
        let start = rec.clone();
        let button = div()
            .id((id.clone(), "rec"))
            .capture_key_down(on_key)
            .on_modifiers_changed(on_mods)
            .child(
                gpui_kit::base::Button::new((id.clone(), "button"))
                    .track_focus(&focus)
                    .when_some(self.label.clone(), |b, l| b.accessibility_label(l))
                    .relative()
                    .flex()
                    .items_center()
                    // `display: inline-flex`: keys start at the padding (Base centers).
                    .justify_start()
                    .gap(px(2.))
                    .min_w(rem(144.))
                    .h(rem(field_h))
                    .px(rem(12.))
                    .bg(fill)
                    .cursor(CursorStyle::PointingHand)
                    .children(content)
                    .child(form::bottom_rule(color(ColorRole::BorderStrong01)))
                    .child(form::inset_outline(2., outline))
                    .when(ring, |b| b.child(parts::focus_ring()))
                    .on_click(move |_, _, cx| {
                        start.update(cx, |r, cx| {
                            r.recording = true;
                            cx.notify();
                        })
                    }),
            );
        // Where focus rests once the clear button is gone: Chromium keeps the
        // Tab starting point where a removed focused element was.
        let anchor = parts::keyed_focus((id.clone(), "anchor-focus"), window, cx);
        let clear = (!value.is_empty() && !recording).then(|| {
            let set = set.clone();
            let anchor = anchor.clone();
            IconButton::new((id.clone(), "clear"), CarbonIcon::Close, clear_label).on_click(
                move |_, window, cx| {
                    // A press focuses the button, which then disappears.
                    if window.focused(cx).is_some() {
                        anchor.focus(window, cx);
                    }
                    set(SharedString::default(), window, cx)
                },
            )
        });
        let message = match &self.conflict {
            Some(c) => parts::error(format!("Already used by “{c}”")),
            None => parts::helper(if recording { rec_help } else { idle_help }),
        };
        parts::field()
            .when_some(self.label.clone(), |f, l| f.child(parts::label(l)))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(rem(4.))
                    .child(button)
                    .children(clear)
                    .child(parts::nav_anchor((id.clone(), "anchor"), &anchor, None, cx)),
            )
            .child(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combos_use_the_reference_glyphs_and_order() {
        let m = Modifiers {
            platform: true,
            shift: true,
            ..Default::default()
        };
        assert_eq!(combo(&m, key_name("k").as_deref()), "⇧+⌘+K");
        let all = Modifiers {
            control: true,
            alt: true,
            shift: true,
            platform: true,
            function: false,
        };
        assert_eq!(combo(&all, None), "⌃+⌥+⇧+⌘");
        assert_eq!(key_name("shift"), None);
        assert_eq!(key_name("enter").as_deref(), Some("↵"));
        assert_eq!(key_name("+").as_deref(), Some("Plus"));
        assert_eq!(key_name("space").as_deref(), Some("Space"));
        assert_eq!(key_name("f5").as_deref(), Some("F5"));
    }

    #[test]
    fn conflict_outline_wins() {
        use crate::{CarbonTheme, theme::color_in};
        crate::theme::tests_set(CarbonTheme::White);
        let c = |r| color_in(CarbonTheme::White, r);
        assert_eq!(
            look(false, false),
            (c(ColorRole::Field01), Hsla::transparent_black())
        );
        assert_eq!(
            look(true, false),
            (c(ColorRole::Highlight), c(ColorRole::Focus))
        );
        assert_eq!(look(true, true).1, c(ColorRole::SupportError));
        assert_eq!(look(false, true).1, c(ColorRole::SupportError));
    }
}
