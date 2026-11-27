//! Chart: a light SVG-style chart (`bar` | `line` | `area` | `donut`) in the
//! Carbon Charts categorical palette (`charts-1…14`, in order).
//!
//! - **Structure** follows `index.jsx` `export function Chart`: a `.cu-chart`
//!   figure (16px padding on `layer-01`), an optional title, a legend, then an
//!   SVG drawn in a `560 × height` viewBox scaled to the figure width
//!   (`width="100%"`), with `pad { l: 40, r: 12, t: 12, b: 28 }`, a "nice" top
//!   tick, five dashed grid lines, axis labels and a baseline. The donut is a
//!   fixed 200 × 200 SVG (`R 80`, `r 52`) with the total in its hole and a
//!   vertical legend beside it.
//! - **Geometry** is a set of pure functions over the viewBox ([`Cartesian`],
//!   [`donut_segments`], [`tip_origin`]), unit-tested against the JSX formulas.
//! - **Drawing**: GPUI `canvas` + `PathBuilder` (dash array for the
//!   `stroke-dasharray: 2 2` grid, arcs for the donut). Kit's `chart` module
//!   draws its own axes, colors and tooltip in Kit's look and lays charts out
//!   with its own margins; the reference geometry is simple enough that drawing
//!   it directly is shorter than restyling Kit's charts, so only GPUI
//!   primitives are used.
//! - **Hover** is real: bars and line/area points show the `.cu-chart-tip`
//!   tooltip (`background-inverse` box, `text-inverse` 11px text, clamped as in
//!   the JSX); a hovered donut segment gets `.cu-chart-seg:hover { opacity: .85 }`
//!   and shows its `<title>` text (`label: value`) in the same tooltip style,
//!   since GPUI has no native SVG `<title>` tooltip; like the native one it
//!   opens at the pointer ([`segment_tip_origin`]), not over the total.
//!
//! ```ignore
//! Chart::new("requests")
//!     .title("Requests by region")
//!     .x_key("m")
//!     .data(vec![ChartRow::new().text("m", "Jan").num("us", 42.).num("eu", 30.)])
//!     .series(vec![ChartSeries::new("us").label("US"), ChartSeries::new("eu").label("EU")])
//! ```
use crate::{
    theme::color,
    tokens::{ColorRole, FontFamilyKey, TypeStyle},
    typography::{TypeStyled as _, font},
};
use gpui_kit::{
    App, Bounds, DispatchPhase, ElementId, Hitbox, HitboxBehavior, Hsla, IntoElement,
    MouseMoveEvent, ParentElement, PathBuilder, Pixels, Point, RenderOnce, SharedString, Styled,
    TextAlign, TextRun, Window, canvas, div, point, prelude::FluentBuilder as _, px, rems,
};
use std::rc::Rc;

/// `index.d.ts` `ChartProps.type`, plus `area` (the line chart with its area filled).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ChartType {
    /// Grouped bars (the default).
    #[default]
    Bar,
    /// A 2px polyline per series with 3.5px-radius points.
    Line,
    /// The line chart with the area under each line filled.
    Area,
    /// A donut of the first series.
    Donut,
}

impl ChartType {
    /// Every type.
    pub const ALL: [ChartType; 4] = [Self::Bar, Self::Line, Self::Area, Self::Donut];
    /// The types `index.d.ts` declares (`'bar' | 'line' | 'donut'`); `Area`
    /// is an extension kept for callers.
    pub const REFERENCE: [ChartType; 3] = [Self::Bar, Self::Line, Self::Donut];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Bar => "bar",
            Self::Line => "line",
            Self::Area => "area",
            Self::Donut => "donut",
        }
    }
}

/// One field value of a [`ChartRow`].
#[derive(Clone, Debug, PartialEq)]
pub enum ChartValue {
    /// A category label (the `xKey` field).
    Text(SharedString),
    /// A series value.
    Number(f64),
}

/// One record of `ChartProps.data` (`Record<string, any>`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChartRow {
    fields: Vec<(SharedString, ChartValue)>,
}

impl ChartRow {
    /// An empty record.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a text field.
    pub fn text(mut self, key: impl Into<SharedString>, value: impl Into<SharedString>) -> Self {
        self.fields
            .push((key.into(), ChartValue::Text(value.into())));
        self
    }

    /// Add a numeric field.
    pub fn num(mut self, key: impl Into<SharedString>, value: f64) -> Self {
        self.fields.push((key.into(), ChartValue::Number(value)));
        self
    }

    /// The field named `key`.
    pub fn get(&self, key: &str) -> Option<&ChartValue> {
        self.fields
            .iter()
            .find(|(k, _)| k.as_ref() == key)
            .map(|(_, v)| v)
    }

    /// The field as a number (0 when missing or text).
    pub fn number(&self, key: &str) -> f64 {
        match self.get(key) {
            Some(ChartValue::Number(v)) => *v,
            _ => 0.,
        }
    }

    /// The field as display text (`${d[key]}`).
    pub fn label(&self, key: &str) -> SharedString {
        match self.get(key) {
            Some(ChartValue::Text(t)) => t.clone(),
            Some(ChartValue::Number(v)) => format_number(*v).into(),
            None => SharedString::default(),
        }
    }
}

