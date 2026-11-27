//! Ghost · Settings › ScopeHeader: a live "Applying to" bar whose menu
//! switches between global and a project, and the two reference bars.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, rems,
};
use u0ui::ghost::settings::ScopeHeader;
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page { scope: None }).into()
}

struct Page {
    scope: Option<SharedString>,
}

const PROJECTS: [&str; 2] = ["carbon-ui", "api-gateway"];
const SCOPES: [&str; 3] = ["global", "carbon-ui", "api-gateway"];

/// `components/ScopeHeader/preview.html`: global and `carbon-ui`, 560px, 12px apart.
fn reference() -> impl IntoElement {
    div()
        .w(rems(35.))
        .max_w_full()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_04)
        .child(ScopeHeader::new("scope-0").projects(PROJECTS))
        .child(
            ScopeHeader::new("scope-1")
                .scope(Some("carbon-ui".into()))
                .projects(PROJECTS),
        )
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(ScopeHeader::new("scope-thumb").projects(PROJECTS))
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let pick = entity.clone();
        let live = div().w(rems(35.)).max_w_full().child(
            ScopeHeader::new("scope-live")
                .scope(self.scope.clone())
                .projects(PROJECTS)
                .on_scope_change(move |s, _, cx| {
                    pick.update(cx, |p, cx| {
                        p.scope = s;
                        cx.notify();
                    })
                }),
        );
        let selected = match self.scope.as_deref() {
            Some("carbon-ui") => 1,
            Some("api-gateway") => 2,
            _ => 0,
        };
        let config = controls::field(
            "scope",
            controls::choices(
                "scope-choice",
                &SCOPES,
                selected,
                controls::setter(&entity, |p: &mut Page, i: usize| {
                    p.scope = (i > 0).then(|| SharedString::from(SCOPES[i]));
                }),
            ),
        );
        Body::new()
            .playground(live)
            .hint("Pick a project from the menu to see the override note")
            .config(config)
            .example(controls::block("Reference preview", reference()))
    }
}
