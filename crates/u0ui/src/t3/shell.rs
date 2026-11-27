//! T3 · Shell, navigation and identity (`src/t3/shell.jsx`).
//!
//! Identity and status primitives, shared by every T3 card:
//! [`ProviderIcon`], [`ProjectMonogram`], [`ProjectFavicon`],
//! [`EnvironmentMachineIcon`], [`ConnectionStatusDot`], [`EnvironmentBadge`],
//! [`StatusPill`] and the split-button chevron [`IconMenu`].
//!
//! The cards: the sidebar ([`sidebar`]) the page chrome ([`chrome`]) and the
//! command palette and list toolbar ([`palette`]), and the frame
//! ([`app_shell`]).
//!
//! Values come from `src/t3.css` (`/* Identity */`), with the later craft
//! passes applied: provider marks are glyphs, never two-letter tiles
//! (`r2-desk`, `r3-gal`), and project monograms are neutral (`r2-desk`).
use super::base::{EASE, EASE_IN_OUT, Status, Tone, pulse};
use crate::{
    components::{
        button::{Button, ButtonSize, ButtonVariant},
        menu::{self, MenuItem, MenuSize},
        overlay,
    },
    icons::{CarbonIcon, Icon},
    motion,
    theme::color,
    tokens::{ColorRole, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    Animation, AnimationExt as _, App, ElementId, FontWeight, Hsla, ImageSource,
    InteractiveElement, IntoElement, ObjectFit, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement as _, Styled, StyledImage as _, Window, canvas, div, img,
    prelude::FluentBuilder as _, px, rgb,
};
use std::rc::Rc;
use std::time::Duration;

pub mod app_shell;
pub mod chrome;
pub mod palette;
pub mod sidebar;

pub use app_shell::AppShell;
pub use chrome::{
    Crumb, LayoutPanel, PanelLayoutControls, RightPanelTabs, SurfaceKind, SurfaceTab,
    WorkspaceBreadcrumb, WorkspacePageHeader,
};
pub use palette::{CommandPalette, ListFilter, ListToolbar, PaletteGroup, PaletteItem};
pub use sidebar::{
    FolderCounts, PrState, ProjectFolderHeader, PullRequestMiniBadge, SidebarBrandHeader,
    SidebarShelfHeader, ThreadRow, UpdatePill, UpdateState,
};

/// A plain hover tooltip standing in for the HTML `title` attribute (a
/// native browser tooltip, not a Carbon `Tooltip`): Kit's window tooltip.
fn title_tooltip(text: SharedString) -> impl Fn(&mut Window, &mut App) -> gpui_kit::AnyView {
    move |window, cx| gpui_kit::component::tooltip::Tooltip::new(text.clone()).build(window, cx)
}

/// `.t3-mono--*` / `.t3-provider--*` sizes (`xs | sm | md | lg`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum IdentitySize {
    /// `xs`: 1rem.
    Xs,
    /// `sm`: 1.25rem.
    Sm,
    /// `md`: 1.5rem (the default).
    #[default]
    Md,
    /// `lg`: 2.5rem.
    Lg,
}

impl IdentitySize {
    /// Every size.
    pub const ALL: [IdentitySize; 4] = [Self::Xs, Self::Sm, Self::Md, Self::Lg];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Xs => "xs",
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
        }
    }

    /// The tile edge in px (`.t3-mono--xs { width: 1rem }`, `--sm` 1.25rem,
    /// `--md` 1.5rem, `--lg` 2.5rem; `.t3-provider--*` the same).
    pub const fn px(self) -> f32 {
        match self {
            Self::Xs => 16.,
            Self::Sm => 20.,
            Self::Md => 24.,
            Self::Lg => 40.,
        }
    }

    /// The monogram letters' font size in px (`.t3-mono--xs { font-size:
    /// 0.5rem }`, `.t3-mono { 0.625rem }` for sm, `--md` 0.6875rem, `--lg` 1rem).
    pub const fn monogram_font(self) -> f32 {
        match self {
            Self::Xs => 8.,
            Self::Sm => 10.,
            Self::Md => 11.,
            Self::Lg => 16.,
        }
    }
}

/// `.t3-mono, .t3-provider, .t3-favicon { border-radius: var(--border-radius-04) }`.
pub const TILE_RADIUS: f32 = 4.;

// ── ProviderIcon ────────────────────────────────────────────────────────

