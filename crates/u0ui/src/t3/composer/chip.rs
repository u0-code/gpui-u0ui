//! [`ContextChip`] (`.t3-chip*`, "Context chips" in `t3.css`): an inline
//! reference (file, image, mention, PR, …) in the kind's `tag-color-*` ink,
//! or a `bubble` card for the attachment strip; `unresolved` strikes the
//! label, `invalid` turns it `text-error`. Inside the composer's context row
//! ([`ContextChip::in_composer`]) every chip is the same 32px bordered square
//! chip (`.t3-composer-ctx .t3-chip--inline`, gemini-G).
//!
//! Behavior: the label part and the remove × are Kit
//! [`gpui_kit::base::Button`]s (tab stops, Enter/Space, a 2px `focus` ring on
//! keyboard focus only). No motion: the reference has no transitions here.
use crate::t3::press::{self, Handler, inset_frame, rem};
use crate::{
    icons::CarbonIcon,
    theme::color,
    tokens::{ColorRole, FontFamilyKey},
    typography::font,
};
use gpui_kit::{
    AnyElement, App, CursorStyle, ElementId, Hsla, InteractiveElement as _, IntoElement,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _, px,
};
use std::rc::Rc;

/// `ContextChip` `kind` (`CHIP_ICON` in `composer.jsx`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum ContextChipKind {
    #[default]
    File,
    Folder,
    Image,
    Video,
    Mention,
    Terminal,
    Element,
    PreviewAnnotation,
    ReviewComment,
    PullRequest,
    PrOpen,
    PrDraft,
    PrMerged,
    PrClosed,
    Skill,
    Model,
    Thread,
    Environment,
    Citation,
    DocPdf,
    Issue,
}

impl ContextChipKind {
    /// Every kind, in `CHIP_ICON` order.
    pub const ALL: [ContextChipKind; 21] = [
        Self::File,
        Self::Image,
        Self::Video,
        Self::Mention,
        Self::Terminal,
        Self::Element,
        Self::PreviewAnnotation,
        Self::ReviewComment,
        Self::PullRequest,
        Self::PrOpen,
        Self::PrDraft,
        Self::PrMerged,
        Self::PrClosed,
        Self::Skill,
        Self::Model,
        Self::Thread,
        Self::Environment,
        Self::Citation,
        Self::DocPdf,
        Self::Folder,
        Self::Issue,
    ];

    /// The reference name (`kind="pr-open"`).
    pub const fn name(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Folder => "folder",
            Self::Image => "image",
            Self::Video => "video",
            Self::Mention => "mention",
            Self::Terminal => "terminal",
            Self::Element => "element",
            Self::PreviewAnnotation => "preview-annotation",
            Self::ReviewComment => "review-comment",
            Self::PullRequest => "pull-request",
            Self::PrOpen => "pr-open",
            Self::PrDraft => "pr-draft",
            Self::PrMerged => "pr-merged",
            Self::PrClosed => "pr-closed",
            Self::Skill => "skill",
            Self::Model => "model",
            Self::Thread => "thread",
            Self::Environment => "environment",
            Self::Citation => "citation",
            Self::DocPdf => "doc-pdf",
            Self::Issue => "issue",
        }
    }

    /// The kind for a reference name; unknown names are `file`
    /// (`CHIP_ICON[kind] || I.Document`).
    pub fn from_name(name: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|k| k.name() == name)
            .unwrap_or_default()
    }

    /// `CHIP_ICON[kind]`.
    pub const fn icon(self) -> CarbonIcon {
        match self {
            Self::File => CarbonIcon::Document,
            Self::Image => CarbonIcon::Image,
            Self::Video => CarbonIcon::Video,
            Self::Mention => CarbonIcon::User,
            Self::Terminal => CarbonIcon::Terminal,
            Self::Element => CarbonIcon::Code,
            Self::PreviewAnnotation => CarbonIcon::Earth,
            Self::ReviewComment | Self::Thread => CarbonIcon::Chat,
            Self::PullRequest | Self::PrOpen | Self::PrDraft | Self::PrClosed => {
                CarbonIcon::PullRequest
            }
            Self::PrMerged => CarbonIcon::Merge,
            Self::Skill => CarbonIcon::MagicWand,
            Self::Model => CarbonIcon::Model,
            Self::Environment => CarbonIcon::Laptop,
            Self::Citation => CarbonIcon::Quotes,
            Self::DocPdf => CarbonIcon::DocumentPdf,
            Self::Folder => CarbonIcon::Folder,
            Self::Issue => CarbonIcon::Ticket,
        }
    }

    /// The kind's `--t3-ink` (`.t3-chip--<kind>`); `issue` keeps the
    /// `.t3-chip` default `link-primary`.
    pub const fn ink(self) -> ColorRole {
        match self {
            Self::File | Self::Folder => ColorRole::TagColorBlue,
            Self::Image | Self::Video => ColorRole::TagColorMagenta,
            Self::Mention | Self::Thread | Self::PrMerged => ColorRole::TagColorPurple,
            Self::Terminal | Self::Environment => ColorRole::TagColorGray,
            Self::Element | Self::PreviewAnnotation | Self::Model => ColorRole::TagColorCyan,
            Self::ReviewComment | Self::Citation => ColorRole::TagColorWarmGray,
            Self::PullRequest | Self::PrOpen => ColorRole::TagColorGreen,
            Self::PrDraft => ColorRole::TextSecondary,
            Self::PrClosed => ColorRole::TagColorRed,
            Self::Skill => ColorRole::TagColorTeal,
            Self::DocPdf => ColorRole::TextPrimary,
            Self::Issue => ColorRole::LinkPrimary,
        }
    }
}

