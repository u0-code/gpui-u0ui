//! `ComboButton` (`.cds--combo-button__*`, Carbon's `ComboButton`): a
//! text-only primary action and, 1px after it, a primary icon button whose
//! chevron opens a menu of related actions. Used by the Disclosures pattern.
//!
//! - **Layout**: `.cds--combo-button__container { display: inline-flex;
//!   column-gap: 0.0625rem }`; the action keeps a minimum width per size
//!   ([`primary_min_width`]) and is at most 14.9375rem wide, ellipsized.
//! - **Behavior**: the action fires `on_click`; the chevron (pointer, Enter
//!   or Space) toggles the menu, which is u0ui's Carbon `Menu` hung under the
//!   container's start edge and at least as wide as the container. The menu
//!   owns keyboard navigation, Escape and click-outside.
//! - **Motion**: the chevron turns 180° while open (`.cds--combo-button__trigger
//!   svg { transition: transform 110ms }`), a cut under reduced motion.
use crate::components::{
    Button, ButtonSize, ButtonVariant,
    menu::{self, MenuItem, MenuSize},
    overlay::{self, OpenChange},
};
use crate::{
    icons::{CarbonIcon, Icon},
    motion,
    tokens::REM_PX,
};
use gpui_kit::{
    App, Bounds, ClickEvent, ElementId, IntoElement, ParentElement, Pixels, RenderOnce,
    SharedString, Styled, Window, canvas, div, px, rems,
};
use std::rc::Rc;

/// `index.d.ts` `Size` of a ComboButton (`sm | md | lg`, default `lg`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ComboButtonSize {
    /// `sm`.
    Sm,
    /// `md`.
    Md,
    /// `lg` (default).
    #[default]
    Lg,
}

impl ComboButtonSize {
    /// Every size.
    pub const ALL: [Self; 3] = [Self::Sm, Self::Md, Self::Lg];

    /// The Carbon prop name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
        }
    }

    /// The button size of both halves.
    pub const fn button(self) -> ButtonSize {
        match self {
            Self::Sm => ButtonSize::Sm,
            Self::Md => ButtonSize::Md,
            Self::Lg => ButtonSize::Lg,
        }
    }

    /// The menu size: Carbon's ComboButton hands its own size to the `Menu`.
    pub const fn menu(self) -> MenuSize {
        match self {
            Self::Sm => MenuSize::Sm,
            Self::Md => MenuSize::Md,
            Self::Lg => MenuSize::Lg,
        }
    }
}

/// The action's minimum width in rem: `.cds--combo-button__container--sm
/// .cds--combo-button__primary-action { min-inline-size: 7.9375rem }`, `--md`
/// 7.4375rem, `--lg` 6.9375rem (each leaves room for the trigger beside it).
pub const fn primary_min_width(size: ComboButtonSize) -> f32 {
    match size {
        ComboButtonSize::Sm => 7.9375,
        ComboButtonSize::Md => 7.4375,
        ComboButtonSize::Lg => 6.9375,
    }
}

/// `.cds--combo-button__primary-action .cds--btn { max-inline-size: 14.9375rem }`.
pub const PRIMARY_MAX_WIDTH: f32 = 14.9375;

/// `.cds--combo-button__container { column-gap: 0.0625rem }`, in px.
pub const GAP: f32 = 1.;

type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// A Carbon combo button. See the module docs.
#[derive(IntoElement)]
pub struct ComboButton {
    id: ElementId,
    label: SharedString,
    menu_label: SharedString,
    size: ComboButtonSize,
    disabled: bool,
    open: Option<bool>,
    on_open_change: Option<OpenChange>,
    on_click: Option<ClickHandler>,
    items: Vec<MenuItem>,
}

