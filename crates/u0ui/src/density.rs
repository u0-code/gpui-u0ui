//! Desktop density.
//!
//! `Compact` is the desktop default (`[data-density="compact"]` in the reference
//! bundle): rows 28/32/36px, fields 28/32px, and Carbon buttons/inputs that are
//! not explicitly sm/xl/2xl shrink to `density-field-md` (32px). `Comfortable`
//! restores Carbon's own sizes (`[data-density="comfortable"]`: rows `size-lg`
//! 48px, fields `size-md` 40px).
//!
//! Like the theme, the current density is context-free ([`current`]) and changed
//! only through [`set_density`] from event handlers.
use crate::tokens::{
    DENSITY_FIELD_MD, DENSITY_FIELD_SM, DENSITY_ROW_LG, DENSITY_ROW_MD, DENSITY_ROW_SM, SIZE_LG,
    SIZE_MD, TokenLength,
};
use gpui_kit::{App, Global};
use std::cell::Cell;

/// How tightly controls are packed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Density {
    /// Desktop default: 32px fields and rows.
    #[default]
    Compact,
    /// Carbon's web sizes: 40px fields, 48px rows and buttons.
    Comfortable,
}

impl Density {
    /// Both densities.
    pub const ALL: [Density; 2] = [Self::Compact, Self::Comfortable];

    /// `compact` / `comfortable`.
    pub const fn id(self) -> &'static str {
        match self {
            Self::Compact => "compact",
            Self::Comfortable => "comfortable",
        }
    }

    /// Default row height: `density-row-md` (32px) compact; `size-lg` (48px)
    /// comfortable (`[data-density="comfortable"] { --density-row-md: var(--size-lg) }`).
    pub const fn row(self) -> TokenLength {
        match self {
            Self::Compact => DENSITY_ROW_MD,
            Self::Comfortable => SIZE_LG,
        }
    }

    /// Small row: `density-row-sm` (28px).
    pub const fn row_sm(self) -> TokenLength {
        match self {
            Self::Compact => DENSITY_ROW_SM,
            Self::Comfortable => SIZE_LG,
        }
    }

    /// Two-line row: `density-row-lg` (36px).
    pub const fn row_lg(self) -> TokenLength {
        match self {
            Self::Compact => DENSITY_ROW_LG,
            Self::Comfortable => SIZE_LG,
        }
    }

    /// Default field height: `density-field-md` (32px) compact; `size-md` (40px)
    /// comfortable (`--density-field-md: var(--size-md)`).
    pub const fn field(self) -> TokenLength {
        match self {
            Self::Compact => DENSITY_FIELD_MD,
            Self::Comfortable => SIZE_MD,
        }
    }

    /// Small field: `density-field-sm` (28px).
    pub const fn field_sm(self) -> TokenLength {
        match self {
            Self::Compact => DENSITY_FIELD_SM,
            Self::Comfortable => SIZE_MD,
        }
    }
}

thread_local! {
    static CURRENT: Cell<Density> = const { Cell::new(Density::Compact) };
}

/// The density as a GPUI global, for observers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ActiveDensity(pub Density);
impl Global for ActiveDensity {}

/// The current density (context-free).
pub fn current() -> Density {
    CURRENT.get()
}

/// Change the density and refresh every window. Event handlers only.
pub fn set_density(cx: &mut App, density: Density) {
    CURRENT.set(density);
    cx.set_global(ActiveDensity(density));
    cx.refresh_windows();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heights_follow_agents_md() {
        let c = Density::Compact;
        assert_eq!(
            [c.row_sm(), c.row(), c.row_lg()].map(|l| l.px_value()),
            [28., 32., 36.]
        );
        assert_eq!([c.field_sm(), c.field()].map(|l| l.px_value()), [28., 32.]);
        let w = Density::Comfortable;
        assert_eq!((w.field().px_value(), w.row().px_value()), (40., 48.));
        assert_eq!(Density::default(), Density::Compact);
    }
}