/// `ContextChip` `state`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ContextChipState {
    /// Resolved (no `state`).
    #[default]
    Normal,
    /// `unresolved`: `text-secondary` ink, label struck through.
    Unresolved,
    /// `invalid`: `text-error` ink.
    Invalid,
}

impl ContextChipState {
    /// Every state.
    pub const ALL: [ContextChipState; 3] = [Self::Normal, Self::Unresolved, Self::Invalid];

    /// The reference name (`""` for none).
    pub const fn name(self) -> &'static str {
        match self {
            Self::Normal => "",
            Self::Unresolved => "unresolved",
            Self::Invalid => "invalid",
        }
    }
}

/// `ContextChip` `shape`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ContextChipShape {
    /// Inline text-like reference (default).
    #[default]
    Inline,
    /// A 14rem card on `layer-02` (attachment strip).
    Bubble,
}

impl ContextChipShape {
    /// Both shapes.
    pub const ALL: [ContextChipShape; 2] = [Self::Inline, Self::Bubble];

    /// The reference name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Inline => "inline",
            Self::Bubble => "bubble",
        }
    }
}

/// The chip's resolved colors.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChipLook {
    /// `--t3-ink`: the kind color after the state override.
    pub ink: Hsla,
    /// The icon fill.
    pub icon: Hsla,
    /// The label color.
    pub label: Hsla,
    /// The remove × color.
    pub remove: Hsla,
    /// The bubble / composer chip fill, `None` when bare.
    pub fill: Option<Hsla>,
    /// The bubble / composer chip hairline.
    pub frame: Option<Hsla>,
}

impl ChipLook {
    /// Colors for `kind × state × shape`, inside the composer or not.
    pub fn of(
        kind: ContextChipKind,
        state: ContextChipState,
        shape: ContextChipShape,
        in_composer: bool,
    ) -> Self {
        // `.t3-chip--unresolved { --t3-ink: text-secondary }`, `.t3-chip--invalid { --t3-ink: text-error }`
        // come after the kind rules, so they win.
        let ink = color(match state {
            ContextChipState::Unresolved => ColorRole::TextSecondary,
            ContextChipState::Invalid => ColorRole::TextError,
            ContextChipState::Normal => kind.ink(),
        });
        let boxed = shape == ContextChipShape::Bubble || in_composer;
        match shape {
            // `.t3-chip--bubble { color: text-primary }`, `.t3-chip--bubble svg { fill: var(--t3-ink) }`
            // (after `.t3-chip--doc-pdf svg`, so a PDF bubble icon is text-primary too),
            // `.t3-chip--bubble .t3-chip-x svg { fill: currentColor }` (d-fix).
            ContextChipShape::Bubble => Self {
                ink,
                icon: ink,
                label: color(ColorRole::TextPrimary),
                remove: color(if in_composer {
                    ColorRole::IconSecondary
                } else {
                    ColorRole::TextPrimary
                }),
                fill: Some(color(ColorRole::Layer02)),
                frame: Some(color(ColorRole::BorderSubtle01)),
            },
            ContextChipShape::Inline => Self {
                ink,
                // `.t3-chip svg { fill: currentColor }`; `.t3-chip--doc-pdf svg { fill: support-error }`.
                icon: if kind == ContextChipKind::DocPdf {
                    color(ColorRole::SupportError)
                } else {
                    ink
                },
                // `.t3-composer-ctx .t3-chip--inline .t3-chip-label { color: text-primary }` (gemini-G).
                label: if in_composer {
                    color(ColorRole::TextPrimary)
                } else {
                    ink
                },
                // `.t3-composer-ctx .t3-chip-x { color: icon-secondary }`, else `color: inherit`.
                remove: if in_composer {
                    color(ColorRole::IconSecondary)
                } else {
                    ink
                },
                fill: boxed.then(|| color(ColorRole::Layer02)),
                frame: boxed.then(|| color(ColorRole::BorderSubtle01)),
            },
        }
    }
}

