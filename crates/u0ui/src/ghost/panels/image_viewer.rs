//! ImageViewer: an image in the right column (`ImageViewer` in
//! `src/ghost/panels.jsx`, `.g-imgv*` / `.g-zoomval` in `src/ghost.css`).
//!
//! A `.g-codehead` (image glyph, mono name, `W×H`, zoom out, the zoom value
//! that toggles Fit ⇄ 100%, zoom in, background toggle) over a 14rem stage.
//! Zoom steps ×1.5 between 25% and 800% (`Math.max(0.25, s / 1.5)`,
//! `Math.min(8, s * 1.5)`); "Fit" contains the image in the stage. The stage
//! background cycles checkerboard → dark → light, for transparent images.
//! When zoomed past the stage, the image pans with the wheel or by dragging.
//! Without a `src` a placeholder of the image's aspect ratio is painted in
//! `color` (the reference's `.g-imgv-ph`).
use super::head::{code_head, dim, ghost_icon, grow, head_icon, head_name};
use crate::{
    ghost::primitives::parts::{focus_ring, focus_visible, keyed_focus},
    icons::CarbonIcon,
    t3::craft::{rem, sans},
    theme::{color, color_in},
    tokens::{CarbonTheme, ColorRole},
};
use gpui_kit::{
    App, Background, Bounds, CursorStyle, ElementId, Hsla, ImageSource, InteractiveElement as _,
    IntoElement, MouseButton, ObjectFit, ParentElement, Pixels, Point, RenderOnce, ScrollHandle,
    SharedString, StatefulInteractiveElement as _, Styled, StyledImage as _, Window, canvas, div,
    fill, img, point, prelude::FluentBuilder as _, px, relative, size,
};
use std::rc::Rc;

/// The zoom (`z`): `'fit'` or a scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Zoom {
    /// Contain the image in the stage (the default).
    Fit,
    /// A scale factor (1 = actual size).
    Scale(f32),
}

impl Zoom {
    /// `setZ(Math.max(0.25, (scale || 1) / 1.5))`.
    pub fn out(self) -> Self {
        Self::Scale((self.base() / 1.5).max(0.25))
    }

    /// `setZ(Math.min(8, (scale || 1) * 1.5))`.
    pub fn zoom_in(self) -> Self {
        Self::Scale((self.base() * 1.5).min(8.))
    }

    /// The zoom-value button: `setZ(z === 'fit' ? 1 : 'fit')`.
    pub fn toggle(self) -> Self {
        match self {
            Self::Fit => Self::Scale(1.),
            Self::Scale(_) => Self::Fit,
        }
    }

    /// `scale || 1`.
    fn base(self) -> f32 {
        match self {
            Self::Fit => 1.,
            Self::Scale(s) => s,
        }
    }

    /// The value label's percentage (`Math.round(scale * 100) + '%'`), or
    /// `None` for Fit.
    pub fn percent(self) -> Option<u32> {
        match self {
            Self::Fit => None,
            Self::Scale(s) => Some((s * 100.).round() as u32),
        }
    }
}

/// The stage background (`bg`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum StageBackground {
    /// `.g-imgv--checker`: a 16px checkerboard of `layer-01` and
    /// `layer-accent-01` (the default).
    #[default]
    Checker,
    /// `.g-imgv--dark { background: #161616 }`.
    Dark,
    /// `.g-imgv--light { background: #ffffff }`.
    Light,
}

impl StageBackground {
    /// `bg === 'checker' ? 'dark' : bg === 'dark' ? 'light' : 'checker'`.
    pub fn next(self) -> Self {
        match self {
            Self::Checker => Self::Dark,
            Self::Dark => Self::Light,
            Self::Light => Self::Checker,
        }
    }

    /// The solid fill (`#161616` is G100's `background`, `#ffffff` White's);
    /// `None` for the checkerboard.
    pub fn solid(self) -> Option<Hsla> {
        match self {
            Self::Checker => None,
            Self::Dark => Some(color_in(CarbonTheme::G100, ColorRole::Background)),
            Self::Light => Some(color_in(CarbonTheme::White, ColorRole::Background)),
        }
    }
}

