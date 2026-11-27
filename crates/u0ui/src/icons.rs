//! Material Symbols Sharp icons (the design system's only icon set).
//!
//! - [`Icon`]: any Material symbol by name (`Icon::named("rocket_launch")`), or a
//!   Carbon-named icon from the reference (`Icon::carbon(CarbonIcon::Add)`), at
//!   FILL 0/1, weight 400, grade 0, optical size 24. Sizes 16/20/24/32
//!   ([`IconSize`]); color is the surrounding text color (`currentColor`) unless set.
//! - [`Assets`]: the asset source serving `material-symbols/…` paths, falling back
//!   to Kit's own assets. Install it with `application().with_assets(u0ui::Assets::new())`.
//! - [`crate::init`] registers a resolver so Kit's built-in icons (chevrons, close,
//!   check, search, …) draw their Sharp equivalents.
use crate::tokens::ColorRole;
use gpui_kit::{
    AbsoluteLength, Animation, AnimationExt as _, App, AssetSource, ElementId, Hsla, IntoElement,
    Radians, RenderOnce, SharedString, StyleRefinement, Styled, Transformation, Window, percentage,
};
pub use gpui_material_symbols::{Axes, Icon as SymbolIcon, MaterialSymbols, Style, Symbol};
use std::time::Duration;
use std::{borrow::Cow, collections::HashMap};

mod kit_mapping;

include!("generated/carbon_icons.rs");

/// Asset path prefix of the reference's custom glyphs ([`CUSTOM_GLYPHS`]),
/// served by [`Assets`] as inline SVG.
const GLYPH_PREFIX: &str = "u0ui/glyphs/";

/// The SVG document of a custom glyph: the bundle's `draw()` (`viewBox
/// "0 -960 960 960"`, one filled path), so it draws like the Material icons.
fn glyph_svg(name: &str) -> Option<String> {
    CUSTOM_GLYPHS
        .iter()
        .find(|(key, _)| *key == name)
        .map(|(_, d)| {
            format!(
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 -960 960 960\" \
             width=\"24\" height=\"24\"><path d=\"{d}\"/></svg>"
            )
        })
}

impl CarbonIcon {
    /// The Material symbol name this Carbon name draws (or, for a custom
    /// glyph such as `CircleDash`, the bundle's path key, e.g. `"dash"`).
    pub fn material_name(self) -> &'static str {
        self.spec().1
    }

    /// The Material symbol this Carbon name draws (`None` for a custom glyph).
    pub fn symbol(self) -> Option<Symbol> {
        Symbol::named(self.material_name())
    }

    /// The path data when the reference draws this icon from its own SVG path
    /// (Carbon `CircleDash`: the 8-segment dashed ring; `PullRequest`) rather
    /// than a Material symbol.
    pub fn custom_glyph(self) -> Option<&'static str> {
        let name = self.material_name();
        CUSTOM_GLYPHS
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, d)| *d)
    }

    /// The Carbon name, e.g. `"ChevronDown"`.
    pub const fn name(self) -> &'static str {
        self.spec().0
    }

    /// The icon element.
    pub fn icon(self) -> Icon {
        Icon::carbon(self)
    }
}

/// Carbon icon sizes (`icon-size-01` 16, `icon-size-02` 20, plus 24 and 32).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum IconSize {
    /// 16px (`icon-size-01`): the default in controls.
    #[default]
    S16,
    /// 20px (`icon-size-02`): notifications, close buttons.
    S20,
    /// 24px.
    S24,
    /// 32px.
    S32,
}

impl IconSize {
    /// Every size.
    pub const ALL: [IconSize; 4] = [Self::S16, Self::S20, Self::S24, Self::S32];

    /// Pixels at 100% zoom.
    pub const fn px(self) -> f32 {
        match self {
            Self::S16 => 16.,
            Self::S20 => 20.,
            Self::S24 => 24.,
            Self::S32 => 32.,
        }
    }

    /// As rems (so icons follow text zoom with the text they sit in).
    pub fn length(self) -> AbsoluteLength {
        gpui_kit::rems(self.px() / crate::tokens::REM_PX).into()
    }
}

/// The asset path of `symbol` in the house style: Sharp, weight 400, grade 0,
/// optical size 24, FILL 0 or 1.
pub fn symbol_path(symbol: Symbol, filled: bool) -> SharedString {
    sharp(symbol, filled).asset_path().into()
}

fn sharp(symbol: Symbol, filled: bool) -> SymbolIcon {
    symbol
        .icon()
        .style(Style::Sharp)
        .axes(Axes::new(if filled { 1. } else { 0. }, 400., 0., 24.).expect("house axes"))
}

