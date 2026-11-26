//! Gallery chrome controls: a content switcher (`.cds--content-switcher`,
//! size sm) drawn here because the always-dark header paints it in a fixed
//! theme, the config-panel switch (u0ui's [`Switch`]) and labeled fields.
use gpui_kit::{
    App, ClickEvent, CursorStyle, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement,
    SharedString, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder as _,
    rems,
};
use std::rc::Rc;
use u0ui::components::switch::{Switch, SwitchSize};
use u0ui::prelude::*;

/// Which theme a control paints in: the active one (`None`), or a fixed one
/// (the always-dark header).
pub type Palette = Option<CarbonTheme>;

fn paint(palette: Palette, role: ColorRole) -> Hsla {
    match palette {
        None => theme::color(role),
        Some(theme) => role.in_theme(theme),
    }
}

type Select = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// A content switcher: one selected option out of `options`.
pub fn switcher(
    id: impl Into<ElementId>,
    palette: Palette,
    options: &[&'static str],
    selected: usize,
    on_select: impl Fn(usize, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let id: ElementId = id.into();
    let on_select: Select = Rc::new(on_select);
    let p = move |role| paint(palette, role);
    div()
        .flex()
        .flex_row()
        .flex_shrink_0()
        // `.cds--content-switcher-btn`: 1px border-inverse top/bottom, 4px outer radius.
        .border_1()
        .border_color(p(ColorRole::BorderInverse))
        .rounded(rems(0.25))
        .children(options.iter().enumerate().map(|(i, label)| {
            let is_selected = i == selected;
            let on_select = on_select.clone();
            div()
                .id(ElementId::NamedChild(
                    std::sync::Arc::new(id.clone()),
                    SharedString::from(*label),
                ))
                .h(rems(1.875))
                .px(rems(0.75))
                .flex()
                .items_center()
                .whitespace_nowrap()
                .type_style(TypeStyle::BodyCompact01)
                .cursor(CursorStyle::PointingHand)
                .rounded(rems(0.1875))
                .map(|this| {
                    if is_selected {
                        // `.cds--content-switcher--selected`: layer-selected-inverse / text-inverse.
                        this.bg(p(ColorRole::LayerSelectedInverse))
                            .text_color(p(ColorRole::TextInverse))
                    } else {
                        this.text_color(p(ColorRole::TextSecondary)).hover(|s| {
                            s.bg(p(ColorRole::LayerHover01))
                                .text_color(p(ColorRole::TextPrimary))
                        })
                    }
                })
                .on_click(move |_: &ClickEvent, window, cx| on_select(i, window, cx))
                .child(*label)
        }))
}

/// A config-panel switch: u0ui's small [`Switch`] whose state text is the
/// option name (`.cds--toggle__appearance--sm`: 32×16 track).
pub fn toggle(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    on: bool,
    on_change: impl Fn(bool, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let label = label.into();
    Switch::new(id)
        .size(SwitchSize::Sm)
        .checked(on)
        .label_on(label.clone())
        .label_off(label)
        .on_checked_change(on_change)
}

/// A labeled control in the config panel (`label-01` in `text-secondary`, 8px gap).
pub fn field(label: impl Into<SharedString>, control: impl IntoElement) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_03)
        .child(
            div()
                .type_style(TypeStyle::Label01)
                .text_color(color(ColorRole::TextSecondary))
                .child(label.into()),
        )
        .child(control)
}

/// A wrapping row of choices for the config panel, for option lists too long
/// for one switcher row.
pub fn choices(
    id: impl Into<ElementId>,
    options: &[&'static str],
    selected: usize,
    on_select: impl Fn(usize, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let id: ElementId = id.into();
    let on_select: Select = Rc::new(on_select);
    div()
        .flex()
        .flex_row()
        .flex_wrap()
        .gap(tokens::SPACING_02)
        .children(options.iter().enumerate().map(|(i, label)| {
            let on_select = on_select.clone();
            let selected = i == selected;
            div()
                .id(ElementId::NamedChild(
                    std::sync::Arc::new(id.clone()),
                    SharedString::from(*label),
                ))
                .h(rems(1.5))
                .px(rems(0.5))
                .flex()
                .items_center()
                .border_1()
                .type_style(TypeStyle::Label01)
                .cursor(CursorStyle::PointingHand)
                .map(|this| {
                    if selected {
                        this.bg(color(ColorRole::LayerSelectedInverse))
                            .border_color(color(ColorRole::LayerSelectedInverse))
                            .text_color(color(ColorRole::TextInverse))
                    } else {
                        this.border_color(color(ColorRole::BorderStrong01))
                            .text_color(color(ColorRole::TextPrimary))
                            .hover(|s| s.bg(color(ColorRole::LayerHover01)))
                    }
                })
                .on_click(move |_, window, cx| on_select(i, window, cx))
                .child(*label)
        }))
}

/// A section heading inside a page (`heading-compact-01`).
pub fn section(title: impl Into<SharedString>) -> impl IntoElement {
    div()
        .type_style(TypeStyle::HeadingCompact01)
        .text_color(color(ColorRole::TextPrimary))
        .child(title.into())
}

/// The standard page body: a live preview on `background` and, when given, a
/// config panel (`layer-01`). They sit side by side when there is room and
/// stack otherwise (no horizontal scrolling).
pub fn page_body(
    preview: impl IntoElement,
    config: Option<gpui_kit::AnyElement>,
) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_start()
        .gap(tokens::SPACING_06)
        .child(
            div()
                .flex_1()
                .min_w(rems(20.))
                .flex()
                .flex_col()
                .gap(tokens::SPACING_06)
                .child(preview),
        )
        .when_some(config, |this, config| {
            this.child(
                div()
                    .w(rems(18.))
                    .flex_shrink_0()
                    .p(tokens::SPACING_05)
                    .bg(color(ColorRole::Layer01))
                    .flex()
                    .flex_col()
                    .gap(tokens::SPACING_06)
                    .child(config),
            )
        })
}

/// A titled page section: a [`section`] heading over `content`, 16px apart.
pub fn block(title: impl Into<SharedString>, content: impl IntoElement) -> gpui_kit::Div {
    div()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_05)
        .child(section(title))
        .child(content)
}

/// A control callback that updates the page entity with `f` and re-renders:
/// `controls::toggle(.., controls::setter(&entity, |this, on| this.flag = on))`.
pub fn setter<T: 'static, V: 'static>(
    entity: &gpui_kit::Entity<T>,
    f: impl Fn(&mut T, V) + 'static,
) -> impl Fn(V, &mut Window, &mut App) + 'static {
    let entity = entity.clone();
    move |value, _, cx| {
        entity.update(cx, |this, cx| {
            f(this, value);
            cx.notify();
        })
    }
}
