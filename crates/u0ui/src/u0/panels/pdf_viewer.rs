//! PdfViewer: the frame of a PDF viewer (`PdfViewer` in
//! `src/ghost/panels.jsx`, `.g-pdf*` in `src/ghost.css`): a `.g-codehead`
//! with previous / next page, a page-number field (`n / N`), zoom out, the
//! zoom value and zoom in; under it a 4.5rem column of page thumbnails and
//! the stage showing the current page on `layer-accent-01`.
//!
//! **Rendering is the host's job.** Like the reference (which hands pdf.js
//! canvases in through `renderPage(n)`), the viewer draws pages the caller
//! renders: [`PdfViewer::render_page`] returns any element for page `n`,
//! typically an `img` of a rasterised page. There is no pure-Rust PDF
//! rasteriser practical to bundle (pdfium is C++, the pure-Rust ones are
//! incomplete and heavy), so without `render_page` the viewer paints the
//! reference's placeholder sheet ("Page n" and grey text bars).
//!
//! The page sheet keeps the A4 ratio (1 : 1.4142) and fills the stage's
//! height at 100% (the reference's `d-fix`: `.g-pdf-stage > .g-pdf-sheet {
//! align-self: flex-start; height: 100% }`); zoom (50–300% in 25% steps)
//! scales it from there and the stage scrolls.
//!
//! The page field commits on Enter or blur: a number in `1..=pages` moves
//! there, anything else reverts (`onBlur`). Page and zoom are controlled
//! with [`PdfViewer::page`] or kept internally from `default_page`.
use super::head::{code_head, ghost_icon, grow, head_icon, head_name, sep};
use crate::{
    icons::CarbonIcon,
    t3::craft::{rem, sans},
    theme::{color, color_in},
    tokens::{self, CarbonTheme, ColorRole},
    u0::primitives::parts::{editor, focus_ring, focus_visible, keyed_focus},
};
use gpui_kit::{
    AnyElement, App, AppContext as _, BoxShadow, Context, CursorStyle, ElementId, Entity,
    Focusable as _, Hsla, InteractiveElement as _, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement as _, Styled, Subscription, Window,
    component::input::{InputEvent, InputState},
    div, hsla, point,
    prelude::FluentBuilder as _,
    px, relative,
};
use std::rc::Rc;

/// The zoom bounds and step: `Math.max(50, zoom - 25)`, `Math.min(300, zoom + 25)`.
pub const ZOOM_MIN: u32 = 50;
/// See [`ZOOM_MIN`].
pub const ZOOM_MAX: u32 = 300;
/// See [`ZOOM_MIN`].
pub const ZOOM_STEP: u32 = 25;

/// `.g-pdf-body { height: 16rem }`.
pub const BODY_HEIGHT: f32 = 256.;
/// `.g-pdf-stage { padding: var(--spacing-05) }`.
pub const STAGE_PAD: f32 = 16.;
/// `.g-pdf-sheet { aspect-ratio: 1 / 1.4142 }`.
pub const A4: f32 = std::f32::consts::SQRT_2;

/// The page a typed draft commits to: a whole number in `1..=pages`, else
/// `None` (the field reverts).
pub fn parse_page(draft: &str, pages: u32) -> Option<u32> {
    // `parseInt(pd, 10)`: leading digits after optional spaces.
    let digits: String = draft
        .trim_start()
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse().ok().filter(|n| (1..=pages).contains(n))
}

/// The page sheet's (width, height) in px at `zoom` percent: the stage's
/// inner height at 100%.
pub fn sheet_size(zoom: u32) -> (f32, f32) {
    let h = (BODY_HEIGHT - 2. * STAGE_PAD) * zoom as f32 / 100.;
    (h / A4, h)
}

