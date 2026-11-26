//! The Carbon UI shell: a 48px header (`.cds--header`, always G100 like
//! Carbon's dark shell), a 16rem side nav (`.cds--side-nav`) and the page area.
use crate::controls;
use crate::registry::{self, PAGES};
use gpui_kit::{
    AnyView, ClickEvent, Context, CursorStyle, ElementId, FontWeight, InteractiveElement,
    IntoElement, ParentElement, Render, SharedString, StatefulInteractiveElement, Styled, Window,
    div, prelude::FluentBuilder as _, rems,
};
use std::collections::{HashMap, HashSet};
use u0ui::motion::ReducedMotion;
use u0ui::prelude::*;

/// The header's fixed theme.
const SHELL: CarbonTheme = CarbonTheme::G100;

/// The gallery window's root view.
pub struct Gallery {
    current: usize,
    collapsed: HashSet<&'static str>,
    views: HashMap<&'static str, AnyView>,
    motion: ReducedMotion,
}

impl Gallery {
    /// Open on `page` (an id from the registry), or the first page.
    pub fn new(page: Option<&str>, _: &mut Window, _: &mut Context<Self>) -> Self {
        Self {
            current: page.and_then(registry::find).unwrap_or(0),
            collapsed: HashSet::new(),
            views: HashMap::new(),
            motion: ReducedMotion::System,
        }
    }

    fn view(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyView {
        let page = &PAGES[self.current];
        self.views
            .entry(page.id)
            .or_insert_with(|| (page.build)(window, cx))
            .clone()
    }

    fn header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let p = |role: ColorRole| role.in_theme(SHELL);
        let theme_index = CarbonTheme::ALL
            .iter()
            .position(|t| *t == theme::current())
            .unwrap_or(0);
        let density_index = Density::ALL
            .iter()
            .position(|d| *d == density::current())
            .unwrap_or(0);
        let motion_index = match self.motion {
            ReducedMotion::System => 0,
            ReducedMotion::Reduce => 1,
            ReducedMotion::Allow => 2,
        };
        let entity = cx.entity();
        div()
            // `.cds--header`: 3rem, background, 1px border-subtle bottom.
            .h(rems(3.))
            .flex_shrink_0()
            .flex()
            .flex_row()
            .items_center()
            .gap(tokens::SPACING_05)
            .pl(tokens::SPACING_05)
            .pr(tokens::SPACING_03)
            .bg(p(ColorRole::Background))
            .border_b_1()
            .border_color(p(ColorRole::BorderSubtle01))
            .child(
                // `.cds--header__name`: body-compact-01, 600, prefix 400.
                div()
                    .flex()
                    .flex_row()
                    .gap(rems(0.25))
                    .flex_shrink_0()
                    .type_style(TypeStyle::BodyCompact01)
                    .text_color(p(ColorRole::TextPrimary))
                    .child(div().child("u0ui"))
                    .child(div().font_weight(FontWeight::SEMIBOLD).child("Gallery")),
            )
            .child(div().flex_1())
            .child(controls::switcher(
                "theme",
                Some(SHELL),
                &["White", "G10", "G90", "G100"],
                theme_index,
                |i, _, cx| theme::set_theme(cx, CarbonTheme::ALL[i]),
            ))
            .child(controls::switcher(
                "density",
                Some(SHELL),
                &["Compact", "Comfortable"],
                density_index,
                |i, _, cx| density::set_density(cx, Density::ALL[i]),
            ))
            .child(controls::switcher(
                "motion",
                Some(SHELL),
                &["Motion: system", "Reduce", "Full"],
                motion_index,
                move |i, _, cx| {
                    let preference = [
                        ReducedMotion::System,
                        ReducedMotion::Reduce,
                        ReducedMotion::Allow,
                    ][i];
                    motion::set_reduced_motion(cx, preference);
                    entity.update(cx, |this, cx| {
                        this.motion = preference;
                        cx.notify();
                    });
                },
            ))
    }

