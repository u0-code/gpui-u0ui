//! TagInput: a creatable multi-value tag field (`TagInput` in
//! `src/ghost/primitives.jsx`; Carbon `FilterableMultiSelect` + `Tag`, which
//! cannot create values).
//!
//! - **Behavior**: Kit's [`InputState`] edits the query. Focus opens the
//!   suggestion list (suggestions not yet chosen whose text contains the
//!   query, then "Create …" for a new value). ↑/↓ move the active option,
//!   Enter or `,` adds it (or the typed text), Escape closes the list,
//!   Backspace in an empty field removes the last tag. A press on an option
//!   adds it. Duplicates (case-insensitive) and tags past `max` are ignored.
//! - **Appearance** (`.g-tags`): a wrapping `field-01` field with a
//!   `border-strong-01` rule and a `focus` outline while focus is anywhere
//!   inside (the input or a chip's remove button); gray pill
//!   tags (`tag-background-gray` / `tag-color-gray`, 20px, a 16px round
//!   remove button with `tag-hover-gray` hover); the list a floating
//!   `layer-01` surface (`0 2px 6px shadow`) under the field, `density-row-md`
//!   options, the active one `layer-selected-01` with a 2px
//!   `border-interactive` start bar.
use super::parts::{self, rem};
use crate::{
    components::{form, overlay},
    density,
    icons::{CarbonIcon, Icon, IconSize},
    theme::color,
    tokens::{ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    App, AppContext as _, BoxShadow, Context, CursorStyle, ElementId, Entity, Focusable as _,
    InteractiveElement, IntoElement, KeyDownEvent, MouseButton, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement as _, Styled, Subscription, Window,
    component::input::{Enter, Escape, InputEvent, InputState, MoveDown, MoveUp},
    deferred, div,
    prelude::FluentBuilder as _,
    px,
};
use std::rc::Rc;

/// A list option: an existing suggestion or "create the typed text".
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TagOption {
    /// A suggestion.
    Suggestion(SharedString),
    /// `Create “q”`.
    Create(String),
}

/// The list for `query` (`opts` then `__create`): suggestions not in `tags`
/// containing the query (case-insensitive), then Create when the trimmed
/// query is new and matches no suggestion exactly.
pub fn options(tags: &[SharedString], suggestions: &[SharedString], query: &str) -> Vec<TagOption> {
    let q = query.to_lowercase();
    let t = query.trim();
    let mut out: Vec<TagOption> = suggestions
        .iter()
        .filter(|s| !tags.contains(s) && s.to_lowercase().contains(&q))
        .map(|s| TagOption::Suggestion(s.clone()))
        .collect();
    let can_create = !t.is_empty()
        && !tags.iter().any(|x| x.as_ref() == t)
        && !suggestions
            .iter()
            .any(|s| s.to_lowercase() == t.to_lowercase());
    if can_create {
        out.push(TagOption::Create(t.to_string()));
    }
    out
}

/// Whether the field shows its focus outline (`.g-tags:focus-within`): the
/// input is focused or any of the chips' remove buttons is.
pub fn focus_within(input: bool, removes: impl IntoIterator<Item = bool>) -> bool {
    input || removes.into_iter().any(|f| f)
}

/// `add(t)`: `None` when empty, a case-insensitive duplicate, or at `max`.
pub fn add(tags: &[SharedString], tag: &str, max: Option<usize>) -> Option<Vec<SharedString>> {
    if tag.is_empty()
        || tags.iter().any(|x| x.to_lowercase() == tag.to_lowercase())
        || max.is_some_and(|m| tags.len() >= m)
    {
        return None;
    }
    let mut next = tags.to_vec();
    next.push(tag.to_string().into());
    Some(next)
}

type Handler = Rc<dyn Fn(Vec<SharedString>, &mut Window, &mut App)>;

struct Model {
    tags: Vec<SharedString>,
    input: Entity<InputState>,
    open: bool,
    active: usize,
    controlled: bool,
    on_change: Option<Handler>,
    _subscription: Subscription,
}

impl Model {
    fn set(&mut self, tags: Vec<SharedString>, window: &mut Window, cx: &mut Context<Self>) {
        if !self.controlled {
            self.tags = tags.clone();
        }
        if let Some(f) = self.on_change.clone() {
            f(tags, window, cx);
        }
        cx.notify();
    }

    fn add(&mut self, tag: &str, max: Option<usize>, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(next) = add(&self.tags, tag, max) {
            self.set(next, window, cx);
            self.input.update(cx, |s, cx| s.set_value("", window, cx));
            self.active = 0;
        }
    }
}

/// A creatable tag field. See the module docs.
#[derive(IntoElement)]
pub struct TagInput {
    id: ElementId,
    label: Option<SharedString>,
    value: Option<Vec<SharedString>>,
    default_value: Vec<SharedString>,
    suggestions: Vec<SharedString>,
    placeholder: SharedString,
    max: Option<usize>,
    fill: Option<gpui_kit::Hsla>,
    on_value_change: Option<Handler>,
}

impl TagInput {
    /// A tag field with a stable `id` (uncontrolled, empty).
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            value: None,
            default_value: Vec::new(),
            suggestions: Vec::new(),
            // The reference's default; override per locale.
            placeholder: "Add tag…".into(),
            max: None,
            fill: None,
            on_value_change: None,
        }
    }

    /// The label above.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Control the tags.
    pub fn value(mut self, tags: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.value = Some(tags.into_iter().map(Into::into).collect());
        self
    }

    /// The initial tags when uncontrolled.
    pub fn default_value(
        mut self,
        tags: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        self.default_value = tags.into_iter().map(Into::into).collect();
        self
    }

    /// Suggested values.
    pub fn suggestions(mut self, s: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.suggestions = s.into_iter().map(Into::into).collect();
        self
    }

    /// Shown while there are no tags (default "Add tag…").
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// At most this many tags (shows "Up to N tags" when reached).
    pub fn max(mut self, max: usize) -> Self {
        self.max = Some(max);
        self
    }

    /// The field fill (default `field-01`; a workflow step card uses
    /// `field-02`: `.g-wfe-card .g-tags { background: var(--field-02) }`).
    pub fn fill(mut self, fill: gpui_kit::Hsla) -> Self {
        self.fill = Some(fill);
        self
    }

    /// Called with the new tag list.
    pub fn on_value_change(
        mut self,
        f: impl Fn(Vec<SharedString>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_value_change = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for TagInput {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let initial = self.value.clone().unwrap_or(self.default_value.clone());
        let model = window.use_keyed_state((id.clone(), "tags"), cx, |window, cx| {
            let input = cx.new(|cx| InputState::new(window, cx));
            let subscription = cx.subscribe_in(&input, window, |this: &mut Model, _, e, _, cx| {
                match e {
                    InputEvent::Change => {
                        this.open = true;
                        this.active = 0;
                    }
                    InputEvent::Focus => this.open = true,
                    InputEvent::Blur => this.open = false,
                    InputEvent::PressEnter { .. } => {}
                }
                cx.notify();
            });
            Model {
                tags: initial,
                input,
                open: false,
                active: 0,
                controlled: false,
                on_change: None,
                _subscription: subscription,
            }
        });
        model.update(cx, |m, _| {
            m.on_change = self.on_value_change.clone();
            m.controlled = self.value.is_some();
            if let Some(v) = &self.value {
                m.tags = v.clone();
            }
        });
        let (tags, input, open, active) = {
            let m = model.read(cx);
            (m.tags.clone(), m.input.clone(), m.open, m.active)
        };
        let query = input.read(cx).value().to_string();
        let all = options(&tags, &self.suggestions, &query);
        let active = active.min(all.len().saturating_sub(1));
        let focus = input.read(cx).focus_handle(cx);
        let focused = focus.is_focused(window);
        let max = self.max;
        let field_h = density::current().field().px_value();
        let row_h = density::current().row().px_value();

        let pick_active = {
            let model = model.clone();
            let all = all.clone();
            move |window: &mut Window, cx: &mut App| {
                model.update(cx, |m, cx| {
                    let q = m.input.read(cx).value().trim().to_string();
                    let t = match all.get(m.active.min(all.len().saturating_sub(1))) {
                        Some(TagOption::Suggestion(s)) => s.to_string(),
                        _ => q,
                    };
                    m.add(&t, max, window, cx);
                })
            }
        };
        let pick_active = Rc::new(pick_active);
        let count = all.len();

        // Each chip is keyed by its text (`key={t}`), not its position, so a
        // removed chip never hands its focus or hover to the one after it.
        let remove_focuses: Vec<_> = tags
            .iter()
            .map(|t| {
                parts::keyed_focus(
                    (id.clone(), SharedString::from(format!("rm-focus-{t}"))),
                    window,
                    cx,
                )
            })
            .collect();
        // `.g-tags:focus-within { outline-color: var(--focus) }`: the input or
        // any chip's remove button.
        let focus_within =
            focus_within(focused, remove_focuses.iter().map(|f| f.is_focused(window)));
        // Where focus rests when a focused chip's remove button removes it:
        // Chromium keeps the Tab starting point where it was.
        let slot = parts::focus_slot((id.clone(), "slot"), window, cx);
        let tag_pills = tags.iter().enumerate().map(|(i, t)| {
            let model = model.clone();
            let slot = slot.clone();
            let remove_focus = remove_focuses[i].clone();
            let vacated = remove_focus.clone();
            let ring = parts::focus_visible(&remove_focus, window);
            let hover = window.use_keyed_state(
                (id.clone(), SharedString::from(format!("rm-hover-{t}"))),
                cx,
                |_, _| false,
            );
            let hovered = *hover.read(cx);
            let tag = t.clone();
            // `.g-tag { height: 1.25rem; padding: 0 2px 0 var(--spacing-03);
            // background: tag-background-gray; color: tag-color-gray; border-radius: 16 }`.
            div()
                .flex()
                .flex_none()
                .items_center()
                .gap(px(2.))
                .h(rem(20.))
                .pl(rem(8.))
                .pr(px(2.))
                .rounded(rem(16.))
                .bg(color(ColorRole::TagBackgroundGray))
                .text_color(color(ColorRole::TagColorGray))
                .text_size(rem(12.))
                .line_height(rem(16.))
                .child(t.clone())
                .child(
                    gpui_kit::base::Button::new((
                        id.clone(),
                        SharedString::from(format!("rm-{t}")),
                    ))
                    .track_focus(&remove_focus)
                    .accessibility_label(format!("Remove {t}"))
                    .relative()
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(rem(16.))
                    .rounded_full()
                    .when(hovered, |b| b.bg(color(ColorRole::TagHoverGray)))
                    .cursor(CursorStyle::PointingHand)
                    .on_hover(move |h, _, cx| {
                        hover.update(cx, |v, cx| {
                            *v = *h;
                            cx.notify();
                        })
                    })
                    .child(
                        Icon::carbon(CarbonIcon::Close)
                            .size_length(px(12.))
                            .color(color(ColorRole::TagColorGray)),
                    )
                    .when(ring, |b| {
                        b.child(
                            div()
                                .absolute()
                                .size_full()
                                .rounded_full()
                                .border_2()
                                .border_color(color(ColorRole::Focus)),
                        )
                    })
                    .on_click(move |_, window, cx| {
                        let tag = tag.clone();
                        parts::vacate_slot(&slot, i, std::slice::from_ref(&vacated), window, cx);
                        model.update(cx, |m, cx| {
                            let next: Vec<_> =
                                m.tags.iter().filter(|x| **x != tag).cloned().collect();
                            m.set(next, window, cx);
                        })
                    }),
                )
                .into_any_element()
        });
        let mut tag_pills: Vec<_> = tag_pills.collect();
        parts::insert_slot_anchor(&mut tag_pills, (id.clone(), "anchor"), &slot, cx);

        let placeholder = (tags.is_empty() && query.is_empty()).then(|| {
            div()
                .absolute()
                .inset_0()
                .flex()
                .items_center()
                .text_color(color(ColorRole::TextPlaceholder))
                .child(self.placeholder.clone())
        });
        let editor = div()
            .relative()
            .flex()
            .flex_1()
            .min_w(rem(80.))
            .h(rem(24.))
            .items_center()
            .child(
                parts::editor(&input)
                    .when_some(self.label.clone(), |e, l| e.aria_label(l))
                    .text_color(color(ColorRole::TextPrimary)),
            )
            .children(placeholder);

        let on_key = {
            let model = model.clone();
            let pick = pick_active.clone();
            move |e: &KeyDownEvent, window: &mut Window, cx: &mut App| match e
                .keystroke
                .key
                .as_str()
            {
                "," => {
                    cx.stop_propagation();
                    pick(window, cx);
                }
                "backspace" => {
                    let empty = model.read(cx).input.read(cx).value().is_empty();
                    if empty {
                        cx.stop_propagation();
                        model.update(cx, |m, cx| {
                            if !m.tags.is_empty() {
                                let mut next = m.tags.clone();
                                next.pop();
                                m.set(next, window, cx);
                            }
                        });
                    }
                }
                _ => {}
            }
        };
        let field = div()
            .id((id.clone(), "field"))
            .relative()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(rem(4.))
            .py(rem(4.))
            .px(rem(8.))
            .min_h(rem(field_h))
            .bg(self.fill.unwrap_or_else(|| color(ColorRole::Field01)))
            .cursor(CursorStyle::IBeam)
            .type_style(TypeStyle::BodyCompact01)
            .on_mouse_down(MouseButton::Left, {
                let focus = focus.clone();
                move |_, window, cx| focus.focus(window, cx)
            })
            .capture_key_down(on_key)
            .capture_action({
                let model = model.clone();
                move |_: &MoveDown, _, cx| {
                    model.update(cx, |m, cx| {
                        m.open = true;
                        m.active = (m.active + 1).min(count.saturating_sub(1));
                        cx.notify();
                    });
                    cx.stop_propagation();
                }
            })
            .capture_action({
                let model = model.clone();
                move |_: &MoveUp, _, cx| {
                    model.update(cx, |m, cx| {
                        m.active = m.active.saturating_sub(1);
                        cx.notify();
                    });
                    cx.stop_propagation();
                }
            })
            .capture_action({
                let pick = pick_active.clone();
                move |_: &Enter, window, cx| {
                    pick(window, cx);
                    cx.stop_propagation();
                }
            })
            .capture_action({
                let model = model.clone();
                move |_: &Escape, _, cx| {
                    model.update(cx, |m, cx| {
                        m.open = false;
                        cx.notify();
                    });
                    cx.stop_propagation();
                }
            })
            .children(tag_pills)
            .child(editor)
            .child(form::bottom_rule(color(ColorRole::BorderStrong01)))
            .when(focus_within, |d| d.child(parts::focus_ring()));

        let popup = (open && focused && !all.is_empty()).then(|| {
            let rows = all.iter().enumerate().map(|(i, o)| {
                let on = i == active;
                let hover = window.use_keyed_state(
                    (id.clone(), SharedString::from(format!("opt-hover-{i}"))),
                    cx,
                    |_, _| false,
                );
                let hovered = *hover.read(cx);
                let model = model.clone();
                let o2 = o.clone();
                div()
                    .id((id.clone(), SharedString::from(format!("opt-{i}"))))
                    .flex()
                    .items_center()
                    .gap(rem(8.))
                    .min_h(rem(row_h))
                    .px(rem(12.))
                    .text_color(color(ColorRole::TextPrimary))
                    .cursor(CursorStyle::PointingHand)
                    // `.g-opt:hover { background: layer-hover-01 }`; `.g-opt--on {
                    // background: layer-selected-01; box-shadow: inset 2px 0 0 border-interactive }`.
                    .when(hovered && !on, |d| d.bg(color(ColorRole::LayerHover01)))
                    .when(on, |d| {
                        d.bg(color(ColorRole::LayerSelected01))
                            .shadow(vec![BoxShadow {
                                color: color(ColorRole::BorderInteractive),
                                offset: gpui_kit::point(px(2.), px(0.)),
                                blur_radius: px(0.),
                                spread_radius: px(0.),
                                inset: true,
                            }])
                    })
                    .on_hover(move |h, _, cx| {
                        hover.update(cx, |v, cx| {
                            *v = *h;
                            cx.notify();
                        })
                    })
                    .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                        cx.stop_propagation();
                        let t = match &o2 {
                            TagOption::Suggestion(s) => s.to_string(),
                            TagOption::Create(t) => t.clone(),
                        };
                        model.update(cx, |m, cx| m.add(&t, max, window, cx));
                    })
                    .map(|d| match o {
                        TagOption::Suggestion(s) => d.child(s.clone()),
                        TagOption::Create(t) => d
                            .child(
                                Icon::carbon(CarbonIcon::Add)
                                    .size(IconSize::S16)
                                    .color(color(ColorRole::TextPrimary)),
                            )
                            .child(format!("Create “{t}”")),
                    })
            });
            deferred(
                div()
                    .absolute()
                    .top_full()
                    .left_0()
                    .right_0()
                    .bg(color(ColorRole::Layer01))
                    .shadow(overlay::floating_shadow())
                    .type_style(TypeStyle::BodyCompact01)
                    .children(rows),
            )
            .with_priority(10)
        });

        let at_max = max.is_some_and(|m| tags.len() >= m);
        parts::field()
            .when_some(self.label.clone(), |f, l| f.child(parts::label(l)))
            .child(div().relative().child(field).children(popup))
            .when(at_max, |f| {
                f.child(parts::helper(format!("Up to {} tags", max.unwrap_or(0))))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<SharedString> {
        v.iter()
            .map(|x| SharedString::from(x.to_string()))
            .collect()
    }

    #[test]
    fn options_filter_and_offer_create() {
        let sug = s(&[
            "frontend",
            "backend",
            "urgent",
            "docs",
            "design-system",
            "infra",
        ]);
        let tags = s(&["frontend", "urgent"]);
        assert_eq!(
            options(&tags, &sug, ""),
            vec![
                TagOption::Suggestion("backend".into()),
                TagOption::Suggestion("docs".into()),
                TagOption::Suggestion("design-system".into()),
                TagOption::Suggestion("infra".into()),
            ]
        );
        assert_eq!(
            options(&tags, &sug, "DE"),
            vec![
                TagOption::Suggestion("design-system".into()),
                TagOption::Create("DE".into()),
            ]
        );
        // An exact suggestion match (any case) offers no Create.
        assert_eq!(
            options(&tags, &sug, "Docs"),
            vec![TagOption::Suggestion("docs".into())]
        );
    }

    #[test]
    fn add_ignores_duplicates_and_max() {
        let tags = s(&["frontend"]);
        assert_eq!(add(&tags, "Frontend", None), None);
        assert_eq!(add(&tags, "", None), None);
        assert_eq!(add(&tags, "api", Some(1)), None);
        assert_eq!(add(&tags, "api", None), Some(s(&["frontend", "api"])));
    }

    #[test]
    fn outline_follows_focus_within() {
        assert!(!focus_within(false, [false, false]));
        assert!(focus_within(true, [false, false]));
        // Tab onto the first chip's remove button (FG1B, `TagInput … tab1`).
        assert!(focus_within(false, [true, false]));
        assert!(focus_within(false, [false, true]));
    }
}