/// One entry of `ChartProps.series`: `{ key, label? }`.
#[derive(Clone, Debug, PartialEq)]
pub struct ChartSeries {
    /// The data field plotted.
    pub key: SharedString,
    /// The legend/tooltip label (defaults to `key`).
    pub label: Option<SharedString>,
}

impl ChartSeries {
    /// A series plotting `key`.
    pub fn new(key: impl Into<SharedString>) -> Self {
        Self {
            key: key.into(),
            label: None,
        }
    }

    /// The display label.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// `s.label || s.key`.
    pub fn display(&self) -> SharedString {
        self.label.clone().unwrap_or_else(|| self.key.clone())
    }
}

/// JS number → string (`${v}`): integers without a fraction, others in the
/// shortest round-trip form.
pub fn format_number(v: f64) -> String {
    if v.is_finite() && v.fract() == 0. && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

/// The categorical palette color for series/segment `i`:
/// `color = (i) => var(--charts-${(i % 14) + 1})`.
pub fn palette_role(i: usize) -> ColorRole {
    const ROLES: [ColorRole; 14] = [
        ColorRole::Charts1,
        ColorRole::Charts2,
        ColorRole::Charts3,
        ColorRole::Charts4,
        ColorRole::Charts5,
        ColorRole::Charts6,
        ColorRole::Charts7,
        ColorRole::Charts8,
        ColorRole::Charts9,
        ColorRole::Charts10,
        ColorRole::Charts11,
        ColorRole::Charts12,
        ColorRole::Charts13,
        ColorRole::Charts14,
    ];
    ROLES[i % 14]
}

/// viewBox width: `const W = 560`.
pub const VIEW_WIDTH: f32 = 560.;
/// Default `height = 220`.
pub const DEFAULT_HEIGHT: f32 = 220.;
/// `pad = { l: 40, r: 12, t: 12, b: 28 }`.
pub const PAD: Pad = Pad {
    l: 40.,
    r: 12.,
    t: 12.,
    b: 28.,
};
/// `.cu-chart-axis { font-size: 11px }` and `.cu-chart-tip text { font-size: 11px }`.
pub const AXIS_FONT: f32 = 11.;
/// `.cu-chart-total { font-size: 24px; font-weight: 300 }`.
pub const TOTAL_FONT: f32 = 24.;
/// Line series: `strokeWidth="2"`; points `r="3.5"`.
pub const LINE_WIDTH: f32 = 2.;
/// Line point radius.
pub const POINT_RADIUS: f32 = 3.5;
/// Area fill opacity (not in the reference: the Carbon Charts area fill).
pub const AREA_OPACITY: f32 = 0.3;
/// `.cu-chart-seg:hover { opacity: .85 }`.
pub const SEGMENT_HOVER_OPACITY: f32 = 0.85;
/// Donut SVG size and radii: `width="200" … R = 80, r = 52, cxp = 100, cyp = 100`.
pub const DONUT_SIZE: f32 = 200.;
/// Donut outer radius.
pub const DONUT_OUTER: f32 = 80.;
/// Donut inner radius.
pub const DONUT_INNER: f32 = 52.;

/// SVG padding inside the viewBox.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pad {
    /// Left (tick labels).
    pub l: f32,
    /// Right.
    pub r: f32,
    /// Top.
    pub t: f32,
    /// Bottom (category labels).
    pub b: f32,
}

/// The "nice" top tick: `nice = 10 ** floor(log10(max)); top = ceil(max / nice) * nice`
/// with `max = Math.max(1, …values)`.
pub fn nice_top(max: f64) -> f64 {
    let max = max.max(1.);
    let nice = 10f64.powf(max.log10().floor());
    (max / nice).ceil() * nice
}

/// A rectangle in viewBox units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    /// Left.
    pub x: f32,
    /// Top.
    pub y: f32,
    /// Width.
    pub w: f32,
    /// Height.
    pub h: f32,
}

impl Rect {
    fn contains(&self, p: (f32, f32)) -> bool {
        p.0 >= self.x && p.0 <= self.x + self.w && p.1 >= self.y && p.1 <= self.y + self.h
    }
}

/// The bar/line/area chart geometry in viewBox units.
#[derive(Clone, Debug, PartialEq)]
pub struct Cartesian {
    /// viewBox height (`height`).
    pub height: f32,
    /// The top tick value.
    pub top: f64,
    /// Number of categories.
    pub count: usize,
    /// Number of series.
    pub series: usize,
}

impl Cartesian {
    /// Geometry for `count` categories × `series` series whose largest value is `max`.
    pub fn new(height: f32, max: f64, count: usize, series: usize) -> Self {
        Self {
            height,
            top: nice_top(max),
            count,
            series,
        }
    }

    /// `iw = W - pad.l - pad.r`.
    pub fn inner_width(&self) -> f32 {
        VIEW_WIDTH - PAD.l - PAD.r
    }

    /// `ih = H - pad.t - pad.b`.
    pub fn inner_height(&self) -> f32 {
        self.height - PAD.t - PAD.b
    }

    /// `y = (v) => pad.t + ih - (v / top) * ih`.
    pub fn y(&self, v: f64) -> f32 {
        PAD.t + self.inner_height() - ((v / self.top) as f32) * self.inner_height()
    }