/// The sheet's paper (`#ffffff`), ink (`#161616`) and text-bar (`#e0e0e0`)
/// colors: White's `background`, `text-primary` and `border-subtle-00`, so
/// a page stays paper-white in every theme.
pub fn paper() -> (Hsla, Hsla, Hsla) {
    (
        color_in(CarbonTheme::White, ColorRole::Background),
        color_in(CarbonTheme::White, ColorRole::TextPrimary),
        color_in(CarbonTheme::White, ColorRole::BorderSubtle00),
    )
}

/// `box-shadow: 0 1px 3px rgba(0,0,0,.25)`.
fn sheet_shadow() -> Vec<BoxShadow> {
    vec![BoxShadow {
        color: hsla(0., 0., 0., 0.25),
        offset: point(px(0.), px(1.)),
        blur_radius: px(3.),
        spread_radius: px(0.),
        inset: false,
    }]
}

/// The viewer's user-facing strings (the reference's English defaults).
#[derive(Clone)]
pub struct PdfViewerTexts {
    /// `"Previous page"`.
    pub previous: SharedString,
    /// `"Next page"`.
    pub next: SharedString,
    /// `"Zoom out"`.
    pub zoom_out: SharedString,
    /// `"Zoom in"`.
    pub zoom_in: SharedString,
    /// The placeholder sheet's title: `"Page {n}"`.
    pub page: Rc<dyn Fn(u32) -> SharedString>,
}

impl Default for PdfViewerTexts {
    fn default() -> Self {
        Self {
            previous: "Previous page".into(),
            next: "Next page".into(),
            zoom_out: "Zoom out".into(),
            zoom_in: "Zoom in".into(),
            page: Rc::new(|n| format!("Page {n}").into()),
        }
    }
}

type PageHandler = Rc<dyn Fn(u32, &mut Window, &mut App)>;
type RenderPage = Rc<dyn Fn(u32, &mut Window, &mut App) -> AnyElement>;

struct Model {
    page: u32,
    zoom: u32,
    pages: u32,
    controlled: bool,
    input: Entity<InputState>,
    on_change: Option<PageHandler>,
    _subscription: Subscription,
}

impl Model {
    /// `set(n)`: move (uncontrolled), report, and show `n` in the field.
    fn set(&mut self, n: u32, window: &mut Window, cx: &mut Context<Self>) {
        if !self.controlled {
            self.page = n;
        }
        let shown = self.page.to_string();
        self.input
            .update(cx, |s, cx| s.set_value(shown, window, cx));
        if let Some(f) = self.on_change.clone() {
            f(n, window, cx);
        }
        cx.notify();
    }

    /// `onBlur`: commit a valid draft, else revert (`setPd(null)`).
    fn commit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let draft = self.input.read(cx).value().to_string();
        match parse_page(&draft, self.pages) {
            Some(n) if n != self.page => self.set(n, window, cx),
            _ => {
                let shown = self.page.to_string();
                self.input
                    .update(cx, |s, cx| s.set_value(shown, window, cx));
                cx.notify();
            }
        }
    }
}

/// A PDF viewer frame. See the module docs.
#[derive(IntoElement)]
pub struct PdfViewer {
    id: ElementId,
    name: SharedString,
    pages: u32,
    page: Option<u32>,
    default_page: u32,
    on_page_change: Option<PageHandler>,
    render_page: Option<RenderPage>,
    texts: PdfViewerTexts,
}

impl PdfViewer {
    /// A viewer for the document `name` with `pages` pages.
    pub fn new(id: impl Into<ElementId>, name: impl Into<SharedString>, pages: u32) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            pages: pages.max(1),
            page: None,
            default_page: 1,
            on_page_change: None,
            render_page: None,
            texts: PdfViewerTexts::default(),
        }
    }

    /// The shown page, controlled (`page`).
    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    /// The first page shown when uncontrolled (`defaultPage`, default 1).
    pub fn default_page(mut self, page: u32) -> Self {
        self.default_page = page;
        self
    }

    /// Called with the page the user moves to (`onPageChange`).
    pub fn on_page_change(mut self, f: impl Fn(u32, &mut Window, &mut App) + 'static) -> Self {
        self.on_page_change = Some(Rc::new(f));
        self
    }

    /// Draw page `n` (`renderPage`); the element fills the A4 sheet.
    pub fn render_page(
        mut self,
        f: impl Fn(u32, &mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.render_page = Some(Rc::new(f));
        self
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: PdfViewerTexts) -> Self {
        self.texts = texts;
        self
    }
}

