//! T3 · Composer › ComposerCommandMenu: the `/rev` suggestions with
//! Commands, Skills (source badges), Files and folders and Pull requests
//! (`components/ComposerCommandMenu/preview.html`). In the playground the
//! page plays the composer's part: ↑/↓ move the active row, a click picks one.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, FocusHandle, InteractiveElement as _,
    IntoElement, KeyDownEvent, ParentElement, Render, SharedString, Styled, Window, div,
};
use u0ui::prelude::*;
use u0ui::t3::composer::{CommandMenuItem, CommandMenuSection, ComposerCommandMenu};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|cx| CommandMenuPage {
        active: 0,
        query: true,
        picked: "Focus the menu, then use ↑ and ↓.".into(),
        focus: cx.focus_handle(),
    })
    .into()
}

struct CommandMenuPage {
    active: usize,
    query: bool,
    picked: SharedString,
    focus: FocusHandle,
}

/// The preview's sections.
pub fn sections() -> Vec<CommandMenuSection> {
    vec![
        CommandMenuSection::new(
            "Commands",
            vec![
                CommandMenuItem::new("/review")
                    .description("Review the current diff")
                    .icon(CarbonIcon::View),
                CommandMenuItem::new("/compact")
                    .description("Summarize and free context")
                    .icon(CarbonIcon::Subtract),
            ],
        ),
        CommandMenuSection::new(
            "Skills",
            vec![
                CommandMenuItem::new("release-notes")
                    .description("Draft release notes from merged PRs")
                    .icon(CarbonIcon::MagicWand)
                    .source("Project Skill"),
                CommandMenuItem::new("review-checklist")
                    .description("Team review checklist")
                    .icon(CarbonIcon::MagicWand)
                    .source("Personal Skill"),
            ],
        ),
        CommandMenuSection::new(
            "Files and folders",
            vec![
                CommandMenuItem::new("src/review/").icon(CarbonIcon::Folder),
                CommandMenuItem::new("src/review/Reviewer.tsx").icon(CarbonIcon::Document),
            ],
        ),
        CommandMenuSection::new(
            "Pull requests",
            vec![CommandMenuItem::new("#482 Dark theme tokens").icon(CarbonIcon::PullRequest)],
        ),
    ]
}

fn titles() -> Vec<SharedString> {
    sections()
        .into_iter()
        .flat_map(|s| s.items.into_iter().map(|i| i.title))
        .collect()
}

impl Render for CommandMenuPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let count = titles().len();
        let mut menu = ComposerCommandMenu::new("cmdmenu-live", sections())
            .active_index(self.active)
            .on_select({
                let entity = entity.clone();
                move |i, _, cx| {
                    entity.update(cx, |this, cx| {
                        this.active = i;
                        this.picked = format!("Picked {}", titles()[i]).into();
                        cx.notify();
                    })
                }
            });
        if self.query {
            menu = menu.query("/rev");
        }
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .w_full()
            .child(
                div()
                    .id("cmdmenu-keys")
                    .track_focus(&self.focus)
                    .w_full()
                    .on_key_down(cx.listener(move |this, e: &KeyDownEvent, _, cx| {
                        match e.keystroke.key.as_str() {
                            "down" => this.active = (this.active + 1) % count,
                            "up" => this.active = (this.active + count - 1) % count,
                            "enter" => {
                                this.picked = format!("Picked {}", titles()[this.active]).into()
                            }
                            _ => return,
                        }
                        cx.stop_propagation();
                        cx.notify();
                    }))
                    .on_mouse_down(gpui_kit::MouseButton::Left, {
                        let focus = self.focus.clone();
                        move |_, window, cx| window.focus(&focus, cx)
                    })
                    .child(menu),
            )
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(self.picked.clone()),
            );
        let indices: Vec<&'static str> = ["0", "1", "2", "3", "4", "5", "6"]
            .into_iter()
            .take(count)
            .collect();
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "activeIndex",
                controls::choices(
                    "cmdmenu-active",
                    &indices,
                    self.active,
                    controls::setter(&entity, |this: &mut Self, i| this.active = i),
                ),
            ))
            .child(controls::toggle(
                "cmdmenu-query",
                "query",
                self.query,
                controls::setter(&entity, |this: &mut Self, on| this.query = on),
            ));
        Body::new()
            .playground(live)
            .hint("Click the menu, then ↑/↓ move the active row and Enter picks it.")
            .config(config)
            .example(controls::block(
                "Reference preview",
                ComposerCommandMenu::new("cmdmenu-ref", sections()).query("/rev"),
            ))
    }
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(gpui_kit::rems(15.))
        .child(ComposerCommandMenu::new(
            "cmdmenu-th",
            sections().into_iter().take(1).collect(),
        ))
        .into_any_element()
}
