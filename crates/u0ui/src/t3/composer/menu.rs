//! [`ComposerCommandMenu`] (`.t3-cmdmenu`, "Command menu" in `t3.css`): the
//! `/` and `@` suggestion list, grouped, with one active row and an optional
//! source badge per skill.
//!
//! The reference is presentational (`activeIndex` comes from the composer,
//! which owns the keys), so this draws the list and reports pointer picks
//! through [`ComposerCommandMenu::on_select`] when given. When the active
//! row changes it is scrolled into view (as a browser keeps the
//! `aria-selected` option visible while arrowing).
use crate::components::overlay::floating_shadow;
use crate::t3::press::rem;
use crate::{
    icons::{CarbonIcon, Icon},
    theme::color,
    tokens::{ColorRole, FontFamilyKey},
    typography::font,
};
use gpui_kit::{
    AnyElement, App, CursorStyle, ElementId, InteractiveElement as _, IntoElement, ParentElement,
    RenderOnce, ScrollHandle, SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _, px,
};
use std::cell::Cell;
use std::rc::Rc;

/// One suggestion (`{ title, description, icon, source }`).
#[derive(Clone)]
pub struct CommandMenuItem {
    /// `title`.
    pub title: SharedString,
    /// `description`.
    pub description: Option<SharedString>,
    /// `icon` (default `Document`).
    pub icon: Icon,
    /// `source`: the skill's origin badge ("Project Skill").
    pub source: Option<SharedString>,
}

impl CommandMenuItem {
    /// An item titled `title` with the default `Document` icon.
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            description: None,
            icon: Icon::carbon(CarbonIcon::Document),
            source: None,
        }
    }

    /// `description`.
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// `icon`.
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = icon.into();
        self
    }

    /// `source`.
    pub fn source(mut self, source: impl Into<SharedString>) -> Self {
        self.source = Some(source.into());
        self
    }
}

/// A labelled group (`{ label, items }`).
#[derive(Clone)]
pub struct CommandMenuSection {
    /// `label`.
    pub label: SharedString,
    /// `items`.
    pub items: Vec<CommandMenuItem>,
}

impl CommandMenuSection {
    /// A group.
    pub fn new(label: impl Into<SharedString>, items: Vec<CommandMenuItem>) -> Self {
        Self {
            label: label.into(),
            items,
        }
    }
}

/// `.t3-cmdmenu { max-width: 28rem; max-height: 20rem }` in px.
pub const COMMAND_MENU_SIZE: (f32, f32) = (448., 320.);

type SelectHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// The grouped suggestion menu. See the module docs.
#[derive(IntoElement)]
pub struct ComposerCommandMenu {
    id: ElementId,
    sections: Vec<CommandMenuSection>,
    active_index: usize,
    query: Option<SharedString>,
    on_select: Option<SelectHandler>,
}

impl ComposerCommandMenu {
    /// A menu of `sections`.
    pub fn new(id: impl Into<ElementId>, sections: Vec<CommandMenuSection>) -> Self {
        Self {
            id: id.into(),
            sections,
            active_index: 0,
            query: None,
            on_select: None,
        }
    }

    /// `activeIndex`: the highlighted row, counted across sections.
    pub fn active_index(mut self, index: usize) -> Self {
        self.active_index = index;
        self
    }

    /// `query`: the typed trigger ("/rev"), shown mono above the list.
    pub fn query(mut self, query: impl Into<SharedString>) -> Self {
        self.query = Some(query.into());
        self
    }

