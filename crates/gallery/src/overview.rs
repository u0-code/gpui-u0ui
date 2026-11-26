//! The overview, like md3.io's /components/: per navigation group, a grid of
//! Carbon tiles, each a small live specimen of the real component over its
//! name and description. A search field and a section switcher filter it;
//! a tile opens its page.
//!
//! The specimens are real components but inert: a transparent cover over
//! them takes the pointer, so a press anywhere on the tile opens the page.
//! The grid computes its column count from the content width, so it reflows
//! instead of scrolling sideways.
use crate::registry::{self, PAGES, PageDef, Section};
use crate::shell::{self, Gallery, Route};
use gpui_kit::{
    AnyElement, AppContext as _, Context, Entity, InteractiveElement, IntoElement, ParentElement,
    Render, SharedString, StatefulInteractiveElement as _, Styled, Subscription, WeakEntity,
    Window, component::input::InputEvent, div, px, rems,
};
use u0ui::components::empty::{Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle};
use u0ui::components::toggle::ToggleSize;
use u0ui::components::{Card, Input, InputState, ToggleGroup, ToggleGroupItem};
use u0ui::prelude::*;

/// A tile's minimum width (17rem): the specimens are drawn for 15rem.
pub const CARD_MIN_WIDTH: f32 = 272.;
/// The gap between tiles (`spacing-05`).
pub const CARD_GAP: f32 = 16.;
/// The specimen area's height.
pub const THUMBNAIL_HEIGHT: f32 = 160.;

/// Tiles per row for a grid `width` px wide.
pub fn columns(width: f32) -> usize {
    (((width + CARD_GAP) / (CARD_MIN_WIDTH + CARD_GAP)).floor() as usize).max(1)
}

/// The page area's content width for a window `viewport` px wide.
pub fn content_width(viewport: f32) -> f32 {
    let nav = if viewport < shell::NAV_BREAKPOINT {
        0.
    } else {
        u0ui::components::sidebar::WIDTH
    };
    // The page area's 2rem side padding and its 90rem cap.
    (viewport - nav).min(1440.) - 64.
}

/// Whether `page` matches the search `query` (title, id or group).
pub fn matches(page: &PageDef, query: &str) -> bool {
    let query = query.trim().to_lowercase();
    query.is_empty()
        || page.title.to_lowercase().contains(&query)
        || page.id.contains(&query)
        || page.group.to_lowercase().contains(&query)
}

/// The overview's state.
pub struct Overview {
    gallery: WeakEntity<Gallery>,
    query: Entity<InputState>,
    /// Only this section's groups (`None`: all).
    pub section: Option<Section>,
    _subscription: Subscription,
}

impl Overview {
    /// A new overview that navigates `gallery`.
    pub fn new(gallery: WeakEntity<Gallery>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let query = cx.new(|cx| InputState::new(window, cx).placeholder("Find a component"));
        let subscription = cx.subscribe(&query, |_, _, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) {
                cx.notify();
            }
        });
        Self {
            gallery,
            query,
            section: None,
            _subscription: subscription,
        }
    }

    fn card(&self, index: usize, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let page = &PAGES[index];
        let open = {
            let gallery = self.gallery.clone();
            move |cx: &mut gpui_kit::App| {
                let _ = gallery.update(cx, |g, cx| g.navigate(Route::Page(index), cx));
            }
        };
        let specimen = match page.thumbnail {
            Some(draw) => draw(window, cx),
            None => Icon::named(registry::group_icon(page.group))
                .size(IconSize::S32)
                .color(color(ColorRole::IconSecondary))
                .into_any_element(),
        };
        let cover = {
            let open = open.clone();
            div()
                .id(("overview-cover", index))
                .absolute()
                .inset_0()
                .occlude()
                .cursor_pointer()
                .on_click(move |_, _, cx| open(cx))
        };
        let card = Card::new(("overview-card", index))
            .label(page.title)
            .on_click(move |_, _, cx| open(cx))
            .size_full()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(tokens::SPACING_05)
                    .child(
                        // The specimen sits on `background` so its controls
                        // paint their layer-one tokens.
                        div()
                            .relative()
                            .h(px(THUMBNAIL_HEIGHT))
                            .overflow_hidden()
                            .bg(color(ColorRole::Background))
                            .flex()
                            .items_center()
                            .justify_center()
                            .px(tokens::SPACING_05)
                            .child(specimen)
                            .child(cover),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(tokens::SPACING_02)
                            .child(
                                div()
                                    .type_style(TypeStyle::HeadingCompact02)
                                    .text_color(color(ColorRole::TextPrimary))
                                    .child(page.title),
                            )
                            .child(
                                div()
                                    .type_style(TypeStyle::BodyCompact01)
                                    .text_color(color(ColorRole::TextSecondary))
                                    .line_clamp(2)
                                    .child(page.description),
                            ),
                    ),
            );
        // Every column the same width, whatever the tile's padding.
        div()
            .flex_1()
            .min_w_0()
            .flex()
            .child(card)
            .into_any_element()
    }

    fn group(
        &self,
        group: &'static str,
        entries: &[usize],
        columns: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let mut grid = div().flex().flex_col().gap(px(CARD_GAP));
        for chunk in entries.chunks(columns) {
            let mut row = div().flex().flex_row().gap(px(CARD_GAP));
            for index in chunk {
                row = row.child(self.card(*index, window, cx));
            }
            // Keep the last row's tiles in the grid's columns.
            for _ in chunk.len()..columns {
                row = row.child(div().flex_1().min_w_0());
            }
            grid = grid.child(row);
        }
        let count = entries.len();
        div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .items_baseline()
                    .gap(tokens::SPACING_04)
                    .child(
                        div()
                            .type_style(TypeStyle::Heading03)
                            .text_color(color(ColorRole::TextPrimary))
                            .child(group),
                    )
                    .child(
                        div()
                            .type_style(TypeStyle::Label01)
                            .text_color(color(ColorRole::TextSecondary))
                            .child(if count == 1 {
                                SharedString::from("1 page")
                            } else {
                                format!("{count} pages").into()
                            }),
                    ),
            )
            .child(grid)
            .into_any_element()
    }
}

