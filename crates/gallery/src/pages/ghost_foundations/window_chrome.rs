//! Ghost · Foundations › WindowChrome: the macOS frame with title bar,
//! traffic-light inset and sidebar material, focused or not.
//!
//! The sidebar rows are the T3 `ThreadRow` (the Ghost · Shell thread row
//! is drawn by that group).
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, rems,
};
use u0ui::components::{Button, ButtonSize, ButtonVariant};
use u0ui::ghost::foundations::WindowChrome;
use u0ui::prelude::*;
use u0ui::t3::Status;
use u0ui::t3::shell::sidebar::ThreadRow;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| WindowChromePage {
        focused: true,
        sidebar: true,
        lights: true,
    })
    .into()
}

struct WindowChromePage {
    focused: bool,
    sidebar: bool,
    lights: bool,
}

fn sidebar(prefix: &'static str) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .child(
            ThreadRow::new((prefix, 0usize), "Add dark theme tokens")
                .status(Status::Working)
                .elapsed(74)
                .active(true),
        )
        .child(
            ThreadRow::new((prefix, 1usize), "Fix flaky date picker test")
                .status(Status::NeedsYou)
                .time("3m"),
        )
        .child(ThreadRow::new((prefix, 2usize), "Write migration guide").time("1h"))
}

fn window(prefix: &'static str, focused: bool, with_sidebar: bool, lights: bool) -> WindowChrome {
    let mut w = WindowChrome::new("Add dark theme tokens")
        .subtitle("carbon-ui · feat/dark-tokens")
        .height(rems(270. / 16.))
        .focused(focused)
        .traffic_lights(lights)
        .action(
            Button::new((prefix, 10usize), "Toggle sidebar")
                .icon(CarbonIcon::SidePanelOpen)
                .icon_only(true)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm),
        )
        .action(
            Button::new((prefix, 11usize), "Toggle right column")
                .icon(CarbonIcon::RightPanelOpen)
                .icon_only(true)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm),
        )
        .child(
            div()
                .p(tokens::SPACING_06)
                .type_style(TypeStyle::BodyCompact01)
                .text_color(color(ColorRole::TextSecondary))
                .child("Thread content"),
        );
    if with_sidebar {
        w = w.sidebar(sidebar(prefix));
    }
    w
}

/// `components/WindowChrome/preview.html`.
pub fn reference() -> impl IntoElement {
    window("chrome-ref", true, true, true)
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(
            WindowChrome::new("Add dark theme tokens")
                .height(rems(6.))
                .child(div()),
        )
        .into_any_element()
}

impl Render for WindowChromePage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::toggle(
                "chrome-focused",
                "focused",
                self.focused,
                controls::setter(&entity, |this: &mut Self, on| this.focused = on),
            ))
            .child(controls::toggle(
                "chrome-sidebar",
                "sidebar",
                self.sidebar,
                controls::setter(&entity, |this: &mut Self, on| this.sidebar = on),
            ))
            .child(controls::toggle(
                "chrome-lights",
                "trafficLights",
                self.lights,
                controls::setter(&entity, |this: &mut Self, on| this.lights = on),
            ));
        Body::new()
            .playground(div().w_full().p(tokens::SPACING_03).child(window(
                "chrome-live",
                self.focused,
                self.sidebar,
                self.lights,
            )))
            .config(config)
            .example(controls::block("Reference preview", reference()))
    }
}
