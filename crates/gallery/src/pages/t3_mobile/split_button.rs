//! T3 · Mobile › SplitButton: the reference pair (primary Commit & push,
//! secondary Approve) with working menus, and a live split button.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, prelude::FluentBuilder as _,
};
use u0ui::components::MenuItem;
use u0ui::prelude::*;
use u0ui::t3::mobile::{SplitButton, SplitKind};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        kind: 0,
        last: None,
    })
    .into()
}

struct Page {
    kind: usize,
    last: Option<SharedString>,
}

fn items(labels: &[&'static str]) -> Vec<MenuItem> {
    labels.iter().map(|l| MenuItem::item(*l)).collect()
}

fn reference(prefix: &'static str) -> gpui_kit::Div {
    div()
        .flex()
        .flex_wrap()
        .items_start()
        .gap(tokens::SPACING_05)
        .child(
            SplitButton::new((prefix, 0usize), "Commit & push").menu(items(&[
                "Commit only",
                "Commit on new branch",
                "Create PR",
            ])),
        )
        .child(
            SplitButton::new((prefix, 1usize), "Approve")
                .kind(SplitKind::Secondary)
                .menu(items(&["Always allow this session", "Decline"])),
        )
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    reference("sb-thumb").into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let pick = |label: &'static str| {
            let entity = entity.clone();
            MenuItem::item(label).on_select(move |_, cx| {
                entity.update(cx, |this, cx| {
                    this.last = Some(label.into());
                    cx.notify();
                })
            })
        };
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(
                SplitButton::new("sb-live", "Commit & push")
                    .kind(SplitKind::ALL[self.kind])
                    .menu([
                        pick("Commit only"),
                        pick("Commit on new branch"),
                        pick("Create PR"),
                    ])
                    .on_press({
                        let entity = entity.clone();
                        move |_, cx| {
                            entity.update(cx, |this, cx| {
                                this.last = Some("Commit & push".into());
                                cx.notify();
                            })
                        }
                    }),
            )
            .when_some(self.last.clone(), |d, last| {
                d.child(
                    div()
                        .type_style(TypeStyle::Label01)
                        .text_color(color(ColorRole::TextSecondary))
                        .child(format!("Chose: {last}")),
                )
            });
        Body::new()
            .playground(live)
            .config(controls::field(
                "kind",
                controls::choices(
                    "sb-kind",
                    &["primary", "secondary"],
                    self.kind,
                    controls::setter(&entity, |this: &mut Page, i| this.kind = i),
                ),
            ))
            .example(controls::block("Reference preview", reference("sb-ref")))
    }
}
