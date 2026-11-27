//! StorageBreakdown: what the app keeps on this Mac and how much each part
//! takes (`StorageBreakdown` in `src/ghost/admin.jsx`, `.g-store` in
//! `src/ghost.css`): the total in large type, a stacked bar colored
//! `charts-1…` in order, and one row per item with its size and a tidy
//! action ("Kept" when there is none).
use crate::{
    components::{Button, ButtonSize, ButtonVariant},
    ghost::rows as row,
    t3::craft::{rem, sans},
    theme::color,
    tokens::{self, ColorRole},
};
use gpui_kit::{
    App, ElementId, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, px, relative,
};
use std::rc::Rc;

/// One stored item (`items: {label, detail, bytes, tidy?}[]`).
#[derive(Clone, Debug, PartialEq)]
pub struct StorageItem {
    /// `label`.
    pub label: SharedString,
    /// `detail`.
    pub detail: SharedString,
    /// `bytes`.
    pub bytes: f64,
    /// `tidy`: the tidy action's label; `None` shows "Kept".
    pub tidy: Option<SharedString>,
}

impl StorageItem {
    /// An item that is kept.
    pub fn new(
        label: impl Into<SharedString>,
        detail: impl Into<SharedString>,
        bytes: f64,
    ) -> Self {
        Self {
            label: label.into(),
            detail: detail.into(),
            bytes,
            tidy: None,
        }
    }

    /// With a tidy action (`tidy`).
    pub fn tidy(mut self, tidy: impl Into<SharedString>) -> Self {
        self.tidy = Some(tidy.into());
        self
    }
}

/// `b >= 1e9 ? (b / 1e9).toFixed(1) + ' GB' : (b / 1e6).toFixed(0) + ' MB'`.
pub fn format_bytes(b: f64) -> String {
    if b >= 1e9 {
        format!("{:.1} GB", b / 1e9)
    } else {
        format!("{:.0} MB", b / 1e6)
    }
}

