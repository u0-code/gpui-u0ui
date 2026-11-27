//! SecretField: a secret typed in (masked, revealable) or referenced from the
//! macOS Keychain or 1Password (`SecretField` in `src/ghost/primitives.jsx`).
//!
//! - **Behavior**: a ghost `sm` [`DropdownMenu`] at the start switches the
//!   source. Typed values are masked until the eye button reveals them
//!   (`aria-pressed`). Reference sources show the reference path in mono
//!   (`op://vault/item/field`, `service / account`) and a source badge; the
//!   secret itself is never shown. Kit's [`InputState`] edits.
//! - **Appearance** (`.g-secret`): a `field-01` field with a
//!   `border-strong-01` rule and a `focus` outline while focused; the source
//!   menu ends in a 1px `border-subtle-01` rule; the badge a purple pill
//!   (`tag-background-purple` / `tag-color-purple`, 20px line).
use super::parts::{self, IconButton, rem};
use crate::{
    components::{DropdownMenu, DropdownMenuKind, DropdownMenuSize, MenuItem, form},
    density,
    icons::CarbonIcon,
    theme::color,
    tokens::ColorRole,
};
use gpui_kit::{
    App, AppContext as _, ElementId, Entity, Focusable as _, InteractiveElement as _, IntoElement,
    MouseButton, ParentElement, RenderOnce, SharedString, Styled, Window,
    component::input::{InputEvent, InputState},
    div,
    prelude::FluentBuilder as _,
    px,
};
use std::rc::Rc;

/// Where the secret comes from (`source`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SecretSource {
    /// `plain`: typed value.
    #[default]
    Plain,
    /// `keychain`: macOS Keychain item.
    Keychain,
    /// `onepassword`: 1Password reference.
    OnePassword,
}

impl SecretSource {
    /// Every source, in menu order.
    pub const ALL: [SecretSource; 3] = [Self::Plain, Self::Keychain, Self::OnePassword];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Plain => "plain",
            Self::Keychain => "keychain",
            Self::OnePassword => "onepassword",
        }
    }

    /// The reference's menu label (`SOURCES[s][0]`).
    pub const fn label(self) -> &'static str {
        match self {
            Self::Plain => "Typed value",
            Self::Keychain => "macOS Keychain",
            Self::OnePassword => "1Password reference",
        }
    }

    /// The menu icon (`SOURCES[s][1]`).
    pub const fn icon(self) -> CarbonIcon {
        match self {
            Self::Plain => CarbonIcon::Password,
            Self::Keychain => CarbonIcon::Locked,
            Self::OnePassword => CarbonIcon::Key,
        }
    }

    /// The placeholder (`ref`): the reference format, or dots for a typed value.
    pub const fn reference(self) -> &'static str {
        match self {
            Self::Plain => "••••••••",
            Self::Keychain => "service / account",
            Self::OnePassword => "op://vault/item/field",
        }
    }

    /// The badge beside a reference (`Keychain` / `1Password`); none when typed.
    pub const fn badge(self) -> Option<&'static str> {
        match self {
            Self::Plain => None,
            Self::Keychain => Some("Keychain"),
            Self::OnePassword => Some("1Password"),
        }
    }
}

type SourceHandler = Rc<dyn Fn(SecretSource, &mut Window, &mut App)>;
type TextHandler = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;

struct Model {
    input: Entity<InputState>,
    source: SecretSource,
    show: bool,
    masked: Option<bool>,
    on_change: Option<TextHandler>,
    _subscription: gpui_kit::Subscription,
}

/// A secret field. See the module docs.
#[derive(IntoElement)]
pub struct SecretField {
    id: ElementId,
    label: Option<SharedString>,
    value: SharedString,
    source: SecretSource,
    sources: Vec<SecretSource>,
    placeholder: Option<SharedString>,
    show_labels: (SharedString, SharedString),
    on_change: Option<TextHandler>,
    on_source_change: Option<SourceHandler>,
}