/// `.g-pdf-sheet i { display: block; height: 6px; background: #e0e0e0 }
/// .g-pdf-sheet i:nth-child(3n) { width: 60% }`: `n` bars, `h` px tall,
/// starting at child number `first`.
fn bars(n: usize, first: usize, h: f32, ink: Hsla) -> Vec<AnyElement> {
    (0..n)
        .map(|i| {
            div()
                .flex_none()
                .h(px(h))
                .bg(ink)
                .when((first + i).is_multiple_of(3), |d| d.w(relative(0.6)))
                .into_any_element()
        })
        .collect()
}

/// `.g-pdf-sheet--sm { width: 2.75rem; padding: 6px; gap: 3px } .g-pdf-sheet--sm i
/// { height: 2px }` (three bars), ringed when its page is current.
fn small_sheet(on: bool) -> impl IntoElement {
    let (paper, _, bar) = paper();
    let mut shadow = sheet_shadow();
    if on {
        // `.g-pdf-thumb--on .g-pdf-sheet { box-shadow: 0 0 0 2px var(--border-interactive) }`.
        shadow = vec![BoxShadow {
            color: color(ColorRole::BorderInteractive),
            offset: point(px(0.), px(0.)),
            blur_radius: px(0.),
            spread_radius: px(2.),
            inset: false,
        }];
    }
    div()
        .flex()
        .flex_col()
        .flex_none()
        .gap(px(3.))
        .w(rem(44.))
        .h(rem(44. * A4))
        .p(px(6.))
        .bg(paper)
        .shadow(shadow)
        .children(bars(3, 1, 2., bar))
}

impl RenderOnce for PdfViewer {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let pages = self.pages;
        let initial = self.page.unwrap_or(self.default_page).clamp(1, pages);
        let model = window.use_keyed_state((id.clone(), "pdf"), cx, |window, cx| {
            let input = cx.new(|cx| InputState::new(window, cx).default_value(initial.to_string()));
            let subscription =
                cx.subscribe_in(&input, window, |m: &mut Model, _, e, window, cx| {
                    match e {
                        InputEvent::Blur => m.commit(window, cx),
                        // `onKeyDown: e.key === 'Enter' && e.currentTarget.blur()`.
                        InputEvent::PressEnter { .. } => {
                            m.commit(window, cx);
                            window.blur(cx);
                        }
                        _ => {}
                    }
                });
            Model {
                page: initial,
                zoom: 100,
                pages,
                controlled: false,
                input,
                on_change: None,
                _subscription: subscription,
            }
        });
        model.update(cx, |m, cx| {
            m.pages = pages;
            m.on_change = self.on_page_change.clone();
            m.controlled = self.page.is_some();
            if let Some(p) = self.page
                && p != m.page
            {
                m.page = p.clamp(1, pages);
                let focused = m.input.read(cx).focus_handle(cx).is_focused(window);
                if !focused {
                    let shown = m.page.to_string();
                    m.input.update(cx, |s, cx| s.set_value(shown, window, cx));
                }
            }
        });
        let (page, zoom, input) = {
            let m = model.read(cx);
            (m.page, m.zoom, m.input.clone())
        };
        let go = |to: u32| {
            let model = model.clone();
            Rc::new(move |window: &mut Window, cx: &mut App| {
                model.update(cx, |m, cx| m.set(to, window, cx))
            }) as Rc<dyn Fn(&mut Window, &mut App)>
        };
        let zoom_by = |up: bool| {
            let model = model.clone();
            Rc::new(move |_: &mut Window, cx: &mut App| {
                model.update(cx, |m, cx| {
                    m.zoom = if up {
                        (m.zoom + ZOOM_STEP).min(ZOOM_MAX)
                    } else {
                        m.zoom.saturating_sub(ZOOM_STEP).max(ZOOM_MIN)
                    };
                    cx.notify();
                })
            }) as Rc<dyn Fn(&mut Window, &mut App)>
        };