/// The provider glyphs of `t3.css` (`r2-desk: provider marks are glyphs`),
/// keyed on the name prefix (`.t3-provider[title^=Claude]`), as SVG path
/// data in a 16×16 viewBox (`fill-rule: evenodd`).
pub const PROVIDER_GLYPHS: [(&str, &str); 7] = [
    (
        "Claude",
        "M8.00 0.50 L8.41 6.45 L11.75 1.50 L9.13 6.87 L14.50 4.25 L9.55 7.59 L15.50 8.00 L9.55 8.41 L14.50 11.75 L9.13 9.13 L11.75 14.50 L8.41 9.55 L8.00 15.50 L7.59 9.55 L4.25 14.50 L6.87 9.13 L1.50 11.75 L6.45 8.41 L0.50 8.00 L6.45 7.59 L1.50 4.25 L6.87 6.87 L4.25 1.50 L7.59 6.45Z",
    ),
    (
        "Codex",
        "M8 .8 14.4 4.4v7.2L8 15.2 1.6 11.6V4.4z M8 3 3.6 5.5v5L8 13l4.4-2.5v-5z M5.2 6.4 7.6 8l-2.4 1.6-.8-1.1L5.3 8l-1-.5z M8.4 9.4h3.2v1.2H8.4z",
    ),
    (
        "Cursor",
        "M8 .8 14.6 4.6v6.8L8 15.2 1.4 11.4V4.6z M8 2.6 3 5.5 8 8.4l5-2.9z",
    ),
    ("Antigravity", "M1.5 14.5 8 1.5l6.5 13h-2.9L8 7.3 4.4 14.5z"),
    (
        "OpenCode",
        "M2.5 2h4.5v2.2H4.8v7.6H7V14H2.5z M13.5 2H9v2.2h2.2v7.6H9V14h4.5z",
    ),
    (
        "Figma",
        "M5 1h3v4.6H5A2.3 2.3 0 0 1 5 1z M8 1h3a2.3 2.3 0 0 1 0 4.6H8z M5 5.6h3v4.6H5a2.3 2.3 0 0 1 0-4.6z M10.3 5.6a2.3 2.3 0 1 1 0 4.6 2.3 2.3 0 0 1 0-4.6z M5 10.2h3v2.4A2.3 2.3 0 1 1 5 10.2z",
    ),
    (
        "Gemini",
        "M8 .5C8.6 4.6 11.4 7.4 15.5 8 11.4 8.6 8.6 11.4 8 15.5 7.4 11.4 4.6 8.6.5 8 4.6 7.4 7.4 4.6 8 .5z",
    ),
];

/// The asset path prefix [`crate::Assets`] serves T3 SVGs under.
pub(crate) const ASSET_PREFIX: &str = "u0ui/t3/";

/// The SVG for a T3 asset name (`provider-Claude`), if any.
pub(crate) fn asset_svg(name: &str) -> Option<String> {
    let key = name.strip_prefix("provider-")?;
    let (_, d) = PROVIDER_GLYPHS.iter().find(|(k, _)| *k == key)?;
    Some(format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 16 16\" width=\"16\" \
         height=\"16\"><path fill-rule=\"evenodd\" d=\"{d}\"/></svg>"
    ))
}

/// The known provider a name starts with (`[title^=Claude]`), if any.
pub fn provider_glyph(name: &str) -> Option<&'static str> {
    PROVIDER_GLYPHS
        .iter()
        .map(|(k, _)| *k)
        .find(|k| name.starts_with(k))
}

/// The glyph color of a provider mark without an image: Claude
/// `#c15f3c` and Gemini `#1a73e8` (brand inks hard-coded by `t3.css`),
/// the other known marks `text-primary`, an unknown agent's generic glyph
/// `icon-secondary` (`.t3-provider:has(> .t3-provider-generic)`).
pub fn provider_glyph_color(name: &str) -> Hsla {
    match provider_glyph(name) {
        // `.t3-provider[title^=Claude]:not(:has(img)) { color: #c15f3c }`.
        Some("Claude") => rgb(0xc15f3c).into(),
        // `.t3-provider[title^=Gemini]:not(:has(img)) { color: #1a73e8 }`.
        Some("Gemini") => rgb(0x1a73e8).into(),
        Some(_) => color(ColorRole::TextPrimary),
        None => color(ColorRole::IconSecondary),
    }
}

/// The Material symbol an unknown agent draws (`.t3-provider-generic`:
/// "one neutral generic glyph, never a two-letter tile").
pub const GENERIC_PROVIDER_SYMBOL: &str = "smart_toy";

/// `.t3-provider-spin { animation: cu-spin 0.8s linear infinite }`.
pub const PROVIDER_SPIN_PERIOD: Duration = Duration::from_millis(800);

/// The agent accent colors (`--agent-accent-1..8`).
const ACCENTS: [ColorRole; 8] = [
    ColorRole::AgentAccent1,
    ColorRole::AgentAccent2,
    ColorRole::AgentAccent3,
    ColorRole::AgentAccent4,
    ColorRole::AgentAccent5,
    ColorRole::AgentAccent6,
    ColorRole::AgentAccent7,
    ColorRole::AgentAccent8,
];

/// An agent provider's mark (`ProviderIcon`). No vendor logo is copied: a
/// known provider (Claude, Codex, Cursor, Antigravity, OpenCode, Figma,
/// Gemini) draws its abstract glyph at 84% of the tile on a transparent
/// ground, an unknown one a neutral generic glyph; pass `src` for an icon
/// you may use (then the tile is `layer-accent-01`, or the agent accent).
/// `starting` adds the 10px spinning ring at the bottom-right corner.
#[derive(IntoElement)]
pub struct ProviderIcon {
    id: Option<ElementId>,
    name: SharedString,
    src: Option<ImageSource>,
    starting: bool,
    size: IdentitySize,
    accent: Option<u8>,
    starting_label: Option<SharedString>,
}

impl ProviderIcon {
    /// The mark for provider `name` (`name`, default `"Agent"` in the JSX).
    pub fn new(name: impl Into<SharedString>) -> Self {
        Self {
            id: None,
            name: name.into(),
            src: None,
            starting: false,
            size: IdentitySize::Md,
            accent: None,
            starting_label: None,
        }
    }