/// `.g-imgv--checker { background: repeating-conic-gradient(layer-01 0 25%,
/// layer-accent-01 0 50%) 0 0 / 16px 16px }`: the ink of the 8px cell at
/// (`col`, `row`). The conic runs clockwise from 12 o'clock: the top-right
/// quarter of each tile is `layer-01`, the bottom-right `layer-accent-01`,
/// and the pattern repeats for the left half.
pub fn checker_cell(col: usize, row: usize) -> ColorRole {
    if (col + row).is_multiple_of(2) {
        ColorRole::LayerAccent01
    } else {
        ColorRole::Layer01
    }
}

/// `.g-imgv-stage { height: 14rem; padding: var(--spacing-05) }`.
pub const STAGE_HEIGHT: f32 = 224.;
/// The stage's padding.
pub const STAGE_PAD: f32 = 16.;

/// The image's drawn size at `zoom` for a `width`×`height` image: the
/// actual size times the scale, or, fitted, the largest size that keeps
/// the ratio inside a stage `inner_h` px tall (the placeholder's width is
/// capped at 70% of the stage by the caller).
pub fn drawn_size(zoom: Zoom, width: f32, height: f32, inner_h: f32) -> (f32, f32) {
    let ratio = if height > 0. { width / height } else { 1. };
    match zoom {
        Zoom::Scale(s) => (width * s, width * s / ratio),
        Zoom::Fit => {
            let h = height.min(inner_h);
            (h * ratio, h)
        }
    }
}

/// The viewer's user-facing strings (the reference's English defaults).
#[derive(Clone)]
pub struct ImageViewerTexts {
    /// `"Zoom out"`.
    pub zoom_out: SharedString,
    /// `"Zoom in"`.
    pub zoom_in: SharedString,
    /// The zoom value's label: `"Toggle fit / actual size"`.
    pub toggle_fit: SharedString,
    /// The value while fitted: `"Fit"`.
    pub fit: SharedString,
    /// `"Toggle background"`.
    pub background: SharedString,
    /// `"{w}×{h}"`.
    pub dimensions: Rc<dyn Fn(u32, u32) -> SharedString>,
}

impl Default for ImageViewerTexts {
    fn default() -> Self {
        Self {
            zoom_out: "Zoom out".into(),
            zoom_in: "Zoom in".into(),
            toggle_fit: "Toggle fit / actual size".into(),
            fit: "Fit".into(),
            background: "Toggle background".into(),
            dimensions: Rc::new(|w, h| format!("{w}×{h}").into()),
        }
    }
}

#[derive(Default)]
struct Drag {
    from: Option<(Point<Pixels>, Point<Pixels>)>,
}

/// An image viewer. See the module docs.
#[derive(IntoElement)]
pub struct ImageViewer {
    id: ElementId,
    src: Option<ImageSource>,
    alt: SharedString,
    name: SharedString,
    width: u32,
    height: u32,
    placeholder: Option<Background>,
    texts: ImageViewerTexts,
}

impl ImageViewer {
    /// A viewer for the image called `name`.
    pub fn new(id: impl Into<ElementId>, name: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            src: None,
            alt: SharedString::default(),
            name: name.into(),
            width: 0,
            height: 0,
            placeholder: None,
            texts: ImageViewerTexts::default(),
        }
    }

    /// The image (`src`).
    pub fn src(mut self, src: impl Into<ImageSource>) -> Self {
        self.src = Some(src.into());
        self
    }

    /// Its description (`alt`), the stage's accessible name.
    pub fn alt(mut self, alt: impl Into<SharedString>) -> Self {
        self.alt = alt.into();
        self
    }

    /// Its natural size in px (`width`, `height`).
    pub fn dimensions(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// The placeholder's paint when there is no `src` (`color`).
    pub fn color(mut self, fill: impl Into<Background>) -> Self {
        self.placeholder = Some(fill.into());
        self
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: ImageViewerTexts) -> Self {
        self.texts = texts;
        self
    }
}

/// The checkerboard, painted under the stage.
fn checkerboard() -> impl IntoElement {
    let a = color(checker_cell(0, 0));
    let b = color(checker_cell(1, 0));
    canvas(
        |_, _, _| (),
        move |bounds: Bounds<Pixels>, _, window, _| {
            let cell = px(8.);
            let cols = (bounds.size.width / cell).ceil() as usize;
            let rows = (bounds.size.height / cell).ceil() as usize;
            window.paint_quad(fill(bounds, b));
            for r in 0..rows {
                for c in 0..cols {
                    if (c + r).is_multiple_of(2) {
                        let origin = bounds.origin + point(cell * c as f32, cell * r as f32);
                        window.paint_quad(fill(Bounds::new(origin, size(cell, cell)), a));
                    }
                }
            }
        },
    )
    .absolute()
    .top_0()
    .left_0()
    .size_full()
}

