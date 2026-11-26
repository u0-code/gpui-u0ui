//! The active Carbon theme, role lookup, layering and the Kit theme bridge.
//!
//! # Reading colors
//!
//! [`color`] is context-free: it answers from a thread-local copy of the active
//! theme, so helpers and builders stay plain functions (GPUI renders on one UI
//! thread). Use it everywhere in render code:
//!
//! ```ignore
//! div().bg(theme::color(ColorRole::Layer01)).text_color(theme::color(ColorRole::TextPrimary))
//! ```
//!
//! [`color_in`] answers for a fixed theme, e.g. the always-dark UI shell header.
//!
//! # Changing the theme
//!
//! [`set_theme`] is the only write path. It runs from an event handler (never from
//! `render`), updates the thread-local and the [`ActiveTheme`] global, projects the
//! Carbon roles into Kit's [`gpui_kit::component::Theme`] (so Kit inputs,
//! scrollbars, menus and lists look Carbon) and refreshes every window.
//!
//! # Layers
//!
//! Carbon's contextual tokens (`layer`, `field`, `border-subtle`, `border-strong`, …)
//! depend on how deep a surface sits: background → layer-01 → layer-02 → layer-03.
//! A component that paints contextual tokens takes a [`LayerLevel`] explicitly
//! (`.layer(LayerLevel::Two)`), defaulting to [`LayerLevel::One`] (a control on the
//! page background). A container that draws `layer-0N` passes `level.next()` to
//! the controls inside it. Explicit beats implicit here: GPUI has no cascading
//! context for `RenderOnce` children, and an explicit level is easy to test.
use crate::tokens::{COLOR_TABLES, CarbonTheme, ColorRole, FontFamilyKey, REM_PX};
use gpui_kit::component::{Theme as KitTheme, ThemeColor, ThemeMode, ThemeTokens};
use gpui_kit::{App, Global, Hsla, Rgba, px};
use std::cell::Cell;

thread_local! {
    static ACTIVE: Cell<CarbonTheme> = const { Cell::new(CarbonTheme::White) };
}

/// The active theme as a GPUI global, for code that prefers `cx` or wants to
/// observe theme changes (`cx.observe_global::<ActiveTheme>`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ActiveTheme(pub CarbonTheme);
impl Global for ActiveTheme {}

/// The active theme (context-free).
pub fn current() -> CarbonTheme {
    ACTIVE.get()
}

/// The active theme's value of `role` (context-free; see the module docs).
pub fn color(role: ColorRole) -> Hsla {
    rgba(role).into()
}

/// The active theme's exact sRGB value of `role`, alpha preserved.
pub fn rgba(role: ColorRole) -> Rgba {
    COLOR_TABLES[current() as usize][role as usize]
}

/// `role` in a fixed `theme`, regardless of the active one.
pub fn color_in(theme: CarbonTheme, role: ColorRole) -> Hsla {
    role.in_theme(theme)
}

/// Whether the active theme is dark (G90 or G100).
pub fn is_dark() -> bool {
    current().is_dark()
}

/// Switch the theme. Call from event handlers or startup, never from `render`.
///
/// Updates [`current`]/[`ActiveTheme`], re-projects Kit's theme and refreshes
/// every window.
pub fn set_theme(cx: &mut App, theme: CarbonTheme) {
    ACTIVE.set(theme);
    cx.set_global(ActiveTheme(theme));
    sync_kit_theme(theme, cx);
    cx.refresh_windows();
}

/// A region drawn in a fixed theme: Carbon's `<Theme theme="g100">` zone, e.g.
/// the always-dark UI shell header.
///
/// `build` runs while the element lays out, with [`current`] and [`color`]
/// answering for `theme`; the components it returns render (and prepaint and
/// paint) in that theme too. Floating surfaces a component defers past the
/// zone (menus, popovers) and Kit-drawn controls (which read Kit's theme)
/// keep the active theme, so put only plain components in a zone.
///
/// ```ignore
/// div().child(theme::zone(CarbonTheme::G100, |_, _| {
///     NavigationMenu::new("header").brand("Gallery").into_any_element()
/// }))
/// ```
pub fn zone(
    theme: CarbonTheme,
    build: impl FnOnce(&mut gpui_kit::Window, &mut App) -> gpui_kit::AnyElement + 'static,
) -> ThemeZone {
    ThemeZone {
        theme,
        build: Some(Box::new(build)),
        child: None,
    }
}