    fn side_nav(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut nav = div()
            .id("side-nav")
            // `.cds--side-nav--expanded`: 16rem; `.cds--side-nav__items`: 1rem top padding.
            .w(rems(16.))
            .flex_shrink_0()
            .h_full()
            .overflow_y_scroll()
            .pt(tokens::SPACING_05)
            .bg(color(ColorRole::Background))
            .border_r_1()
            .border_color(color(ColorRole::BorderSubtle01))
            .flex()
            .flex_col();
        for group in registry::groups() {
            let expanded = !self.collapsed.contains(group);
            let rotation = motion::chevron_rotation(
                ElementId::NamedChild(std::sync::Arc::new("nav-chevron".into()), group.into()),
                expanded,
                window,
                cx,
            );
            let entity = cx.entity();
            // `.cds--side-nav__submenu`: heading-compact-01, 2rem, 0 1rem, text-secondary.
            nav = nav.child(
                div()
                    .id(SharedString::from(format!("group-{group}")))
                    .h(rems(2.))
                    .flex_shrink_0()
                    .px(tokens::SPACING_05)
                    .flex()
                    .flex_row()
                    .items_center()
                    .type_style(TypeStyle::HeadingCompact01)
                    .text_color(color(ColorRole::TextSecondary))
                    .cursor(CursorStyle::PointingHand)
                    .hover(|s| {
                        s.bg(color(ColorRole::BackgroundHover))
                            .text_color(color(ColorRole::TextPrimary))
                    })
                    .on_click(move |_: &ClickEvent, _, cx| {
                        entity.update(cx, |this, cx| {
                            if !this.collapsed.remove(group) {
                                this.collapsed.insert(group);
                            }
                            cx.notify();
                        })
                    })
                    .child(div().flex_1().child(group))
                    .child(
                        Icon::carbon(CarbonIcon::ChevronDown)
                            .size(IconSize::S16)
                            .rotate(gpui_kit::radians(rotation)),
                    ),
            );
            if !expanded {
                continue;
            }
            for (index, page) in PAGES.iter().enumerate().filter(|(_, p)| p.group == group) {
                let current = index == self.current;
                let entity = cx.entity();
                // `.cds--side-nav__menu .cds--side-nav__link`: 2rem, 2rem left padding;
                // current: background-selected, text-primary 600, 3px border-interactive bar.
                nav = nav.child(
                    div()
                        .id(page.id)
                        .relative()
                        .h(rems(2.))
                        .flex_shrink_0()
                        .pl(rems(2.))
                        .pr(tokens::SPACING_05)
                        .flex()
                        .items_center()
                        .type_style(TypeStyle::BodyCompact01)
                        .cursor(CursorStyle::PointingHand)
                        .map(|this| {
                            if current {
                                this.bg(color(ColorRole::BackgroundSelected))
                                    .text_color(color(ColorRole::TextPrimary))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(
                                        div()
                                            .absolute()
                                            .left_0()
                                            .top_0()
                                            .bottom_0()
                                            .w(gpui_kit::px(3.))
                                            .bg(color(ColorRole::BorderInteractive)),
                                    )
                            } else {
                                this.text_color(color(ColorRole::TextSecondary)).hover(|s| {
                                    s.bg(color(ColorRole::BackgroundHover))
                                        .text_color(color(ColorRole::TextPrimary))
                                })
                            }
                        })
                        .on_click(move |_: &ClickEvent, _, cx| {
                            entity.update(cx, |this, cx| {
                                this.current = index;
                                cx.notify();
                            })
                        })
                        .child(page.title),
                );
            }
        }
        nav
    }
}

impl Render for Gallery {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let page = &PAGES[self.current];
        let view = self.view(window, cx);
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(color(ColorRole::Background))
            .text_color(color(ColorRole::TextPrimary))
            .type_style(TypeStyle::BodyCompact01)
            .child(self.header(cx))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .flex_row()
                    .child(self.side_nav(window, cx))
                    .child(
                        div()
                            .id("page")
                            .flex_1()
                            .min_w_0()
                            .h_full()
                            .overflow_y_scroll()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(tokens::SPACING_07)
                                    .p(tokens::SPACING_07)
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap(tokens::SPACING_03)
                                            .child(
                                                div()
                                                    .type_style(TypeStyle::Heading04)
                                                    .child(page.title),
                                            )
                                            .child(
                                                div()
                                                    .type_style(TypeStyle::Body01)
                                                    .text_color(color(ColorRole::TextSecondary))
                                                    .child(page.description),
                                            ),
                                    )
                                    .child(view),
                            ),
                    ),
            )
    }
}