    /// A stable id (defaults to one derived from the name).
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// An image to show instead of the glyph (`src`).
    pub fn src(mut self, src: impl Into<ImageSource>) -> Self {
        self.src = Some(src.into());
        self
    }

    /// Show the starting ring (`starting`).
    pub fn starting(mut self, starting: bool) -> Self {
        self.starting = starting;
        self
    }

    /// The size (`size`).
    pub fn size(mut self, size: IdentitySize) -> Self {
        self.size = size;
        self
    }

    /// The agent accent 1..=8 (`accent`, `--agent-accent-N`): the tile
    /// behind an image. Without an image the glyph ground stays transparent
    /// (`.t3-provider:not(:has(img))` outranks `.t3-provider--accent`).
    pub fn accent(mut self, accent: Option<u8>) -> Self {
        self.accent = accent;
        self
    }

    /// The hover title while starting (default `"<name> is starting"`).
    pub fn starting_label(mut self, label: impl Into<SharedString>) -> Self {
        self.starting_label = Some(label.into());
        self
    }
}

/// Paint the `.t3-provider-spin` ring: a 10px disc in `background` with a
/// 2px `layer-accent-01` border whose top quarter is `interactive`.
fn paint_provider_spin(bounds: gpui_kit::Bounds<gpui_kit::Pixels>, turn: f32, window: &mut Window) {
    use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, TAU};
    let center = bounds.center();
    let edge = f32::from(bounds.size.width);
    let border = edge / 5.;
    window.paint_quad(
        gpui_kit::fill(bounds, color(ColorRole::Background)).corner_radii(px(edge / 2.)),
    );
    let r = (edge - border) / 2.;
    super::base::stroke_arc(
        window,
        center,
        r,
        border,
        0.,
        TAU,
        color(ColorRole::LayerAccent01),
    );
    // `border-top-color`: the top edge spans 315°..45° around 12 o'clock.
    let start = -FRAC_PI_2 - FRAC_PI_4 + turn * TAU;
    super::base::stroke_arc(
        window,
        center,
        r,
        border,
        start,
        FRAC_PI_2,
        color(ColorRole::Interactive),
    );
}

impl RenderOnce for ProviderIcon {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self
            .id
            .unwrap_or_else(|| ElementId::Name(format!("t3-provider-{}", self.name).into()));
        let edge = self.size.px();
        let title: SharedString = if self.starting {
            self.starting_label
                .unwrap_or_else(|| format!("{} is starting", self.name).into())
        } else {
            self.name.clone()
        };
        let has_src = self.src.is_some();
        let reduced = motion::reduced_motion(cx);
        let mark = match self.src {
            // `.t3-provider img { width: 100%; height: 100%; object-fit: contain }`.
            Some(src) => img(src)
                .size_full()
                .object_fit(ObjectFit::Contain)
                .into_any_element(),
            None => {
                // `::before { width: 84%; height: 84% }` / `.t3-provider-generic { 84% }`.
                let glyph = px(edge * 0.84);
                let icon = match provider_glyph(&self.name) {
                    Some(key) => Icon::from_path(format!("{ASSET_PREFIX}provider-{key}.svg")),
                    None => Icon::named(GENERIC_PROVIDER_SYMBOL),
                };
                icon.size_length(glyph)
                    .color(provider_glyph_color(&self.name))
                    .into_any_element()
            }
        };
        let ground = if has_src {
            // `.t3-provider { background: var(--layer-accent-01) }`,
            // `.t3-provider--accent { background: var(--pa) }`.
            match self.accent.filter(|a| (1..=8).contains(a)) {
                Some(a) => color(ACCENTS[a as usize - 1]),
                None => color(ColorRole::LayerAccent01),
            }
        } else {
            gpui_kit::transparent_black()
        };
        let spin = self.starting.then(|| {
            let ring = div().size_full().with_animation(
                (id.clone(), "spin"),
                Animation::new(PROVIDER_SPIN_PERIOD).repeat(),
                move |el, t| {
                    // Reduced motion: rotation becomes a pulse (u0ui motion rules).
                    let el = if reduced {
                        el.opacity(super::base::pulse_opacity(t, EASE_IN_OUT))
                    } else {
                        el
                    };
                    let turn = if reduced { 0. } else { t };
                    el.child(
                        canvas(
                            |_, _, _| {},
                            move |b, _, window, _| paint_provider_spin(b, turn, window),
                        )
                        .size_full(),
                    )
                },
            );
            // `.t3-provider-spin { position: absolute; right: -4px; bottom: -4px; 10×10 }`.
            div()
                .absolute()
                .right(px(-4.))
                .bottom(px(-4.))
                .size(px(10.))
                .child(ring)
        });
        div()
            .id(id)
            .relative()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .size(px(edge))
            .rounded(px(TILE_RADIUS))
            .bg(ground)
            .when(has_src, |this| this.overflow_hidden())
            .tooltip(title_tooltip(title))
            .child(mark)
            .children(spin)
    }
}

// ── ProjectMonogram / ProjectFavicon ────────────────────────────────────

/// The tag color pair a monogram is hashed onto (`MONO_TONES`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MonoTone {
    /// `blue`.
    Blue,
    /// `teal`.
    Teal,
    /// `purple`.
    Purple,
    /// `magenta`.
    Magenta,
    /// `cyan`.
    Cyan,
    /// `green`.
    Green,
    /// `red`.
    Red,
    /// `warm-gray`.
    WarmGray,
}