/// What an [`Icon`] draws.
#[derive(Clone, Debug, PartialEq)]
enum Source {
    Symbol { symbol: Symbol, filled: bool },
    Path(SharedString),
}

/// A Material Symbols Sharp icon, drawn through Kit's `Icon` (so Kit's path
/// resolution and SVG cache apply). 16px and `currentColor` by default.
#[derive(IntoElement, Clone)]
pub struct Icon {
    source: Source,
    size: AbsoluteLength,
    color: Option<Hsla>,
    transformation: Option<Transformation>,
    style: StyleRefinement,
}

impl Icon {
    fn from_source(source: Source) -> Self {
        Self {
            source,
            size: IconSize::S16.length(),
            color: None,
            transformation: None,
            style: StyleRefinement::default(),
        }
    }

    /// Any Material symbol by its snake_case name (`"rocket_launch"`). An unknown
    /// name draws nothing (and logs in debug builds).
    pub fn named(name: &str) -> Self {
        match Symbol::named(name) {
            Some(symbol) => Self::symbol(symbol),
            None => {
                #[cfg(debug_assertions)]
                eprintln!("u0ui: unknown Material symbol {name:?}");
                Self::from_path("")
            }
        }
    }

    /// A Material symbol, outlined (FILL 0).
    pub fn symbol(symbol: Symbol) -> Self {
        Self::from_source(Source::Symbol {
            symbol,
            filled: false,
        })
    }

    /// A Carbon-named reference icon (keeps the reference's filled flag).
    pub fn carbon(icon: CarbonIcon) -> Self {
        if icon.custom_glyph().is_some() {
            return Self::from_path(format!("{GLYPH_PREFIX}{}.svg", icon.material_name()));
        }
        Self::named(icon.material_name()).filled(icon.spec().2)
    }

    /// An icon from a raw asset path (e.g. an application SVG).
    pub fn from_path(path: impl Into<SharedString>) -> Self {
        Self::from_source(Source::Path(path.into()))
    }

    /// The filled (FILL 1) or outlined (FILL 0) variant of a Material symbol.
    pub fn filled(mut self, filled: bool) -> Self {
        if let Source::Symbol { filled: f, .. } = &mut self.source {
            *f = filled;
        }
        self
    }

    /// The asset path this icon draws.
    pub fn asset_path(&self) -> SharedString {
        match &self.source {
            Source::Symbol { symbol, filled } => symbol_path(*symbol, *filled),
            Source::Path(path) => path.clone(),
        }
    }

    /// One of Carbon's icon sizes.
    pub fn size(mut self, size: IconSize) -> Self {
        self.size = size.length();
        self
    }

    /// A custom size.
    pub fn size_length(mut self, size: impl Into<AbsoluteLength>) -> Self {
        self.size = size.into();
        self
    }

    /// Paint in a fixed color (default: the current text color).
    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Paint in a theme role (`icon-primary`, `support-error`, …).
    pub fn role(self, role: ColorRole) -> Self {
        self.color(crate::theme::color(role))
    }

    /// Rotate by `radians` (replaces any transformation).
    pub fn rotate(self, radians: impl Into<Radians>) -> Self {
        self.transform(Transformation::rotate(radians))
    }

    /// Apply an SVG transformation (replaces any rotation).
    pub fn transform(mut self, transformation: Transformation) -> Self {
        self.transformation = Some(transformation);
        self
    }
}

impl Styled for Icon {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for Icon {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let mut icon = gpui_kit::component::Icon::default().path(self.asset_path());
        *icon.style() = self.style;
        let mut icon = icon.size(self.size);
        if let Some(color) = self.color {
            icon = icon.text_color(color);
        }
        if let Some(transformation) = self.transformation {
            icon = icon.transform(transformation);
        }
        icon
    }
}

impl From<Symbol> for Icon {
    fn from(symbol: Symbol) -> Self {
        Self::symbol(symbol)
    }
}

impl From<CarbonIcon> for Icon {
    fn from(icon: CarbonIcon) -> Self {
        Self::carbon(icon)
    }
}

/// Asset source: Material Symbols first, then `fallback` (Kit's assets by default).
pub struct Assets<F: AssetSource = gpui_kit::assets::Assets> {
    inner: gpui_material_symbols::WithMaterialSymbols<F>,
}

impl Assets {
    /// Material Symbols over Kit's bundled assets.
    pub fn new() -> Self {
        Self::with(gpui_kit::assets::Assets)
    }
}

impl Default for Assets {
    fn default() -> Self {
        Self::new()
    }
}