/// `var(--charts-{k + 1})`: the k-th categorical chart color (14, cycling).
pub fn chart_color(k: usize) -> Hsla {
    const CHARTS: [ColorRole; 14] = [
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
    color(CHARTS[k % CHARTS.len()])
}

/// Each item's share of the bar (`bytes / sum`).
pub fn shares(items: &[StorageItem]) -> Vec<f32> {
    let sum: f64 = items.iter().map(|i| i.bytes).sum();
    items
        .iter()
        .map(|i| if sum > 0. { (i.bytes / sum) as f32 } else { 0. })
        .collect()
}

/// The words after the total, given the free space.
pub type UsedText = Rc<dyn Fn(Option<&str>) -> SharedString>;

/// The breakdown's user-facing strings (the reference's English defaults).
#[derive(Clone)]
pub struct StorageTexts {
    /// After the total: `"used by this app"` + `" of {total} free"`.
    pub used: UsedText,
    /// `"Kept"`.
    pub kept: SharedString,
}

impl Default for StorageTexts {
    fn default() -> Self {
        Self {
            used: Rc::new(|total| match total {
                Some(t) => format!("used by this app of {t} free").into(),
                None => "used by this app".into(),
            }),
            kept: "Kept".into(),
        }
    }
}

type TidyHandler = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;

/// The storage breakdown. See the module docs.
#[derive(IntoElement)]
pub struct StorageBreakdown {
    id: ElementId,
    items: Vec<StorageItem>,
    total: Option<f64>,
    texts: StorageTexts,
    on_tidy: Option<TidyHandler>,
}

impl StorageBreakdown {
    /// A breakdown of `items` (`items`).
    pub fn new(id: impl Into<ElementId>, items: impl IntoIterator<Item = StorageItem>) -> Self {
        Self {
            id: id.into(),
            items: items.into_iter().collect(),
            total: None,
            texts: StorageTexts::default(),
            on_tidy: None,
        }
    }

    /// The disk's free space in bytes (`total`).
    pub fn total(mut self, total: f64) -> Self {
        self.total = Some(total);
        self
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: StorageTexts) -> Self {
        self.texts = texts;
        self
    }

    /// A tidy action was pressed (`onTidy(label)`).
    pub fn on_tidy(mut self, f: impl Fn(SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_tidy = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for StorageBreakdown {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let sum: f64 = self.items.iter().map(|i| i.bytes).sum();
        let total = self.total.map(format_bytes);
        let parts = shares(&self.items);
        // `.g-store-bar { display: flex; height: 0.75rem; gap: 2px }`,
        // `i { min-width: 2px }` with each item's share and chart color.
        let bar =
            div()
                .flex()
                .h(rem(12.))
                .gap(px(2.))
                .w_full()
                .children(parts.iter().enumerate().map(|(k, f)| {
                    div()
                        .h_full()
                        .flex_basis(relative(0.))
                        .min_w(px(2.))
                        .bg(chart_color(k))
                        .map_grow(*f)
                }));
        let rows = self.items.iter().enumerate().map(|(k, it)| {
            let action = match &it.tidy {
                Some(t) => {
                    let f = self.on_tidy.clone();
                    let label = it.label.clone();
                    Button::new(
                        (id.clone(), SharedString::from(format!("tidy-{k}"))),
                        t.clone(),
                    )
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm)
                    .on_click(move |_, w, cx| {
                        if let Some(f) = &f {
                            f(label.clone(), w, cx);
                        }
                    })
                    .into_any_element()
                }
                // `.g-store-keep { width: 4.5rem; text-align: center; font-size: 0.75rem }`.
                None => row::dim(self.texts.kept.clone(), 12.)
                    .w(rem(72.))
                    .flex()
                    .justify_center()
                    .into_any_element(),
            };
            // `.g-store-list li { display: flex; align-items: center; gap:
            // var(--spacing-04); min-height: var(--density-row-lg); padding:
            // var(--spacing-02) var(--spacing-02) var(--spacing-02)
            // var(--spacing-04); background: var(--layer-01); border-bottom: 1px
            // solid var(--border-subtle-01); font-size: 0.875rem }`.
            sans(div(), 14., 400)
                .flex()
                .items_center()
                .gap(tokens::SPACING_04)
                .min_h(crate::density::current().row_lg())
                .py(tokens::SPACING_02)
                .pr(tokens::SPACING_02)
                .pl(tokens::SPACING_04)
                .bg(color(ColorRole::Layer01))
                .border_b_1()
                .border_color(color(ColorRole::BorderSubtle01))
                .text_color(color(ColorRole::TextPrimary))
                // `.g-store-dot { width: 10px; height: 10px; flex: none }`.
                .child(div().flex_none().size(px(10.)).bg(chart_color(k)))
                .child(row::slrow_main(
                    row::title(color(ColorRole::TextPrimary))
                        .child(row::title_text(it.label.clone())),
                    [row::dim(it.detail.clone(), 12.).into_any_element()],
                ))
                .child(
                    div()
                        .flex_none()
                        .whitespace_nowrap()
                        .child(format_bytes(it.bytes)),
                )
                .child(row::tail(action))
        });

        // `.g-store { display: flex; flex-direction: column; gap: var(--spacing-04) }`.
        div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_04)
            .w_full()
            .child(
                // `.g-store-head { display: flex; align-items: baseline; gap:
                // var(--spacing-03) }`, `b { font-size: 1.5rem; font-weight: 400 }`,
                // `.g-dim { font-size: 0.875rem }`.
                div()
                    .flex()
                    .items_baseline()
                    .gap(tokens::SPACING_03)
                    .text_color(color(ColorRole::TextPrimary))
                    .child(sans(div(), 24., 400).child(format_bytes(sum)))
                    .child(row::dim((self.texts.used)(total.as_deref()), 14.)),
            )
            .child(bar)
            .child(div().flex().flex_col().children(rows))
    }
}

/// `flex-grow` by share so the segments split the width like the
/// reference's `width: share%` minus the 2px gaps.
trait MapGrow {
    fn map_grow(self, share: f32) -> Self;
}

impl MapGrow for gpui_kit::Div {
    fn map_grow(mut self, share: f32) -> Self {
        self.style().flex_grow = Some(share.max(0.0001));
        self.style().flex_shrink = Some(1.);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CarbonTheme;

    #[test]
    fn bytes_format_like_the_reference() {
        assert_eq!(format_bytes(820e6), "820 MB");
        assert_eq!(format_bytes(2.1e9), "2.1 GB");
        assert_eq!(format_bytes(95e6), "95 MB");
        assert_eq!(format_bytes(412e9), "412.0 GB");
    }

    #[test]
    fn shares_and_colors_follow_items() {
        let items = [
            StorageItem::new("a", "", 1e9),
            StorageItem::new("b", "", 3e9),
        ];
        assert_eq!(shares(&items), vec![0.25, 0.75]);
        crate::theme::tests_set(CarbonTheme::White);
        assert_eq!(chart_color(0), color(ColorRole::Charts1));
        assert_eq!(chart_color(4), color(ColorRole::Charts5));
    }

    #[test]
    fn used_line_mentions_free_space() {
        let t = StorageTexts::default();
        assert_eq!(
            (t.used)(Some("412.0 GB")),
            SharedString::from("used by this app of 412.0 GB free")
        );
        assert_eq!((t.used)(None), SharedString::from("used by this app"));
    }
}
