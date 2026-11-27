//! Carbon's fluid fields (`FluidTextInput`, `FluidPasswordInput`,
//! `FluidDropdown`): the label sits inside a 4rem box that fills its
//! container, so stacked fields touch (`FluidForm`). Used by the Fluid form
//! pattern; the regular fields are [`crate::components::Input`] and
//! [`crate::components::Select`].
//!
//! - **Box**: `field` fill (contextual to the layer), 1px `border-strong`
//!   bottom rule, at least 4rem tall (`.cds--text-input--fluid .cds--text-input
//!   { min-block-size: 4rem }`); focus draws the 2px inset `focus` outline.
//! - **Label** (`.cds--text-input--fluid .cds--label { position: absolute;
//!   inset-block-start: 0.8125rem; inset-inline-start: 1rem; block-size: 1rem }`),
//!   `label-01` in `text-secondary`.
//! - **Text input**: Kit's editor (`InputState`: caret, selection, IME,
//!   clipboard) without chrome, padded `2rem 1rem 0.8125rem`. A password adds
//!   the visibility toggle, 2rem square, 1.625rem from the top and 0.5rem from
//!   the end, and pads the end 2.5rem.
//! - **Dropdown**: the value padded `2.0625rem 0 0.8125rem 1rem`, a 1rem
//!   chevron 1rem from the end that turns 180° while open, `field-hover` on
//!   hover; Enter, Space or a click opens u0ui's Carbon `Menu` (its own keyboard
//!   navigation, Escape and click-outside) as wide as the field.
use crate::components::{
    form::{FIELD_RULE, bottom_rule, inset_outline},
    input::FIELD_MOTION,
    menu::{self, MenuItem, MenuSize},
    overlay,
};
use crate::{
    icons::{CarbonIcon, Icon, IconSize},
    motion,
    theme::{LayerLevel, color},
    tokens::{ColorRole, REM_PX, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    AnyElement, App, Bounds, CursorStyle, ElementId, Entity, Focusable as _, Hsla,
    InteractiveElement as _, IntoElement, KeyDownEvent, ParentElement, Pixels, RenderOnce,
    SharedString, StatefulInteractiveElement as _, Styled, Window, canvas,
    component::{Sizable as _, Size, input::Input as KitInput, input::InputState},
    div,
    prelude::FluentBuilder as _,
    rems,
};
use std::rc::Rc;

/// The fluid box's measurements, in rem.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FluidGeometry {
    /// `min-block-size: 4rem`.
    pub min_height: f32,
    /// The label's `inset-block-start: 0.8125rem`.
    pub label_top: f32,
    /// The label's and the text's `inset-inline-start` / `padding-inline`: 1rem.
    pub inline: f32,
    /// `.cds--text-input--fluid .cds--text-input { padding: 2rem 1rem 0.8125rem }`.
    pub input_top: f32,
    /// The bottom padding, 0.8125rem.
    pub input_bottom: f32,
    /// `.cds--list-box__wrapper--fluid .cds--list-box__field { padding-block:
    /// 2.0625rem 0.8125rem }`.
    pub dropdown_top: f32,
    /// `.cds--text-input--fluid .cds--text-input.cds--password-input {
    /// padding-inline-end: 2.5rem }`.
    pub password_end: f32,
    /// The visibility toggle: `block-size: 2rem; inline-size: 2rem;
    /// inset-block-start: 1.625rem; inset-inline-end: 0.5rem`.
    pub toggle: (f32, f32, f32),
}

impl FluidGeometry {
    /// The reference geometry.
    pub const REFERENCE: Self = Self {
        min_height: 4.,
        label_top: 0.8125,
        inline: 1.,
        input_top: 2.,
        input_bottom: 0.8125,
        dropdown_top: 2.0625,
        password_end: 2.5,
        toggle: (2., 1.625, 0.5),
    };
}

/// The box's colors.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FluidLook {
    /// `background: var(--field)`.
    pub background: Hsla,
    /// `.cds--list-box:hover { background-color: var(--field-hover) }` (dropdown).
    pub hover: Hsla,
    /// `border-block-end: 1px solid var(--border-strong)`.
    pub rule: Hsla,
    /// `.cds--label { color: var(--text-secondary) }`.
    pub label: Hsla,
    /// The value, `text-primary`.
    pub text: Hsla,
    /// `::placeholder`, `text-placeholder`.
    pub placeholder: Hsla,
    /// The outline while focused (`focus`), else transparent.
    pub outline: Hsla,
}