impl<F: AssetSource> Assets<F> {
    /// Material Symbols over a caller-owned source.
    pub fn with(fallback: F) -> Self {
        Self {
            inner: MaterialSymbols.with_fallback(fallback),
        }
    }
}

impl<F: AssetSource> AssetSource for Assets<F> {
    fn load(&self, path: &str) -> gpui_kit::Result<Option<Cow<'static, [u8]>>> {
        if let Some(name) = path
            .strip_prefix(GLYPH_PREFIX)
            .and_then(|p| p.strip_suffix(".svg"))
        {
            return Ok(glyph_svg(name).map(|svg| Cow::Owned(svg.into_bytes())));
        }
        if let Some(name) = path
            .strip_prefix(crate::t3::shell::ASSET_PREFIX)
            .and_then(|p| p.strip_suffix(".svg"))
        {
            return Ok(crate::t3::shell::asset_svg(name).map(|svg| Cow::Owned(svg.into_bytes())));
        }
        self.inner.load(path)
    }
    fn list(&self, prefix: &str) -> gpui_kit::Result<Vec<SharedString>> {
        self.inner.list(prefix)
    }
}

/// A continuously spinning element (loading indicators), one turn per `period`.
/// Under reduced motion it pulses its opacity instead of rotating.
pub fn spin(icon: Icon, id: impl Into<ElementId>, period: Duration, cx: &App) -> impl IntoElement {
    let reduced = crate::motion::reduced_motion(cx);
    icon.with_animation(id, Animation::new(period).repeat(), move |icon, delta| {
        if reduced {
            // A pulse: 1 → 0.4 → 1.
            let opacity = 0.4 + 0.6 * (1. - (delta * 2. - 1.).abs());
            icon.opacity(opacity)
        } else {
            icon.transform(Transformation::rotate(percentage(delta)))
        }
    })
}

/// Make Kit's default icons resolve to Sharp equivalents.
pub(crate) fn init(cx: &mut App) {
    let paths: HashMap<&'static str, SharedString> = kit_mapping::mappings()
        .map(|(path, icon)| (path, icon.asset_path().into()))
        .collect();
    gpui_kit::component::set_icon_resolver(cx, move |path, _| paths.get(path).cloned());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn house_style_paths_load() {
        let assets = Assets::new();
        let symbol = Symbol::named("add").unwrap();
        for filled in [false, true] {
            let path = symbol_path(symbol, filled);
            let parsed = SymbolIcon::from_asset_path(&path).unwrap();
            assert_eq!(parsed.visual_style(), Style::Sharp);
            assert_eq!(parsed.variations().fill(), if filled { 1. } else { 0. });
            assert_eq!(parsed.variations().weight(), 400.);
            assert_eq!(parsed.variations().optical_size(), 24.);
            assert!(assets.load(&path).unwrap().unwrap().starts_with(b"<svg"));
        }
    }

    #[test]
    fn every_carbon_icon_resolves() {
        let missing: Vec<_> = CarbonIcon::ALL
            .iter()
            .filter(|icon| icon.symbol().is_none() && icon.custom_glyph().is_none())
            .map(|icon| format!("{} → {}", icon.name(), icon.material_name()))
            .collect();
        assert!(missing.is_empty(), "{missing:?}");
    }

    #[test]
    fn circle_dash_draws_the_reference_dashed_ring() {
        // `CircleDash = draw("dash")`: eight arc segments, not a plain circle.
        let d = CarbonIcon::CircleDash.custom_glyph().unwrap();
        assert_eq!(d.matches('Z').count(), 8);
        let path = Icon::carbon(CarbonIcon::CircleDash).asset_path();
        assert_eq!(path.as_ref(), "u0ui/glyphs/dash.svg");
        let svg = Assets::new().load(&path).unwrap().unwrap();
        let svg = std::str::from_utf8(&svg).unwrap();
        assert!(svg.starts_with("<svg") && svg.contains("viewBox=\"0 -960 960 960\""));
        assert!(svg.contains(d));
        assert!(CarbonIcon::Add.custom_glyph().is_none());
    }

    #[test]
    fn kit_defaults_map_to_sharp() {
        let mapped: Vec<&str> = kit_mapping::mappings().map(|(path, _)| path).collect();
        for (path, icon) in kit_mapping::mappings() {
            assert!(path.starts_with("icons/"));
            assert_eq!(icon.visual_style(), Style::Sharp);
        }
        assert!(
            kit_mapping::APPROXIMATE_PATHS
                .iter()
                .all(|p| mapped.contains(p))
        );
        assert!(
            !kit_mapping::UNSUPPORTED_PATHS
                .iter()
                .any(|p| mapped.contains(p))
        );
    }
}
