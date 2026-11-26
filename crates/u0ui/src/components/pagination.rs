//! Pagination: Carbon `Pagination` (the bar under a table: items per page,
//! item range, page picker, previous/next) and `PaginationNav` (numbered page
//! buttons) (`.cds--pagination`, `.cds--pagination-nav`).
//!
//! - **Behavior**: page and page-size pickers open the Carbon [`super::menu`]
//!   (keyboard included); previous/next and page buttons are Base `Button`s
//!   (focus, Enter/Space, pointer); disabled ends do nothing. Page state is
//!   controlled or uncontrolled. Kit's `pagination` renders its own rounded
//!   buttons, so only its idea (page items + ellipses) is reused, as the pure
//!   [`nav_slots`].
//! - **Appearance**: the bar is `size-md` on `layer` with a 1px
//!   `border-subtle` top line; texts body-compact-01 (`text-primary`, the item
//!   range `text-secondary`); pickers are `size-md` with a caret, hover
//!   `layer-hover`; arrow buttons are square with a 1px `border-subtle` start
//!   line, hover `layer-hover`, disabled `icon-disabled`, focus a 2px inset
//!   outline. Nav pages are 3rem squares, hover `background-hover`, the active
//!   page 600 weight with a 1rem × 4px `border-interactive` bar.
//! - **Motion**: background 110ms productive standard; the active bar grows
//!   from the center (`transition: width 110ms`), a cut under reduced motion.
use super::menu::{self, MenuItem, MenuSize};
use super::overlay::{self, rem};
use crate::{
    icons::{CarbonIcon, Icon, IconSize},
    motion::{self, Curve, Motion, Speed},
    theme::color,
    tokens::{ColorRole, SIZE_LG, SIZE_MD, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    AnyElement, App, CursorStyle, ElementId, FontWeight, Hsla, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _, px,
};
use std::rc::Rc;

/// `transition: background-color 110ms cubic-bezier(0.2, 0, 0.38, 0.9)`.
pub const BUTTON_MOTION: Motion = Motion::new(Curve::ProductiveStandard, Speed::Fast02);
/// `.cds--pagination-nav__page::after { transition: width 110ms }`.
pub const BAR_MOTION: Motion = Motion::new(Curve::ProductiveStandard, Speed::Fast02).spatial(true);

/// The number of pages for `total` items `size` per page (at least 1).
pub fn page_count(total: usize, size: usize) -> usize {
    total.div_ceil(size.max(1)).max(1)
}

/// The 1-based item range on 1-based `page`: `(first, last)`.
pub fn item_range(page: usize, size: usize, total: usize) -> (usize, usize) {
    let first = ((page.max(1) - 1) * size + 1).min(total.max(1));
    let last = (page.max(1) * size).min(total);
    (if total == 0 { 0 } else { first }, last)
}

/// One slot of a page navigation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NavSlot {
    /// A page (0-based).
    Page(usize),
    /// Hidden pages (0-based), shown as an overflow picker.
    Ellipsis(Vec<usize>),
}

/// The slots for `total` pages around 0-based `page`, using at most `shown`
/// slots (Carbon `itemsShown`, minimum 5): first and last page always, with
/// overflow pickers for the pages in between.
pub fn nav_slots(total: usize, page: usize, shown: usize) -> Vec<NavSlot> {
    let shown = shown.max(5);
    if total <= shown {
        return (0..total).map(NavSlot::Page).collect();
    }
    let last = total - 1;
    let middle = shown - 2;
    if page < middle - 1 {
        // Near the start: 0..middle, …, last.
        let mut out: Vec<_> = (0..middle).map(NavSlot::Page).collect();
        out.push(NavSlot::Ellipsis((middle..last).collect()));
        out.push(NavSlot::Page(last));
        out
    } else if page > last - (middle - 1) {
        let start = total - middle;
        let mut out = vec![NavSlot::Page(0), NavSlot::Ellipsis((1..start).collect())];
        out.extend((start..total).map(NavSlot::Page));
        out
    } else {
        let window = shown - 4;
        let start = page - (window - 1) / 2;
        let end = start + window;
        let mut out = vec![NavSlot::Page(0), NavSlot::Ellipsis((1..start).collect())];
        out.extend((start..end).map(NavSlot::Page));
        out.push(NavSlot::Ellipsis((end..last).collect()));
        out.push(NavSlot::Page(last));
        out
    }
}