impl FluidLook {
    /// The look at `level`, `focused` or not.
    pub fn of(level: LayerLevel, focused: bool) -> Self {
        Self {
            background: color(level.field()),
            hover: color(level.field_hover()),
            rule: color(level.border_strong()),
            label: color(ColorRole::TextSecondary),
            text: color(ColorRole::TextPrimary),
            placeholder: color(ColorRole::TextPlaceholder),
            outline: if focused {
                color(ColorRole::Focus)
            } else {
                Hsla::transparent_black()
            },
        }
    }
}

fn label(text: SharedString, look: FluidLook) -> impl IntoElement {
    let g = FluidGeometry::REFERENCE;
    div()
        .absolute()
        .top(rems(g.label_top))
        .left(rems(g.inline))
        .right(rems(g.inline))
        .h(rems(1.))
        .flex()
        .items_center()
        .overflow_hidden()
        .whitespace_nowrap()
        .type_style(TypeStyle::Label01)
        .text_color(look.label)
        .child(text)
}

/// A fluid text or password input over a Kit `InputState`.
#[derive(IntoElement)]
pub struct FluidTextInput {
    id: ElementId,
    state: Entity<InputState>,
    label: SharedString,
    placeholder: Option<SharedString>,
    password: bool,
    toggle_labels: (SharedString, SharedString),
    level: LayerLevel,
}

impl FluidTextInput {
    /// A fluid input with a stable `id`, editing `state`, labelled `label`.
    pub fn new(
        id: impl Into<ElementId>,
        state: &Entity<InputState>,
        label: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            state: state.clone(),
            label: label.into(),
            placeholder: None,
            password: false,
            // Carbon's `showPasswordLabel` / `hidePasswordLabel` defaults; override per locale.
            toggle_labels: ("Show password".into(), "Hide password".into()),
            level: LayerLevel::One,
        }
    }

    /// The placeholder.
    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    /// A `FluidPasswordInput`: masked, with the visibility toggle.
    pub fn password(mut self, password: bool) -> Self {
        self.password = password;
        self
    }

    /// The toggle's names while masked and while shown.
    pub fn toggle_labels(
        mut self,
        show: impl Into<SharedString>,
        hide: impl Into<SharedString>,
    ) -> Self {
        self.toggle_labels = (show.into(), hide.into());
        self
    }

    /// The layer the field sits on (default 01).
    pub fn layer(mut self, level: LayerLevel) -> Self {
        self.level = level;
        self
    }
}

impl RenderOnce for FluidTextInput {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let g = FluidGeometry::REFERENCE;
        let id = self.id;
        let state = self.state;
        crate::components::form::select_all_on_tab_in(&state, window, cx);
        let focused = state.read(cx).focus_handle(cx).is_focused(window);
        let empty = state.read(cx).value().is_empty();
        if self.password {
            let applied = window.use_keyed_state((id.clone(), "masked-init"), cx, |_, _| {
                None::<gpui_kit::EntityId>
            });
            if *applied.read(cx) != Some(state.entity_id()) {
                applied.update(cx, |a, _| *a = Some(state.entity_id()));
                state.update(cx, |s, cx| s.set_masked(true, window, cx));
            }
        }
        let masked = state.read(cx).presentation().is_masked();
        let look = FluidLook::of(self.level, focused);
        let outline = motion::animate_color(
            (id.clone(), "outline"),
            look.outline,
            FIELD_MOTION,
            window,
            cx,
        );
        let end = if self.password {
            g.password_end
        } else {
            g.inline
        };
        let editor = KitInput::new(&state)
            .appearance(false)
            .with_size(Size::XSmall)
            .aria_label(self.label.clone())
            .p_0()
            .w_full()
            .text_size(rems(14. / REM_PX))
            .line_height(rems(18. / REM_PX))
            .text_color(look.text);
        // One 18px line: Kit's `XSmall` input is 20px tall (`h_5`), which
        // made the row 66px instead of `.cds--text-input--fluid`'s 64. Kit's
        // `Input` has its own inherent `h()` that only sizes multi-line
        // inputs, so the `Styled` one is named explicitly.
        let editor = Styled::h(editor, rems(18. / REM_PX));
        let placeholder = self.placeholder.filter(|_| empty).map(|text| {
            div()
                .absolute()
                .top_0()
                .left_0()
                .right_0()
                .overflow_hidden()
                .whitespace_nowrap()
                .text_color(look.placeholder)
                .child(text)
        });
        let toggle = self.password.then(|| {
            let (show, hide) = self.toggle_labels.clone();
            let focus = window
                .use_keyed_state((id.clone(), "toggle-focus"), cx, |_, cx| cx.focus_handle())
                .read(cx)
                .clone();
            let toggle_focused = focus.is_focused(window);
            let state = state.clone();
            let (side, top, end) = g.toggle;
            gpui_kit::base::Button::new((id.clone(), "visibility"))
                .track_focus(&focus)
                .accessibility_label(if masked { show } else { hide })
                .absolute()
                .top(rems(top))
                .right(rems(end))
                .size(rems(side))
                .flex()
                .items_center()
                .justify_center()
                .cursor(CursorStyle::PointingHand)
                .child(
                    Icon::carbon(if masked {
                        CarbonIcon::View
                    } else {
                        CarbonIcon::ViewOff
                    })
                    .size(IconSize::S16)
                    .color(color(ColorRole::IconPrimary)),
                )
                .when(toggle_focused, |b| {
                    b.child(inset_outline(2., color(ColorRole::Focus)))
                })
                .on_click(move |_, window, cx| {
                    state.update(cx, |s, cx| {
                        let next = !s.presentation().is_masked();
                        s.set_masked(next, window, cx);
                    })
                })
        });
        div()
            .id(id.clone())
            .relative()
            .w_full()
            .min_h(rems(g.min_height))
            .pt(rems(g.input_top))
            .pb(rems(g.input_bottom + FIELD_RULE / REM_PX))
            .pl(rems(g.inline))
            .pr(rems(end))
            .bg(look.background)
            .type_style(TypeStyle::BodyCompact01)
            .cursor(CursorStyle::IBeam)
            .child(label(self.label, look))
            .child(
                div()
                    .relative()
                    .w_full()
                    .children(placeholder)
                    .child(editor),
            )
            .children(toggle)
            .child(bottom_rule(look.rule))
            .child(inset_outline(2., outline))
    }
}

type ChangeHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// A fluid dropdown: a label, the chosen item and a chevron; the items open
/// in a Carbon menu.
#[derive(IntoElement)]
pub struct FluidDropdown {
    id: ElementId,
    label: SharedString,
    placeholder: SharedString,
    items: Vec<SharedString>,
    value: Option<usize>,
    on_change: Option<ChangeHandler>,
    level: LayerLevel,
}

impl FluidDropdown {
    /// A fluid dropdown with a stable `id`, titled `label` (`titleText`).
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            placeholder: SharedString::default(),
            items: Vec::new(),
            value: None,
            on_change: None,
            level: LayerLevel::One,
        }
    }

    /// The text shown before a choice (Carbon's `label`).
    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = text.into();
        self
    }

    /// The items.
    pub fn items<S: Into<SharedString>>(mut self, items: impl IntoIterator<Item = S>) -> Self {
        self.items = items.into_iter().map(Into::into).collect();
        self
    }

    /// The chosen item's index (controlled).
    pub fn value(mut self, index: Option<usize>) -> Self {
        self.value = index;
        self
    }

    /// Called with an item's index when it is chosen.
    pub fn on_change(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }

    /// The layer the field sits on (default 01).
    pub fn layer(mut self, level: LayerLevel) -> Self {
        self.level = level;
        self
    }
}

