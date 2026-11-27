//! KeyValueEditor: environment variables, as name / value / secret / remove
//! rows (`KeyValueEditor` in `src/ghost/primitives.jsx`).
//!
//! - **Behavior**: names are forced to `UPPER_SNAKE` as typed (anything but
//!   `A-Z0-9_` becomes `_`); a name used twice marks its rows (red outline on
//!   the name, "Duplicate name" under the row). The lock button toggles a
//!   row's secret flag (secret values are masked). "Add variable" appends an
//!   empty row; "Paste .env" appends the clipboard's `KEY=value` lines. Kit's
//!   [`InputState`] edits each cell.
//! - **Appearance** (`.g-kv-*`): a `layer-01` block; the header row (12px
//!   `text-secondary` on `background`) and each row lay out
//!   `2fr 3fr 32px 32px` with 1px gaps; cells are `field-01` mono inputs
//!   with a 2px `focus` outline; rows end in a `border-subtle-01` rule; the
//!   footer holds two ghost `sm` buttons on `background`.
use super::parts::{self, IconButton, rem};
use crate::{
    components::{Button, ButtonSize, ButtonVariant},
    density,
    icons::CarbonIcon,
    theme::color,
    tokens::ColorRole,
};
use gpui_kit::{
    App, AppContext as _, Context, ElementId, Entity, Focusable as _, InteractiveElement as _,
    IntoElement, MouseButton, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement as _, Styled, Subscription, Window,
    component::input::{InputEvent, InputState},
    div,
    prelude::FluentBuilder as _,
    px,
};
use std::rc::Rc;

/// One variable.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EnvVar {
    /// Name (`API_URL`).
    pub key: String,
    /// Value.
    pub value: String,
    /// Stored in the keychain, shown masked.
    pub secret: bool,
}

impl EnvVar {
    /// A non-secret variable.
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            secret: false,
        }
    }

    /// Mark it secret.
    pub fn secret(mut self, secret: bool) -> Self {
        self.secret = secret;
        self
    }
}

/// `e.target.value.toUpperCase().replace(/[^A-Z0-9_]/g, '_')`.
pub fn env_name(text: &str) -> String {
    text.to_uppercase()
        .chars()
        .map(|c| {
            if c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// `dup(k)`: a non-empty name used by more than one row.
pub fn is_duplicate(rows: &[EnvVar], key: &str) -> bool {
    !key.is_empty() && rows.iter().filter(|r| r.key == key).count() > 1
}

/// Parse `.env` text: `KEY=value` lines (an optional `export `, surrounding
/// quotes stripped); blank lines and `#` comments skipped.
pub fn parse_dotenv(text: &str) -> Vec<EnvVar> {
    text.lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let line = line.strip_prefix("export ").unwrap_or(line);
            let (k, v) = line.split_once('=')?;
            let v = v.trim();
            let v = v
                .strip_prefix('"')
                .and_then(|v| v.strip_suffix('"'))
                .or_else(|| v.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))
                .unwrap_or(v);
            Some(EnvVar::new(env_name(k.trim()), v))
        })
        .collect()
}

type Handler = Rc<dyn Fn(Vec<EnvVar>, &mut Window, &mut App)>;

struct Row {
    key: Entity<InputState>,
    value: Entity<InputState>,
    secret: bool,
    _subscriptions: [Subscription; 2],
}

struct Model {
    rows: Vec<Row>,
    on_change: Option<Handler>,
}

impl Model {
    fn vars(&self, cx: &App) -> Vec<EnvVar> {
        self.rows
            .iter()
            .map(|r| EnvVar {
                key: r.key.read(cx).value().to_string(),
                value: r.value.read(cx).value().to_string(),
                secret: r.secret,
            })
            .collect()
    }

    fn changed(&self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(f) = self.on_change.clone() {
            let vars = self.vars(cx);
            f(vars, window, cx);
        }
        cx.notify();
    }

    fn push(&mut self, var: EnvVar, window: &mut Window, cx: &mut Context<Self>) {
        let key = cx.new(|cx| InputState::new(window, cx).default_value(var.key));
        let value = cx.new(|cx| {
            let mut s = InputState::new(window, cx).default_value(var.value);
            if var.secret {
                s.set_masked(true, window, cx);
            }
            s
        });
        let on_key = cx.subscribe_in(&key, window, |this: &mut Model, key, e, window, cx| {
            if matches!(e, InputEvent::Change) {
                // Force UPPER_SNAKE as typed.
                let text = key.read(cx).value().to_string();
                let name = env_name(&text);
                if name != text {
                    key.update(cx, |s, cx| s.set_value(name, window, cx));
                }
                this.changed(window, cx);
            }
        });
        let on_value = cx.subscribe_in(&value, window, |this: &mut Model, _, e, window, cx| {
            if matches!(e, InputEvent::Change) {
                this.changed(window, cx);
            }
        });
        self.rows.push(Row {
            key,
            value,
            secret: var.secret,
            _subscriptions: [on_key, on_value],
        });
    }
}