    /// The baseline: `y1 = pad.t + ih`.
    pub fn baseline(&self) -> f32 {
        PAD.t + self.inner_height()
    }

    /// `bw = iw / data.length`.
    pub fn band(&self) -> f32 {
        self.inner_width() / self.count.max(1) as f32
    }

    /// `pad.l + bw * i + bw / 2`.
    pub fn center(&self, i: usize) -> f32 {
        PAD.l + self.band() * i as f32 + self.band() / 2.
    }

    /// `ticks = [0, 0.25, 0.5, 0.75, 1].map((f) => f * top)`.
    pub fn ticks(&self) -> [f64; 5] {
        [0., 0.25, 0.5, 0.75, 1.].map(|f| f * self.top)
    }

    /// Bar `j` of category `i`: `w = (bw * 0.6) / series.length;
    /// x = pad.l + bw * i + bw * 0.2 + w * j`, `width = w - 1`,
    /// `height = pad.t + ih - y(v)`.
    pub fn bar(&self, i: usize, j: usize, v: f64) -> Rect {
        let bw = self.band();
        let w = bw * 0.6 / self.series.max(1) as f32;
        let x = PAD.l + bw * i as f32 + bw * 0.2 + w * j as f32;
        let y = self.y(v);
        Rect {
            x,
            y,
            w: (w - 1.).max(0.),
            h: self.baseline() - y,
        }
    }

    /// The tooltip anchor of bar `(i, j)`: `{ x: x + w / 2, y: y(v) }`.
    pub fn bar_anchor(&self, i: usize, j: usize, v: f64) -> (f32, f32) {
        let r = self.bar(i, j, v);
        let w = self.band() * 0.6 / self.series.max(1) as f32;
        (r.x + w / 2., r.y)
    }
}

/// The `.cu-chart-tip` translation: `translate(min(max(x, 90), W - 90), max(y - 10, 22))`.
pub fn tip_origin(x: f32, y: f32, view_width: f32) -> (f32, f32) {
    (x.max(90.).min(view_width - 90.), (y - 10.).max(22.))
}

/// The tooltip box relative to its origin: `<rect x="-86" y="-22" width="172" height="20" />`.
pub const TIP_BOX: Rect = Rect {
    x: -86.,
    y: -22.,
    w: 172.,
    h: 20.,
};
/// The tooltip text baseline: `<text y="-8" textAnchor="middle">`.
pub const TIP_BASELINE: f32 = -8.;
/// How far below the pointer a donut segment's tooltip starts: the segment's
/// SVG `<title>` is a native tooltip, which opens under the pointer (clear of
/// the arrow), not at a fixed place in the chart.
pub const SEGMENT_TIP_GAP: f32 = 20.;

/// The window position of a donut segment tooltip's top-left corner for a
/// pointer at `pointer` (window px).
pub fn segment_tip_origin(pointer: (f32, f32)) -> (f32, f32) {
    (pointer.0, pointer.1 + SEGMENT_TIP_GAP)
}