type PageChange = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// Texts of the bar (Carbon's English defaults; override per locale).
#[derive(Clone)]
pub struct PaginationText {
    /// `itemsPerPageText`.
    pub items_per_page: SharedString,
    /// `itemRangeText(min, max, total)`.
    pub item_range: Rc<dyn Fn(usize, usize, usize) -> String>,
    /// `pageRangeText(current, total)`.
    pub page_range: Rc<dyn Fn(usize, usize) -> String>,
    /// `backwardText`.
    pub backward: SharedString,
    /// `forwardText`.
    pub forward: SharedString,
}

impl Default for PaginationText {
    fn default() -> Self {
        Self {
            items_per_page: "Items per page:".into(),
            item_range: Rc::new(|a, b, t| format!("{a}–{b} of {t} items")),
            page_range: Rc::new(|_, t| format!("of {t} {}", if t == 1 { "page" } else { "pages" })),
            backward: "Previous page".into(),
            forward: "Next page".into(),
        }
    }
}

/// A square arrow/page button's colors: `(background, icon/text)`.
pub fn button_colors(hovered: bool, disabled: bool) -> (Hsla, Hsla) {
    if disabled {
        // `.cds--pagination__button--no-index { fill: var(--icon-disabled) }`.
        (Hsla::transparent_black(), color(ColorRole::IconDisabled))
    } else if hovered {
        (
            color(ColorRole::LayerHover01),
            color(ColorRole::IconPrimary),
        )
    } else {
        (Hsla::transparent_black(), color(ColorRole::IconPrimary))
    }
}

