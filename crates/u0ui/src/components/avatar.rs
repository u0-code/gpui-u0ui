//! Avatar: a round user picture or initials (`.cu-avatar*` in `custom.css`;
//! Carbon's UI shell user avatar style, no core component).
//!
//! - **Appearance**: `.cu-avatar { width: 2rem; height: 2rem; border-radius:
//!   var(--border-radius-max); overflow: hidden }`; sm 1.5rem, lg 3rem, xl
//!   4rem. The fallback fills it with initials (`.cu-avatar-fallback {
//!   font-size: 0.75rem; font-weight: 600 }`) in a `tag-background-*` /
//!   `tag-color-*` pair. The fallback's own `font-size: 0.75rem` wins over the
//!   avatar's size font (`.cu-avatar--lg { font-size: 1rem }`), so initials are
//!   12px at every size, as in the reference.
//! - **Image**: `AvatarImage` covers the circle (`object-fit: cover`); while it
//!   loads or when it fails, the fallback shows (`onError → setOk(false)`).
//! - **Group**: `.cu-avatar-group > .cu-avatar { box-shadow: 0 0 0 2px
//!   var(--background) }` and `+ .cu-avatar { margin-left: -0.5rem }`.
//! - **Behavior**: none. Kit's `Avatar` derives colors from the name and draws
//!   its own sizes; Carbon's palette is the tag pairs, so this is drawn here.
use crate::{
    theme::color,
    tokens::{ColorRole, FontFamilyKey},
    typography::font,
};
use gpui_kit::{
    AnyElement, App, BoxShadow, Hsla, ImageSource, IntoElement, ObjectFit, ParentElement,
    RenderOnce, SharedString, Styled, StyledImage as _, Window, div, img,
    prelude::FluentBuilder as _, px, rems,
};

/// `index.d.ts` `AvatarProps.size`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AvatarSize {
    /// 1.5rem.
    Sm,
    /// 2rem, the default.
    #[default]
    Md,
    /// 3rem.
    Lg,
    /// 4rem.
    Xl,
}

impl AvatarSize {
    /// Every size.
    pub const ALL: [AvatarSize; 4] = [Self::Sm, Self::Md, Self::Lg, Self::Xl];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
            Self::Xl => "xl",
        }
    }

    /// Diameter in px (`.cu-avatar` 2rem; `--sm` 1.5rem, `--lg` 3rem, `--xl` 4rem).
    pub const fn diameter(self) -> f32 {
        match self {
            Self::Sm => 24.,
            Self::Md => 32.,
            Self::Lg => 48.,
            Self::Xl => 64.,
        }
    }
}

/// `index.d.ts` `AvatarFallback.color`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AvatarColor {
    /// `blue`, the default.
    #[default]
    Blue,
    /// `teal`.
    Teal,
    /// `purple`.
    Purple,
    /// `magenta`.
    Magenta,
    /// `gray`.
    Gray,
}

impl AvatarColor {
    /// Every color.
    pub const ALL: [AvatarColor; 5] = [
        Self::Blue,
        Self::Teal,
        Self::Purple,
        Self::Magenta,
        Self::Gray,
    ];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Blue => "blue",
            Self::Teal => "teal",
            Self::Purple => "purple",
            Self::Magenta => "magenta",
            Self::Gray => "gray",
        }
    }

    /// (background, text): `.cu-avatar--<c> { background: var(--tag-background-<c>);
    /// color: var(--tag-color-<c>) }`.
    pub fn colors(self) -> (Hsla, Hsla) {
        use ColorRole::*;
        let (bg, fg) = match self {
            Self::Blue => (TagBackgroundBlue, TagColorBlue),
            Self::Teal => (TagBackgroundTeal, TagColorTeal),
            Self::Purple => (TagBackgroundPurple, TagColorPurple),
            Self::Magenta => (TagBackgroundMagenta, TagColorMagenta),
            Self::Gray => (TagBackgroundGray, TagColorGray),
        };
        (color(bg), color(fg))
    }
}

/// `.cu-avatar-fallback { font-size: 0.75rem }`.
pub const FALLBACK_FONT_SIZE: f32 = 12.;
/// `.cu-avatar-group > .cu-avatar { box-shadow: 0 0 0 2px var(--background) }`.
pub const GROUP_RING: f32 = 2.;
/// `.cu-avatar-group > .cu-avatar + .cu-avatar { margin-left: -0.5rem }`.
pub const GROUP_OVERLAP: f32 = 8.;

/// `AvatarFallback`: initials on a tag color pair.
#[derive(Clone)]
pub struct AvatarFallback {
    text: SharedString,
    color: AvatarColor,
}