type ZoneBuild = Box<dyn FnOnce(&mut gpui_kit::Window, &mut App) -> gpui_kit::AnyElement>;

/// The element made by [`zone`].
pub struct ThemeZone {
    theme: CarbonTheme,
    build: Option<ZoneBuild>,
    child: Option<gpui_kit::AnyElement>,
}

impl ThemeZone {
    fn scoped<R>(theme: CarbonTheme, f: impl FnOnce() -> R) -> R {
        let previous = ACTIVE.replace(theme);
        let result = f();
        ACTIVE.set(previous);
        result
    }
}

impl gpui_kit::Element for ThemeZone {
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
        let id = Self::scoped(self.theme, || {
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
            Self::scoped(self.theme, || {
                child.prepaint(window, cx);
            });
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
            Self::scoped(self.theme, || child.paint(window, cx));
        }
    }
}

/// Set the thread-local theme without a `cx` (unit tests of pure look functions).
#[cfg(test)]
pub(crate) fn tests_set(theme: CarbonTheme) {
    ACTIVE.set(theme);
}

/// Install the theme state (called by [`crate::init`]).
pub(crate) fn init(cx: &mut App) {
    let theme = cx
        .try_global::<ActiveTheme>()
        .map(|t| t.0)
        .unwrap_or_default();
    set_theme(cx, theme);
}

/// How deep a surface sits in Carbon's layering model.
///
/// `One` is a control or tile on `background`; `Two` sits on a `layer-01`
/// surface; `Three` on `layer-02`. See the module docs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LayerLevel {
    /// On `background`: paints `layer-01`, `field-01`, …
    #[default]
    One,
    /// On a `layer-01` surface: paints `layer-02`, `field-02`, …
    Two,
    /// On a `layer-02` surface: paints `layer-03`, `field-03`, …
    Three,
}