impl MonoTone {
    /// `MONO_TONES`, in order.
    pub const ALL: [MonoTone; 8] = [
        Self::Blue,
        Self::Teal,
        Self::Purple,
        Self::Magenta,
        Self::Cyan,
        Self::Green,
        Self::Red,
        Self::WarmGray,
    ];

    /// The reference name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Blue => "blue",
            Self::Teal => "teal",
            Self::Purple => "purple",
            Self::Magenta => "magenta",
            Self::Cyan => "cyan",
            Self::Green => "green",
            Self::Red => "red",
            Self::WarmGray => "warm-gray",
        }
    }

    /// The tone `name` hashes to (`MONO_TONES[hash(name) % 8]`).
    pub fn for_name(name: &str) -> Self {
        Self::ALL[(monogram_hash(name) % 8) as usize]
    }
}

/// `hash = (s) => [...s].reduce((a, c) => (a * 31 + c.charCodeAt(0)) >>> 0, 7)`:
/// per code point, its first UTF-16 unit, folded modulo 2³².
pub fn monogram_hash(name: &str) -> u32 {
    name.chars().fold(7u32, |a, c| {
        let mut units = [0u16; 2];
        let unit = c.encode_utf16(&mut units)[0];
        a.wrapping_mul(31).wrapping_add(unit as u32)
    })
}

/// The monogram letters: non-alphanumerics become spaces, the first letter
/// of the first two words, upper-cased; `"?"` when nothing is left.
pub fn monogram_letters(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { ' ' })
        .collect();
    let letters: String = cleaned
        .split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase();
    if letters.is_empty() {
        "?".into()
    } else {
        letters
    }
}

/// A project's monogram tile (`ProjectMonogram`): the initials of `name`,
/// 600 weight, on a 4px-rounded tile. The hashed tag tone is kept for API
/// parity, but the final rule draws every monogram neutral
/// (`r2-desk: .t3-mono[class*="t3-tone--"] { background:
/// var(--layer-accent-01); color: var(--text-secondary) }`), so a project
/// avatar never reads as a status or a provider tile.
#[derive(IntoElement, Clone)]
pub struct ProjectMonogram {
    name: SharedString,
    size: IdentitySize,
    tone: Option<MonoTone>,
}

impl ProjectMonogram {
    /// The monogram of `name`.
    pub fn new(name: impl Into<SharedString>) -> Self {
        Self {
            name: name.into(),
            size: IdentitySize::Md,
            tone: None,
        }
    }

    /// The size (`size`).
    pub fn size(mut self, size: IdentitySize) -> Self {
        self.size = size;
        self
    }

    /// Force a tone (`tone`); hashed from the name otherwise.
    pub fn tone(mut self, tone: MonoTone) -> Self {
        self.tone = Some(tone);
        self
    }

    /// The tone in effect.
    pub fn resolved_tone(&self) -> MonoTone {
        self.tone.unwrap_or_else(|| MonoTone::for_name(&self.name))
    }

    /// (background, text): `layer-accent-01` / `text-secondary` for every tone.
    pub fn colors() -> (Hsla, Hsla) {
        (
            color(ColorRole::LayerAccent01),
            color(ColorRole::TextSecondary),
        )
    }
}

fn tile(size: IdentitySize) -> gpui_kit::Div {
    // `.t3-mono { display: inline-flex; flex: none; align-items: center;
    // justify-content: center; border-radius: var(--border-radius-04) }`.
    div()
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .size(px(size.px()))
        .rounded(px(TILE_RADIUS))
}

impl RenderOnce for ProjectMonogram {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let _tone = self.resolved_tone();
        let (bg, fg) = Self::colors();
        let font = self.size.monogram_font();
        tile(self.size)
            .bg(bg)
            .text_color(fg)
            .font_family(crate::typography::font(crate::tokens::FontFamilyKey::Sans, 600).family)
            .font_weight(FontWeight::SEMIBOLD)
            .text_size(px(font))
            .line_height(px(font))
            .child(monogram_letters(&self.name))
    }
}

/// A project's favicon (`ProjectFavicon`): the image `src` (falling back to
/// the monogram when it fails to load), else an `icon` tile
/// (`.t3-mono--icon`: `layer-accent-01`, `icon-primary`, 16px glyph), else
/// the [`ProjectMonogram`].
#[derive(IntoElement)]
pub struct ProjectFavicon {
    name: SharedString,
    src: Option<ImageSource>,
    icon: Option<Icon>,
    size: IdentitySize,
}

impl ProjectFavicon {
    /// The favicon of project `name`.
    pub fn new(name: impl Into<SharedString>) -> Self {
        Self {
            name: name.into(),
            src: None,
            icon: None,
            size: IdentitySize::Md,
        }
    }

    /// The favicon image (`src`).
    pub fn src(mut self, src: impl Into<ImageSource>) -> Self {
        self.src = Some(src.into());
        self
    }

    /// An icon instead of the monogram (`icon`).
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// The size (`size`).
    pub fn size(mut self, size: IdentitySize) -> Self {
        self.size = size;
        self
    }
}