impl SecretField {
    /// A secret field with a stable `id`.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            value: SharedString::default(),
            source: SecretSource::Plain,
            sources: SecretSource::ALL.to_vec(),
            placeholder: None,
            // The reference's `aria-label`s; override per locale.
            show_labels: ("Show value".into(), "Hide value".into()),
            on_change: None,
            on_source_change: None,
        }
    }

    /// The label above.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// The initial text (`defaultValue`).
    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = value.into();
        self
    }

    /// The initial source.
    pub fn source(mut self, source: SecretSource) -> Self {
        self.source = source;
        self
    }

    /// The sources offered in the menu.
    pub fn sources(mut self, sources: impl IntoIterator<Item = SecretSource>) -> Self {
        self.sources = sources.into_iter().collect();
        self
    }

    /// Placeholder (default: the source's reference format).
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Accessible names of the eye button (show, hide).
    pub fn show_labels(
        mut self,
        show: impl Into<SharedString>,
        hide: impl Into<SharedString>,
    ) -> Self {
        self.show_labels = (show.into(), hide.into());
        self
    }

    /// Called with the text on each edit.
    pub fn on_change(mut self, f: impl Fn(SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }

    /// Called when the source changes.
    pub fn on_source_change(
        mut self,
        f: impl Fn(SecretSource, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_source_change = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for SecretField {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let initial = self.value.clone();
        let source0 = self.source;
        let model = window.use_keyed_state((id.clone(), "secret"), cx, |window, cx| {
            let input = cx.new(|cx| InputState::new(window, cx).default_value(initial));
            let subscription =
                cx.subscribe_in(&input, window, |this: &mut Model, input, e, window, cx| {
                    if matches!(e, InputEvent::Change)
                        && let Some(f) = this.on_change.clone()
                    {
                        let v = input.read(cx).value();
                        f(v, window, cx);
                    }
                });
            Model {
                input,
                source: source0,
                show: false,
                masked: None,
                on_change: None,
                _subscription: subscription,
            }
        });
        model.update(cx, |m, _| m.on_change = self.on_change.clone());
        let (input, source, show) = {
            let m = model.read(cx);
            (m.input.clone(), m.source, m.show)
        };
        // `type={src === 'plain' && !show ? 'password' : 'text'}`.
        let want_masked = source == SecretSource::Plain && !show;
        if model.read(cx).masked != Some(want_masked) {
            model.update(cx, |m, cx| {
                m.masked = Some(want_masked);
                m.input
                    .update(cx, |s, cx| s.set_masked(want_masked, window, cx));
            });
        }
        let focused = input.read(cx).focus_handle(cx).is_focused(window);
        let empty = input.read(cx).value().is_empty();
        let field_h = density::current().field().px_value();
        let mono = source != SecretSource::Plain;
        let placeholder: SharedString = self
            .placeholder
            .clone()
            .unwrap_or_else(|| source.reference().into());

        let menu = DropdownMenu::new((id.clone(), "source"))
            .label(source.label())
            .kind(DropdownMenuKind::Ghost)
            .size(DropdownMenuSize::Sm)
            .items(self.sources.iter().map(|&s| {
                let model = model.clone();
                let f = self.on_source_change.clone();
                MenuItem::item(s.label())
                    .icon(s.icon())
                    .on_select(move |window, cx| {
                        model.update(cx, |m, cx| {
                            m.source = s;
                            cx.notify();
                        });
                        if let Some(f) = &f {
                            f(s, window, cx);
                        }
                    })
            }));

        let editor = div()
            .relative()
            .flex()
            .flex_1()
            .min_w_0()
            .h(rem(field_h))
            .px(rem(12.))
            .items_center()
            .on_mouse_down(MouseButton::Left, {
                let focus = input.read(cx).focus_handle(cx);
                move |_, window, cx| focus.focus(window, cx)
            })
            .child(
                if mono {
                    parts::mono_editor(&input)
                } else {
                    parts::editor(&input)
                }
                .when_some(self.label.clone(), |e, l| e.aria_label(l))
                .text_color(color(ColorRole::TextPrimary)),
            )
            .when(empty, |d| {
                d.child(
                    div()
                        .absolute()
                        .left(rem(12.))
                        .right(rem(12.))
                        .top_0()
                        .bottom_0()
                        .flex()
                        .items_center()
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .text_color(color(ColorRole::TextPlaceholder))
                        .map(|d| {
                            if mono {
                                d.font(crate::typography::font(
                                    crate::tokens::FontFamilyKey::Mono,
                                    400,
                                ))
                                .text_size(rem(12.))
                            } else {
                                d.text_size(rem(14.))
                            }
                        })
                        .child(placeholder),
                )
            });

        let end = match source.badge() {
            None => {
                let (show_label, hide_label) = self.show_labels.clone();
                let model = model.clone();
                IconButton::new(
                    (id.clone(), "show"),
                    if show {
                        CarbonIcon::ViewOff
                    } else {
                        CarbonIcon::View
                    },
                    if show { hide_label } else { show_label },
                )
                .on_click(move |_, _, cx| {
                    model.update(cx, |m, cx| {
                        m.show = !m.show;
                        cx.notify();
                    })
                })
                .into_any_element()
            }
            Some(badge) => div()
                // `.g-secret-badge { margin-right: spacing-03; padding: 0 spacing-03;
                // background: tag-background-purple; color: tag-color-purple;
                // font-size: 0.75rem; line-height: 1.25rem; border-radius: 16 }`.
                .flex_none()
                .mr(rem(8.))
                .px(rem(8.))
                .rounded(rem(16.))
                .bg(color(ColorRole::TagBackgroundPurple))
                .text_color(color(ColorRole::TagColorPurple))
                .text_size(rem(12.))
                .line_height(rem(20.))
                .child(badge)
                .into_any_element(),
        };

        let body = div()
            .relative()
            .flex()
            .items_center()
            .min_h(rem(field_h))
            .bg(color(ColorRole::Field01))
            .child(
                // `.g-secret-src { flex: none; border-right: 1px solid var(--border-subtle-01) }`.
                div()
                    .flex_none()
                    .border_r(px(1.))
                    .border_color(color(ColorRole::BorderSubtle01))
                    .child(menu),
            )
            .child(editor)
            .child(end)
            .child(form::bottom_rule(color(ColorRole::BorderStrong01)))
            .when(focused, |d| d.child(parts::focus_ring()));
        parts::field()
            .when_some(self.label.clone(), |f, l| f.child(parts::label(l)))
            .child(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sources_follow_the_reference_table() {
        assert_eq!(
            SecretSource::OnePassword.reference(),
            "op://vault/item/field"
        );
        assert_eq!(SecretSource::Keychain.reference(), "service / account");
        assert_eq!(SecretSource::Plain.badge(), None);
        assert_eq!(SecretSource::Keychain.badge(), Some("Keychain"));
        assert_eq!(SecretSource::Plain.label(), "Typed value");
        assert_eq!(SecretSource::OnePassword.icon(), CarbonIcon::Key);
    }
}
