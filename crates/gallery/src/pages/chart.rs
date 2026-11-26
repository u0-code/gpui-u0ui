//! Data display › Chart: the reference "Requests by region" bar chart and
//! "Storage by class" donut, and a live chart with every `ChartProps` prop.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled, Window,
    div, rems,
};
use u0ui::components::chart::{Chart, ChartRow, ChartSeries, ChartType};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| ChartPage {
        kind: 0,
        series: 1,
        height: 1,
        title: true,
        months: 1,
    })
    .into()
}

struct ChartPage {
    kind: usize,
    series: usize,
    height: usize,
    title: bool,
    months: usize,
}

const HEIGHTS: [(&str, f32); 3] = [("160", 160.), ("220", 220.), ("300", 300.)];
const SERIES: [&str; 3] = ["1 (us)", "2 (us, eu)", "3 (us, eu, ap)"];
const MONTHS: [(&str, usize); 3] = [("2", 2), ("4", 4), ("8", 8)];

/// The reference bar data: `[{m:'Jan',us:42,eu:30}, …]`.
fn regions() -> Vec<ChartRow> {
    [
        ("Jan", 42., 30.),
        ("Feb", 51., 38.),
        ("Mar", 47., 45.),
        ("Apr", 60., 52.),
    ]
    .map(|(m, us, eu)| ChartRow::new().text("m", m).num("us", us).num("eu", eu))
    .to_vec()
}

/// A longer series for the live chart.
fn monthly(n: usize) -> Vec<ChartRow> {
    const NAMES: [&str; 8] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug"];
    const US: [f64; 8] = [42., 51., 47., 60., 66., 58., 71., 77.];
    const EU: [f64; 8] = [30., 38., 45., 52., 49., 55., 61., 64.];
    const AP: [f64; 8] = [12., 18., 26., 24., 33., 41., 38., 47.];
    (0..n)
        .map(|i| {
            ChartRow::new()
                .text("m", NAMES[i])
                .num("us", US[i])
                .num("eu", EU[i])
                .num("ap", AP[i])
        })
        .collect()
}

/// The reference donut data: `[{c:'Standard',v:58},{c:'Vault',v:24},{c:'Cold',v:18}]`.
fn storage() -> Vec<ChartRow> {
    [("Standard", 58.), ("Vault", 24.), ("Cold", 18.)]
        .map(|(c, v)| ChartRow::new().text("c", c).num("v", v))
        .to_vec()
}

impl Render for ChartPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();

        // `components/Chart/preview.html`: a `cu-row` (16px gap, top-aligned)
        // with the bar chart in a 520px box and the donut beside it.
        let reference = div()
            .flex()
            .flex_row()
            .flex_wrap()
            .items_start()
            .gap(tokens::SPACING_05)
            .child(
                div().w(rems(32.5)).max_w_full().child(
                    Chart::new("ref-chart-bar")
                        .title("Requests by region")
                        .x_key("m")
                        .data(regions())
                        .series(vec![
                            ChartSeries::new("us").label("US"),
                            ChartSeries::new("eu").label("EU"),
                        ]),
                ),
            )
            .child(
                Chart::new("ref-chart-donut")
                    .kind(ChartType::Donut)
                    .title("Storage by class")
                    .x_key("c")
                    .data(storage())
                    .series(vec![ChartSeries::new("v").label("TB")]),
            );

        let kind = ChartType::ALL[self.kind];
        let all = [
            ChartSeries::new("us").label("US"),
            ChartSeries::new("eu").label("EU"),
            ChartSeries::new("ap").label("AP"),
        ];
        let mut live = Chart::new(("live-chart", self.kind))
            .kind(kind)
            .height(HEIGHTS[self.height].1);
        if self.title {
            live = live.title(match kind {
                ChartType::Donut => "Storage by class",
                _ => "Requests by region",
            });
        }
        live = if kind == ChartType::Donut {
            live.x_key("c")
                .data(storage())
                .series(vec![ChartSeries::new("v").label("TB")])
        } else {
            live.x_key("m")
                .data(monthly(MONTHS[self.months].1))
                .series(all[..=self.series].to_vec())
        };

        let kinds = ChartType::ALL.map(|k| k.name());
        let heights = HEIGHTS.map(|h| h.0);
        let months = MONTHS.map(|m| m.0);
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "type",
                controls::choices(
                    "chart-type",
                    &kinds,
                    self.kind,
                    controls::setter(&entity, |this: &mut ChartPage, i| this.kind = i),
                ),
            ))
            .child(controls::field(
                "series",
                controls::choices(
                    "chart-series",
                    &SERIES,
                    self.series,
                    controls::setter(&entity, |this: &mut ChartPage, i| this.series = i),
                ),
            ))
            .child(controls::field(
                "data (categories)",
                controls::choices(
                    "chart-months",
                    &months,
                    self.months,
                    controls::setter(&entity, |this: &mut ChartPage, i| this.months = i),
                ),
            ))
            .child(controls::field(
                "height",
                controls::choices(
                    "chart-height",
                    &heights,
                    self.height,
                    controls::setter(&entity, |this: &mut ChartPage, i| this.height = i),
                ),
            ))
            .child(controls::toggle(
                "chart-title",
                "title",
                self.title,
                controls::setter(&entity, |this: &mut ChartPage, on| this.title = on),
            ));

        Body::new()
            .playground(div().max_w(rems(40.)).child(live))
            .config(config)
            .example(controls::block("Reference preview", reference))
    }
}