impl RenderOnce for ProjectFavicon {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let monogram = ProjectMonogram::new(self.name.clone()).size(self.size);
        let edge = px(self.size.px());
        match (self.src, self.icon) {
            // `.t3-favicon { border-radius: var(--border-radius-04); object-fit: cover }`
            // with the `.t3-mono--<size>` box; `onError` falls back to the monogram.
            (Some(src), icon) => {
                let fallback_icon = icon.clone();
                let size = self.size;
                img(src)
                    .flex_none()
                    .size(edge)
                    .rounded(px(TILE_RADIUS))
                    .object_fit(ObjectFit::Cover)
                    .with_fallback(move || match &fallback_icon {
                        Some(icon) => icon_tile(icon.clone(), size).into_any_element(),
                        None => monogram.clone().into_any_element(),
                    })
                    .into_any_element()
            }
            (None, Some(icon)) => icon_tile(icon, self.size).into_any_element(),
            (None, None) => monogram.into_any_element(),
        }
    }
}

fn icon_tile(icon: Icon, size: IdentitySize) -> gpui_kit::Div {
    // `.t3-mono--icon { background: var(--layer-accent-01); color: var(--icon-primary) }`.
    tile(size).bg(color(ColorRole::LayerAccent01)).child(
        icon.size_length(px(16.))
            .color(color(ColorRole::IconPrimary)),
    )
}

// ── Environment ─────────────────────────────────────────────────────────

/// `EnvironmentMachineIcon` `kind`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MachineKind {
    /// `laptop` (the default).
    #[default]
    Laptop,
    /// `desktop`.
    Desktop,
    /// `server`.
    Server,
    /// `cloud`.
    Cloud,
    /// `phone`.
    Phone,
}

impl MachineKind {
    /// Every kind.
    pub const ALL: [MachineKind; 5] = [
        Self::Laptop,
        Self::Desktop,
        Self::Server,
        Self::Cloud,
        Self::Phone,
    ];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Laptop => "laptop",
            Self::Desktop => "desktop",
            Self::Server => "server",
            Self::Cloud => "cloud",
            Self::Phone => "phone",
        }
    }

    /// The glyph (`MACHINE`): Laptop, Screen, Cloud; `server` draws the
    /// `dns` glyph (`index.d.ts`), `phone` a smartphone (`phone_iphone`,
    /// the `r2-set` fix for Carbon `Phone`'s handset).
    pub fn icon(self) -> Icon {
        match self {
            Self::Laptop => Icon::carbon(CarbonIcon::Laptop),
            Self::Desktop => Icon::carbon(CarbonIcon::Screen),
            Self::Server => Icon::named("dns"),
            Self::Cloud => Icon::carbon(CarbonIcon::Cloud),
            Self::Phone => Icon::named("phone_iphone"),
        }
    }
}

/// A machine glyph (`EnvironmentMachineIcon`, `.t3-machine { fill:
/// var(--icon-secondary) }`), 16px by default.
#[derive(IntoElement)]
pub struct EnvironmentMachineIcon {
    kind: MachineKind,
    size: f32,
}

impl EnvironmentMachineIcon {
    /// The glyph for `kind`.
    pub fn new(kind: MachineKind) -> Self {
        Self { kind, size: 16. }
    }

    /// The size in px (`size`).
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }
}

impl RenderOnce for EnvironmentMachineIcon {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        self.kind
            .icon()
            .size_length(px(self.size))
            .color(color(ColorRole::IconSecondary))
            .flex_none()
    }
}

/// `ConnectionStatusDot` `status`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ConnectionStatus {
    /// `connected` (the default).
    #[default]
    Connected,
    /// `connecting`: pulses.
    Connecting,
    /// `offline`: hollow.
    Offline,
    /// `error`.
    Error,
    /// `sleeping`: hollow, like offline.
    Sleeping,
}

impl ConnectionStatus {
    /// Every status.
    pub const ALL: [ConnectionStatus; 5] = [
        Self::Connected,
        Self::Connecting,
        Self::Offline,
        Self::Error,
        Self::Sleeping,
    ];

    /// The reference name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Connected => "connected",
            Self::Connecting => "connecting",
            Self::Offline => "offline",
            Self::Error => "error",
            Self::Sleeping => "sleeping",
        }
    }

    /// The default label (`aria-label` / `title`); callers may override it.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Connected => "Connected",
            Self::Connecting => "Connecting…",
            Self::Offline => "Offline",
            Self::Error => "Connection error",
            Self::Sleeping => "Computer is off",
        }
    }

    /// (fill, inset ring): `.t3-dot--connected { background:
    /// var(--support-success) }`, `--connecting` `support-warning`,
    /// `--error` `support-error`; `--offline, --sleeping` are transparent
    /// with `box-shadow: inset 0 0 0 1.5px var(--icon-secondary)`.
    pub fn colors(self) -> (Hsla, Option<Hsla>) {
        match self {
            Self::Connected => (color(ColorRole::SupportSuccess), None),
            Self::Connecting => (color(ColorRole::SupportWarning), None),
            Self::Error => (color(ColorRole::SupportError), None),
            Self::Offline | Self::Sleeping => (
                gpui_kit::transparent_black(),
                Some(color(ColorRole::IconSecondary)),
            ),
        }
    }
}

