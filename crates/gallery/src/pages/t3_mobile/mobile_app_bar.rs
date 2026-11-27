//! T3 · Mobile › MobileAppBar: the reference bars (thread with subtitle,
//! Share and More; large searchable Threads with Menu and Filter), a live
//! bar, and the same bars in the shared phone chrome (`[data-mb]`).
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, rems,
};
use u0ui::prelude::*;
use u0ui::t3::mobile::{BarAction, Leading, MobileAppBar};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        leading: 0,
        large: false,
        searchable: false,
        subtitle: true,
        actions: 2,
        phone: false,
    })
    .into()
}

struct Page {
    leading: usize,
    large: bool,
    searchable: bool,
    subtitle: bool,
    actions: usize,
    phone: bool,
}

fn framed(bar: impl IntoElement) -> gpui_kit::Div {
    // `width: 340; outline: 1px solid var(--border-subtle-01)`.
    div()
        .w(rems(21.25))
        .border_1()
        .border_color(color(ColorRole::BorderSubtle01))
        .child(bar)
}

fn thread_bar(id: &'static str, phone: bool) -> MobileAppBar {
    MobileAppBar::new(id, "Add dark theme tokens")
        .subtitle("carbon-ui")
        .actions([
            BarAction::new("Share", CarbonIcon::Share),
            BarAction::new("More", CarbonIcon::OverflowMenuHorizontal),
        ])
        .phone(phone)
}

fn list_bar(id: &'static str, phone: bool) -> MobileAppBar {
    MobileAppBar::new(id, "Threads")
        .leading(Leading::Menu)
        .large(true)
        .searchable(true)
        .actions([BarAction::new("Filter", CarbonIcon::Filter)])
        .phone(phone)
}

fn pair(a: impl IntoElement, b: impl IntoElement) -> gpui_kit::Div {
    div()
        .flex()
        .flex_wrap()
        .items_start()
        .gap(tokens::SPACING_05)
        .child(framed(a))
        .child(framed(b))
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(thread_bar("mab-thumb", false))
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let all = [
            BarAction::new("Share", CarbonIcon::Share),
            BarAction::new("More", CarbonIcon::OverflowMenuHorizontal),
        ];
        let mut live = MobileAppBar::new("mab-live", "Add dark theme tokens")
            .leading([Leading::Back, Leading::Menu, Leading::None][self.leading])
            .large(self.large)
            .searchable(self.searchable)
            .actions(all.into_iter().take(self.actions))
            .phone(self.phone);
        if self.subtitle {
            live = live.subtitle("carbon-ui");
        }
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::field(
                "leading",
                controls::choices(
                    "mab-leading",
                    &["back", "menu", "none"],
                    self.leading,
                    controls::setter(&entity, |this: &mut Page, i| this.leading = i),
                ),
            ))
            .child(controls::field(
                "actions",
                controls::choices(
                    "mab-actions",
                    &["0", "1", "2"],
                    self.actions,
                    controls::setter(&entity, |this: &mut Page, i| this.actions = i),
                ),
            ))
            .child(controls::toggle(
                "mab-subtitle",
                "subtitle",
                self.subtitle,
                controls::setter(&entity, |this: &mut Page, v| this.subtitle = v),
            ))
            .child(controls::toggle(
                "mab-large",
                "large",
                self.large,
                controls::setter(&entity, |this: &mut Page, v| this.large = v),
            ))
            .child(controls::toggle(
                "mab-search",
                "searchable",
                self.searchable,
                controls::setter(&entity, |this: &mut Page, v| this.searchable = v),
            ))
            .child(controls::toggle(
                "mab-phone",
                "phone chrome",
                self.phone,
                controls::setter(&entity, |this: &mut Page, v| this.phone = v),
            ));
        Body::new()
            .playground(framed(live))
            .config(config)
            .example(controls::block(
                "Reference preview",
                pair(thread_bar("mab-ref-0", false), list_bar("mab-ref-1", false)),
            ))
            .example(controls::block(
                "Phone chrome ([data-mb])",
                pair(thread_bar("mab-mb-0", true), list_bar("mab-mb-1", true)),
            ))
    }
}