/// Chip geometry in px.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChipGeometry {
    /// Fixed width (`.t3-chip--bubble { width: 14rem }`), else content width.
    pub width: Option<f32>,
    /// Max width (`max-width: 14rem` in the composer).
    pub max_width: Option<f32>,
    /// Fixed height (32 in the composer), else the line box.
    pub height: Option<f32>,
    /// (top/bottom, left, right) padding.
    pub padding: (f32, f32, f32),
    /// Gap between icon, label and meta.
    pub gap: f32,
    /// The remove ×'s padding around its 12px glyph.
    pub remove_padding: f32,
    /// The remove ×'s corner radius.
    pub remove_radius: f32,
    /// Label weight: 600 inline, 400 boxed.
    pub weight: u16,
}

impl ChipGeometry {
    /// Geometry for `shape`, inside the composer or not.
    pub const fn of(shape: ContextChipShape, in_composer: bool) -> Self {
        // `.t3-chip-x { padding: 2px; border-radius: border-radius-02 }`;
        // `.t3-composer-ctx .t3-chip-x { padding: 4px; border-radius: 0 }`.
        let (remove_padding, remove_radius) = if in_composer { (4., 0.) } else { (2., 2.) };
        match (shape, in_composer) {
            // `.t3-chip--bubble { width: 14rem; gap: spacing-03; padding: spacing-03 spacing-04; font-weight: 400 }`.
            (ContextChipShape::Bubble, false) => Self {
                width: Some(224.),
                max_width: None,
                height: None,
                padding: (8., 12., 12.),
                gap: 8.,
                remove_padding,
                remove_radius,
                weight: 400,
            },
            // `.t3-composer-ctx .t3-chip--bubble { width: auto; max-width: 14rem; height: 2rem; padding: 0 spacing-03 }`.
            (ContextChipShape::Bubble, true) => Self {
                width: None,
                max_width: Some(224.),
                height: Some(32.),
                padding: (0., 8., 8.),
                gap: 8.,
                remove_padding,
                remove_radius,
                weight: 400,
            },
            // `.t3-chip { gap: spacing-02; font-weight: 600; line-height: 1.25rem }`.
            (ContextChipShape::Inline, false) => Self {
                width: None,
                max_width: None,
                height: None,
                padding: (0., 0., 0.),
                gap: 4.,
                remove_padding,
                remove_radius,
                weight: 600,
            },
            // `.t3-composer-ctx .t3-chip--inline { height: 2rem; padding: 0 spacing-02 0 spacing-03; gap: spacing-02; font-weight: 400 }`.
            (ContextChipShape::Inline, true) => Self {
                width: None,
                max_width: None,
                height: Some(32.),
                padding: (0., 8., 4.),
                gap: 4.,
                remove_padding,
                remove_radius,
                weight: 400,
            },
        }
    }
}

/// An inline context reference. See the module docs.
#[derive(IntoElement)]
pub struct ContextChip {
    id: ElementId,
    kind: ContextChipKind,
    label: SharedString,
    meta: Option<SharedString>,
    state: ContextChipState,
    shape: ContextChipShape,
    in_composer: bool,
    remove_label: Option<SharedString>,
    on_click: Option<Handler>,
    on_remove: Option<Handler>,
}