/// `.t3-dot { width: 8px; height: 8px; border-radius: max }`.
pub const DOT_SIZE: f32 = 8.;
/// `.t3-dot--offline { box-shadow: inset 0 0 0 1.5px … }`.
pub const DOT_RING: f32 = 1.5;

/// A connection status dot (`ConnectionStatusDot`): 8px, green / pulsing
/// yellow / hollow / red, titled with its label.
#[derive(IntoElement)]
pub struct ConnectionStatusDot {
    id: Option<ElementId>,
    status: ConnectionStatus,
    label: Option<SharedString>,
    ring: Option<Hsla>,
}

impl ConnectionStatusDot {
    /// A dot for `status`.
    pub fn new(status: ConnectionStatus) -> Self {
        Self {
            id: None,
            status,
            label: None,
            ring: None,
        }
    }

    /// A stable id (defaults to one derived from the status).
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Override the label (`label`).
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// An outer 2px ring in the surface color, for a dot that sits on a
    /// glyph (`.t3-envrow-icon .t3-dot { box-shadow: 0 0 0 2px var(--layer-01) }`).
    pub fn cutout(mut self, surface: Hsla) -> Self {
        self.ring = Some(surface);
        self
    }
}

impl RenderOnce for ConnectionStatusDot {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let id = self
            .id
            .unwrap_or_else(|| ElementId::Name(format!("t3-dot-{}", self.status.name()).into()));
        let label = self.label.unwrap_or_else(|| self.status.label().into());
        let (fill, inset) = self.status.colors();
        let ring = |color: Hsla, spread: f32, inset: bool| gpui_kit::BoxShadow {
            color,
            offset: gpui_kit::point(px(0.), px(0.)),
            blur_radius: px(0.),
            spread_radius: px(spread),
            inset,
        };
        let mut shadows = Vec::new();
        if let Some(c) = inset {
            shadows.push(ring(c, DOT_RING, true));
        }
        if let Some(surface) = self.ring {
            shadows.push(ring(surface, 2., false));
        }
        let dot = div()
            .flex_none()
            .size(px(DOT_SIZE))
            .rounded_full()
            .bg(fill)
            .shadow(shadows);
        let dot = if self.status == ConnectionStatus::Connecting {
            // `.t3-dot--connecting { animation: t3-pulse 1.2s ease-in-out infinite }`.
            pulse(dot, (id.clone(), "pulse"), EASE_IN_OUT).into_any_element()
        } else {
            dot.into_any_element()
        };
        div()
            .id(id)
            .flex()
            .flex_none()
            .tooltip(title_tooltip(label))
            .child(dot)
    }
}

/// `EnvironmentBadge` `size`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BadgeSize {
    /// `sm`: 1.25rem tall, 4px inline padding.
    Sm,
    /// `md` (the default): 1.5rem tall, 8px inline padding.
    #[default]
    Md,
}

impl BadgeSize {
    /// Every size.
    pub const ALL: [BadgeSize; 2] = [Self::Sm, Self::Md];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
        }
    }

    /// (height, inline padding) in px: `.t3-envbadge { height: 1.5rem;
    /// padding: 0 var(--spacing-03) }`, `--sm { height: 1.25rem; padding: 0
    /// var(--spacing-02) }`.
    pub const fn geometry(self) -> (f32, f32) {
        match self {
            Self::Sm => (20., 4.),
            Self::Md => (24., 8.),
        }
    }
}

/// A machine badge (`EnvironmentBadge`): machine glyph (14px), name and
/// connection dot on a square `layer-accent-01` chip, `label-01` text in
/// `text-primary`, 4px gaps.
#[derive(IntoElement)]
pub struct EnvironmentBadge {
    name: SharedString,
    kind: MachineKind,
    status: ConnectionStatus,
    size: BadgeSize,
    id: Option<ElementId>,
}

impl EnvironmentBadge {
    /// The badge for machine `name`.
    pub fn new(name: impl Into<SharedString>) -> Self {
        Self {
            name: name.into(),
            kind: MachineKind::Laptop,
            status: ConnectionStatus::Connected,
            size: BadgeSize::Md,
            id: None,
        }
    }

    /// The machine kind (`kind`).
    pub fn kind(mut self, kind: MachineKind) -> Self {
        self.kind = kind;
        self
    }

    /// The connection status (`status`).
    pub fn status(mut self, status: ConnectionStatus) -> Self {
        self.status = status;
        self
    }

    /// The size (`size`).
    pub fn size(mut self, size: BadgeSize) -> Self {
        self.size = size;
        self
    }

    /// A stable id for the dot (defaults to one derived from the name).
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }
}

impl RenderOnce for EnvironmentBadge {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let (height, pad) = self.size.geometry();
        let id = self
            .id
            .unwrap_or_else(|| ElementId::Name(format!("t3-envbadge-{}", self.name).into()));
        div()
            .flex()
            .flex_none()
            .items_center()
            .gap(px(4.))
            .h(px(height))
            .px(px(pad))
            .min_w_0()
            .overflow_hidden()
            .bg(color(ColorRole::LayerAccent01))
            .type_style(TypeStyle::Label01)
            .text_color(color(ColorRole::TextPrimary))
            .whitespace_nowrap()
            .child(EnvironmentMachineIcon::new(self.kind).size(14.))
            .child(div().min_w_0().truncate().child(self.name))
            .child(ConnectionStatusDot::new(self.status).id(id))
    }
}

