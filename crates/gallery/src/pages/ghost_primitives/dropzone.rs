//! Ghost · Primitives › Dropzone: a live drop zone (files/folder, busy,
//! error) reporting what it received, plus the four reference zones.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, rems,
};
use u0ui::ghost::primitives::Dropzone;
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| DropzonePage {
        folder: false,
        busy: false,
        error: false,
        received: "nothing yet".into(),
    })
    .into()
}

struct DropzonePage {
    folder: bool,
    busy: bool,
    error: bool,
    received: SharedString,
}

/// `components/Dropzone/preview.html`: a 2 × 2 grid, 16px gaps.
pub fn reference(prefix: &'static str) -> impl IntoElement {
    let cell = |z: Dropzone| div().flex_1().min_w_0().child(z);
    div()
        .flex()
        .flex_col()
        .gap(tokens::SPACING_05)
        .child(
            div()
                .flex()
                .gap(tokens::SPACING_05)
                .child(cell(
                    Dropzone::new((prefix, 0usize))
                        .label("Drop a VS Code theme")
                        .hint(".json or .vsix")
                        .accept([".json", ".vsix"]),
                ))
                .child(cell(
                    Dropzone::new((prefix, 1usize))
                        .label("Add a project folder")
                        .hint("or click to choose")
                        .folder(true),
                )),
        )
        .child(
            div()
                .flex()
                .gap(tokens::SPACING_05)
                .child(cell(
                    Dropzone::new((prefix, 2usize))
                        .label("Drop a file")
                        .busy(true),
                ))
                .child(cell(
                    Dropzone::new((prefix, 3usize))
                        .label("Drop a VS Code theme")
                        .hint(".json or .vsix")
                        .error("That file is not a colour theme."),
                )),
        )
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(
            Dropzone::new("thumb-drop")
                .label("Drop a VS Code theme")
                .hint(".json or .vsix"),
        )
        .into_any_element()
}

impl Render for DropzonePage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let mut zone = Dropzone::new("drop-live")
            .label(if self.folder {
                "Add a project folder"
            } else {
                "Drop a VS Code theme"
            })
            .hint(if self.folder {
                "or click to choose"
            } else {
                ".json or .vsix"
            })
            .folder(self.folder)
            .busy(self.busy)
            .on_files(controls::setter(
                &entity,
                |this: &mut Self, paths: Vec<std::path::PathBuf>| {
                    this.received = if paths.is_empty() {
                        "no accepted files".into()
                    } else {
                        paths
                            .iter()
                            .map(|p| p.display().to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                            .into()
                    };
                },
            ));
        if !self.folder {
            zone = zone.accept([".json", ".vsix"]);
        }
        if self.error {
            zone = zone.error("That file is not a colour theme.");
        }
        let live = div()
            .w(rems(24.))
            .max_w_full()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(zone)
            .child(
                div()
                    .type_style(TypeStyle::Code01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(format!("received: {}", self.received)),
            );
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::toggle(
                "drop-folder",
                "folder",
                self.folder,
                controls::setter(&entity, |this: &mut Self, on| this.folder = on),
            ))
            .child(controls::toggle(
                "drop-busy",
                "state = busy",
                self.busy,
                controls::setter(&entity, |this: &mut Self, on| this.busy = on),
            ))
            .child(controls::toggle(
                "drop-error",
                "error",
                self.error,
                controls::setter(&entity, |this: &mut Self, on| this.error = on),
            ));
        Body::new()
            .playground(live)
            .hint("Drop files from Finder, or press (Enter/Space) to choose")
            .config(config)
            .example(controls::block("Reference preview", reference("drop-ref")))
    }
}