/// A hovered donut segment's `<title>` text (`label: value`) in the
/// `.cu-chart-tip` look, at the pointer like the native tooltip.
fn segment_tip(text: SharedString, pointer: Point<Pixels>, bg: Hsla, fg: Hsla) -> impl IntoElement {
    let (x, y) = segment_tip_origin((f32::from(pointer.x), f32::from(pointer.y)));
    gpui_kit::deferred(
        gpui_kit::anchored()
            .position(point(px(x), px(y)))
            .snap_to_window()
            .child(
                div()
                    .h(px(TIP_BOX.h))
                    .min_w(px(TIP_BOX.w))
                    .px(px(6.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(bg)
                    .text_color(fg)
                    .font(font(FontFamilyKey::Sans, 400))
                    .text_size(px(AXIS_FONT))
                    .whitespace_nowrap()
                    .child(text),
            ),
    )
    .with_priority(1)
}

/// Donut segment angles (radians, clockwise from 12 o'clock):
/// `a0 = (acc / tot) * 2π; acc += v; a1 = (acc / tot) * 2π`.
pub fn donut_segments(values: &[f64]) -> Vec<(f32, f32)> {
    let total: f64 = values.iter().sum();
    let mut acc = 0.;
    values
        .iter()
        .map(|v| {
            let a0 = if total > 0. { acc / total } else { 0. };
            acc += v;
            let a1 = if total > 0. { acc / total } else { 0. };
            (
                (a0 * std::f64::consts::TAU) as f32,
                (a1 * std::f64::consts::TAU) as f32,
            )
        })
        .collect()
}

/// `pt = (a, rr) => [cxp + rr * sin(a), cyp - rr * cos(a)]`.
pub fn donut_point(angle: f32, radius: f32) -> (f32, f32) {
    let c = DONUT_SIZE / 2.;
    (c + radius * angle.sin(), c - radius * angle.cos())
}

/// The segment index under `p` (donut viewBox units), if any.
pub fn donut_hit(segments: &[(f32, f32)], p: (f32, f32)) -> Option<usize> {
    let c = DONUT_SIZE / 2.;
    let (dx, dy) = (p.0 - c, p.1 - c);
    let r = (dx * dx + dy * dy).sqrt();
    if !(DONUT_INNER..=DONUT_OUTER).contains(&r) {
        return None;
    }
    let mut a = dx.atan2(-dy);
    if a < 0. {
        a += std::f32::consts::TAU;
    }
    segments.iter().position(|&(a0, a1)| a >= a0 && a < a1)
}

/// What the pointer is over.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ChartHover {
    /// Bar or point of series `series` at category `index`.
    Mark {
        /// Category index.
        index: usize,
        /// Series index.
        series: usize,
    },
    /// Donut segment `index`, with the pointer position in viewBox units.
    Segment {
        /// Segment index.
        index: usize,
        /// Pointer x.
        x: f32,
        /// Pointer y.
        y: f32,
    },
}

/// The prepared data the canvas paints and hit-tests.
struct Plot {
    kind: ChartType,
    geometry: Cartesian,
    categories: Vec<SharedString>,
    /// `values[i][j]`: category i, series j.
    values: Vec<Vec<f64>>,
    series: Vec<SharedString>,
    hover: Option<ChartHover>,
    colors: Colors,
}

/// The theme colors, read once per render.
#[derive(Clone)]
struct Colors {
    palette: Vec<Hsla>,
    grid: Hsla,
    baseline: Hsla,
    axis: Hsla,
    total: Hsla,
    tip_bg: Hsla,
    tip_text: Hsla,
}

impl Colors {
    fn read(n: usize) -> Self {
        Self {
            palette: (0..n.max(1)).map(|i| color(palette_role(i))).collect(),
            // `.cu-chart-grid { stroke: var(--border-subtle-01) }`.
            grid: color(ColorRole::BorderSubtle01),
            // `.cu-chart-baseline { stroke: var(--border-strong-01) }`.
            baseline: color(ColorRole::BorderStrong01),
            // `.cu-chart-axis { fill: var(--text-secondary) }`.
            axis: color(ColorRole::TextSecondary),
            // `.cu-chart-total { fill: var(--text-primary) }`.
            total: color(ColorRole::TextPrimary),
            // `.cu-chart-tip rect { fill: var(--background-inverse) }`.
            tip_bg: color(ColorRole::BackgroundInverse),
            // `.cu-chart-tip text { fill: var(--text-inverse) }`.
            tip_text: color(ColorRole::TextInverse),
        }
    }
}

impl Plot {
    fn hit(&self, p: (f32, f32)) -> Option<ChartHover> {
        let g = &self.geometry;
        match self.kind {
            ChartType::Bar => {
                for (i, row) in self.values.iter().enumerate() {
                    for (j, v) in row.iter().enumerate() {
                        if g.bar(i, j, *v).contains(p) {
                            return Some(ChartHover::Mark {
                                index: i,
                                series: j,
                            });
                        }
                    }
                }
                None
            }
            ChartType::Line | ChartType::Area => {
                // Later series paint on top, so they win.
                for j in (0..self.series.len()).rev() {
                    for (i, row) in self.values.iter().enumerate() {
                        let (cx, cy) = (g.center(i), g.y(row[j]));
                        let (dx, dy) = (p.0 - cx, p.1 - cy);
                        if dx * dx + dy * dy <= POINT_RADIUS * POINT_RADIUS {
                            return Some(ChartHover::Mark {
                                index: i,
                                series: j,
                            });
                        }
                    }
                }
                None
            }
            ChartType::Donut => None,
        }
    }

    fn tip(&self) -> Option<((f32, f32), SharedString)> {
        let ChartHover::Mark { index, series } = self.hover? else {
            return None;
        };
        let v = *self.values.get(index)?.get(series)?;
        let anchor = match self.kind {
            ChartType::Bar => self.geometry.bar_anchor(index, series, v),
            _ => (self.geometry.center(index), self.geometry.y(v)),
        };
        // `${d[xKey]} · ${s.label || s.key}: ${d[s.key]}`.
        let text = format!(
            "{} · {}: {}",
            self.categories[index],
            self.series[series],
            format_number(v)
        );
        Some((anchor, text.into()))
    }
}

/// Text anchor (`textAnchor`).
#[derive(Clone, Copy)]
enum Anchor {
    Middle,
    End,
}

/// Paint `text` with its baseline at `baseline` (window px).
#[allow(clippy::too_many_arguments)]
fn paint_text(
    text: SharedString,
    size: f32,
    weight: u16,
    fill: Hsla,
    anchor: Anchor,
    x: Pixels,
    baseline: Pixels,
    window: &mut Window,
    cx: &mut App,
) {
    if text.is_empty() {
        return;
    }
    let run = TextRun {
        len: text.len(),
        font: font(FontFamilyKey::Sans, weight),
        color: fill,
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    let line = window
        .text_system()
        .shape_line(text, px(size), &[run], None);
    let left = match anchor {
        Anchor::Middle => x - line.width / 2.,
        Anchor::End => x - line.width,
    };
    let line_height = line.ascent + line.descent;
    let _ = line.paint(
        point(left, baseline - line.ascent),
        line_height,
        TextAlign::Left,
        None,
        window,
        cx,
    );
}

/// Maps viewBox units to window pixels.
#[derive(Clone, Copy)]
struct View {
    origin: Point<Pixels>,
    scale: f32,
}

impl View {
    fn of(bounds: Bounds<Pixels>, view_width: f32) -> Self {
        Self {
            origin: bounds.origin,
            scale: f32::from(bounds.size.width) / view_width,
        }
    }
    fn p(&self, x: f32, y: f32) -> Point<Pixels> {
        self.origin + point(px(x * self.scale), px(y * self.scale))
    }
    fn len(&self, v: f32) -> Pixels {
        px(v * self.scale)
    }
    fn local(&self, p: Point<Pixels>) -> (f32, f32) {
        let d = p - self.origin;
        (
            f32::from(d.x) / self.scale.max(f32::EPSILON),
            f32::from(d.y) / self.scale.max(f32::EPSILON),
        )
    }
}

fn stroke_line(
    view: View,
    from: (f32, f32),
    to: (f32, f32),
    fill: Hsla,
    dashed: bool,
    w: &mut Window,
) {
    let mut path = PathBuilder::stroke(view.len(1.));
    if dashed {
        // `.cu-chart-grid { stroke-dasharray: 2 2 }`.
        path = path.dash_array(&[view.len(2.), view.len(2.)]);
    }
    path.move_to(view.p(from.0, from.1));
    path.line_to(view.p(to.0, to.1));
    if let Ok(path) = path.build() {
        w.paint_path(path, fill);
    }
}

fn fill_rect(view: View, r: Rect, fill: Hsla, w: &mut Window) {
    if r.w <= 0. || r.h <= 0. {
        return;
    }
    w.paint_quad(gpui_kit::fill(
        Bounds::new(
            view.p(r.x, r.y),
            gpui_kit::size(view.len(r.w), view.len(r.h)),
        ),
        fill,
    ));
}

fn fill_circle(view: View, c: (f32, f32), r: f32, fill: Hsla, w: &mut Window) {
    let mut path = PathBuilder::fill();
    let rr = view.len(r);
    path.move_to(view.p(c.0 - r, c.1));
    path.arc_to(point(rr, rr), px(0.), false, true, view.p(c.0 + r, c.1));
    path.arc_to(point(rr, rr), px(0.), false, true, view.p(c.0 - r, c.1));
    path.close();
    if let Ok(path) = path.build() {
        w.paint_path(path, fill);
    }
}

fn paint_tip(
    view: View,
    at: (f32, f32),
    text: SharedString,
    colors: &Colors,
    w: &mut Window,
    cx: &mut App,
) {
    fill_rect(
        view,
        Rect {
            x: at.0 + TIP_BOX.x,
            y: at.1 + TIP_BOX.y,
            ..TIP_BOX
        },
        colors.tip_bg,
        w,
    );
    let o = view.p(at.0, at.1 + TIP_BASELINE);
    paint_text(
        text,
        AXIS_FONT * view.scale,
        400,
        colors.tip_text,
        Anchor::Middle,
        o.x,
        o.y,
        w,
        cx,
    );
}

fn paint_cartesian(plot: &Plot, view: View, w: &mut Window, cx: &mut App) {
    let g = &plot.geometry;
    let c = &plot.colors;
    let font = AXIS_FONT * view.scale;
    let right = VIEW_WIDTH - PAD.r;
    // Grid lines and tick labels: `x = pad.l - 6, y = y(t) + 4, textAnchor end`.
    for t in g.ticks() {
        let y = g.y(t);
        stroke_line(view, (PAD.l, y), (right, y), c.grid, true, w);
        let o = view.p(PAD.l - 6., y + 4.);
        paint_text(
            format_number(t).into(),
            font,
            400,
            c.axis,
            Anchor::End,
            o.x,
            o.y,
            w,
            cx,
        );
    }
    // Category labels: `x = center, y = H - 8, textAnchor middle`.
    for (i, label) in plot.categories.iter().enumerate() {
        let o = view.p(g.center(i), g.height - 8.);
        paint_text(
            label.clone(),
            font,
            400,
            c.axis,
            Anchor::Middle,
            o.x,
            o.y,
            w,
            cx,
        );
    }
    match plot.kind {
        ChartType::Bar => {
            for (i, row) in plot.values.iter().enumerate() {
                for (j, v) in row.iter().enumerate() {
                    fill_rect(view, g.bar(i, j, *v), c.palette[j % c.palette.len()], w);
                }
            }
        }
        ChartType::Line | ChartType::Area => {
            let points = |j: usize| -> Vec<(f32, f32)> {
                plot.values
                    .iter()
                    .enumerate()
                    .map(|(i, row)| (g.center(i), g.y(row[j])))
                    .collect()
            };
            // Areas first, so no fill covers another series' line.
            if plot.kind == ChartType::Area {
                for j in 0..plot.series.len() {
                    let pts = points(j);
                    if pts.len() < 2 {
                        continue;
                    }
                    let mut area = PathBuilder::fill();
                    area.move_to(view.p(pts[0].0, g.baseline()));
                    for p in &pts {
                        area.line_to(view.p(p.0, p.1));
                    }
                    area.line_to(view.p(pts[pts.len() - 1].0, g.baseline()));
                    area.close();
                    if let Ok(path) = area.build() {
                        w.paint_path(path, c.palette[j % c.palette.len()].opacity(AREA_OPACITY));
                    }
                }
            }
            for j in 0..plot.series.len() {
                let fill = c.palette[j % c.palette.len()];
                let pts = points(j);
                if pts.len() > 1 {
                    let mut line = PathBuilder::stroke(view.len(LINE_WIDTH));
                    line.move_to(view.p(pts[0].0, pts[0].1));
                    for p in &pts[1..] {
                        line.line_to(view.p(p.0, p.1));
                    }
                    if let Ok(path) = line.build() {
                        w.paint_path(path, fill);
                    }
                }
                for p in &pts {
                    fill_circle(view, *p, POINT_RADIUS, fill, w);
                }
            }
        }
        ChartType::Donut => {}
    }
    stroke_line(
        view,
        (PAD.l, g.baseline()),
        (right, g.baseline()),
        c.baseline,
        false,
        w,
    );
    if let Some((anchor, text)) = plot.tip() {
        let at = tip_origin(anchor.0, anchor.1, VIEW_WIDTH);
        paint_tip(view, at, text, c, w, cx);
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_donut(
    segments: &[(f32, f32)],
    total: f64,
    unit: SharedString,
    hover: Option<ChartHover>,
    colors: &Colors,
    view: View,
    w: &mut Window,
    cx: &mut App,
) {
    for (i, &(a0, a1)) in segments.iter().enumerate() {
        let fill = colors.palette[i % colors.palette.len()];
        let fill = match hover {
            Some(ChartHover::Segment { index, .. }) if index == i => {
                fill.opacity(SEGMENT_HOVER_OPACITY)
            }
            _ => fill,
        };
        let mut path = PathBuilder::fill();
        let (ro, ri) = (view.len(DONUT_OUTER), view.len(DONUT_INNER));
        if a1 - a0 >= std::f32::consts::TAU - 1e-4 {
            // A single full segment: the SVG arc would be degenerate; draw the ring.
            let c = DONUT_SIZE / 2.;
            path.move_to(view.p(c, c - DONUT_OUTER));
            path.arc_to(
                point(ro, ro),
                px(0.),
                false,
                true,
                view.p(c, c + DONUT_OUTER),
            );
            path.arc_to(
                point(ro, ro),
                px(0.),
                false,
                true,
                view.p(c, c - DONUT_OUTER),
            );
            path.move_to(view.p(c, c - DONUT_INNER));
            path.arc_to(
                point(ri, ri),
                px(0.),
                false,
                false,
                view.p(c, c + DONUT_INNER),
            );
            path.arc_to(
                point(ri, ri),
                px(0.),
                false,
                false,
                view.p(c, c - DONUT_INNER),
            );
            path.close();
        } else {
            let big = a1 - a0 > std::f32::consts::PI;
            let (p0, p1) = (donut_point(a0, DONUT_OUTER), donut_point(a1, DONUT_OUTER));
            let (p2, p3) = (donut_point(a1, DONUT_INNER), donut_point(a0, DONUT_INNER));
            // `M x0 y0 A R R 0 big 1 x1 y1 L x2 y2 A r r 0 big 0 x3 y3 Z`.
            path.move_to(view.p(p0.0, p0.1));
            path.arc_to(point(ro, ro), px(0.), big, true, view.p(p1.0, p1.1));
            path.line_to(view.p(p2.0, p2.1));
            path.arc_to(point(ri, ri), px(0.), big, false, view.p(p3.0, p3.1));
            path.close();
        }
        if let Ok(path) = path.build() {
            w.paint_path(path, fill);
        }
    }
    // `<text x="100" y="96" className="cu-chart-total">` and `y="116" cu-chart-axis`.
    let o = view.p(100., 96.);
    paint_text(
        format_number(total).into(),
        TOTAL_FONT * view.scale,
        300,
        colors.total,
        Anchor::Middle,
        o.x,
        o.y,
        w,
        cx,
    );
    let o = view.p(100., 116.);
    paint_text(
        unit,
        AXIS_FONT * view.scale,
        400,
        colors.axis,
        Anchor::Middle,
        o.x,
        o.y,
        w,
        cx,
    );
}

/// A chart. See the module docs.
#[derive(IntoElement)]
pub struct Chart {
    id: ElementId,
    kind: ChartType,
    data: Vec<ChartRow>,
    x_key: SharedString,
    series: Vec<ChartSeries>,
    height: f32,
    title: Option<SharedString>,
}

impl Chart {
    /// An empty bar chart (`xKey = 'label'`, `height = 220`).
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            kind: ChartType::Bar,
            data: Vec::new(),
            x_key: "label".into(),
            series: Vec::new(),
            height: DEFAULT_HEIGHT,
            title: None,
        }
    }

    /// `type`.
    pub fn kind(mut self, kind: ChartType) -> Self {
        self.kind = kind;
        self
    }

    /// `data`.
    pub fn data(mut self, data: Vec<ChartRow>) -> Self {
        self.data = data;
        self
    }

    /// `xKey`: the category field.
    pub fn x_key(mut self, key: impl Into<SharedString>) -> Self {
        self.x_key = key.into();
        self
    }

    /// `series`.
    pub fn series(mut self, series: Vec<ChartSeries>) -> Self {
        self.series = series;
        self
    }

    /// `height`: the viewBox height of bar/line/area charts.
    pub fn height(mut self, height: f32) -> Self {
        self.height = height.max(PAD.t + PAD.b + 1.);
        self
    }

    /// `title`.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }
}

fn rem(px_value: f32) -> gpui_kit::Rems {
    rems(px_value / crate::tokens::REM_PX)
}

/// `.cu-chart-legend i`: a 12px swatch, 4px before the label.
pub fn legend_item(fill: Hsla, label: SharedString) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .child(
            div()
                .flex_none()
                .size(rem(12.))
                .mr(crate::tokens::SPACING_02)
                .bg(fill),
        )
        .child(label)
}

/// `.cu-chart-legend { display: flex; flex-wrap: wrap; gap: spacing-02 spacing-05;
/// font-size: 0.75rem; color: text-secondary }`: the legend row (or column).
pub fn legend(vertical: bool) -> gpui_kit::Div {
    div()
        .flex()
        .when(vertical, |d| d.flex_col().justify_center())
        .when(!vertical, |d| d.flex_row().flex_wrap())
        .gap_x(crate::tokens::SPACING_05)
        .gap_y(crate::tokens::SPACING_02)
        .type_style(TypeStyle::Label01)
        .text_color(color(ColorRole::TextSecondary))
}

fn register_hover(
    hitbox: Hitbox,
    view: View,
    hover: gpui_kit::Entity<Option<ChartHover>>,
    hit: impl Fn((f32, f32)) -> Option<ChartHover> + 'static,
    window: &mut Window,
) {
    window.on_mouse_event(move |e: &MouseMoveEvent, phase, window, cx| {
        if phase != DispatchPhase::Bubble {
            return;
        }
        let target = if hitbox.is_hovered(window) {
            hit(view.local(e.position))
        } else {
            None
        };
        hover.update(cx, |h, cx| {
            if *h != target {
                *h = target;
                cx.notify();
            }
        });
    });
}

impl RenderOnce for Chart {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let hover_state = window.use_keyed_state((self.id.clone(), "hover"), cx, |_, _| None);
        let hover = *hover_state.read(cx);

        // `.cu-chart { padding: spacing-05; background: layer-01; color: text-primary }`.
        let figure = div()
            .p(crate::tokens::SPACING_05)
            .bg(color(ColorRole::Layer01))
            .text_color(color(ColorRole::TextPrimary))
            .flex()
            .flex_col()
            .when_some(self.title.clone(), |d, title| {
                // `.cu-chart-title { font-size: 0.875rem; font-weight: 600; margin-bottom: spacing-03 }`.
                d.child(
                    div()
                        .type_style(TypeStyle::HeadingCompact01)
                        .mb(crate::tokens::SPACING_03)
                        .child(title),
                )
            });

        if self.kind == ChartType::Donut {
            let Some(series) = self.series.first().cloned() else {
                return figure.into_any_element();
            };
            let values: Vec<f64> = self.data.iter().map(|d| d.number(&series.key)).collect();
            let total: f64 = values.iter().sum();
            let segments = donut_segments(&values);
            let colors = Colors::read(self.data.len());
            let titles: Vec<SharedString> = self
                .data
                .iter()
                .map(|d| format!("{}: {}", d.label(&self.x_key), d.label(&series.key)).into())
                .collect();
            let unit = series.display();
            let palette = colors.palette.clone();
            let tip = match hover {
                Some(ChartHover::Segment { index, .. }) => titles.get(index).map(|t| {
                    segment_tip(
                        t.clone(),
                        window.mouse_position(),
                        colors.tip_bg,
                        colors.tip_text,
                    )
                }),
                _ => None,
            };
            let svg = canvas(
                |bounds, window, _| window.insert_hitbox(bounds, HitboxBehavior::Normal),
                move |bounds, hitbox, window, cx| {
                    let view = View::of(bounds, DONUT_SIZE);
                    paint_donut(&segments, total, unit, hover, &colors, view, window, cx);
                    let segments = segments.clone();
                    register_hover(
                        hitbox,
                        view,
                        hover_state,
                        move |p| {
                            donut_hit(&segments, p).map(|index| ChartHover::Segment {
                                index,
                                x: p.0,
                                y: p.1,
                            })
                        },
                        window,
                    );
                },
            )
            .flex_none()
            .size(rem(DONUT_SIZE));
            // `.cu-chart-donut { display: flex; gap: spacing-06; align-items: center }`.
            return figure
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(crate::tokens::SPACING_06)
                        .child(svg)
                        .children(tip)
                        .child(legend(true).children(self.data.iter().enumerate().map(
                            |(i, d)| legend_item(palette[i % palette.len()], d.label(&self.x_key)),
                        ))),
                )
                .into_any_element();
        }

        let values: Vec<Vec<f64>> = self
            .data
            .iter()
            .map(|d| self.series.iter().map(|s| d.number(&s.key)).collect())
            .collect();
        let max = values.iter().flatten().copied().fold(1f64, f64::max);
        let colors = Colors::read(self.series.len());
        let legend = legend(false).mb(crate::tokens::SPACING_03).children(
            self.series
                .iter()
                .enumerate()
                .map(|(i, s)| legend_item(colors.palette[i % colors.palette.len()], s.display())),
        );
        let plot = Rc::new(Plot {
            kind: self.kind,
            geometry: Cartesian::new(self.height, max, self.data.len(), self.series.len()),
            categories: self.data.iter().map(|d| d.label(&self.x_key)).collect(),
            values,
            series: self.series.iter().map(|s| s.display()).collect(),
            hover,
            colors,
        });
        let svg = canvas(
            |bounds, window, _| window.insert_hitbox(bounds, HitboxBehavior::Normal),
            move |bounds, hitbox, window, cx| {
                let view = View::of(bounds, VIEW_WIDTH);
                paint_cartesian(&plot, view, window, cx);
                let plot = plot.clone();
                register_hover(hitbox, view, hover_state, move |p| plot.hit(p), window);
            },
        )
        .w_full()
        .aspect_ratio(VIEW_WIDTH / self.height);
        figure.child(legend).child(svg).into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    #[test]
    fn nice_top_matches_jsx() {
        // `nice = 10 ** floor(log10(max)); top = ceil(max / nice) * nice`.
        assert_eq!(nice_top(60.), 60.);
        assert_eq!(nice_top(61.), 70.);
        assert_eq!(nice_top(0.), 1.);
        assert_eq!(nice_top(123.), 200.);
        assert_eq!(nice_top(7.5), 8.);
    }

    #[test]
    fn cartesian_geometry_matches_jsx() {
        // Preview: 4 months, 2 series, max 60, height 220.
        let g = Cartesian::new(220., 60., 4, 2);
        assert_eq!(g.inner_width(), 508.);
        assert_eq!(g.inner_height(), 180.);
        assert_eq!(g.ticks(), [0., 15., 30., 45., 60.]);
        assert_eq!(g.y(0.), 192.);
        assert_eq!(g.y(60.), 12.);
        assert_eq!(g.baseline(), 192.);
        assert_eq!(g.band(), 127.);
        assert_eq!(g.center(0), 40. + 63.5);
        // bar width 60% of the band split per series, minus 1.
        let b = g.bar(1, 1, 30.);
        let w = 127. * 0.6 / 2.;
        assert!((b.x - (40. + 127. + 127. * 0.2 + w)).abs() < 1e-4);
        assert!((b.w - (w - 1.)).abs() < 1e-4);
        assert_eq!(b.y, 102.);
        assert_eq!(b.h, 90.);
        assert_eq!(g.bar_anchor(1, 1, 30.).0, b.x + w / 2.);
    }

    #[test]
    fn tip_is_clamped() {
        // `translate(min(max(x, 90), W - 90), max(y - 10, 22))`.
        assert_eq!(tip_origin(10., 5., 560.), (90., 22.));
        assert_eq!(tip_origin(555., 100., 560.), (470., 90.));
        assert_eq!(TIP_BOX.w, 172.);
        assert_eq!(TIP_BASELINE, -8.);
    }

    #[test]
    fn donut_segments_and_hit() {
        let s = donut_segments(&[58., 24., 18.]);
        assert_eq!(s[0].0, 0.);
        assert!((s[0].1 - 0.58 * std::f32::consts::TAU).abs() < 1e-5);
        assert!((s[2].1 - std::f32::consts::TAU).abs() < 1e-5);
        // 12 o'clock on the ring hits the first segment; the hole hits nothing.
        assert_eq!(donut_hit(&s, (100., 100. - 66.)), Some(0));
        assert_eq!(donut_hit(&s, (100., 100.)), None);
        // 9 o'clock (3/4 turn) is in the second segment (0.58..0.82).
        assert_eq!(donut_hit(&s, (100. - 66., 100.)), Some(1));
        // Just left of 12 o'clock is in the last one (0.82..1).
        assert_eq!(donut_hit(&s, (99., 100. - 66.)), Some(2));
        assert_eq!(donut_point(0., 80.), (100., 20.));
    }

    #[test]
    fn palette_is_charts_1_to_14_in_order() {
        crate::theme::tests_set(CarbonTheme::White);
        assert_eq!(palette_role(0), ColorRole::Charts1);
        assert_eq!(palette_role(13), ColorRole::Charts14);
        assert_eq!(palette_role(14), ColorRole::Charts1);
        let c = Colors::read(2);
        assert_eq!(c.grid, color(ColorRole::BorderSubtle01));
        assert_eq!(c.baseline, color(ColorRole::BorderStrong01));
        assert_eq!(c.tip_bg, color(ColorRole::BackgroundInverse));
        assert_eq!(c.tip_text, color(ColorRole::TextInverse));
    }

    #[test]
    fn rows_and_numbers() {
        let r = ChartRow::new().text("m", "Jan").num("us", 42.);
        assert_eq!(r.label("m").as_ref(), "Jan");
        assert_eq!(r.number("us"), 42.);
        assert_eq!(r.number("eu"), 0.);
        assert_eq!(format_number(17.5), "17.5");
        assert_eq!(format_number(60.), "60");
        assert_eq!(ChartSeries::new("v").display().as_ref(), "v");
        assert_eq!(ChartType::Area.name(), "area");
    }

    #[test]
    fn donut_tip_opens_at_the_pointer() {
        // Below the pointer, wherever it is: never pinned over the centre total.
        assert_eq!(segment_tip_origin((480., 500.)), (480., 520.));
        assert_eq!(segment_tip_origin((10., 20.)), (10., 40.));
    }

    #[test]
    fn reference_types_match_index_d_ts() {
        let names = ChartType::REFERENCE.map(|t| t.name());
        assert_eq!(names, ["bar", "line", "donut"]);
    }
}