impl ContextChip {
    /// A `kind` chip showing `label`.
    pub fn new(
        id: impl Into<ElementId>,
        kind: ContextChipKind,
        label: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            kind,
            label: label.into(),
            meta: None,
            state: ContextChipState::Normal,
            shape: ContextChipShape::Inline,
            in_composer: false,
            remove_label: None,
            on_click: None,
            on_remove: None,
        }
    }

    /// `meta`: a quiet 12px note after the label ("60 KB").
    pub fn meta(mut self, meta: impl Into<SharedString>) -> Self {
        self.meta = Some(meta.into());
        self
    }

    /// `state`.
    pub fn state(mut self, state: ContextChipState) -> Self {
        self.state = state;
        self
    }

    /// `shape`.
    pub fn shape(mut self, shape: ContextChipShape) -> Self {
        self.shape = shape;
        self
    }

    /// Drawn inside the composer's context row (`.t3-composer-ctx`).
    pub fn in_composer(mut self, in_composer: bool) -> Self {
        self.in_composer = in_composer;
        self
    }

    /// `onClick`: the label part becomes a button (underlined on hover).
    pub fn on_click(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }

    /// `onRemove`: adds the remove ×.
    pub fn on_remove(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_remove = Some(Rc::new(f));
        self
    }

    /// The remove ×'s accessible name (default `Remove <label>`).
    pub fn remove_label(mut self, label: impl Into<SharedString>) -> Self {
        self.remove_label = Some(label.into());
        self
    }
}

impl RenderOnce for ContextChip {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let look = ChipLook::of(self.kind, self.state, self.shape, self.in_composer);
        let geo = ChipGeometry::of(self.shape, self.in_composer);
        let bubble = self.shape == ContextChipShape::Bubble;
        let id = self.id.clone();
        let (main_hovered, main_hover) = press::hover((id.clone(), "main-hover"), window, cx);
        let label = div()
            .min_w_0()
            .overflow_hidden()
            .whitespace_nowrap()
            .text_ellipsis()
            .text_color(look.label)
            .when(bubble, |this| this.flex_1())
            .when(self.state == ContextChipState::Unresolved, |this| {
                this.line_through()
            })
            // `button.t3-chip-main:hover .t3-chip-label { text-decoration: underline }`.
            .when(self.on_click.is_some() && main_hovered, |this| {
                this.underline()
            })
            .child(self.label.clone());
        let mut parts: Vec<AnyElement> = vec![
            press::icon(self.kind.icon(), 16., look.icon).into_any_element(),
            label.into_any_element(),
        ];
        if let Some(meta) = self.meta.clone() {
            // `.t3-chip-meta { font-weight: 400; color: text-secondary; font-size: 0.75rem }`.
            parts.push(
                div()
                    .flex_none()
                    .font_weight(gpui_kit::FontWeight::NORMAL)
                    .text_size(rem(12.))
                    .text_color(color(ColorRole::TextSecondary))
                    .child(meta)
                    .into_any_element(),
            );
        }
        fn main_box<E: Styled + ParentElement>(this: E, gap: f32, bubble: bool) -> E {
            let this = this
                .flex()
                .flex_row()
                .items_center()
                .min_w_0()
                .gap(rem(gap));
            // `.t3-chip--bubble .t3-chip-main { flex: 1 }`.
            if bubble { this.flex_1() } else { this }
        }
        let main: AnyElement = match self.on_click.clone() {
            Some(handler) => {
                let (focus, ring) = press::focus((id.clone(), "main-focus"), window, cx);
                main_box(
                    gpui_kit::base::Button::new((id.clone(), "main"))
                        .track_focus(&focus)
                        .accessibility_label(self.label.clone())
                        .relative()
                        .cursor(CursorStyle::PointingHand)
                        .on_hover(press::track_hover(main_hover))
                        .on_click(move |_, window, cx| handler(window, cx)),
                    geo.gap,
                    bubble,
                )
                .children(parts)
                .when(ring, |this| this.child(press::ring()))
                .into_any_element()
            }
            None => main_box(div(), geo.gap, bubble)
                .children(parts)
                .into_any_element(),
        };
        let remove = self.on_remove.clone().map(|handler| {
            let (focus, ring) = press::focus((id.clone(), "x-focus"), window, cx);
            let (hovered, hover) = press::hover((id.clone(), "x-hover"), window, cx);
            let name = self
                .remove_label
                .clone()
                .unwrap_or_else(|| format!("Remove {}", self.label).into());
            gpui_kit::base::Button::new((id.clone(), "remove"))
                .track_focus(&focus)
                .accessibility_label(name)
                .relative()
                .flex()
                .flex_none()
                .p(rem(geo.remove_padding))
                .rounded(px(geo.remove_radius))
                .cursor(CursorStyle::PointingHand)
                // `.t3-chip-x:hover { background: var(--background-hover) }`.
                .when(hovered, |this| this.bg(color(ColorRole::BackgroundHover)))
                .on_hover(press::track_hover(hover))
                .on_click({
                    // Removing the chip keeps the Tab starting point there.
                    let focus = focus.clone();
                    move |_, window, cx| {
                        crate::u0::primitives::parts::vacate_focus(&focus, window, cx);
                        handler(window, cx)
                    }
                })
                .child(press::icon(CarbonIcon::Close, 12., look.remove))
                .when(ring, |this| this.child(press::ring()))
        });
        let (py, pl, pr) = geo.padding;
        div()
            .id(id)
            .flex()
            .flex_row()
            .flex_none()
            .items_center()
            .max_w_full()
            .gap(rem(geo.gap))
            .font(font(FontFamilyKey::Sans, geo.weight))
            // `font-size: 0.875rem; line-height: 1.25rem`.
            .text_size(rem(14.))
            .line_height(rem(20.))
            .text_color(look.ink)
            .when_some(geo.width, |this, w| this.w(rem(w)))
            .when_some(geo.max_width, |this, w| this.max_w(rem(w)))
            .when_some(geo.height, |this, h| this.h(rem(h)))
            .pt(rem(py))
            .pb(rem(py))
            .pl(rem(pl))
            .pr(rem(pr))
            .when_some(look.fill, |this, fill| this.bg(fill))
            .when_some(look.frame, |this, frame| this.shadow(inset_frame(frame)))
            .child(main)
            .children(remove)
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
    fn kinds_take_their_tag_ink() {
        crate::theme::tests_set(CarbonTheme::White);
        let ink =
            |k| hex(ChipLook::of(k, ContextChipState::Normal, ContextChipShape::Inline, false).ink);
        // `.t3-chip--file { --t3-ink: var(--tag-color-blue) }` (#0043ce in White).
        assert_eq!(ink(ContextChipKind::File), "#0043ce");
        // `.t3-chip--pr-closed { --t3-ink: var(--tag-color-red) }`.
        assert_eq!(ink(ContextChipKind::PrClosed), "#a2191f");
        // `.t3-chip--pr-draft { --t3-ink: var(--text-secondary) }`.
        assert_eq!(ink(ContextChipKind::PrDraft), "#525252");
        // `issue` has no rule: `.t3-chip { --t3-ink: var(--link-primary) }`.
        assert_eq!(ink(ContextChipKind::Issue), "#0f62fe");
        assert_eq!(
            ContextChipKind::from_name("pr-merged"),
            ContextChipKind::PrMerged
        );
        assert_eq!(ContextChipKind::from_name("nope"), ContextChipKind::File);
    }