/// A hover-tracked focusable square used by both components.
#[allow(clippy::too_many_arguments)]
fn square(
    id: ElementId,
    label: SharedString,
    size: f32,
    border_start: bool,
    disabled: bool,
    content: AnyElement,
    on_click: impl Fn(&mut Window, &mut App) + 'static,
    hover_color: Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let hover = window.use_keyed_state((id.clone(), "hover"), cx, |_, _| false);
    let focus = window
        .use_keyed_state((id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
        .read(cx)
        .clone();
    let focused = focus.is_focused(window);
    let hovered = *hover.read(cx) && !disabled;
    let bg = motion::animate_color(
        (id.clone(), "bg"),
        if hovered {
            hover_color
        } else {
            Hsla::transparent_black()
        },
        BUTTON_MOTION,
        window,
        cx,
    );
    gpui_kit::base::Button::new(id)
        .track_focus(&focus)
        .disabled(disabled)
        .accessibility_label(label)
        .relative()
        .flex_shrink_0()
        .min_w(rem(size))
        .h(rem(size))
        .flex()
        .items_center()
        .justify_center()
        .bg(bg)
        .when(border_start && !focused, |this| {
            this.border_l_1()
                .border_color(color(ColorRole::BorderSubtle01))
        })
        .cursor(if disabled {
            CursorStyle::OperationNotAllowed
        } else {
            CursorStyle::PointingHand
        })
        .on_hover(move |h, _, cx| {
            hover.update(cx, |v, cx| {
                *v = *h;
                cx.notify();
            })
        })
        .on_click(move |_, window, cx| on_click(window, cx))
        .child(content)
        .when(focused, |this| {
            this.child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .size_full()
                    .border_2()
                    .border_color(color(ColorRole::Focus)),
            )
        })
        .into_any_element()
}

/// A Carbon inline select in the bar: value + caret, opening a menu of
/// options.
#[allow(clippy::too_many_arguments)]
fn picker(
    id: ElementId,
    value: String,
    options: Vec<String>,
    selected: usize,
    on_pick: PageChange,
    label: SharedString,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let openness = overlay::openness(&id, None, false, None, window, cx);
    let open = openness.open;
    let set_open = openness.setter();
    let hover = window.use_keyed_state((id.clone(), "hover"), cx, |_, _| false);
    let bg = motion::animate_color(
        (id.clone(), "bg"),
        if *hover.read(cx) || open {
            color(ColorRole::LayerHover01)
        } else {
            Hsla::transparent_black()
        },
        BUTTON_MOTION,
        window,
        cx,
    );
    // `.cds--pagination .cds--select-input { padding: 0 2.25rem 0 1rem; line-height: 2.5rem }`.
    let trigger = div()
        .id((id.clone(), "value"))
        .relative()
        .h(SIZE_MD.rems())
        .pl(rem(16.))
        .pr(rem(36.))
        .flex()
        .items_center()
        .bg(bg)
        .cursor(CursorStyle::PointingHand)
        .on_hover(move |h, _, cx| {
            hover.update(cx, |v, cx| {
                *v = *h;
                cx.notify();
            })
        })
        .child(value)
        .child(
            div()
                .absolute()
                .right(rem(12.))
                .top_0()
                .bottom_0()
                .flex()
                .items_center()
                .child(
                    Icon::carbon(CarbonIcon::ChevronDown)
                        .size(IconSize::S16)
                        .color(color(ColorRole::IconPrimary)),
                ),
        )
        .into_any_element();
    let trigger = overlay::trigger_wrapper((id.clone(), "trigger"), trigger, {
        let set_open = set_open.clone();
        Rc::new(move |window, cx| set_open(!open, window, cx))
    })
    .into_any_element();
    let on_pick = on_pick.clone();
    menu::anchored_menu(
        &id,
        trigger,
        menu::MenuSpec {
            items: vec![
                MenuItem::radio_group(label.clone(), options)
                    .selected_item(Some(selected))
                    .on_select_item(move |i, window, cx| on_pick(i, window, cx)),
            ],
            size: MenuSize::Sm,
            label,
            open,
            set_open,
            min_width: None,
            on_unhandled_key: None,
        },
        window,
        cx,
    )
}

/// The Carbon pagination bar. See the module docs.
#[derive(IntoElement)]
pub struct Pagination {
    id: ElementId,
    total_items: usize,
    page: Option<usize>,
    default_page: usize,
    page_size: usize,
    page_sizes: Vec<usize>,
    on_page_change: Option<PageChange>,
    on_page_size_change: Option<PageChange>,
    text: PaginationText,
}

impl Pagination {
    /// A pagination bar with a stable `id` over `total_items`.
    pub fn new(id: impl Into<ElementId>, total_items: usize) -> Self {
        Self {
            id: id.into(),
            total_items,
            page: None,
            default_page: 1,
            page_size: 10,
            page_sizes: vec![10, 20, 30, 40, 50],
            on_page_change: None,
            on_page_size_change: None,
            text: PaginationText::default(),
        }
    }

    /// Controlled 1-based page.
    pub fn page(mut self, page: usize) -> Self {
        self.page = Some(page);
        self
    }

    /// Initial 1-based page when uncontrolled (default 1).
    pub fn default_page(mut self, page: usize) -> Self {
        self.default_page = page;
        self
    }

    /// Initial items per page (default 10).
    pub fn page_size(mut self, size: usize) -> Self {
        self.page_size = size;
        self
    }

    /// The page sizes offered (default 10–50).
    pub fn page_sizes(mut self, sizes: impl IntoIterator<Item = usize>) -> Self {
        self.page_sizes = sizes.into_iter().collect();
        self
    }

    /// Called with the new 1-based page.
    pub fn on_page_change(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_page_change = Some(Rc::new(f));
        self
    }

    /// Called with the new items-per-page when the size picker changes (the
    /// page then resets to 1 through [`Pagination::on_page_change`]).
    pub fn on_page_size_change(
        mut self,
        f: impl Fn(usize, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_page_size_change = Some(Rc::new(f));
        self
    }

    /// Localized texts.
    pub fn text(mut self, text: PaginationText) -> Self {
        self.text = text;
        self
    }
}

struct BarState {
    page: usize,
    size: usize,
}

impl RenderOnce for Pagination {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let (default_page, default_size) = (self.default_page, self.page_size);
        let state = window.use_keyed_state((id.clone(), "state"), cx, |_, _| BarState {
            page: default_page,
            size: default_size,
        });
        let size = state.read(cx).size;
        let pages = page_count(self.total_items, size);
        let page = self.page.unwrap_or(state.read(cx).page).clamp(1, pages);
        let go: PageChange = {
            let state = state.clone();
            let on_change = self.on_page_change.clone();
            Rc::new(move |p, window, cx| {
                state.update(cx, |s, cx| {
                    s.page = p;
                    cx.notify();
                });
                if let Some(f) = &on_change {
                    f(p, window, cx);
                }
            })
        };
        let (first, last) = item_range(page, size, self.total_items);
        let text_primary = color(ColorRole::TextPrimary);

        let sizes = self.page_sizes.clone();
        let size_index = sizes.iter().position(|s| *s == size).unwrap_or(0);
        let size_picker = picker(
            ElementId::NamedChild(std::sync::Arc::new(id.clone()), "size".into()),
            size.to_string(),
            sizes.iter().map(|s| s.to_string()).collect(),
            size_index,
            {
                let state = state.clone();
                let go = go.clone();
                let on_size = self.on_page_size_change.clone();
                Rc::new(move |i, window, cx| {
                    state.update(cx, |s, cx| {
                        s.size = sizes[i];
                        cx.notify();
                    });
                    if let Some(f) = &on_size {
                        f(sizes[i], window, cx);
                    }
                    go(1, window, cx);
                })
            },
            self.text.items_per_page.clone(),
            window,
            cx,
        );
        let page_picker = picker(
            ElementId::NamedChild(std::sync::Arc::new(id.clone()), "page".into()),
            page.to_string(),
            (1..=pages).map(|p| p.to_string()).collect(),
            page - 1,
            {
                let go = go.clone();
                Rc::new(move |i, window, cx| go(i + 1, window, cx))
            },
            (self.text.page_range)(page, pages).into(),
            window,
            cx,
        );
        let arrow = |name: &'static str,
                     icon: CarbonIcon,
                     label: SharedString,
                     target: Option<usize>,
                     window: &mut Window,
                     cx: &mut App| {
            let go = go.clone();
            let (_, fg) = button_colors(false, target.is_none());
            square(
                ElementId::NamedChild(std::sync::Arc::new(id.clone()), name.into()),
                label,
                SIZE_MD.px_value(),
                true,
                target.is_none(),
                Icon::carbon(icon)
                    .size(IconSize::S16)
                    .color(fg)
                    .into_any_element(),
                move |window, cx| {
                    if let Some(p) = target {
                        go(p, window, cx)
                    }
                },
                color(ColorRole::LayerHover01),
                window,
                cx,
            )
        };
        let back = arrow(
            "back",
            CarbonIcon::CaretLeft,
            self.text.backward.clone(),
            (page > 1).then(|| page - 1),
            window,
            cx,
        );
        let forward = arrow(
            "forward",
            CarbonIcon::CaretRight,
            self.text.forward.clone(),
            (page < pages).then(|| page + 1),
            window,
            cx,
        );

        div()
            .w_full()
            .min_h(SIZE_MD.rems())
            .flex()
            .flex_row()
            .flex_wrap()
            .items_center()
            .justify_between()
            .bg(color(ColorRole::Layer01))
            .border_t_1()
            .border_color(color(ColorRole::BorderSubtle01))
            .type_style(TypeStyle::BodyCompact01)
            .text_color(text_primary)
            .child(
                // `.cds--pagination__left { padding: 0 1rem }`.
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .pl(rem(16.))
                    .child(
                        div()
                            .whitespace_nowrap()
                            .child(self.text.items_per_page.clone()),
                    )
                    .child(size_picker)
                    .child(
                        // `span.cds--pagination__text.cds--pagination__items-count`.
                        div()
                            .ml(rem(16.))
                            .whitespace_nowrap()
                            .text_color(color(ColorRole::TextSecondary))
                            .child((self.text.item_range)(first, last, self.total_items)),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .child(page_picker)
                    .child(
                        div()
                            .ml(px(1.))
                            .mr(rem(16.))
                            .whitespace_nowrap()
                            .child((self.text.page_range)(page, pages)),
                    )
                    .child(back)
                    .child(forward),
            )
    }
}

/// Carbon `PaginationNav`: numbered page buttons. See the module docs.
#[derive(IntoElement)]
pub struct PaginationNav {
    id: ElementId,
    total_items: usize,
    page: Option<usize>,
    items_shown: usize,
    on_page_change: Option<PageChange>,
    text: PaginationText,
}

impl PaginationNav {
    /// A page navigation with a stable `id` over `total_items` pages.
    pub fn new(id: impl Into<ElementId>, total_items: usize) -> Self {
        Self {
            id: id.into(),
            total_items,
            page: None,
            items_shown: 7,
            on_page_change: None,
            text: PaginationText::default(),
        }
    }

    /// The 0-based current page (controlled if `on_page_change` updates it;
    /// otherwise the initial page).
    pub fn page(mut self, page: usize) -> Self {
        self.page = Some(page);
        self
    }

    /// Slots shown at most (default 7, minimum 5).
    pub fn items_shown(mut self, shown: usize) -> Self {
        self.items_shown = shown;
        self
    }

    /// Called with the new 0-based page.
    pub fn on_page_change(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_page_change = Some(Rc::new(f));
        self
    }

    /// Localized texts (backward/forward labels).
    pub fn text(mut self, text: PaginationText) -> Self {
        self.text = text;
        self
    }
}

impl RenderOnce for PaginationNav {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let initial = self.page.unwrap_or(0);
        let state = window.use_keyed_state((id.clone(), "page"), cx, |_, _| initial);
        let controlled = self.page.is_some() && self.on_page_change.is_some();
        let total = self.total_items.max(1);
        let page = if controlled {
            self.page.unwrap_or(0)
        } else {
            *state.read(cx)
        }
        .min(total - 1);
        let go: PageChange = {
            let state = state.clone();
            let on_change = self.on_page_change.clone();
            Rc::new(move |p, window, cx| {
                state.update(cx, |s, cx| {
                    *s = p;
                    cx.notify();
                });
                if let Some(f) = &on_change {
                    f(p, window, cx);
                }
            })
        };
        let size = SIZE_LG.px_value();
        let child_id =
            |name: String| ElementId::NamedChild(std::sync::Arc::new(id.clone()), name.into());
        let mut row = div()
            .flex()
            .flex_row()
            .items_center()
            .type_style(TypeStyle::BodyCompact01)
            .text_color(color(ColorRole::TextPrimary));
        let direction = |name: &str,
                         icon: CarbonIcon,
                         label: SharedString,
                         target: Option<usize>,
                         window: &mut Window,
                         cx: &mut App| {
            let go = go.clone();
            let (_, fg) = button_colors(false, target.is_none());
            square(
                child_id(name.to_string()),
                label,
                size,
                false,
                target.is_none(),
                Icon::carbon(icon)
                    .size(IconSize::S16)
                    .color(fg)
                    .into_any_element(),
                move |window, cx| {
                    if let Some(p) = target {
                        go(p, window, cx)
                    }
                },
                color(ColorRole::BackgroundHover),
                window,
                cx,
            )
        };
        row = row.child(direction(
            "back",
            CarbonIcon::CaretLeft,
            self.text.backward.clone(),
            (page > 0).then(|| page - 1),
            window,
            cx,
        ));
        for (slot_index, slot) in nav_slots(total, page, self.items_shown)
            .into_iter()
            .enumerate()
        {
            match slot {
                NavSlot::Page(p) => {
                    let active = p == page;
                    let bar_id = child_id(format!("bar-{p}"));
                    let bar = motion::animate(
                        bar_id,
                        if active { 16f32 } else { 0. },
                        BAR_MOTION,
                        window,
                        cx,
                    );
                    let go = go.clone();
                    let content = div()
                        .relative()
                        .size_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .when(active, |this| this.font_weight(FontWeight::SEMIBOLD))
                        .child((p + 1).to_string())
                        .child(
                            // `::after { block-size: 0.25rem; inline-size: 1rem; inset-inline-start: calc(50% - 0.5rem) }`.
                            div()
                                .absolute()
                                .bottom_0()
                                .left_0()
                                .right_0()
                                .flex()
                                .justify_center()
                                .child(
                                    div()
                                        .w(rem(bar))
                                        .h(rem(4.))
                                        .bg(color(ColorRole::BorderInteractive)),
                                ),
                        )
                        .into_any_element();
                    row = row.child(square(
                        child_id(format!("page-{p}")),
                        (p + 1).to_string().into(),
                        size,
                        false,
                        false,
                        content,
                        move |window, cx| go(p, window, cx),
                        color(ColorRole::BackgroundHover),
                        window,
                        cx,
                    ));
                }
                NavSlot::Ellipsis(hidden) => {
                    let picker_id = child_id(format!("more-{slot_index}"));
                    let go = go.clone();
                    let options: Vec<String> = hidden.iter().map(|p| (p + 1).to_string()).collect();
                    let hidden_pages = hidden.clone();
                    row = row.child(div().w(rem(size)).flex().justify_center().child(picker(
                        picker_id,
                        "…".into(),
                        options,
                        usize::MAX,
                        Rc::new(move |i, window, cx| go(hidden_pages[i], window, cx)),
                        SharedString::default(),
                        window,
                        cx,
                    )));
                }
            }
        }
        row.child(direction(
            "forward",
            CarbonIcon::CaretRight,
            self.text.forward.clone(),
            (page + 1 < total).then(|| page + 1),
            window,
            cx,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use NavSlot::*;

    #[test]
    fn ranges_follow_carbon_pagination() {
        assert_eq!(page_count(103, 10), 11);
        assert_eq!(page_count(0, 10), 1);
        assert_eq!(item_range(2, 10, 103), (11, 20));
        assert_eq!(item_range(11, 10, 103), (101, 103));
        let text = PaginationText::default();
        assert_eq!((text.item_range)(11, 20, 103), "11–20 of 103 items");
        assert_eq!((text.page_range)(2, 11), "of 11 pages");
    }

    #[test]
    fn nav_slots_keep_first_and_last() {
        assert_eq!(nav_slots(5, 2, 7), (0..5).map(Page).collect::<Vec<_>>());
        // The reference: 12 pages, page index 3, 7 shown.
        assert_eq!(
            nav_slots(12, 3, 7),
            vec![
                Page(0),
                Page(1),
                Page(2),
                Page(3),
                Page(4),
                Ellipsis((5..11).collect()),
                Page(11)
            ]
        );
        assert_eq!(
            nav_slots(12, 6, 7),
            vec![
                Page(0),
                Ellipsis(vec![1, 2, 3, 4]),
                Page(5),
                Page(6),
                Page(7),
                Ellipsis(vec![8, 9, 10]),
                Page(11)
            ]
        );
        assert_eq!(
            nav_slots(12, 10, 7),
            vec![
                Page(0),
                Ellipsis((1..7).collect()),
                Page(7),
                Page(8),
                Page(9),
                Page(10),
                Page(11)
            ]
        );
        for page in 0..12 {
            assert_eq!(nav_slots(12, page, 7).len(), 7);
        }
    }

    #[test]
    fn arrow_colors() {
        crate::theme::tests_set(crate::tokens::CarbonTheme::White);
        let (bg, fg) = button_colors(true, false);
        assert_eq!(
            (bg, fg),
            (
                color(ColorRole::LayerHover01),
                color(ColorRole::IconPrimary)
            )
        );
        assert_eq!(button_colors(true, true).1, color(ColorRole::IconDisabled));
    }
}
