//! T3 · Pull requests & issues › MergeControls: the reference pair (ready
//! to squash, blocked by conflicts) and a live box whose method menu
//! changes the button.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, prelude::FluentBuilder as _, rems,
};
use u0ui::prelude::*;
use u0ui::t3::review::{MergeControls, MergeMethod};

const CONFLICT: &str = "Conflicts with main. Resolve them before merging.";

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        method: 0,
        blocked: false,
        merged: false,
    })
    .into()
}

struct Page {
    method: usize,
    blocked: bool,
    merged: bool,
}

fn stack() -> gpui_kit::Div {
    // `cu-stack` with `gap: 8; maxWidth: 640`.
    div()
        .flex()
        .flex_col()
        .gap(rems(0.5))
        .w_full()
        .max_w(rems(40.))
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    stack()
        .w(rems(15.))
        .child(MergeControls::new("mc-thumb"))
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let live = MergeControls::new("mc-live")
            .method(MergeMethod::ALL[self.method])
            .when(self.blocked, |m| m.blocked(CONFLICT))
            .on_method(controls::setter(
                &entity,
                |this: &mut Page, m: MergeMethod| {
                    this.method = MergeMethod::ALL.iter().position(|x| *x == m).unwrap_or(0)
                },
            ))
            .on_merge({
                let entity = entity.clone();
                move |_, cx| {
                    entity.update(cx, |this, cx| {
                        this.merged = true;
                        cx.notify();
                    })
                }
            });
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::field(
                "method",
                controls::choices(
                    "mc-method",
                    &["squash", "merge", "rebase"],
                    self.method,
                    controls::setter(&entity, |this: &mut Page, i| this.method = i),
                ),
            ))
            .child(controls::toggle(
                "mc-blocked",
                "blocked",
                self.blocked,
                controls::setter(&entity, |this: &mut Page, v| this.blocked = v),
            ));
        Body::new()
            .playground(stack().child(live).when(self.merged, |d| {
                d.child(
                    div()
                        .type_style(TypeStyle::Label01)
                        .text_color(color(ColorRole::TextSecondary))
                        .child("onMerge()"),
                )
            }))
            .hint("The chevron opens the method menu; choosing a method relabels the button")
            .config(config)
            .example(controls::block(
                "Reference preview",
                stack()
                    .child(MergeControls::new("mc-ref-0"))
                    .child(MergeControls::new("mc-ref-1").blocked(CONFLICT)),
            ))
    }
}