    #[test]
    fn states_override_the_kind_ink() {
        crate::theme::tests_set(CarbonTheme::White);
        let u = ChipLook::of(
            ContextChipKind::File,
            ContextChipState::Unresolved,
            ContextChipShape::Inline,
            false,
        );
        assert_eq!(hex(u.label), "#525252");
        let i = ChipLook::of(
            ContextChipKind::File,
            ContextChipState::Invalid,
            ContextChipShape::Inline,
            false,
        );
        // `.t3-chip--invalid { --t3-ink: var(--text-error) }`.
        assert_eq!(hex(i.label), "#da1e28");
    }

    #[test]
    fn pdf_icon_is_red_inline_and_ink_in_a_bubble() {
        crate::theme::tests_set(CarbonTheme::White);
        let inline = ChipLook::of(
            ContextChipKind::DocPdf,
            ContextChipState::Normal,
            ContextChipShape::Inline,
            false,
        );
        assert_eq!(hex(inline.icon), "#da1e28");
        let bubble = ChipLook::of(
            ContextChipKind::DocPdf,
            ContextChipState::Normal,
            ContextChipShape::Bubble,
            false,
        );
        // `.t3-chip--bubble svg { fill: var(--t3-ink) }` comes later.
        assert_eq!(hex(bubble.icon), "#161616");
        assert_eq!(hex(bubble.fill.unwrap()), "#ffffff");
    }

    #[test]
    fn composer_chips_are_32px_boxes() {
        let inline = ChipGeometry::of(ContextChipShape::Inline, true);
        assert_eq!(inline.height, Some(32.));
        assert_eq!(inline.padding, (0., 8., 4.));
        assert_eq!(inline.remove_padding, 4.);
        assert_eq!(inline.weight, 400);
        let bubble = ChipGeometry::of(ContextChipShape::Bubble, false);
        assert_eq!(bubble.width, Some(224.));
        assert_eq!(bubble.padding, (8., 12., 12.));
        assert_eq!(
            ChipGeometry::of(ContextChipShape::Inline, false).weight,
            600
        );
    }
}