/// An environment-variable editor. See the module docs.
#[derive(IntoElement)]
pub struct KeyValueEditor {
    id: ElementId,
    label: SharedString,
    rows: Vec<EnvVar>,
    texts: [SharedString; 7],
    on_change: Option<Handler>,
}

impl KeyValueEditor {
    /// An editor with a stable `id`, seeded with `rows` (one empty row when none).
    pub fn new(id: impl Into<ElementId>, rows: impl IntoIterator<Item = EnvVar>) -> Self {
        Self {
            id: id.into(),
            // The reference's texts; override per locale with [`Self::texts`].
            label: "Environment variables".into(),
            rows: rows.into_iter().collect(),
            texts: [
                "Name".into(),
                "Value".into(),
                "Secret".into(),
                "Duplicate name".into(),
                "Add variable".into(),
                "Paste .env".into(),
                "value".into(),
            ],
            on_change: None,
        }
    }

    /// The group's accessible name (default "Environment variables").
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    /// Texts: the three column headers, the duplicate error, the two footer
    /// buttons and the value placeholder.
    pub fn texts(mut self, texts: [SharedString; 7]) -> Self {
        self.texts = texts;
        self
    }

    /// Called with all rows after each change.
    pub fn on_change(mut self, f: impl Fn(Vec<EnvVar>, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for KeyValueEditor {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let seed = if self.rows.is_empty() {
            vec![EnvVar::default()]
        } else {
            self.rows.clone()
        };
        let model = window.use_keyed_state((id.clone(), "kv"), cx, |window, cx| {
            let mut m = Model {
                rows: Vec::new(),
                on_change: None,
            };
            for var in seed {
                m.push(var, window, cx);
            }
            m
        });
        model.update(cx, |m, _| m.on_change = self.on_change.clone());
        let [
            h_name,
            h_value,
            h_secret,
            dup_text,
            add_text,
            paste_text,
            value_ph,
        ] = self.texts.clone();
        let field_h = density::current().field().px_value();
        let vars = model.read(cx).vars(cx);
        let rows: Vec<(Entity<InputState>, Entity<InputState>, bool)> = model
            .read(cx)
            .rows
            .iter()
            .map(|r| (r.key.clone(), r.value.clone(), r.secret))
            .collect();

        // `grid-template-columns: minmax(0, 2fr) minmax(0, 3fr) var(--density-field-md) var(--density-field-md); gap: 1px`.
        let grid = || div().flex().items_center().gap(px(1.));
        let col = |grow: f32| {
            let mut d = div().flex_basis(px(0.)).min_w_0();
            d.style().flex_grow = Some(grow);
            d.style().flex_shrink = Some(1.);
            d
        };
        let head = grid()
            .pb(rem(4.))
            .bg(color(ColorRole::Background))
            .text_size(rem(12.))
            .line_height(rem(16.))
            .text_color(color(ColorRole::TextSecondary))
            .child(col(2.).child(h_name))
            .child(col(3.).child(h_value))
            .child(
                div()
                    .w(rem(field_h))
                    .flex_none()
                    .whitespace_nowrap()
                    .child(h_secret),
            )
            .child(div().w(rem(field_h)).flex_none());

        let cell = |state: &Entity<InputState>,
                    label: String,
                    placeholder: SharedString,
                    error: bool,
                    window: &mut Window,
                    cx: &mut App| {
            let focused = state.read(cx).focus_handle(cx).is_focused(window);
            let empty = state.read(cx).value().is_empty();
            let outline = if error {
                Some(color(ColorRole::SupportError))
            } else if focused {
                Some(color(ColorRole::Focus))
            } else {
                None
            };
            div()
                .relative()
                .flex()
                .items_center()
                .h(rem(field_h))
                .px(rem(12.))
                .bg(color(ColorRole::Field01))
                .on_mouse_down(MouseButton::Left, {
                    let focus = state.read(cx).focus_handle(cx);
                    move |_, window, cx| focus.focus(window, cx)
                })
                .child(
                    parts::mono_editor(state)
                        .aria_label(label)
                        .text_color(color(ColorRole::TextPrimary)),
                )
                .when(empty, |d| {
                    d.child(
                        div()
                            .absolute()
                            .left(rem(12.))
                            .top_0()
                            .bottom_0()
                            .flex()
                            .items_center()
                            .font(crate::typography::font(
                                crate::tokens::FontFamilyKey::Mono,
                                400,
                            ))
                            .text_size(rem(12.))
                            .text_color(color(ColorRole::TextPlaceholder))
                            .child(placeholder),
                    )
                })
                .when_some(outline, |d, c| {
                    d.child(crate::components::form::inset_outline(2., c))
                })
        };

        let mut body = Vec::new();
        for (i, (key, value, secret)) in rows.iter().enumerate() {
            let k = &vars[i].key;
            let dup = is_duplicate(&vars, k);
            let name_for = if k.is_empty() {
                format!("row {}", i + 1)
            } else {
                k.clone()
            };
            let toggle = {
                let model = model.clone();
                IconButton::new(
                    (id.clone(), SharedString::from(format!("secret-{i}"))),
                    if *secret {
                        CarbonIcon::Locked
                    } else {
                        CarbonIcon::Unlocked
                    },
                    format!("Treat {name_for} as secret"),
                )
                .on(*secret)
                .on_click(move |_, window, cx| {
                    model.update(cx, |m, cx| {
                        if let Some(r) = m.rows.get_mut(i) {
                            r.secret = !r.secret;
                            let masked = r.secret;
                            r.value.update(cx, |s, cx| s.set_masked(masked, window, cx));
                        }
                        m.changed(window, cx);
                    })
                })
            };
            let remove = {
                let model = model.clone();
                IconButton::new(
                    (id.clone(), SharedString::from(format!("remove-{i}"))),
                    CarbonIcon::TrashCan,
                    format!("Remove {name_for}"),
                )
                .on_click(move |_, window, cx| {
                    model.update(cx, |m, cx| {
                        if i < m.rows.len() {
                            m.rows.remove(i);
                        }
                        m.changed(window, cx);
                    })
                })
            };
            let ph: SharedString = if *secret {
                "••••••".into()
            } else {
                value_ph.clone()
            };
            let row = div()
                .flex()
                .flex_col()
                .border_b(px(1.))
                .border_color(color(ColorRole::BorderSubtle01))
                .child(
                    grid()
                        .child(col(2.).child(cell(
                            key,
                            format!("Name {}", i + 1),
                            "API_URL".into(),
                            dup,
                            window,
                            cx,
                        )))
                        .child(col(3.).child(cell(
                            value,
                            format!("Value {}", i + 1),
                            ph,
                            false,
                            window,
                            cx,
                        )))
                        .child(toggle)
                        .child(remove),
                )
                .when(dup, |d| {
                    // `.g-kv-err { grid-column: 1 / -1; padding: spacing-02 spacing-04;
                    // font-size: 0.75rem; color: var(--text-error) }`.
                    d.child(
                        div()
                            .py(rem(4.))
                            .px(rem(12.))
                            .text_size(rem(12.))
                            .line_height(rem(16.))
                            .text_color(color(ColorRole::TextError))
                            .child(dup_text.clone()),
                    )
                });
            body.push(row);
        }

        let add = {
            let model = model.clone();
            Button::new((id.clone(), "add"), add_text)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .icon(CarbonIcon::Add)
                .on_click(move |_, window, cx| {
                    model.update(cx, |m, cx| {
                        m.push(EnvVar::default(), window, cx);
                        m.changed(window, cx);
                    })
                })
        };
        let paste = {
            let model = model.clone();
            Button::new((id.clone(), "paste"), paste_text)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .on_click(move |_, window, cx| {
                    let text = cx
                        .read_from_clipboard()
                        .and_then(|c| c.text())
                        .unwrap_or_default();
                    let vars = parse_dotenv(&text);
                    if vars.is_empty() {
                        return;
                    }
                    model.update(cx, |m, cx| {
                        for v in vars {
                            m.push(v, window, cx);
                        }
                        m.changed(window, cx);
                    })
                })
        };
        let foot = div()
            .flex()
            .gap(rem(8.))
            .py(rem(4.))
            .bg(color(ColorRole::Background))
            .child(add)
            .child(paste);

        div()
            .id(id.clone())
            .role(gpui_kit::Role::Group)
            .aria_label(self.label.clone())
            .flex()
            .flex_col()
            .bg(color(ColorRole::Layer01))
            .text_size(rem(14.))
            .child(head)
            .children(body)
            .child(foot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_upper_snake() {
        assert_eq!(env_name("api-url"), "API_URL");
        assert_eq!(env_name("Key 2.x"), "KEY_2_X");
    }

    #[test]
    fn duplicates_ignore_empty_names() {
        let rows = vec![
            EnvVar::new("API_URL", "a"),
            EnvVar::new("OPENAI_API_KEY", "b").secret(true),
            EnvVar::new("API_URL", "c"),
            EnvVar::new("", ""),
            EnvVar::new("", ""),
        ];
        assert!(is_duplicate(&rows, "API_URL"));
        assert!(!is_duplicate(&rows, "OPENAI_API_KEY"));
        assert!(!is_duplicate(&rows, ""));
    }

    #[test]
    fn dotenv_parsing() {
        let vars = parse_dotenv(
            "# comment\nexport API_URL=\"https://x\"\n\nport=3000\nbad line\nTOKEN='a=b'\n",
        );
        assert_eq!(
            vars,
            vec![
                EnvVar::new("API_URL", "https://x"),
                EnvVar::new("PORT", "3000"),
                EnvVar::new("TOKEN", "a=b"),
            ]
        );
    }
}