impl Render for Overview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let query = self.query.read(cx).value().to_string();
        let columns = columns(content_width(window.viewport_size().width.into()));
        let mut groups = Vec::new();
        for group in registry::groups() {
            if self.section.is_some_and(|s| Section::of_group(group) != s) {
                continue;
            }
            let entries: Vec<usize> = registry::pages_in(group)
                .filter(|(_, p)| matches(p, &query))
                .map(|(i, _)| i)
                .collect();
            if !entries.is_empty() {
                groups.push((group, entries));
            }
        }
        let total: usize = groups.iter().map(|(_, e)| e.len()).sum();
        let (title, summary) = match self.section {
            None => (
                "Overview",
                "Every page of the gallery: the foundations, each component and the patterns. Open one to try it live.",
            ),
            Some(section) => (section.title(), section.summary()),
        };
        let gallery = self.gallery.clone();
        let filters = ["All", "Foundations", "Components", "Patterns"];
        let mut switcher = ToggleGroup::new("overview-section")
            .size(ToggleSize::Sm)
            .value([match self.section {
                None => "All",
                Some(s) => s.title(),
            }])
            .on_value_change(move |value, _, cx| {
                let section = Section::ALL
                    .into_iter()
                    .find(|s| value.first().is_some_and(|v| v.as_ref() == s.title()));
                let _ = gallery.update(cx, |g, cx| g.navigate(Route::Overview(section), cx));
            });
        for filter in filters {
            switcher = switcher.item(ToggleGroupItem::new(filter).child(filter));
        }
        let empty = groups.is_empty().then(|| {
            let media = EmptyMedia::new();
            let icon = Icon::named(if self.section == Some(Section::Patterns) {
                "dashboard"
            } else {
                "search_off"
            })
            .size(IconSize::S32)
            .color(media.icon_color());
            let (title, text) = if query.trim().is_empty() {
                (
                    "Nothing here yet",
                    "The pattern libraries (Carbon patterns, T3 Code and Ghost) arrive with the next waves.",
                )
            } else {
                (
                    "No page matches",
                    "Try a component name such as Button, or clear the search.",
                )
            };
            Empty::new().child(
                EmptyHeader::new()
                    .child(media.child(icon))
                    .child(EmptyTitle::new(title))
                    .child(EmptyDescription::new(text)),
            )
        });
        let mut body = div().flex().flex_col().gap(tokens::SPACING_09);
        for (group, entries) in &groups {
            body = body.child(self.group(group, entries, columns, window, cx));
        }
        div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_07)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(tokens::SPACING_05)
                    .child(
                        div()
                            .type_style(TypeStyle::Heading05)
                            .text_color(color(ColorRole::TextPrimary))
                            .child(title),
                    )
                    .child(
                        div()
                            .max_w(rems(40.))
                            .type_style(TypeStyle::Body02)
                            .text_color(color(ColorRole::TextSecondary))
                            .child(summary),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .items_end()
                    .gap(tokens::SPACING_05)
                    .child(
                        div().w(rems(20.)).max_w_full().child(
                            Input::new("overview-search", &self.query)
                                .label("Search")
                                .hide_label(true),
                        ),
                    )
                    .child(div().w(rems(32.)).max_w_full().child(switcher))
                    .child(
                        div()
                            .type_style(TypeStyle::Label01)
                            .text_color(color(ColorRole::TextSecondary))
                            .child(format!("{total} of {} pages", PAGES.len())),
                    ),
            )
            .children(empty)
            .child(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_grid_drops_columns_as_it_narrows() {
        assert_eq!(columns(1200.), 4);
        assert_eq!(columns(736.), 2);
        assert_eq!(columns(400.), 1);
        assert_eq!(columns(100.), 1);
    }

    #[test]
    fn search_matches_title_id_and_group() {
        let button = &PAGES[registry::find("button").unwrap()];
        assert!(matches(button, "butt"));
        assert!(matches(button, "ACTIONS"));
        assert!(matches(button, " "));
        assert!(!matches(button, "calendar"));
    }
}
