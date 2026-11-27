//! T3 · Thread › ChatHeader: the reference header (working thread on
//! build-box with a 6-file git action and the layout toggles) and a live
//! header whose toggles and title respond.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, prelude::FluentBuilder as _, rems,
};
use u0ui::prelude::*;
use u0ui::t3::Status;
use u0ui::t3::review::{ChatHeader, GitActionsControl, GitState, HeaderEnvironment, HeaderLayout};
use u0ui::t3::shell::{ConnectionStatus, LayoutPanel, MachineKind};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        layout: HeaderLayout::default(),
        show_layout: true,
        inline: false,
        status: 0,
        renames: 0,
    })
    .into()
}

struct Page {
    layout: HeaderLayout,
    show_layout: bool,
    inline: bool,
    status: usize,
    renames: usize,
}

const STATUSES: [Status; 4] = [
    Status::Working,
    Status::NeedsYou,
    Status::Done,
    Status::Failed,
];

fn reference(id: &'static str) -> ChatHeader {
    ChatHeader::new(id, "Add dark theme tokens")
        .project("carbon-ui")
        .branch("feat/dark-tokens")
        .status(Status::Working)
        .environment(HeaderEnvironment::new(
            "build-box",
            MachineKind::Server,
            ConnectionStatus::Connected,
        ))
        .action(GitActionsControl::new((id, 1usize), GitState::Changes).files(6))
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(
            ChatHeader::new("ch-thumb", "Add dark theme tokens")
                .project("carbon-ui")
                .status(Status::Working)
                .layout(None),
        )
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let live = div().w_full().child(
            reference("ch-live")
                .status(STATUSES[self.status])
                .inline(self.inline)
                .layout(self.show_layout.then_some(self.layout))
                .on_layout(controls::setter(
                    &entity,
                    |this: &mut Page, p: LayoutPanel| match p {
                        LayoutPanel::Sidebar => this.layout.sidebar = !this.layout.sidebar,
                        LayoutPanel::Terminal => this.layout.terminal = !this.layout.terminal,
                        LayoutPanel::Right => this.layout.right_panel = !this.layout.right_panel,
                    },
                ))
                .on_rename({
                    let entity = entity.clone();
                    move |_, cx| {
                        entity.update(cx, |this, cx| {
                            this.renames += 1;
                            cx.notify();
                        })
                    }
                }),
        );
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .w_full()
            .child(live)
            .when(self.renames > 0, |d| {
                d.child(
                    div()
                        .type_style(TypeStyle::Label01)
                        .text_color(color(ColorRole::TextSecondary))
                        .child(format!("onRename × {}", self.renames)),
                )
            });
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::field(
                "status",
                controls::choices(
                    "ch-status",
                    &["working", "needs-you", "done", "failed"],
                    self.status,
                    controls::setter(&entity, |this: &mut Page, i| this.status = i),
                ),
            ))
            .child(controls::toggle(
                "ch-layout",
                "layout controls",
                self.show_layout,
                controls::setter(&entity, |this: &mut Page, v| this.show_layout = v),
            ))
            .child(controls::toggle(
                "ch-inline",
                "inline",
                self.inline,
                controls::setter(&entity, |this: &mut Page, v| this.inline = v),
            ));
        Body::new()
            .playground(live)
            .stage_min_width(40.)
            .hint("The layout toggles press; the title reports onRename")
            .config(config)
            .example(controls::block("Reference preview", reference("ch-ref")))
    }
}