        // `.g-pdf-page input { width: 2.25rem; height: 1.5rem; padding: 0 4px;
        // border-bottom: 1px solid var(--border-strong-01); background:
        // var(--field-01); text-align: center }`.
        let field_focus = input.read(cx).focus_handle(cx);
        let field = div()
            .relative()
            .flex()
            .items_center()
            .flex_none()
            .w(rem(36.))
            .h(rem(24.))
            .px(px(4.))
            .bg(color(ColorRole::Field01))
            .border_b_1()
            .border_color(color(ColorRole::BorderStrong01))
            .child(editor(&input).text_size(rem(12.)).text_center())
            .when(field_focus.is_focused(window), |d| d.child(focus_ring()));
        let head = code_head()
            .child(head_icon(CarbonIcon::DocumentPdf))
            .child(head_name(self.name.clone(), ColorRole::TextSecondary))
            .child(grow())
            .child(ghost_icon(
                (id.clone(), "prev"),
                self.texts.previous.clone(),
                CarbonIcon::ChevronUp,
                page <= 1,
                Some(go(page.saturating_sub(1).max(1))),
            ))
            // `.g-pdf-page { display: inline-flex; gap: var(--spacing-02);
            // font-size: 0.75rem; font-variant-numeric: tabular-nums }`.
            .child(
                sans(div(), 12., 400)
                    .flex()
                    .flex_none()
                    .items_center()
                    .gap(tokens::SPACING_02)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(field)
                    .child(format!("/ {pages}")),
            )
            .child(ghost_icon(
                (id.clone(), "next"),
                self.texts.next.clone(),
                CarbonIcon::ChevronDown,
                page >= pages,
                Some(go((page + 1).min(pages))),
            ))
            .child(sep())
            .child(ghost_icon(
                (id.clone(), "zout"),
                self.texts.zoom_out.clone(),
                CarbonIcon::ZoomOut,
                false,
                Some(zoom_by(false)),
            ))
            .child(
                sans(div(), 12., 400)
                    .flex_none()
                    .text_color(color(ColorRole::TextSecondary))
                    .child(format!("{zoom}%")),
            )
            .child(ghost_icon(
                (id.clone(), "zin"),
                self.texts.zoom_in.clone(),
                CarbonIcon::ZoomIn,
                false,
                Some(zoom_by(true)),
            ));

        // `.g-pdf-thumbs { gap: var(--spacing-03); width: 4.5rem; padding:
        // var(--spacing-03); border-right: 1px solid var(--border-subtle-01) }`.
        let thumbs = div()
            .id((id.clone(), "thumbs"))
            .flex()
            .flex_col()
            .flex_none()
            .gap(tokens::SPACING_03)
            .w(rem(72.))
            .p(tokens::SPACING_03)
            .overflow_y_scroll()
            .border_r_1()
            .border_color(color(ColorRole::BorderSubtle01))
            .children((1..=pages).map(|n| {
                let tid: ElementId = (id.clone(), SharedString::from(format!("thumb-{n}"))).into();
                let focus = keyed_focus((tid.clone(), "focus"), window, cx);
                let ring = focus_visible(&focus, window);
                let hovered = window.use_keyed_state((tid.clone(), "hover"), cx, |_, _| false);
                let over = *hovered.read(cx);
                let on = n == page;
                let select = go(n);
                // `.g-pdf-thumb { flex-direction: column; align-items: center;
                // gap: 2px; padding: var(--spacing-02); font-size: 0.6875rem;
                // color: var(--text-secondary) }`, hover `layer-hover-01`,
                // `--on` `state-selected-background` / `text-primary`.
                sans(gpui_kit::base::Button::new(tid), 11., 400)
                    .track_focus(&focus)
                    .relative()
                    .flex()
                    .flex_col()
                    .flex_none()
                    .items_center()
                    .gap(px(2.))
                    .p(tokens::SPACING_02)
                    .cursor(CursorStyle::PointingHand)
                    .text_color(color(if on {
                        ColorRole::TextPrimary
                    } else {
                        ColorRole::TextSecondary
                    }))
                    .when(on, |b| b.bg(color(ColorRole::StateSelectedBackground)))
                    .when(over && !on, |b| b.bg(color(ColorRole::LayerHover01)))
                    .on_hover(move |h, _, cx| {
                        hovered.update(cx, |v, cx| {
                            if *v != *h {
                                *v = *h;
                                cx.notify();
                            }
                        })
                    })
                    .on_click(move |_, w, cx| select(w, cx))
                    .child(small_sheet(on))
                    .child(n.to_string())
                    .when(ring, |b| b.child(focus_ring()))
            }));

