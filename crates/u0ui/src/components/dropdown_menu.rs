//! DropdownMenu: an action menu opened by a button. With a `label` it is
//! Carbon `MenuButton` (`.cds--menu-button__trigger`, a chevron that turns
//! 180° in 110ms while open); without, a ghost overflow icon button
//! (`OverflowMenuVertical`) whose trigger lifts onto `layer-01` with the
//! floating shadow while open (`.cu-overflow-trigger--open`). The menu itself
//! is [`super::menu`]'s Carbon `Menu`, anchored under the trigger's start
//! edge by Base's `Popup`.
//!
//! ```ignore
//! DropdownMenu::new("actions")
//!     .label("Actions")
//!     .item(MenuItem::item("Restart"))
//!     .item(MenuItem::divider())
//!     .item(MenuItem::item("Delete").danger(true))
//! ```
use super::button::{Button, ButtonSize, ButtonVariant};
use super::menu::{self, MenuItem, MenuSize};
use super::overlay::{self, OpenChange};
use crate::{
    icons::{CarbonIcon, Icon},
    motion,
    theme::color,
    tokens::ColorRole,
};
use gpui_kit::{
    App, Bounds, ElementId, IntoElement, ParentElement, Pixels, RenderOnce, SharedString, Styled,
    TestSupportExt as _, Window, canvas, div, prelude::FluentBuilder as _,
};
use std::rc::Rc;

/// `index.d.ts` `DropdownMenuProps.kind`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DropdownMenuKind {
    /// `primary` (default).
    #[default]
    Primary,
    /// `secondary`.
    Secondary,
    /// `tertiary`.
    Tertiary,
    /// `ghost`.
    Ghost,
}

impl DropdownMenuKind {
    /// Every kind.
    pub const ALL: [DropdownMenuKind; 4] =
        [Self::Primary, Self::Secondary, Self::Tertiary, Self::Ghost];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Secondary => "secondary",
            Self::Tertiary => "tertiary",
            Self::Ghost => "ghost",
        }
    }

    const fn variant(self) -> ButtonVariant {
        match self {
            Self::Primary => ButtonVariant::Primary,
            Self::Secondary => ButtonVariant::Secondary,
            Self::Tertiary => ButtonVariant::Tertiary,
            Self::Ghost => ButtonVariant::Ghost,
        }
    }
}

/// `index.d.ts` `Size` (sm | md | lg).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DropdownMenuSize {
    /// `sm`.
    Sm,
    /// `md`.
    Md,
    /// `lg` (the MenuButton default).
    #[default]
    Lg,
}

impl DropdownMenuSize {
    /// Every size.
    pub const ALL: [DropdownMenuSize; 3] = [Self::Sm, Self::Md, Self::Lg];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
        }
    }

    /// The trigger button size.
    pub const fn button(self) -> ButtonSize {
        match self {
            Self::Sm => ButtonSize::Sm,
            Self::Md => ButtonSize::Md,
            Self::Lg => ButtonSize::Lg,
        }
    }

    /// The menu size: a MenuButton's menu matches the button; the overflow
    /// menu is `lg` only for `lg` (`index.jsx`: `size === 'lg' ? 'lg' : 'sm'`).
    pub const fn menu(self, labeled: bool) -> MenuSize {
        match (labeled, self) {
            (true, Self::Sm) | (false, Self::Sm | Self::Md) => MenuSize::Sm,
            (true, Self::Md) => MenuSize::Md,
            (_, Self::Lg) => MenuSize::Lg,
        }
    }
}

/// `.cds--menu-button__trigger:not(.cds--btn--ghost) { min-inline-size: 12rem }`.
pub const TRIGGER_MIN_WIDTH: f32 = 192.;

/// A dropdown menu. See the module docs.
#[derive(IntoElement)]
pub struct DropdownMenu {
    id: ElementId,
    label: Option<SharedString>,
    overflow_label: SharedString,
    overflow_icon: CarbonIcon,
    kind: DropdownMenuKind,
    size: Option<DropdownMenuSize>,
    open: Option<bool>,
    default_open: bool,
    on_open_change: Option<OpenChange>,
    items: Vec<MenuItem>,
}