impl ComboButton {
    /// A combo button with a stable `id` whose action reads `label`.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            // Carbon's default `translateWithId` for the trigger; override per locale.
            menu_label: "Additional actions".into(),
            size: ComboButtonSize::Lg,
            disabled: false,
            open: None,
            on_open_change: None,
            on_click: None,
            items: Vec::new(),
        }
    }

    /// The trigger's accessible name and tooltip.
    pub fn menu_label(mut self, label: impl Into<SharedString>) -> Self {
        self.menu_label = label.into();
        self
    }

    /// Size (default `lg`).
    pub fn size(mut self, size: ComboButtonSize) -> Self {
        self.size = size;
        self
    }

    /// Disable both halves.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Controlled open state of the menu.
    pub fn open(mut self, open: bool) -> Self {
        self.open = Some(open);
        self
    }

    /// Called whenever the menu asks to open or close.
    pub fn on_open_change(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_open_change = Some(Rc::new(f));
        self
    }

    /// Called when the primary action is clicked.
    pub fn on_click(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }

    /// Append a menu item (`MenuItem`).
    pub fn item(mut self, item: MenuItem) -> Self {
        self.items.push(item);
        self
    }
}

impl RenderOnce for ComboButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let openness = overlay::openness(&id, self.open, false, self.on_open_change, window, cx);
        let open = openness.open && !self.disabled;
        let set_open = openness.setter();
        let size = self.size;
        let bounds = window.use_keyed_state((id.clone(), "bounds"), cx, |_, _| {
            Bounds::<Pixels>::default()
        });
        let width_rem =
            f32::from(bounds.read(cx).size.width) / (f32::from(window.rem_size()) / REM_PX);
        let rotation = motion::chevron_rotation((id.clone(), "chevron"), open, window, cx);

        let mut action = Button::new((id.clone(), "action"), self.label)
            .variant(ButtonVariant::Primary)
            .size(size.button())
            .disabled(self.disabled)
            .full_width(true);
        if let Some(f) = self.on_click {
            action = action.on_click(move |e, window, cx| f(e, window, cx));
        }
        let chevron = Button::new((id.clone(), "trigger"), self.menu_label.clone())
            .variant(ButtonVariant::Primary)
            .size(size.button())
            .icon(Icon::carbon(CarbonIcon::ChevronDown).rotate(gpui_kit::radians(rotation)))
            .icon_only(true)
            .tooltip(!open)
            .disabled(self.disabled);
        let trigger = if self.disabled {
            div().child(chevron).into_any_element()
        } else {
            let set_open = set_open.clone();
            overlay::trigger_wrapper(
                (id.clone(), "trigger-wrap"),
                chevron.into_any_element(),
                Rc::new(move |window, cx| set_open(!open, window, cx)),
            )
            .into_any_element()
        };
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
        let container = div()
            .relative()
            .flex()
            .flex_row()
            .gap(px(GAP))
            .child(
                div()
                    .min_w(rems(primary_min_width(size)))
                    .max_w(rems(PRIMARY_MAX_WIDTH))
                    .overflow_hidden()
                    .child(action),
            )
            .child(trigger)
            .child(measure)
            .into_any_element();

        menu::anchored_menu(
            &id,
            container,
            menu::MenuSpec {
                items: self.items,
                size: size.menu(),
                label: self.menu_label,
                open,
                set_open,
                min_width: Some(width_rem),
                on_unhandled_key: None,
            },
            window,
            cx,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widths_match_the_css() {
        assert_eq!(primary_min_width(ComboButtonSize::Sm) * REM_PX, 127.);
        assert_eq!(primary_min_width(ComboButtonSize::Md) * REM_PX, 119.);
        assert_eq!(primary_min_width(ComboButtonSize::Lg) * REM_PX, 111.);
        assert_eq!(PRIMARY_MAX_WIDTH * REM_PX, 239.);
        assert_eq!(GAP, 1.);
    }

    #[test]
    fn sizes_map_to_button_and_menu() {
        assert_eq!(ComboButtonSize::default(), ComboButtonSize::Lg);
        assert_eq!(ComboButtonSize::Sm.button(), ButtonSize::Sm);
        assert_eq!(ComboButtonSize::Md.menu(), MenuSize::Md);
        assert_eq!(
            ComboButtonSize::ALL.map(ComboButtonSize::name),
            ["sm", "md", "lg"]
        );
    }
}