        let (sw, sh) = sheet_size(zoom);
        let (paper, ink, bar) = paper();
        let content = match &self.render_page {
            Some(f) => f(page, window, cx),
            None => div()
                .flex()
                .flex_col()
                .gap(px(8.))
                .child(
                    // `.g-pdf-sheet b`: no weight rule, so Carbon's
                    // `b { font: inherit }` keeps it 400.
                    sans(div(), 12., 400)
                        .text_color(ink)
                        .child((self.texts.page)(page)),
                )
                .children(bars(5, 2, 6., bar))
                .into_any_element(),
        };
        // `.g-pdf-sheet { display: flex; flex-direction: column; gap: 8px;
        // padding: 8%; background: #ffffff; color: #161616 }`.
        let sheet = div()
            .flex()
            .flex_col()
            .flex_none()
            .w(rem(sw))
            .h(rem(sh))
            .p(rem(sw * 0.08))
            .bg(paper)
            .text_color(ink)
            .shadow(sheet_shadow())
            .overflow_hidden()
            .child(content);
        // `.g-pdf-stage { flex: 1; overflow: auto; padding: var(--spacing-05);
        // background: var(--layer-accent-01); display: flex; justify-content: center }`.
        let stage = div()
            .id((id.clone(), "stage"))
            .flex_1()
            .min_w_0()
            .overflow_scroll()
            .bg(color(ColorRole::LayerAccent01))
            .child(
                div()
                    .flex()
                    .justify_center()
                    .items_start()
                    .min_w_full()
                    .p(rem(STAGE_PAD))
                    .child(sheet),
            );

        div()
            .id(id)
            .flex()
            .flex_col()
            .min_w_0()
            .bg(color(ColorRole::Layer01))
            .child(head)
            .child(
                div()
                    .flex()
                    .h(rem(BODY_HEIGHT))
                    .min_h_0()
                    .child(thumbs)
                    .child(stage),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_drafts_commit_like_parse_int() {
        assert_eq!(parse_page("3", 4), Some(3));
        assert_eq!(parse_page(" 2x", 4), Some(2));
        // Out of range or not a number reverts.
        assert_eq!(parse_page("0", 4), None);
        assert_eq!(parse_page("5", 4), None);
        assert_eq!(parse_page("", 4), None);
        assert_eq!(parse_page("x2", 4), None);
    }

    #[test]
    fn sheet_keeps_a4_and_scales() {
        // 100% fills the 16rem body less 2 × 16px padding.
        let (w, h) = sheet_size(100);
        assert_eq!(h, 224.);
        assert!((h / w - A4).abs() < 1e-4);
        assert_eq!(sheet_size(200).1, 448.);
    }

    #[test]
    fn paper_is_white_in_every_theme() {
        assert_eq!(
            paper(),
            (
                gpui_kit::rgb(0xffffff).into(),
                gpui_kit::rgb(0x161616).into(),
                gpui_kit::rgb(0xe0e0e0).into()
            )
        );
    }
}
