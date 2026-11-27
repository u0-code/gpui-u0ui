//! T3 · Shell: CommandPalette and ListToolbar pages.
use super::stack;
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, Entity, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, px,
};
use u0ui::prelude::*;
use u0ui::t3::shell::{
    CommandPalette, IdentitySize, ListFilter, ListToolbar, PaletteGroup, PaletteItem,
    ProjectMonogram,
};

fn status_line(text: impl Into<SharedString>) -> impl IntoElement {
    div()
        .type_style(TypeStyle::Label01)
        .text_color(color(ColorRole::TextSecondary))
        .child(text.into())
}

// ── CommandPalette ──────────────────────────────────────────────────────

struct PalettePage {
    selected: SharedString,
    custom_empty: bool,
}

/// Build the CommandPalette page.
pub fn build_command_palette(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| PalettePage {
        selected: "none".into(),
        custom_empty: false,
    })
    .into()
}

/// The reference groups (`components/CommandPalette/preview.html`); each
/// item reports its selection through `pick`.
fn groups<T: 'static>(
    entity: Option<&Entity<T>>,
    pick: fn(&mut T, SharedString),
) -> Vec<PaletteGroup> {
    let item = |title: &'static str| {
        let mut it = PaletteItem::new(title);
        if let Some(e) = entity {
            let e = e.clone();
            it = it.on_select(move |_, cx| {
                e.update(cx, |this, cx| {
                    pick(this, title.into());
                    cx.notify();
                })
            });
        }
        it
    };
    vec![
        PaletteGroup::new(
            "Threads",
            [
                item("Add dark theme tokens")
                    .subtitle(["carbon-ui", "Working"])
                    .icon(CarbonIcon::Chat),
                item("Fix flaky date picker test")
                    .subtitle(["carbon-ui", "3m ago"])
                    .icon(CarbonIcon::Chat),
            ],
        ),
        PaletteGroup::new(
            "Projects",
            [
                item("carbon-ui")
                    .subtitle(["~/Developer/carbon-ui"])
                    .leading(ProjectMonogram::new("carbon-ui").size(IdentitySize::Sm)),
                item("marketing-site")
                    .subtitle(["build-box · ~/sites/marketing"])
                    .leading(ProjectMonogram::new("marketing-site").size(IdentitySize::Sm)),
            ],
        ),
        PaletteGroup::new(
            "Commands",
            [
                item("New thread").icon(CarbonIcon::Add).shortcut("⌘ N"),
                item("Toggle terminal")
                    .icon(CarbonIcon::Terminal)
                    .shortcut("⌘ J"),
                item("Settings").icon(CarbonIcon::Settings).shortcut("⌘ ,"),
            ],
        ),
    ]
}

/// Overview specimen.
pub fn palette_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    let mut palette = CommandPalette::new("thumb-palette").auto_focus(false);
    for g in groups::<()>(None, |_, _| {}).into_iter().take(1) {
        palette = palette.group(g);
    }
    div().w(px(260.)).child(palette).into_any_element()
}

impl Render for PalettePage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let mut palette = CommandPalette::new("live-palette");
        for g in groups(Some(&entity), |this: &mut PalettePage, title| {
            this.selected = title
        }) {
            palette = palette.group(g);
        }
        if self.custom_empty {
            palette = palette.empty("Nothing here. Try another word.");
        }
        let live = stack()
            .w_full()
            .gap(px(16.))
            .child(palette)
            .child(status_line(format!("Selected: {}", self.selected)));
        let config = stack().gap(tokens::SPACING_06).child(controls::toggle(
            "palette-empty",
            "custom empty text",
            self.custom_empty,
            controls::setter(&entity, |this: &mut PalettePage, on| this.custom_empty = on),
        ));
        let mut reference = CommandPalette::new("ref-palette").auto_focus(false);
        for g in groups::<PalettePage>(None, |_, _| {}) {
            reference = reference.group(g);
        }
        Body::new()
            .playground(live)
            .hint("Type to filter; ↑/↓ move, Enter opens, hover highlights")
            .config(config)
            .example(controls::block("Reference preview", reference))
    }
}