/// `.g-zoomval { min-width: 3rem; height: 2rem; padding: 0
/// var(--spacing-02); font-size: 0.75rem; font-variant-numeric:
/// tabular-nums; color: var(--text-primary) }`, hover `layer-hover-01`.
fn zoom_value(
    id: ElementId,
    label: SharedString,
    name: SharedString,
    on_click: impl Fn(&mut Window, &mut App) + 'static,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let focus = keyed_focus((id.clone(), "focus"), window, cx);
    let ring = focus_visible(&focus, window);
    let hovered = window.use_keyed_state((id.clone(), "hover"), cx, |_, _| false);
    let over = *hovered.read(cx);
    sans(gpui_kit::base::Button::new(id), 12., 400)
        .track_focus(&focus)
        .accessibility_label(name)
        .relative()
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .min_w(rem(48.))
        .h(rem(32.))
        .px(rem(4.))
        .text_color(color(ColorRole::TextPrimary))
        .when(over, |b| b.bg(color(ColorRole::LayerHover01)))
        .cursor(CursorStyle::PointingHand)
        .on_hover(move |h, _, cx| {
            hovered.update(cx, |v, cx| {
                if *v != *h {
                    *v = *h;
                    cx.notify();
                }
            })
        })
        .on_click(move |_, w, cx| on_click(w, cx))
        .child(label)
        .when(ring, |b| b.child(focus_ring()))
}

impl RenderOnce for ImageViewer {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let zoom = window.use_keyed_state((id.clone(), "zoom"), cx, |_, _| Zoom::Fit);
        let bg = window.use_keyed_state((id.clone(), "bg"), cx, |_, _| StageBackground::Checker);
        let drag = window.use_keyed_state((id.clone(), "drag"), cx, |_, _| Drag::default());
        let scroll = window
            .use_keyed_state((id.clone(), "scroll"), cx, |_, _| ScrollHandle::new())
            .read(cx)
            .clone();
        let z = *zoom.read(cx);
        let stage_bg = *bg.read(cx);
        let set_zoom = |f: fn(Zoom) -> Zoom| {
            let zoom = zoom.clone();
            move |_: &mut Window, cx: &mut App| {
                zoom.update(cx, |z, cx| {
                    *z = f(*z);
                    cx.notify();
                })
            }
        };
        let value = match z.percent() {
            None => self.texts.fit.clone(),
            Some(p) => format!("{p}%").into(),
        };
        let head = code_head()
            .child(head_icon(CarbonIcon::Image))
            .child(head_name(self.name.clone(), ColorRole::TextSecondary))
            .when(self.width > 0, |h| {
                h.child(dim((self.texts.dimensions)(self.width, self.height), 12.))
            })
            .child(grow())
            .child(ghost_icon(
                (id.clone(), "out"),
                self.texts.zoom_out.clone(),
                CarbonIcon::ZoomOut,
                false,
                Some(Rc::new(set_zoom(Zoom::out))),
            ))
            .child(zoom_value(
                (id.clone(), "value").into(),
                value,
                self.texts.toggle_fit.clone(),
                set_zoom(Zoom::toggle),
                window,
                cx,
            ))
            .child(ghost_icon(
                (id.clone(), "in"),
                self.texts.zoom_in.clone(),
                CarbonIcon::ZoomIn,
                false,
                Some(Rc::new(set_zoom(Zoom::zoom_in))),
            ))
            .child(ghost_icon(
                (id.clone(), "bg"),
                self.texts.background.clone(),
                CarbonIcon::ColorPalette,
                false,
                Some(Rc::new({
                    let bg = bg.clone();
                    move |_, cx| {
                        bg.update(cx, |b, cx| {
                            *b = b.next();
                            cx.notify();
                        })
                    }
                })),
            ));

