//! Layout › Resizable: the reference Sidebar | Editor / Terminal split and a
//! configurable live group (orientation, panel count, grip, sizes, minimum).
use crate::controls;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled, Window,
    div, rems,
};
use u0ui::components::resizable::{
    ResizableHandle, ResizableOrientation, ResizablePanel, ResizablePanelGroup,
};
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| ResizablePage {
        orientation: 0,
        panels: 0,
        with_handle: true,
        split: 1,
        min: 0,
    })
    .into()
}

struct ResizablePage {
    orientation: usize,
    panels: usize,
    with_handle: bool,
    split: usize,
    min: usize,
}

const PANELS: [&str; 2] = ["2", "3"];
const SPLITS: [&str; 3] = ["even", "30 / 70", "70 / 30"];
const MINS: [&str; 3] = ["10 (default)", "20", "40"];

/// `preview.html`: `h('div', { style: { padding: 16 } }, label)`.
fn pane(label: &'static str) -> impl IntoElement {
    div()
        .p(tokens::SPACING_05)
        .type_style(TypeStyle::BodyCompact01)
        .text_color(color(ColorRole::TextPrimary))
        .child(label)
}

impl Render for ResizablePage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();

        // `components/Resizable/preview.html`: a 200px box holding a horizontal
        // 30/70 group (with grip) whose right panel is a vertical 60/40 group.
        let reference = div().h(rems(12.5)).child(
            ResizablePanelGroup::new("rz-ref")
                .panel(
                    ResizablePanel::new()
                        .default_size(30.)
                        .child(pane("Sidebar")),
                )
                .handle(ResizableHandle::new().with_handle(true))
                .panel(
                    ResizablePanel::new().default_size(70.).child(
                        ResizablePanelGroup::new("rz-ref-inner")
                            .orientation(ResizableOrientation::Vertical)
                            .panel(
                                ResizablePanel::new()
                                    .default_size(60.)
                                    .child(pane("Editor")),
                            )
                            .handle(ResizableHandle::new())
                            .panel(
                                ResizablePanel::new()
                                    .default_size(40.)
                                    .child(pane("Terminal")),
                            ),
                    ),
                ),
        );

        let orientation = ResizableOrientation::ALL[self.orientation];
        let count = self.panels + 2;
        let min = [10., 20., 40.][self.min];
        let defaults: Vec<Option<f32>> = match (count, self.split) {
            (_, 0) => vec![None; count],
            (2, 1) => vec![Some(30.), Some(70.)],
            (2, _) => vec![Some(70.), Some(30.)],
            (_, 1) => vec![Some(30.), Some(35.), Some(35.)],
            _ => vec![Some(50.), Some(25.), Some(25.)],
        };
        const NAMES: [&str; 3] = ["One", "Two", "Three"];
        let mut group = ResizablePanelGroup::new(("rz-live", self.orientation * 10 + count))
            .orientation(orientation);
        for (i, default) in defaults.into_iter().enumerate() {
            if i > 0 {
                group = group.handle(ResizableHandle::new().with_handle(self.with_handle));
            }
            let mut panel = ResizablePanel::new().min_size(min).child(pane(NAMES[i]));
            if let Some(size) = default {
                panel = panel.default_size(size);
            }
            group = group.panel(panel);
        }
        let live = div().h(rems(15.)).child(group);

        let orientation_names = ResizableOrientation::ALL.map(|o| o.name());
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "orientation",
                controls::choices(
                    "rz-orientation",
                    &orientation_names,
                    self.orientation,
                    controls::setter(&entity, |this: &mut ResizablePage, i| this.orientation = i),
                ),
            ))
            .child(controls::field(
                "panels",
                controls::choices(
                    "rz-panels",
                    &PANELS,
                    self.panels,
                    controls::setter(&entity, |this: &mut ResizablePage, i| this.panels = i),
                ),
            ))
            .child(controls::field(
                "defaultSize",
                controls::choices(
                    "rz-split",
                    &SPLITS,
                    self.split,
                    controls::setter(&entity, |this: &mut ResizablePage, i| this.split = i),
                ),
            ))
            .child(controls::field(
                "minSize (%)",
                controls::choices(
                    "rz-min",
                    &MINS,
                    self.min,
                    controls::setter(&entity, |this: &mut ResizablePage, i| this.min = i),
                ),
            ))
            .child(controls::toggle(
                "rz-with-handle",
                "withHandle",
                self.with_handle,
                controls::setter(&entity, |this: &mut ResizablePage, on| {
                    this.with_handle = on
                }),
            ));

        let note = div()
            .type_style(TypeStyle::Label01)
            .text_color(color(ColorRole::TextSecondary))
            .child("Drag a handle, or Tab to it and press the arrow keys (±5%).");

        controls::page_body(
            div()
                .flex()
                .flex_col()
                .gap(tokens::SPACING_07)
                .child(controls::block("Reference preview", reference))
                .child(controls::block(
                    "Live",
                    div()
                        .flex()
                        .flex_col()
                        .gap(tokens::SPACING_04)
                        .child(live)
                        .child(note),
                )),
            Some(config.into_any_element()),
        )
    }
}