macro_rules! contextual {
    ($($(#[$doc:meta])* $fn:ident => $one:ident, $two:ident, $three:ident;)+) => {
        impl LayerLevel {
            $(
                $(#[$doc])*
                pub const fn $fn(self) -> ColorRole {
                    match self {
                        Self::One => ColorRole::$one,
                        Self::Two => ColorRole::$two,
                        Self::Three => ColorRole::$three,
                    }
                }
            )+
        }
    };
}

contextual! {
    /// Carbon `$layer`: the surface color at this level.
    layer => Layer01, Layer02, Layer03;
    /// Carbon `$layer-hover`.
    layer_hover => LayerHover01, LayerHover02, LayerHover03;
    /// Carbon `$layer-active`.
    layer_active => LayerActive01, LayerActive02, LayerActive03;
    /// Carbon `$layer-selected`.
    layer_selected => LayerSelected01, LayerSelected02, LayerSelected03;
    /// Carbon `$layer-selected-hover`.
    layer_selected_hover => LayerSelectedHover01, LayerSelectedHover02, LayerSelectedHover03;
    /// Carbon `$layer-accent`.
    layer_accent => LayerAccent01, LayerAccent02, LayerAccent03;
    /// Carbon `$layer-accent-hover`.
    layer_accent_hover => LayerAccentHover01, LayerAccentHover02, LayerAccentHover03;
    /// Carbon `$layer-accent-active`.
    layer_accent_active => LayerAccentActive01, LayerAccentActive02, LayerAccentActive03;
    /// Carbon `$field`: text inputs, selects.
    field => Field01, Field02, Field03;
    /// Carbon `$field-hover`.
    field_hover => FieldHover01, FieldHover02, FieldHover03;
    /// Carbon `$border-subtle`: dividers.
    border_subtle => BorderSubtle01, BorderSubtle02, BorderSubtle03;
    /// Carbon `$border-subtle-selected`.
    border_subtle_selected => BorderSubtleSelected01, BorderSubtleSelected02, BorderSubtleSelected03;
    /// Carbon `$border-strong`: the field bottom rule.
    border_strong => BorderStrong01, BorderStrong02, BorderStrong03;
    /// Carbon `$border-tile`.
    border_tile => BorderTile01, BorderTile02, BorderTile03;
    /// Carbon `$layer-background` (the background of a `<Layer>` region).
    layer_background => LayerBackground01, LayerBackground02, LayerBackground03;
}

impl LayerLevel {
    /// The level of controls placed on a surface painted at this level
    /// (Three stays Three, as in Carbon).
    pub const fn next(self) -> Self {
        match self {
            Self::One => Self::Two,
            Self::Two | Self::Three => Self::Three,
        }
    }
}

// ───────────────────────── Kit bridge ─────────────────────────

/// Carbon role → Kit `ThemeColor` field. Only colors; geometry and fonts are
/// written in [`write_kit_projection`].
macro_rules! kit_projection {
    ($($field:ident => $role:ident $(@ $alpha:expr)?),* $(,)?) => {
        fn project_colors(colors: &mut ThemeColor, theme: CarbonTheme) {
            $(colors.$field = kit_projection!(@color theme, $role $(, $alpha)?);)*
        }
    };
    (@color $theme:ident, $role:ident) => { ColorRole::$role.in_theme($theme) };
    (@color $theme:ident, $role:ident, $alpha:expr) => {{
        let color = ColorRole::$role.in_theme($theme);
        color.opacity($alpha)
    }};
}

kit_projection! {
    background => Background,
    foreground => TextPrimary,
    border => BorderSubtle01,
    input => BorderStrong01,
    ring => Focus,
    caret => TextPrimary,
    selection => Highlight,
    overlay => Overlay,
    window_border => BorderSubtle01,
    drag_border => BorderInteractive,
    drop_target => Highlight,

    primary => ButtonPrimary,
    primary_hover => ButtonPrimaryHover,
    primary_active => ButtonPrimaryActive,
    primary_foreground => TextOnColor,
    secondary => Layer01,
    secondary_hover => LayerHover01,
    secondary_active => LayerActive01,
    secondary_foreground => TextPrimary,
    muted => Layer01,
    muted_foreground => TextSecondary,
    accent => LayerHover01,
    accent_foreground => TextPrimary,
    popover => Layer01,
    popover_foreground => TextPrimary,

    danger => SupportError,
    danger_hover => ButtonDangerHover,
    danger_active => ButtonDangerActive,
    danger_foreground => TextOnColor,
    success => SupportSuccess,
    success_hover => SupportSuccess,
    success_active => SupportSuccess,
    success_foreground => TextOnColor,
    warning => SupportWarning,
    warning_hover => SupportWarning,
    warning_active => SupportWarning,
    warning_foreground => TextPrimary,
    info => SupportInfo,
    info_hover => SupportInfo,
    info_active => SupportInfo,
    info_foreground => TextOnColor,

    button => Field01,
    button_hover => FieldHover01,
    button_active => LayerActive01,
    button_foreground => TextPrimary,
    button_primary => ButtonPrimary,
    button_primary_hover => ButtonPrimaryHover,
    button_primary_active => ButtonPrimaryActive,
    button_primary_foreground => TextOnColor,
    button_secondary => ButtonSecondary,
    button_secondary_hover => ButtonSecondaryHover,
    button_secondary_active => ButtonSecondaryActive,
    button_secondary_foreground => TextOnColor,
    button_danger => ButtonDangerPrimary,
    button_danger_hover => ButtonDangerHover,
    button_danger_active => ButtonDangerActive,
    button_danger_foreground => TextOnColor,
    button_info => ButtonPrimary,
    button_info_hover => ButtonPrimaryHover,
    button_info_active => ButtonPrimaryActive,
    button_info_foreground => TextOnColor,
    button_success => SupportSuccess,
    button_success_hover => SupportSuccess,
    button_success_active => SupportSuccess,
    button_success_foreground => TextOnColor,
    button_warning => SupportWarning,
    button_warning_hover => SupportWarning,
    button_warning_active => SupportWarning,
    button_warning_foreground => TextPrimary,

    link => LinkPrimary,
    link_hover => LinkPrimaryHover,
    link_active => LinkPrimary,

    list => Background,
    list_even => Background,
    list_head => LayerAccent01,
    list_hover => LayerHover01,
    list_active => LayerSelected01,
    list_active_border => BorderInteractive,
    list_highlight_foreground => TextPrimary,

    table => Layer01,
    table_even => Layer01,
    table_head => LayerAccent01,
    table_head_foreground => TextPrimary,
    table_foot => LayerAccent01,
    table_foot_foreground => TextPrimary,
    table_hover => LayerHover01,
    table_active => LayerSelected01,
    table_active_border => BorderInteractive,
    table_row_border => BorderSubtle01,

    // Carbon defines no scrollbar; the thumb is a strong border on no track.
    scrollbar => Background @ 0.,
    scrollbar_thumb => BorderStrong01,
    scrollbar_thumb_hover => IconSecondary,

    tab => Background @ 0.,
    tab_active => Layer01,
    tab_active_foreground => TextPrimary,
    tab_bar => Background,
    tab_bar_segmented => LayerAccent01,
    tab_foreground => TextSecondary,

    sidebar => Layer01,
    sidebar_foreground => TextSecondary,
    sidebar_border => BorderSubtle01,
    sidebar_accent => LayerHover01,
    sidebar_accent_foreground => TextPrimary,
    sidebar_primary => Interactive,
    sidebar_primary_foreground => TextOnColor,

    title_bar => Background,
    title_bar_border => BorderSubtle01,
    status_bar => Background,
    status_bar_border => BorderSubtle01,

    accordion => Layer01,
    group_box => Layer01,
    group_box_foreground => TextPrimary,
    description_list_label => LayerAccent01,
    description_list_label_foreground => TextSecondary,
    skeleton => SkeletonBackground,
    progress_bar => Interactive,
    slider_bar => Interactive,
    slider_thumb => Interactive,
    switch => ToggleOff,
    switch_thumb => IconOnColor,

    chart_1 => Charts1,
    chart_2 => Charts2,
    chart_3 => Charts3,
    chart_4 => Charts4,
    chart_5 => Charts5,
    chart_bullish => SupportSuccess,
    chart_bearish => SupportError,
    chart_grid => BorderSubtle01,

    red => TagColorRed,
    red_light => TagBackgroundRed,
    green => TagColorGreen,
    green_light => TagBackgroundGreen,
    blue => TagColorBlue,
    blue_light => TagBackgroundBlue,
    yellow => SupportWarning,
    yellow_light => NotificationBackgroundWarning,
    magenta => TagColorMagenta,
    magenta_light => TagBackgroundMagenta,
    cyan => TagColorCyan,
    cyan_light => TagBackgroundCyan,
}

/// Carbon motion projected into Kit's semantic motion tokens.
fn project_motion(motion: &mut gpui_kit::component::MotionTokens) {
    use crate::motion::{Curve, Speed};
    motion.duration_fast = Speed::Fast01.duration();
    motion.duration_normal = Speed::Fast02.duration();
    motion.duration_slow = Speed::Moderate02.duration();
    motion.easing_enter = Curve::ProductiveEntrance.easing();
    motion.easing_exit = Curve::ProductiveExit.easing();
    motion.easing_move = Curve::ProductiveStandard.easing();
}

/// Write the whole Carbon projection (colors, tokens, fonts, geometry) onto a
/// Kit theme value. Pure: tests call it on a `KitTheme::default()`.
fn write_kit_projection(kit: &mut KitTheme, theme: CarbonTheme) {
    project_colors(&mut kit.colors, theme);
    kit.tokens = ThemeTokens::from(kit.colors);
    kit.font_family = FontFamilyKey::Sans.primary().into();
    kit.mono_font_family = FontFamilyKey::Mono.primary().into();
    // Kit's font size doubles as the window rem size (Root::prepare), and every
    // Carbon length is in 16px rems; body text sets its own 14px type style.
    kit.font_size = px(REM_PX);
    // `code-02`: 0.875rem.
    kit.mono_font_size = px(crate::tokens::TypeStyle::Code02.spec().font_size_px());
    // `border-radius-00`: Carbon is square.
    kit.radius = px(0.);
    kit.radius_lg = px(0.);
    kit.window_radius = px(0.);
    // Floating layers (menus, popovers) carry Carbon's `0 2px 6px shadow`.
    kit.shadow = true;
    kit.focus_ring = true;
    project_motion(&mut kit.motion);
}

/// Whether `kit` already shows `theme` (avoids needless refreshes).
fn kit_projection_matches(kit: &KitTheme, theme: CarbonTheme) -> bool {
    let mut expected = kit.clone();
    write_kit_projection(&mut expected, theme);
    expected.colors == kit.colors
        && expected.tokens == kit.tokens
        && expected.font_family == kit.font_family
        && expected.font_size == kit.font_size
        && expected.radius == kit.radius
        && kit.mode.is_dark() == theme.is_dark()
}

/// Project `theme` into Kit's global theme. Returns whether anything changed.
fn sync_kit_theme(theme: CarbonTheme, cx: &mut App) -> bool {
    if cx.try_global::<KitTheme>().is_none() {
        return false;
    }
    if kit_projection_matches(KitTheme::global(cx), theme) {
        return false;
    }
    let mode = if theme.is_dark() {
        ThemeMode::Dark
    } else {
        ThemeMode::Light
    };
    if KitTheme::global(cx).mode != mode {
        // A mode change reloads Kit's registered config for that mode, which
        // would overwrite our colors; do it first, in its own update.
        KitTheme::update(cx, |kit| kit.mode = mode);
    }
    KitTheme::update(cx, |kit| write_kit_projection(kit, theme));
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_zone_answers_for_its_theme_and_restores_the_active_one() {
        tests_set(CarbonTheme::White);
        let inside = ThemeZone::scoped(CarbonTheme::G100, || {
            (current(), color(ColorRole::Background))
        });
        assert_eq!(
            inside,
            (
                CarbonTheme::G100,
                ColorRole::Background.in_theme(CarbonTheme::G100)
            )
        );
        assert_eq!(current(), CarbonTheme::White);
    }

    #[test]
    fn layer_levels_map_to_contextual_tokens() {
        assert_eq!(LayerLevel::One.layer(), ColorRole::Layer01);
        assert_eq!(LayerLevel::Two.field(), ColorRole::Field02);
        assert_eq!(LayerLevel::Three.border_strong(), ColorRole::BorderStrong03);
        assert_eq!(LayerLevel::One.next(), LayerLevel::Two);
        assert_eq!(LayerLevel::Three.next(), LayerLevel::Three);
    }

    #[test]
    fn kit_projection_is_carbon() {
        for theme in CarbonTheme::ALL {
            let mut kit = KitTheme::default();
            write_kit_projection(&mut kit, theme);
            assert_eq!(kit.colors.background, ColorRole::Background.in_theme(theme));
            assert_eq!(kit.colors.primary, ColorRole::ButtonPrimary.in_theme(theme));
            assert_eq!(kit.colors.ring, ColorRole::Focus.in_theme(theme));
            assert_eq!(kit.tokens.primary.color, kit.colors.primary);
            assert_eq!(kit.colors.scrollbar.a, 0.);
            assert_eq!(kit.radius, px(0.));
            assert_eq!(kit.font_family.as_ref(), "IBM Plex Sans");
            assert_eq!(kit.font_size, px(16.));
        }
    }

    #[test]
    fn set_theme_updates_context_free_reads() {
        tests_set(CarbonTheme::G100);
        assert!(is_dark());
        assert_eq!(
            color(ColorRole::Background),
            ColorRole::Background.in_theme(CarbonTheme::G100)
        );
        tests_set(CarbonTheme::White);
        assert!(!is_dark());
    }
}