// ── StatusPill ──────────────────────────────────────────────────────────

/// (background, text, dot) of a status pill tone: `.t3-pill--interactive`
/// blue tag, `--warning` `support-warning` with `#161616` ink and dot,
/// `--success` green tag, `--error` red tag, `--neutral` gray tag; the dot
/// is `currentColor` otherwise.
pub fn pill_colors(tone: Tone) -> (Hsla, Hsla, Hsla) {
    let (bg, fg) = match tone {
        Tone::Interactive => (ColorRole::TagBackgroundBlue, ColorRole::TagColorBlue),
        Tone::Success => (ColorRole::TagBackgroundGreen, ColorRole::TagColorGreen),
        Tone::Error => (ColorRole::TagBackgroundRed, ColorRole::TagColorRed),
        Tone::Neutral => (ColorRole::TagBackgroundGray, ColorRole::TagColorGray),
        Tone::Warning => {
            // `.t3-pill--warning { color: #161616 }` in every theme.
            let ink = super::base::ink();
            return (color(ColorRole::SupportWarning), ink, ink);
        }
    };
    let fg = color(fg);
    (color(bg), fg, fg)
}

/// `.t3-pill { height: 1.25rem; padding: 0 var(--spacing-03); gap: var(--spacing-02) }`.
pub const PILL_HEIGHT: f32 = 20.;
/// `.t3-pill-dot { width: 6px; height: 6px }`.
pub const PILL_DOT: f32 = 6.;

/// A status pill (`StatusPill`): dot + label on a 16px-radius pill in the
/// status tone's tag colors; the working dot pulses.
#[derive(IntoElement)]
pub struct StatusPill {
    id: Option<ElementId>,
    status: Status,
    label: Option<SharedString>,
}

impl StatusPill {
    /// The pill for `status` with its default label.
    pub fn new(status: Status) -> Self {
        Self {
            id: None,
            status,
            label: None,
        }
    }

    /// Replace the label (`children`).
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// A stable id (defaults to one derived from the status).
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }
}

impl RenderOnce for StatusPill {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let tone = self.status.tone();
        let (bg, fg, dot_color) = pill_colors(tone);
        let id = self
            .id
            .unwrap_or_else(|| ElementId::Name(format!("t3-pill-{}", self.status.name()).into()));
        let dot = div()
            .flex_none()
            .size(px(PILL_DOT))
            .rounded_full()
            .bg(dot_color);
        let dot = if tone == Tone::Interactive {
            // `.t3-pill--interactive .t3-pill-dot { animation: t3-pulse 1.2s infinite }` (ease).
            pulse(dot, (id, "pulse"), EASE).into_any_element()
        } else {
            dot.into_any_element()
        };
        div()
            .flex()
            .flex_none()
            .items_center()
            .gap(px(4.))
            .h(px(PILL_HEIGHT))
            .px(px(8.))
            .rounded(px(16.))
            .bg(bg)
            .text_color(fg)
            .type_style(TypeStyle::Label01)
            .whitespace_nowrap()
            .child(dot)
            .child(self.label.unwrap_or_else(|| self.status.label().into()))
    }
}

// ── IconMenu ────────────────────────────────────────────────────────────

/// An icon-only menu trigger (`IconMenu`, the chevron half of a split
/// button): a Carbon `IconButton` (tooltip = `label`) whose `Menu` hangs
/// from the trigger's bottom-end corner (`menuAlignment="bottom-end"`).
#[derive(IntoElement)]
pub struct IconMenu {
    id: ElementId,
    label: SharedString,
    icon: Icon,
    variant: ButtonVariant,
    size: ButtonSize,
    disabled: bool,
    items: Vec<MenuItem>,
}

impl IconMenu {
    /// A menu trigger labelled `label` (`label`, default `"More options"` in
    /// the JSX), drawing `ChevronDown`, secondary, `md`.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: Icon::carbon(CarbonIcon::ChevronDown),
            variant: ButtonVariant::Secondary,
            size: ButtonSize::Md,
            disabled: false,
            items: Vec::new(),
        }
    }

    /// The trigger glyph (`icon`).
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = icon.into();
        self
    }

    /// The trigger kind (`kind`, default `secondary`).
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// The trigger size (`size`, default `md`).
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    /// Disable the trigger (`disabled`).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Add a menu item (`children`).
    pub fn item(mut self, item: MenuItem) -> Self {
        self.items.push(item);
        self
    }

    /// Add menu items.
    pub fn items(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.items.extend(items);
        self
    }

    /// The menu size for a trigger size: `size === 'sm' ? 'sm' : 'md'`.
    pub const fn menu_size(size: ButtonSize) -> MenuSize {
        match size {
            ButtonSize::Sm => MenuSize::Sm,
            _ => MenuSize::Md,
        }
    }
}

