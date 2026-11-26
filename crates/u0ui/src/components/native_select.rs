//! NativeSelect: shadcn `NativeSelect` → Carbon `Select` / `SelectItem` /
//! `SelectItemGroup` (`.cds--select`).
//!
//! The field is Carbon's `.cds--select-input`: `field` fill, 1px
//! `border-strong` rule, hover `field-hover`, 2px `focus` outline, a static
//! 16px chevron 1rem from the end (`.cds--select__arrow`), `padding-inline:
//! 1rem 3rem` (4rem with the status icon), invalid/warn icons, disabled
//! (`border-block-end-color: transparent`) and read-only looks.
//!
//! Desktop GPUI has no native `<select>` popup, so the option list is drawn
//! as Carbon's list-box menu with `<optgroup>` headings, and the behavior is
//! the same Kit-backed one as [`super::Select`] (↑/↓, Enter, Escape,
//! type-ahead, which selects directly while closed like a native select).
//!
//! ```ignore
//! NativeSelect::new("region")
//!     .label("Region")
//!     .group("Europe", [("fra", "Frankfurt"), ("lon", "London")])
//!     .group("Americas", [("dal", "Dallas")])
//!     .default_value("fra")
//! ```
use super::form::FieldSize;
use super::list_box::ListItem;
use super::select::Select;
use crate::{density::Density, theme::LayerLevel};
use gpui_kit::{App, ElementId, IntoElement, RenderOnce, SharedString, Window};
use std::rc::Rc;

type ChangeHandler = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

/// A Carbon (native) select. See the module docs.
#[derive(IntoElement)]
pub struct NativeSelect {
    id: ElementId,
    label: Option<SharedString>,
    options: Vec<ListItem>,
    value: Option<SharedString>,
    default_value: Option<SharedString>,
    helper_text: Option<SharedString>,
    invalid: bool,
    invalid_text: Option<SharedString>,
    warn: bool,
    warn_text: Option<SharedString>,
    size: FieldSize,
    density: Option<Density>,
    disabled: bool,
    readonly: bool,
    level: LayerLevel,
    on_value_change: Option<ChangeHandler>,
}

impl NativeSelect {
    /// A select with a stable `id`.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            options: Vec::new(),
            value: None,
            default_value: None,
            helper_text: None,
            invalid: false,
            invalid_text: None,
            warn: false,
            warn_text: None,
            size: FieldSize::Md,
            density: None,
            disabled: false,
            readonly: false,
            level: LayerLevel::One,
            on_value_change: None,
        }
    }

    /// `label`.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// `options`: strings or `(value, label)` pairs (`NativeSelectOption`s).
    pub fn options<I: Into<ListItem>>(mut self, options: impl IntoIterator<Item = I>) -> Self {
        self.options.extend(options.into_iter().map(Into::into));
        self
    }

    /// One option (`NativeSelectOption`).
    pub fn option(mut self, option: impl Into<ListItem>) -> Self {
        self.options.push(option.into());
        self
    }

    /// A `NativeSelectOptGroup` with its options.
    pub fn group<I: Into<ListItem>>(
        mut self,
        label: impl Into<SharedString>,
        options: impl IntoIterator<Item = I>,
    ) -> Self {
        let label = label.into();
        for (k, option) in options.into_iter().enumerate() {
            let mut option: ListItem = option.into();
            if k == 0 {
                option.group = Some(label.clone());
            }
            self.options.push(option);
        }
        self
    }

    /// Controlled `value`.
    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// `defaultValue` (a native select shows its first option otherwise).
    pub fn default_value(mut self, value: impl Into<SharedString>) -> Self {
        self.default_value = Some(value.into());
        self
    }

    /// `helperText`.
    pub fn helper_text(mut self, text: impl Into<SharedString>) -> Self {
        self.helper_text = Some(text.into());
        self
    }

    /// `invalid`.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// `invalidText`.
    pub fn invalid_text(mut self, text: impl Into<SharedString>) -> Self {
        self.invalid_text = Some(text.into());
        self
    }

    /// `warn`.
    pub fn warn(mut self, warn: bool) -> Self {
        self.warn = warn;
        self
    }

    /// `warnText`.
    pub fn warn_text(mut self, text: impl Into<SharedString>) -> Self {
        self.warn_text = Some(text.into());
        self
    }

    /// `size`.
    pub fn size(mut self, size: FieldSize) -> Self {
        self.size = size;
        self
    }

    /// Override the app density.
    pub fn density(mut self, density: Density) -> Self {
        self.density = Some(density);
        self
    }

    /// Disabled.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Read-only.
    pub fn readonly(mut self, readonly: bool) -> Self {
        self.readonly = readonly;
        self
    }

    /// The layer the field sits on.
    pub fn layer(mut self, level: LayerLevel) -> Self {
        self.level = level;
        self
    }

    /// `onChange` with the chosen value.
    pub fn on_value_change(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_value_change = Some(Rc::new(handler));
        self
    }

    /// The value a native select starts on: `defaultValue`, else the first
    /// enabled option.
    pub fn initial_value(&self) -> Option<SharedString> {
        self.default_value.clone().or_else(|| {
            self.options
                .iter()
                .find(|o| !o.disabled)
                .map(|o| o.value.clone())
        })
    }
}

impl RenderOnce for NativeSelect {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let initial = self.initial_value();
        let mut select = Select::new(self.id)
            .native(true)
            .items(self.options)
            .size(self.size)
            .invalid(self.invalid)
            .warn(self.warn)
            .disabled(self.disabled)
            .readonly(self.readonly)
            .layer(self.level);
        if let Some(label) = self.label {
            select = select.label(label);
        }
        if let Some(text) = self.helper_text {
            select = select.helper_text(text);
        }
        if let Some(text) = self.invalid_text {
            select = select.invalid_text(text);
        }
        if let Some(text) = self.warn_text {
            select = select.warn_text(text);
        }
        if let Some(density) = self.density {
            select = select.density(density);
        }
        select = match self.value {
            Some(value) => select.value(Some(value)),
            None => match initial {
                Some(value) => select.default_value(value),
                None => select,
            },
        };
        if let Some(handler) = self.on_value_change {
            select = select.on_value_change(move |value, window, cx| {
                if let Some(value) = value {
                    handler(value, window, cx)
                }
            });
        }
        select
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_mark_their_first_option_and_default_to_the_first() {
        let select = NativeSelect::new("r")
            .group("Europe", [("fra", "Frankfurt"), ("lon", "London")])
            .group("Americas", [("dal", "Dallas")]);
        let groups: Vec<_> = select.options.iter().map(|o| o.group.clone()).collect();
        assert_eq!(
            groups,
            [Some("Europe".into()), None, Some("Americas".into())]
        );
        assert_eq!(select.initial_value().as_deref(), Some("fra"));
        let chosen = NativeSelect::new("r")
            .options(["a", "b"])
            .default_value("b");
        assert_eq!(chosen.initial_value().as_deref(), Some("b"));
    }
}