impl RenderOnce for FluidDropdown {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let g = FluidGeometry::REFERENCE;
        let id = self.id;
        let openness = overlay::openness(&id, None, false, None, window, cx);
        let open = openness.open;
        let set_open = openness.setter();
        let focus = window
            .use_keyed_state((id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let focused = focus.is_focused(window) || open;
        let look = FluidLook::of(self.level, focused);
        let outline = motion::animate_color(
            (id.clone(), "outline"),
            look.outline,
            FIELD_MOTION,
            window,
            cx,
        );
        let rotation = motion::chevron_rotation((id.clone(), "chevron"), open, window, cx);
        let bounds = window.use_keyed_state((id.clone(), "bounds"), cx, |_, _| {
            Bounds::<Pixels>::default()
        });
        let width_rem =
            f32::from(bounds.read(cx).size.width) / (f32::from(window.rem_size()) / REM_PX);
        let measure = canvas(
            move |b, _, cx| {
                if *bounds.read(cx) != b {
                    bounds.update(cx, |s, _| *s = b);
                }
            },
            |_, _, _, _| {},
        )
        .absolute()
        .top_0()
        .left_0()
        .size_full();
        let (shown, shown_color) = match self.value.and_then(|i| self.items.get(i)) {
            Some(item) => (item.clone(), look.text),
            None => (self.placeholder.clone(), look.text),
        };
        let toggle = {
            let set_open = set_open.clone();
            move |window: &mut Window, cx: &mut App| set_open(!open, window, cx)
        };
        let on_key = toggle.clone();
        let field = div()
            .id((id.clone(), "field"))
            .track_focus(&focus.tab_index(0).tab_stop(true))
            .relative()
            .w_full()
            .min_h(rems(g.min_height))
            .pt(rems(g.dropdown_top))
            // `.cds--list-box__field { padding-block: 2.0625rem 0.8125rem }`
            // fills the 64px box: the list box's 1px bottom border lies under
            // it, not after it (computed: field and list box both 64px).
            .pb(rems(g.input_bottom))
            .pl(rems(g.inline))
            .pr(rems(g.inline * 3.))
            .bg(look.background)
            .hover(move |s| s.bg(look.hover))
            .cursor(CursorStyle::PointingHand)
            .type_style(TypeStyle::BodyCompact01)
            // GPUI already turns Enter/Space on a focused clickable element into a
            // click (on key up); the key handler below acts on the key itself, so a
            // keyboard click is ignored here or the action would run twice.
            .on_click(move |e, window, cx| {
                if !e.is_keyboard() {
                    toggle(window, cx)
                }
            })
            .on_key_down(move |e: &KeyDownEvent, window, cx| {
                if matches!(e.keystroke.key.as_str(), "enter" | "space" | "down") && !e.is_held {
                    on_key(window, cx);
                    cx.stop_propagation();
                }
            })
            .child(label(self.label.clone(), look))
            .child(
                div()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_color(shown_color)
                    .child(shown),
            )
            .child(
                // `:has(.cds--dropdown) .cds--list-box__menu-icon { inset-inline-end: 1rem }`.
                div()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .right(rems(g.inline))
                    .flex()
                    .items_center()
                    .child(
                        Icon::carbon(CarbonIcon::ChevronDown)
                            .size(IconSize::S16)
                            .rotate(gpui_kit::radians(rotation))
                            .color(color(ColorRole::IconPrimary)),
                    ),
            )
            .child(bottom_rule(look.rule))
            .child(inset_outline(2., outline))
            .child(measure)
            .into_any_element();

        let on_change = self.on_change;
        let items = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let on_change = on_change.clone();
                MenuItem::item(item.clone()).on_select(move |window, cx| {
                    if let Some(f) = &on_change {
                        f(i, window, cx)
                    }
                })
            })
            .collect::<Vec<_>>();
        let element: AnyElement = menu::anchored_menu(
            &id,
            field,
            menu::MenuSpec {
                items,
                size: MenuSize::Lg,
                label: self.label,
                open,
                set_open,
                min_width: Some(width_rem),
                on_unhandled_key: None,
            },
            window,
            cx,
        );
        div().w_full().child(element)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry_matches_the_css() {
        let g = FluidGeometry::REFERENCE;
        assert_eq!(g.min_height * REM_PX, 64.);
        assert_eq!(g.label_top * REM_PX, 13.);
        assert_eq!(g.inline * REM_PX, 16.);
        assert_eq!(g.input_top * REM_PX, 32.);
        assert_eq!(g.input_bottom * REM_PX, 13.);
        assert_eq!(g.dropdown_top * REM_PX, 33.);
        assert_eq!(g.password_end * REM_PX, 40.);
        assert_eq!(g.toggle, (2., 1.625, 0.5));
        // 32 + 18 (body-compact-01 line) + 13 + the 1px rule fill the 64px box.
        assert_eq!(32. + 18. + 13. + FIELD_RULE, 64.);
        // The dropdown: 33 + 18 + 13, the rule under the padding.
        assert_eq!(33. + 18. + 13., 64.);
    }

    #[test]
    fn look_follows_the_layer_and_focus() {
        let rest = FluidLook::of(LayerLevel::One, false);
        assert_eq!(rest.background, color(ColorRole::Field01));
        assert_eq!(rest.hover, color(ColorRole::FieldHover01));
        assert_eq!(rest.rule, color(ColorRole::BorderStrong01));
        assert_eq!(rest.label, color(ColorRole::TextSecondary));
        assert_eq!(rest.outline, Hsla::transparent_black());
        let two = FluidLook::of(LayerLevel::Two, true);
        assert_eq!(two.background, color(ColorRole::Field02));
        assert_eq!(two.outline, color(ColorRole::Focus));
    }
}