impl RenderOnce for IconMenu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let openness = overlay::openness(&id, None, false, None, window, cx);
        let open = openness.open && !self.disabled;
        let set_open = openness.setter();
        let button = Button::new((id.clone(), "button"), self.label.clone())
            .variant(self.variant)
            .size(self.size)
            .icon(self.icon)
            .icon_only(true)
            .disabled(self.disabled)
            .tooltip(!open)
            .into_any_element();
        let disabled = self.disabled;
        let trigger = overlay::trigger_wrapper((id.clone(), "trigger"), button, {
            let set_open = set_open.clone();
            Rc::new(move |window, cx| {
                if !disabled {
                    set_open(!open, window, cx)
                }
            })
        })
        .into_any_element();
        menu::anchored_menu_aligned(
            &id,
            trigger,
            menu::MenuSpec {
                items: self.items,
                size: Self::menu_size(self.size),
                label: self.label,
                open,
                set_open,
                min_width: None,
                on_unhandled_key: None,
            },
            true,
            window,
            cx,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    fn hex(c: Hsla) -> String {
        let c = c.to_rgb();
        let b = |v: f32| (v * 255.).round() as u8;
        format!("#{:02x}{:02x}{:02x}", b(c.r), b(c.g), b(c.b))
    }

    #[test]
    fn monogram_letters_and_hash_follow_shell_jsx() {
        assert_eq!(monogram_letters("carbon-ui"), "CU");
        assert_eq!(monogram_letters("Marketing Site"), "MS");
        assert_eq!(monogram_letters("docs"), "D");
        assert_eq!(monogram_letters("my big project"), "MB");
        assert_eq!(monogram_letters("—"), "?");
        // `[...'ab'].reduce((a, c) => (a * 31 + c.charCodeAt(0)) >>> 0, 7)`
        // = (7·31 + 97)·31 + 98 = 9832.
        assert_eq!(monogram_hash("ab"), 9832);
        assert_eq!(monogram_hash(""), 7);
        assert_eq!(MonoTone::for_name("ab"), MonoTone::ALL[9832 % 8]);
        // Wraps modulo 2³² like `>>> 0`.
        let long = "x".repeat(40);
        let mut a: u64 = 7;
        for _ in 0..40 {
            a = (a * 31 + 120) % (1 << 32);
        }
        assert_eq!(monogram_hash(&long) as u64, a);
    }

    #[test]
    fn identity_sizes_follow_t3_css() {
        assert_eq!(IdentitySize::ALL.map(|s| s.px()), [16., 20., 24., 40.]);
        assert_eq!(
            IdentitySize::ALL.map(|s| s.monogram_font()),
            [8., 10., 11., 16.]
        );
        assert_eq!(BadgeSize::Md.geometry(), (24., 8.));
        assert_eq!(BadgeSize::Sm.geometry(), (20., 4.));
    }

    #[test]
    fn provider_glyphs_key_on_the_name_prefix() {
        crate::theme::tests_set(CarbonTheme::White);
        assert_eq!(provider_glyph("Claude Code"), Some("Claude"));
        assert_eq!(provider_glyph("Codex"), Some("Codex"));
        assert_eq!(provider_glyph("Kiro"), None);
        assert_eq!(hex(provider_glyph_color("Claude")), "#c15f3c");
        assert_eq!(hex(provider_glyph_color("Gemini")), "#1a73e8");
        assert_eq!(hex(provider_glyph_color("Cursor")), "#161616");
        assert_eq!(hex(provider_glyph_color("Kiro")), "#525252");
        assert!(asset_svg("provider-Figma").unwrap().contains("evenodd"));
        assert!(asset_svg("provider-Kiro").is_none());
        assert!(gpui_material_symbols::Symbol::named(GENERIC_PROVIDER_SYMBOL).is_some());
        assert!(gpui_material_symbols::Symbol::named("dns").is_some());
        assert!(gpui_material_symbols::Symbol::named("phone_iphone").is_some());
    }

    #[test]
    fn dots_and_pills_follow_t3_css() {
        crate::theme::tests_set(CarbonTheme::White);
        let (fill, ring) = ConnectionStatus::Connected.colors();
        assert_eq!((hex(fill), ring), ("#24a148".into(), None));
        let (fill, ring) = ConnectionStatus::Offline.colors();
        assert_eq!((fill.a, ring.map(hex)), (0., Some("#525252".into())));
        assert_eq!(hex(ConnectionStatus::Error.colors().0), "#da1e28");
        assert_eq!(ConnectionStatus::Sleeping.label(), "Computer is off");

        let (bg, fg, dot) = pill_colors(Tone::Warning);
        assert_eq!(
            (hex(bg), hex(fg), hex(dot)),
            ("#f1c21b".into(), "#161616".into(), "#161616".into())
        );
        let (bg, fg, _) = pill_colors(Tone::Interactive);
        assert_eq!((hex(bg), hex(fg)), ("#d0e2ff".into(), "#0043ce".into()));
        // The warning ink stays #161616 on dark themes.
        crate::theme::tests_set(CarbonTheme::G100);
        assert_eq!(hex(pill_colors(Tone::Warning).1), "#161616");
        crate::theme::tests_set(CarbonTheme::White);
    }

    #[test]
    fn monograms_are_neutral_and_menus_follow_trigger_size() {
        crate::theme::tests_set(CarbonTheme::White);
        let (bg, fg) = ProjectMonogram::colors();
        assert_eq!((hex(bg), hex(fg)), ("#e0e0e0".into(), "#525252".into()));
        assert_eq!(IconMenu::menu_size(ButtonSize::Sm), MenuSize::Sm);
        assert_eq!(IconMenu::menu_size(ButtonSize::Lg), MenuSize::Md);
    }
}