        let (w, h) = drawn_size(
            z,
            self.width.max(1) as f32,
            self.height.max(1) as f32,
            STAGE_HEIGHT - 2. * STAGE_PAD,
        );
        let picture = match &self.src {
            Some(src) => match z {
                // `{ maxWidth: '100%', maxHeight: '100%' }`.
                Zoom::Fit => img(src.clone())
                    .size_full()
                    .object_fit(ObjectFit::Contain)
                    .into_any_element(),
                // `{ width: width * scale, maxWidth: 'none' }`.
                Zoom::Scale(_) => img(src.clone())
                    .flex_none()
                    .w(rem(w))
                    .h(rem(h))
                    .into_any_element(),
            },
            // `.g-imgv-ph { width: scale ? width * scale : '70%'; aspect-ratio }`.
            None => div()
                .flex_none()
                .w(rem(w))
                .h(rem(h))
                .when(z == Zoom::Fit, |d| d.max_w(relative(0.7)))
                .when_some(self.placeholder, |d, f| d.bg(f))
                .into_any_element(),
        };
        let zoomed = z != Zoom::Fit;
        let down = drag.clone();
        let moving = drag.clone();
        let up = drag.clone();
        let pan = scroll.clone();
        let start = scroll.clone();
        let stage = div()
            .id((id.clone(), "stage"))
            .relative()
            .h(rem(STAGE_HEIGHT))
            .overflow_scroll()
            .track_scroll(&scroll)
            .when_some(stage_bg.solid(), |s, c| s.bg(c))
            .when(stage_bg == StageBackground::Checker, |s| {
                s.child(checkerboard())
            })
            .when(zoomed, |s| s.cursor(CursorStyle::OpenHand))
            .on_mouse_down(MouseButton::Left, move |e, _, cx| {
                down.update(cx, |d, _| d.from = Some((e.position, start.offset())));
            })
            .on_mouse_move(move |e, window, cx| {
                if let Some((from, offset)) = moving.read(cx).from {
                    if !e.dragging() {
                        return;
                    }
                    let max = pan.max_offset();
                    let next = offset + (e.position - from);
                    pan.set_offset(point(
                        next.x.clamp(-max.x, px(0.)),
                        next.y.clamp(-max.y, px(0.)),
                    ));
                    window.refresh();
                }
            })
            .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                up.update(cx, |d, _| d.from = None);
            })
            .child(
                div()
                    .relative()
                    .flex()
                    .items_center()
                    .justify_center()
                    .min_w_full()
                    .min_h_full()
                    .when(!zoomed, |d| d.size_full())
                    .p(rem(STAGE_PAD))
                    .child(picture),
            );

        div()
            .id(id)
            .flex()
            .flex_col()
            .min_w_0()
            .bg(color(ColorRole::Layer01))
            .child(head)
            .child(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoom_steps_follow_the_reference() {
        // Fit → in: (1) × 1.5; out: 1 / 1.5.
        assert_eq!(Zoom::Fit.zoom_in(), Zoom::Scale(1.5));
        assert_eq!(Zoom::Fit.out(), Zoom::Scale(1. / 1.5));
        // Clamped to 25%..800%.
        assert_eq!(Zoom::Scale(0.3).out(), Zoom::Scale(0.25));
        assert_eq!(Zoom::Scale(6.).zoom_in(), Zoom::Scale(8.));
        // The value button toggles Fit ⇄ 100%.
        assert_eq!(Zoom::Fit.toggle(), Zoom::Scale(1.));
        assert_eq!(Zoom::Scale(2.25).toggle(), Zoom::Fit);
        assert_eq!(Zoom::Scale(1.5).percent(), Some(150));
        assert_eq!(Zoom::Fit.percent(), None);
    }

    #[test]
    fn backgrounds_cycle_and_paint() {
        use StageBackground::*;
        assert_eq!(Checker.next(), Dark);
        assert_eq!(Dark.next(), Light);
        assert_eq!(Light.next(), Checker);
        // `.g-imgv--dark { background: #161616 }`, `--light { #ffffff }`.
        assert_eq!(Dark.solid(), Some(gpui_kit::rgb(0x161616).into()));
        assert_eq!(Light.solid(), Some(gpui_kit::rgb(0xffffff).into()));
        assert_eq!(checker_cell(0, 0), ColorRole::LayerAccent01);
        assert_eq!(checker_cell(1, 0), ColorRole::Layer01);
        assert_eq!(checker_cell(1, 1), ColorRole::LayerAccent01);
    }

    #[test]
    fn drawn_size_keeps_the_ratio() {
        // Fitted into the 192px stage interior.
        assert_eq!(drawn_size(Zoom::Fit, 1440., 900., 192.), (307.2, 192.));
        // Actual size times the scale.
        assert_eq!(
            drawn_size(Zoom::Scale(0.5), 1440., 900., 192.),
            (720., 450.)
        );
        // A small image is not blown up to fit.
        assert_eq!(drawn_size(Zoom::Fit, 100., 50., 192.), (100., 50.));
    }
}
