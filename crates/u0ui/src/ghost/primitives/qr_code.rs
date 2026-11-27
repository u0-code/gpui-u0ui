//! QRCode: a scannable QR code (`QRCode` in `src/ghost/primitives.jsx`).
//!
//! - Always dark modules on white with a four-module quiet zone, in every
//!   theme (never inverted, so phones can scan it): the reference's `#ffffff`
//!   and `#161616` are Carbon White's `background` and `text-primary`, read
//!   from the White theme whatever the active one is.
//! - Encoding is a real QR encoder (the `qrcode` crate, MIT OR Apache-2.0; the
//!   reference uses qrcode-generator, MIT), smallest version for the data at
//!   the error-correction `level` (L/M/Q/H, default M).
//! - Drawn on a canvas with module edges snapped to device pixels
//!   (`shape-rendering: crispEdges`), runs of dark modules merged per row.
use crate::{CarbonTheme, theme::color_in, tokens::ColorRole};
use gpui_kit::{
    App, Bounds, ElementId, Hsla, InteractiveElement as _, IntoElement, ParentElement, RenderOnce,
    Role, SharedString, StatefulInteractiveElement as _, Styled, Window, canvas, div, fill, point,
    px, size,
};
use qrcode::{Color, EcLevel, QrCode as Encoder};
use std::rc::Rc;

/// The quiet zone in modules (`const q = 4`).
pub const QUIET_ZONE: usize = 4;

/// Error-correction level (`level`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum QrLevel {
    /// ~7% recovery.
    L,
    /// ~15% recovery (default).
    #[default]
    M,
    /// ~25% recovery.
    Q,
    /// ~30% recovery.
    H,
}

impl QrLevel {
    /// Every level.
    pub const ALL: [QrLevel; 4] = [Self::L, Self::M, Self::Q, Self::H];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::L => "L",
            Self::M => "M",
            Self::Q => "Q",
            Self::H => "H",
        }
    }

    fn ec(self) -> EcLevel {
        match self {
            Self::L => EcLevel::L,
            Self::M => EcLevel::M,
            Self::Q => EcLevel::Q,
            Self::H => EcLevel::H,
        }
    }
}

/// An encoded symbol: `n × n` modules, row-major, `true` = dark.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Matrix {
    /// Modules per side (`getModuleCount()`).
    pub n: usize,
    /// Row-major dark flags.
    pub dark: Vec<bool>,
}

impl Matrix {
    /// Encode `value` at `level`; `None` when it does not fit any version.
    pub fn encode(value: &str, level: QrLevel) -> Option<Self> {
        let code = Encoder::with_error_correction_level(value.as_bytes(), level.ec()).ok()?;
        let n = code.width();
        let dark = code
            .to_colors()
            .into_iter()
            .map(|c| c == Color::Dark)
            .collect();
        Some(Self { n, dark })
    }

    /// Whether module (`row`, `col`) is dark (`isDark(r, c)`).
    pub fn is_dark(&self, row: usize, col: usize) -> bool {
        self.dark[row * self.n + col]
    }

    /// Horizontal runs of dark modules per row: `(row, first col, length)`.
    pub fn runs(&self) -> Vec<(usize, usize, usize)> {
        let mut out = Vec::new();
        for r in 0..self.n {
            let mut c = 0;
            while c < self.n {
                if self.is_dark(r, c) {
                    let start = c;
                    while c < self.n && self.is_dark(r, c) {
                        c += 1;
                    }
                    out.push((r, start, c - start));
                } else {
                    c += 1;
                }
            }
        }
        out
    }
}

/// The module size for a `size`px code of `n` modules (`size / (n + q * 2)`).
pub fn cell(size: f32, n: usize) -> f32 {
    size / (n + QUIET_ZONE * 2) as f32
}

/// (light, dark): White's `background` (#ffffff) and `text-primary` (#161616).
pub fn colors() -> (Hsla, Hsla) {
    (
        color_in(CarbonTheme::White, ColorRole::Background),
        color_in(CarbonTheme::White, ColorRole::TextPrimary),
    )
}

/// A QR code. See the module docs.
#[derive(IntoElement)]
pub struct QrCode {
    id: ElementId,
    value: SharedString,
    size: f32,
    label: SharedString,
    level: QrLevel,
}

