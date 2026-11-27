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

/// The ⌘K / ⌘P palette input height, the one exception to the field sizes
/// (`palette-input-height`, 44px, in both densities).
pub const fn palette_input() -> TokenLength {
    crate::tokens::PALETTE_INPUT_HEIGHT
}

/// Lay `build`'s element out at `density`, whatever the app density is: the
/// reference's `<Density value>` / `[data-density]` container, which tightens
/// (or relaxes) every u0ui field, button, menu and table inside it. The
/// density is scoped while the child renders, lays out, prepaints and paints.
///
/// ```ignore
/// density::zone(Density::Comfortable, |_, _| Stepper::new("s").into_any_element())
/// ```
pub fn zone(
    density: Density,
    build: impl FnOnce(&mut gpui_kit::Window, &mut App) -> gpui_kit::AnyElement + 'static,
) -> DensityZone {
    DensityZone {
        density,
        build: Some(Box::new(build)),
        child: None,
    }
}

type ZoneBuild = Box<dyn FnOnce(&mut gpui_kit::Window, &mut App) -> gpui_kit::AnyElement>;

/// The element made by [`zone`].
pub struct DensityZone {
    density: Density,
    build: Option<ZoneBuild>,
    child: Option<gpui_kit::AnyElement>,
}

impl DensityZone {
    fn scoped<R>(density: Density, f: impl FnOnce() -> R) -> R {
        let previous = CURRENT.replace(density);
        let result = f();
        CURRENT.set(previous);
        result
    }
}

impl gpui_kit::Element for DensityZone {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<gpui_kit::ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&gpui_kit::GlobalElementId>,
        _: Option<&gpui_kit::InspectorElementId>,
        window: &mut gpui_kit::Window,
        cx: &mut App,
    ) -> (gpui_kit::LayoutId, ()) {
        let build = self.build.take();
        let child = &mut self.child;
        let id = Self::scoped(self.density, || {
            let mut element = match build {
                Some(build) => build(window, cx),
                None => gpui_kit::IntoElement::into_any_element(gpui_kit::Empty),
            };
            let id = element.request_layout(window, cx);
            *child = Some(element);
            id
        });
        (id, ())
    }

    fn prepaint(
        &mut self,
        _: Option<&gpui_kit::GlobalElementId>,
        _: Option<&gpui_kit::InspectorElementId>,
        _: gpui_kit::Bounds<gpui_kit::Pixels>,
        _: &mut (),
        window: &mut gpui_kit::Window,
        cx: &mut App,
    ) {
        if let Some(child) = &mut self.child {
            Self::scoped(self.density, || child.prepaint(window, cx));
        }
    }

    fn paint(
        &mut self,
        _: Option<&gpui_kit::GlobalElementId>,
        _: Option<&gpui_kit::InspectorElementId>,
        _: gpui_kit::Bounds<gpui_kit::Pixels>,
        _: &mut (),
        _: &mut (),
        window: &mut gpui_kit::Window,
        cx: &mut App,
    ) {
        if let Some(child) = &mut self.child {
            Self::scoped(self.density, || child.paint(window, cx));
        }
    }
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
        // `palette-input-height` stays 44px in both densities.
        assert_eq!(palette_input().px_value(), 44.);
        assert_eq!(Density::default(), Density::Compact);
    }
}