    /// A pointer pick of row `i` (counted across sections).
    pub fn on_select(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for ComposerCommandMenu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let secondary = color(ColorRole::TextSecondary);
        // The scroll position and the active row it was last scrolled to.
        let (scroll, scrolled_to) = window
            .use_keyed_state((self.id.clone(), "scroll"), cx, |_, _| {
                (ScrollHandle::new(), Rc::new(Cell::new(None::<usize>)))
            })
            .read(cx)
            .clone();
        // Headers and rows are the scroller's direct children (the groups are
        // flattened) so `scroll_to_item` can reach the active row.
        let mut children: Vec<AnyElement> = Vec::new();
        let mut active_child = None;
        let mut index = 0usize;
        for section in self.sections {
            // `.t3-cmdmenu-h { padding: spacing-04 spacing-04 spacing-02; font-size: 0.75rem; color: text-secondary }`.
            children.push(
                div()
                    .flex_none()
                    .pt(rem(12.))
                    .px(rem(12.))
                    .pb(rem(4.))
                    .text_size(rem(12.))
                    .line_height(rem(16.))
                    .text_color(secondary)
                    .child(section.label)
                    .into_any_element(),
            );
            for item in section.items {
                let i = index;
                index += 1;
                let on = i == self.active_index;
                let on_select = self.on_select.clone();
                if on {
                    active_child = Some(children.len());
                }
                children.push(
                    // `.t3-cmdmenu-item { display: flex; align-items: center; gap: spacing-03;
                    // min-height: size-sm; padding: spacing-02 spacing-04; font-size: 0.875rem }`.
                    div()
                        .id((self.id.clone(), SharedString::from(format!("item-{i}"))))
                        .relative()
                        .flex_none()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(rem(8.))
                        .min_h(rem(32.))
                        .px(rem(12.))
                        .py(rem(4.))
                        .text_size(rem(14.))
                        .line_height(rem(18.))
                        .text_color(color(ColorRole::TextPrimary))
                        .cursor(CursorStyle::PointingHand)
                        // `.t3-cmdmenu-item--on { background: layer-selected-01 }` and the
                        // later `{ box-shadow: inset 2px 0 0 border-interactive }`.
                        .when(on, |this| {
                            this.bg(color(ColorRole::LayerSelected01)).child(
                                div()
                                    .absolute()
                                    .top_0()
                                    .bottom_0()
                                    .left_0()
                                    .w(px(2.))
                                    .bg(color(ColorRole::BorderInteractive)),
                            )
                        })
                        .when_some(on_select, |this, f| {
                            this.on_click(move |_, window, cx| f(i, window, cx))
                        })
                        // `.t3-cmdmenu-item svg { fill: icon-secondary; flex: none }`.
                        .child(
                            item.icon
                                .size_length(rem(16.))
                                .color(color(ColorRole::IconSecondary)),
                        )
                        .child(div().flex_none().whitespace_nowrap().child(item.title))
                        .when_some(item.description, |this, d| {
                            // `.t3-cmdmenu-desc { flex: 1; text-overflow: ellipsis; font-size: 0.75rem; color: text-secondary }`.
                            this.child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .overflow_hidden()
                                    .whitespace_nowrap()
                                    .text_ellipsis()
                                    .text_size(rem(12.))
                                    .line_height(rem(16.))
                                    .text_color(secondary)
                                    .child(d),
                            )
                        })
                        .when_some(item.source, |this, source| {
                            // `.t3-srcbadge { margin-left: auto; padding: 0 spacing-02; font-size: 0.6875rem;
                            // line-height: 1rem; background: layer-accent-01; color: text-secondary;
                            // border-radius: border-radius-04 }`.
                            this.child(
                                div()
                                    .ml_auto()
                                    .flex_none()
                                    .px(rem(4.))
                                    .text_size(rem(11.))
                                    .line_height(rem(16.))
                                    .bg(color(ColorRole::LayerAccent01))
                                    .text_color(secondary)
                                    .rounded(px(4.))
                                    .whitespace_nowrap()
                                    .child(source),
                            )
                        })
                        .into_any_element(),
                );
            }
        }
        let has_query = self.query.is_some();
        if let Some(child) = active_child
            && scrolled_to.get() != Some(self.active_index)
        {
            scrolled_to.set(Some(self.active_index));
            scroll.scroll_to_item(child + usize::from(has_query));
        }
        // `.t3-cmdmenu { width: 100%; max-width: 28rem; max-height: 20rem; overflow-y: auto;
        // padding-bottom: spacing-03; background: layer-01; box-shadow: 0 2px 6px shadow }`.
        div()
            .id(self.id.clone())
            .flex()
            .flex_col()
            .w_full()
            .max_w(rem(COMMAND_MENU_SIZE.0))
            .max_h(rem(COMMAND_MENU_SIZE.1))
            .overflow_y_scroll()
            .track_scroll(&scroll)
            .pb(rem(8.))
            .bg(color(ColorRole::Layer01))
            .shadow(floating_shadow())
            .when_some(self.query, |this, q| {
                // `.t3-cmdmenu-q { padding: spacing-03 spacing-04; border-bottom: 1px solid border-subtle-01;
                // font-family: mono; font-size: 0.75rem; color: text-secondary }`.
                this.child(
                    div()
                        .flex_none()
                        .px(rem(12.))
                        .py(rem(8.))
                        .border_b_1()
                        .border_color(color(ColorRole::BorderSubtle01))
                        .font(font(FontFamilyKey::Mono, 400))
                        .text_size(rem(12.))
                        .line_height(rem(16.))
                        .text_color(secondary)
                        .child(q),
                )
            })
            .children(children)
    }
}