impl DropdownMenu {
    /// A dropdown menu with a stable `id`.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            // The reference overflow trigger's `label="Options"`; override per locale.
            overflow_label: "Options".into(),
            overflow_icon: CarbonIcon::OverflowMenuVertical,
            kind: DropdownMenuKind::Primary,
            size: None,
            open: None,
            default_open: false,
            on_open_change: None,
            items: Vec::new(),
        }
    }

    /// The button label; without one the trigger is an overflow icon button.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Accessible name and tooltip of the overflow trigger.
    pub fn overflow_label(mut self, label: impl Into<SharedString>) -> Self {
        self.overflow_label = label.into();
        self
    }

    /// The overflow trigger's icon (default `OverflowMenuVertical`; toolbars
    /// use `OverflowMenuHorizontal`).
    pub fn overflow_icon(mut self, icon: CarbonIcon) -> Self {
        self.overflow_icon = icon;
        self
    }

    /// The MenuButton kind (default `primary`).
    pub fn kind(mut self, kind: DropdownMenuKind) -> Self {
        self.kind = kind;
        self
    }

    /// Size (default `lg`, for both triggers: `index.jsx` `DropdownMenu({
    /// size = 'lg' })` hands it to the overflow trigger too).
    pub fn size(mut self, size: DropdownMenuSize) -> Self {
        self.size = Some(size);
        self
    }

    /// Controlled open state.
    pub fn open(mut self, open: bool) -> Self {
        self.open = Some(open);
        self
    }

    /// Initial open state when uncontrolled.
    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }

    /// Called whenever the menu asks to open or close.
    pub fn on_open_change(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_open_change = Some(Rc::new(f));
        self
    }

    /// Append a menu item.
    pub fn item(mut self, item: MenuItem) -> Self {
        self.items.push(item);
        self
    }

    /// Append menu items.
    pub fn items(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.items.extend(items);
        self
    }
}

impl RenderOnce for DropdownMenu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let openness = overlay::openness(
            &id,
            self.open,
            self.default_open,
            self.on_open_change.clone(),
            window,
            cx,
        );
        let open = openness.open;
        let set_open = openness.setter();
        let labeled = self.label.is_some();
        let kind = self.kind;
        let size = self.size.unwrap_or_default();
        let trigger_bounds = window.use_keyed_state((id.clone(), "trigger-bounds"), cx, |_, _| {
            Bounds::<Pixels>::default()
        });
        let trigger_width = f32::from(trigger_bounds.read(cx).size.width)
            / (f32::from(window.rem_size()) / crate::tokens::REM_PX);

        let button = match &self.label {
            Some(label) => {
                let rotation = motion::chevron_rotation((id.clone(), "chevron"), open, window, cx);
                Button::new((id.clone(), "button"), label.clone())
                    .variant(self.kind.variant())
                    .size(size.button())
                    .icon(Icon::carbon(CarbonIcon::ChevronDown).rotate(gpui_kit::radians(rotation)))
                    .full_width(true)
                    .into_any_element()
            }
            None => Button::new((id.clone(), "button"), self.overflow_label.clone())
                .variant(ButtonVariant::Ghost)
                .size(size.button())
                .icon(self.overflow_icon)
                .icon_only(true)
                .tooltip(!open)
                .selected(open)
                .into_any_element(),
        };
        let measure = canvas(
            move |bounds, _, cx| {
                if *trigger_bounds.read(cx) != bounds {
                    trigger_bounds.update(cx, |b, _| *b = bounds);
                }
            },
            |_, _, _, _| {},
        )
        .absolute()
        .top_0()
        .left_0()
        .size_full();
        let trigger = overlay::trigger_wrapper(
            (id.clone(), "trigger"),
            div()
                .relative()
                .when(labeled && kind != DropdownMenuKind::Ghost, |this| {
                    this.min_w(overlay::rem(TRIGGER_MIN_WIDTH))
                })
                .when(!labeled && open, |this| {
                    // `.cu-overflow-trigger--open { background: var(--layer-01);
                    // box-shadow: 0 2px 6px var(--shadow) }`.
                    this.bg(color(ColorRole::Layer01))
                        .shadow(overlay::floating_shadow())
                })
                .child(button)
                .child(measure)
                .into_any_element(),
            {
                let set_open = set_open.clone();
                Rc::new(move |window, cx| set_open(!open, window, cx))
            },
        )
        .test_support()
        .into_any_element();

        menu::anchored_menu(
            &id,
            trigger,
            menu::MenuSpec {
                items: self.items,
                size: size.menu(labeled),
                label: self.label.clone().unwrap_or(self.overflow_label.clone()),
                open,
                set_open,
                min_width: labeled.then_some(trigger_width),
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
    fn menu_size_follows_index_jsx() {
        assert_eq!(DropdownMenuSize::Md.menu(true), MenuSize::Md);
        assert_eq!(DropdownMenuSize::Md.menu(false), MenuSize::Sm);
        assert_eq!(DropdownMenuSize::Lg.menu(false), MenuSize::Lg);
        assert_eq!(
            DropdownMenuKind::Tertiary.variant(),
            ButtonVariant::Tertiary
        );
        assert_eq!(TRIGGER_MIN_WIDTH, 192.);
    }
}
