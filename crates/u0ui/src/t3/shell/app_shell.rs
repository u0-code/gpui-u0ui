//! The T3 application frame (`shell.jsx` "App shell"): [`AppShell`].
use super::super::press::rem;
use crate::{theme::color, tokens::ColorRole};
use gpui_kit::{
    AnyElement, App, ElementId, InteractiveElement as _, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement as _, Styled, Window, div, prelude::FluentBuilder as _,
};

/// `.t3-shell-sidebar { width: 16rem }`.
pub const SIDEBAR_WIDTH: f32 = 256.;
/// `.t3-shell-right { width: 20rem }`.
pub const RIGHT_WIDTH: f32 = 320.;
/// `.t3-shell-terminal { height: 9rem }`.
pub const TERMINAL_HEIGHT: f32 = 144.;
/// `AppShell` `height` default.
pub const DEFAULT_HEIGHT: f32 = 520.;

/// The application frame (`AppShell`, `.t3-shell`): a `background` box
/// with a 1px `border-subtle-01` inner outline holding the 16rem
/// `layer-01` sidebar (scrolling, right rule), the main column (header,
/// the scrolling body, a 9rem terminal drawer over a top rule) and the
/// 20rem `layer-01` right panel (left rule). Every part but the body is
/// optional.
#[derive(IntoElement)]
pub struct AppShell {
    id: ElementId,
    sidebar: Option<AnyElement>,
    header: Option<AnyElement>,
    body: Vec<AnyElement>,
    right_panel: Option<AnyElement>,
    terminal: Option<AnyElement>,
    height: Option<f32>,
}

impl AppShell {
    /// An empty shell.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            sidebar: None,
            header: None,
            body: Vec::new(),
            right_panel: None,
            terminal: None,
            height: Some(DEFAULT_HEIGHT),
        }
    }

    /// The sidebar (`sidebar`).
    pub fn sidebar(mut self, sidebar: impl IntoElement) -> Self {
        self.sidebar = Some(sidebar.into_any_element());
        self
    }

    /// The main column's header (`header`).
    pub fn header(mut self, header: impl IntoElement) -> Self {
        self.header = Some(header.into_any_element());
        self
    }

    /// The right panel (`rightPanel`).
    pub fn right_panel(mut self, panel: impl IntoElement) -> Self {
        self.right_panel = Some(panel.into_any_element());
        self
    }

    /// The terminal drawer (`terminal`).
    pub fn terminal(mut self, terminal: impl IntoElement) -> Self {
        self.terminal = Some(terminal.into_any_element());
        self
    }

    /// The height in px (`height`, default 520); `None` fills the parent.
    pub fn height(mut self, height: Option<f32>) -> Self {
        self.height = height;
        self
    }
}

impl ParentElement for AppShell {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.body.extend(elements);
    }
}

impl RenderOnce for AppShell {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let id = self.id;
        let rule = color(ColorRole::BorderSubtle01);
        let sidebar = self.sidebar.map(|s| {
            // `.t3-shell-sidebar { flex: none; width: 16rem; overflow-y: auto;
            // background: var(--layer-01); border-right: 1px solid var(--border-subtle-01) }`.
            div()
                .id((id.clone(), "sidebar"))
                .flex()
                .flex_col()
                .flex_none()
                .w(rem(SIDEBAR_WIDTH))
                .overflow_y_scroll()
                .bg(color(ColorRole::Layer01))
                .border_r_1()
                .border_color(rule)
                .child(s)
        });
        let main = div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            .overflow_hidden()
            .children(self.header.map(|h| div().flex_none().child(h)))
            .child(
                // `.t3-shell-body { flex: 1; min-height: 0; overflow-y: auto }`.
                div()
                    .id((id.clone(), "body"))
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .children(self.body),
            )
            .children(self.terminal.map(|t| {
                // `.t3-shell-terminal { flex: none; height: 9rem; border-top }`.
                div()
                    .flex()
                    .flex_col()
                    .flex_none()
                    .h(rem(TERMINAL_HEIGHT))
                    .overflow_hidden()
                    .border_t_1()
                    .border_color(rule)
                    .child(t)
            }));
        let right = self.right_panel.map(|r| {
            // `.t3-shell-right { display: flex; flex: none; width: 20rem; flex-direction:
            // column; background: var(--layer-01); border-left }`.
            div()
                .flex()
                .flex_col()
                .flex_none()
                .w(rem(RIGHT_WIDTH))
                .bg(color(ColorRole::Layer01))
                .border_l_1()
                .border_color(rule)
                .child(r)
        });
        // `.t3-shell { display: flex; overflow: hidden; background: var(--background);
        // outline: 1px solid var(--border-subtle-01); outline-offset: -1px }`.
        div()
            .relative()
            .flex()
            .w_full()
            .map(|d| match self.height {
                Some(h) => d.h(rem(h)),
                None => d.h_full(),
            })
            .overflow_hidden()
            .bg(color(ColorRole::Background))
            .children(sidebar)
            .child(main)
            .children(right)
            .child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .bottom_0()
                    .border_1()
                    .border_color(rule),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_geometry_follows_t3_css() {
        assert_eq!(
            (SIDEBAR_WIDTH, RIGHT_WIDTH, TERMINAL_HEIGHT, DEFAULT_HEIGHT),
            (256., 320., 144., 520.)
        );
    }
}