impl AvatarFallback {
    /// Initials (or any short text, e.g. `+3`).
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            color: AvatarColor::Blue,
        }
    }

    /// The color pair (default blue).
    pub fn color(mut self, color: AvatarColor) -> Self {
        self.color = color;
        self
    }

    fn element(&self) -> AnyElement {
        let (bg, fg) = self.color.colors();
        // GPUI clips children to rectangles, so every layer rounds itself.
        div()
            .size_full()
            .rounded_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(bg)
            .text_color(fg)
            .font(font(FontFamilyKey::Sans, 600))
            .text_size(rems(FALLBACK_FONT_SIZE / crate::tokens::REM_PX))
            .line_height(rems(1.))
            .child(self.text.clone())
            .into_any_element()
    }
}

/// An avatar. See the module docs.
#[derive(IntoElement, Clone, Default)]
pub struct Avatar {
    size: AvatarSize,
    image: Option<ImageSource>,
    fallback: Option<AvatarFallback>,
    ring: Option<Hsla>,
}

impl Avatar {
    /// An empty `md` avatar.
    pub fn new() -> Self {
        Self::default()
    }

    /// The size.
    pub fn size(mut self, size: AvatarSize) -> Self {
        self.size = size;
        self
    }

    /// `AvatarImage`: a picture source (path, URL or image data).
    pub fn image(mut self, source: impl Into<ImageSource>) -> Self {
        self.image = Some(source.into());
        self
    }

    /// `AvatarFallback`: shown without an image, while it loads, or if it fails.
    pub fn fallback(mut self, fallback: AvatarFallback) -> Self {
        self.fallback = Some(fallback);
        self
    }

    /// The group ring (set by [`AvatarGroup`]).
    fn ring(mut self, ring: Hsla) -> Self {
        self.ring = Some(ring);
        self
    }
}

impl RenderOnce for Avatar {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let d = rems(self.size.diameter() / crate::tokens::REM_PX);
        let fallback = self.fallback.clone();
        let content: Option<AnyElement> = match (self.image, fallback) {
            (Some(source), fallback) => {
                let loading = fallback.clone();
                Some(
                    img(source)
                        .size_full()
                        .rounded_full()
                        .object_fit(ObjectFit::Cover)
                        .with_fallback(move || {
                            fallback
                                .as_ref()
                                .map(|f| f.element())
                                .unwrap_or_else(|| div().into_any_element())
                        })
                        .with_loading(move || {
                            loading
                                .as_ref()
                                .map(|f| f.element())
                                .unwrap_or_else(|| div().into_any_element())
                        })
                        .into_any_element(),
                )
            }
            (None, Some(fallback)) => Some(fallback.element()),
            (None, None) => None,
        };
        div()
            .flex_none()
            .size(d)
            .rounded_full()
            .overflow_hidden()
            .when_some(self.ring, |this, ring| {
                this.shadow(vec![
                    BoxShadow::new(px(0.), px(0.), ring).spread_radius(px(GROUP_RING)),
                ])
            })
            .children(content)
    }
}

/// `AvatarGroup`: overlapping avatars, each ringed in `background`.
#[derive(IntoElement, Default)]
pub struct AvatarGroup {
    avatars: Vec<Avatar>,
}

impl AvatarGroup {
    /// An empty group.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an avatar.
    pub fn avatar(mut self, avatar: Avatar) -> Self {
        self.avatars.push(avatar);
        self
    }

    /// Add avatars.
    pub fn avatars(mut self, avatars: impl IntoIterator<Item = Avatar>) -> Self {
        self.avatars.extend(avatars);
        self
    }
}

impl RenderOnce for AvatarGroup {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let ring = color(ColorRole::Background);
        div()
            .flex()
            .flex_row()
            .flex_none()
            .children(self.avatars.into_iter().enumerate().map(|(i, avatar)| {
                div()
                    .when(i > 0, |this| {
                        this.ml(rems(-GROUP_OVERLAP / crate::tokens::REM_PX))
                    })
                    .child(avatar.ring(ring))
            }))
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
    fn sizes_and_colors_follow_custom_css() {
        crate::theme::tests_set(CarbonTheme::White);
        assert_eq!(AvatarSize::ALL.map(|s| s.diameter()), [24., 32., 48., 64.]);
        // `.cu-avatar--blue { background: tag-background-blue #d0e2ff; color: tag-color-blue #0043ce }`.
        let (bg, fg) = AvatarColor::Blue.colors();
        assert_eq!((hex(bg), hex(fg)), ("#d0e2ff".into(), "#0043ce".into()));
        let (bg, _) = AvatarColor::Gray.colors();
        assert_eq!(hex(bg), "#e0e0e0");
        assert_eq!(
            (FALLBACK_FONT_SIZE, GROUP_RING, GROUP_OVERLAP),
            (12., 2., 8.)
        );
    }
}