// ── ListToolbar ─────────────────────────────────────────────────────────

const VIEWS: [&str; 3] = ["Needs my review", "My drafts", "Failing checks"];

struct ToolbarPage {
    view: SharedString,
    state: bool,
    review: bool,
    views: bool,
    actions: bool,
    action: SharedString,
}

/// Build the ListToolbar page.
pub fn build_list_toolbar(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| ToolbarPage {
        view: "Needs my review".into(),
        state: true,
        review: true,
        views: true,
        actions: false,
        action: "none".into(),
    })
    .into()
}

fn reference_toolbar(id: &'static str) -> ListToolbar {
    ListToolbar::new(id)
        .placeholder("Search pull requests")
        .filter(ListFilter::new("State").value("Open"))
        .filter(ListFilter::new("Author"))
        .filter(ListFilter::new("Review").value("Requested"))
        .filter(ListFilter::new("Checks"))
        .views(VIEWS)
        .view("Needs my review")
}

/// Overview specimen.
pub fn toolbar_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(px(260.))
        .child(
            ListToolbar::new("thumb-listbar")
                .placeholder("Search pull requests")
                .filter(ListFilter::new("State").value("Open"))
                .filter(ListFilter::new("Author")),
        )
        .into_any_element()
}

impl Render for ToolbarPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let clicked = |what: &'static str| {
            let entity = entity.clone();
            move |_: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    this.action = format!("{what} filter").into();
                    cx.notify();
                })
            }
        };
        let mut state = ListFilter::new("State").on_click(clicked("State"));
        if self.state {
            state = state.value("Open");
        }
        let mut review = ListFilter::new("Review").on_click(clicked("Review"));
        if self.review {
            review = review.value("Requested");
        }
        let mut toolbar = ListToolbar::new("live-listbar")
            .placeholder("Search pull requests")
            .filter(state)
            .filter(ListFilter::new("Author").on_click(clicked("Author")))
            .filter(review)
            .filter(ListFilter::new("Checks").on_click(clicked("Checks")));
        if self.views {
            let entity = entity.clone();
            toolbar =
                toolbar
                    .views(VIEWS)
                    .view(self.view.clone())
                    .on_view_change(move |v, _, cx| {
                        entity.update(cx, |this, cx| {
                            this.view = v;
                            cx.notify();
                        })
                    });
        }
        if self.actions {
            toolbar = toolbar.action(
                Button::new("listbar-new", "New pull request")
                    .size(ButtonSize::Md)
                    .icon(CarbonIcon::Add),
            );
        }
        let live = stack()
            .w_full()
            .gap(px(16.))
            .child(toolbar)
            .child(status_line(format!(
                "View: {} · last action: {}",
                self.view, self.action
            )));
        let config = stack()
            .gap(tokens::SPACING_06)
            .child(controls::toggle(
                "listbar-state",
                "State filter set",
                self.state,
                controls::setter(&entity, |this: &mut ToolbarPage, on| this.state = on),
            ))
            .child(controls::toggle(
                "listbar-review",
                "Review filter set",
                self.review,
                controls::setter(&entity, |this: &mut ToolbarPage, on| this.review = on),
            ))
            .child(controls::toggle(
                "listbar-views",
                "views",
                self.views,
                controls::setter(&entity, |this: &mut ToolbarPage, on| this.views = on),
            ))
            .child(controls::toggle(
                "listbar-actions",
                "actions",
                self.actions,
                controls::setter(&entity, |this: &mut ToolbarPage, on| this.actions = on),
            ));
        Body::new()
            .playground(live)
            .hint("Type in the search; pick a saved view")
            .config(config)
            .example(controls::block(
                "Reference preview",
                div().w_full().child(reference_toolbar("ref-listbar")),
            ))
    }
}