impl QrCode {
    /// A QR code with a stable `id` encoding `value` (160px, level M).
    pub fn new(id: impl Into<ElementId>, value: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            value: value.into(),
            size: 160.,
            // The reference's default accessible name; override per locale.
            label: "QR code".into(),
            level: QrLevel::M,
        }
    }

    /// Side length in px (default 160).
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// The accessible name (`role="img" aria-label`).
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    /// Error-correction level (default M).
    pub fn level(mut self, level: QrLevel) -> Self {
        self.level = level;
        self
    }
}

impl RenderOnce for QrCode {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // Encode once per (value, level).
        let key = (self.value.clone(), self.level);
        let cache = window.use_keyed_state((self.id.clone(), "qr"), cx, |_, _| {
            None::<((SharedString, QrLevel), Option<Rc<Matrix>>)>
        });
        let matrix = match cache.read(cx) {
            Some((k, m)) if *k == key => m.clone(),
            _ => {
                let m = Matrix::encode(&self.value, self.level).map(Rc::new);
                cache.update(cx, |c, _| *c = Some((key, m.clone())));
                m
            }
        };
        let (light, dark) = colors();
        let side = px(self.size);
        div()
            .id(self.id.clone())
            .role(Role::Image)
            .aria_label(self.label.clone())
            .size(side)
            .flex_none()
            .bg(light)
            .child(
                canvas(
                    |_, _, _| {},
                    move |bounds: Bounds<gpui_kit::Pixels>, _, window, _| {
                        let Some(m) = matrix.as_ref() else { return };
                        let scale = window.scale_factor();
                        let size_px = f32::from(bounds.size.width);
                        let cell = cell(size_px, m.n);
                        let ox = f32::from(bounds.origin.x);
                        let oy = f32::from(bounds.origin.y);
                        // Snap to device pixels so neighbouring modules share edges.
                        let snap = |v: f32| (v * scale).round() / scale;
                        for (r, c, len) in m.runs() {
                            let x0 = snap(ox + (c + QUIET_ZONE) as f32 * cell);
                            let x1 = snap(ox + (c + len + QUIET_ZONE) as f32 * cell);
                            let y0 = snap(oy + (r + QUIET_ZONE) as f32 * cell);
                            let y1 = snap(oy + (r + 1 + QUIET_ZONE) as f32 * cell);
                            window.paint_quad(fill(
                                Bounds::new(point(px(x0), px(y0)), size(px(x1 - x0), px(y1 - y0))),
                                dark,
                            ));
                        }
                    },
                )
                .size_full(),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_the_reference_values() {
        let m = Matrix::encode("ghost://pair/7f3a-9c2e-41b0", QrLevel::M).unwrap();
        // 27 bytes at level M need version 3 (29 × 29; version 2 holds 26).
        assert_eq!(m.n, 29);
        // Finder patterns: dark corners, light separator.
        for (r, c) in [(0, 0), (0, m.n - 1), (m.n - 1, 0), (3, 3)] {
            assert!(m.is_dark(r, c), "({r}, {c})");
        }
        assert!(!m.is_dark(7, 7));
        let small = Matrix::encode("https://example.com", QrLevel::M).unwrap();
        // 19 bytes at M: version 2 (version 1 holds 14).
        assert_eq!(small.n, 25);
        // Higher correction needs more modules for the same data.
        let h = Matrix::encode("https://example.com", QrLevel::H).unwrap();
        assert!(h.n > small.n);
    }

    #[test]
    fn runs_cover_every_dark_module() {
        let m = Matrix::encode("https://example.com", QrLevel::M).unwrap();
        let covered: usize = m.runs().iter().map(|r| r.2).sum();
        assert_eq!(covered, m.dark.iter().filter(|d| **d).count());
        assert_eq!(cell(168., 25), 168. / 33.);
    }

    #[test]
    fn colors_never_invert() {
        crate::theme::tests_set(CarbonTheme::G100);
        let (light, dark) = colors();
        assert_eq!(light, color_in(CarbonTheme::White, ColorRole::Background));
        assert_eq!(dark, color_in(CarbonTheme::White, ColorRole::TextPrimary));
        assert!(light.l > 0.99 && dark.l < 0.1);
    }
}
