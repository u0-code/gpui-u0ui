//! T3 · Diff & review › DiffFileTree: the reference tree (four files in
//! three folders, DatePicker selected, Calendar viewed) and a live tree whose
//! files can be selected and marked viewed.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, rems,
};
use u0ui::prelude::*;
use u0ui::t3::review::{DiffFileTree, FileStatus, TreeFile};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        selected: Some("src/components/DatePicker.tsx".into()),
        viewed: [false, true, false, false],
    })
    .into()
}

struct Page {
    selected: Option<SharedString>,
    viewed: [bool; 4],
}

/// `components/DiffFileTree/preview.html` files.
fn files(viewed: [bool; 4]) -> Vec<TreeFile> {
    vec![
        TreeFile::new("src/components/DatePicker.tsx", FileStatus::Modified, 24, 9)
            .viewed(viewed[0]),
        TreeFile::new("src/components/Calendar.tsx", FileStatus::Modified, 3, 1).viewed(viewed[1]),
        TreeFile::new("src/tokens/g90.json", FileStatus::Added, 217, 0).viewed(viewed[2]),
        TreeFile::new("test/date.test.ts", FileStatus::Renamed, 2, 2).viewed(viewed[3]),
    ]
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .bg(color(ColorRole::Layer01))
        .child(
            DiffFileTree::new("dft-thumb")
                .files(files([false, true, false, false]).into_iter().take(2))
                .default_selected("src/components/DatePicker.tsx"),
        )
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        // `width: 340; background: var(--layer-01)`.
        let reference = div().w(rems(21.25)).bg(color(ColorRole::Layer01)).child(
            DiffFileTree::new("dft-ref")
                .files(files([false, true, false, false]))
                .selected(Some("src/components/DatePicker.tsx".into())),
        );
        let live = div().w(rems(21.25)).bg(color(ColorRole::Layer01)).child(
            DiffFileTree::new("dft-live")
                .files(files(self.viewed))
                .selected(self.selected.clone())
                .on_select(controls::setter(&entity, |this: &mut Page, p| {
                    this.selected = Some(p)
                })),
        );
        let names = ["DatePicker.tsx", "Calendar.tsx", "g90.json", "date.test.ts"];
        let mut config = div().flex().flex_col().gap(tokens::SPACING_05);
        for (i, name) in names.into_iter().enumerate() {
            config = config.child(controls::toggle(
                ("dft-viewed", i),
                format!("{name} viewed"),
                self.viewed[i],
                controls::setter(&entity, move |this: &mut Page, v| this.viewed[i] = v),
            ));
        }
        Body::new()
            .playground(live)
            .hint("Click a file or use Tab and Enter/Space to select it")
            .config(config)
            .example(controls::block("Reference preview", reference))
    }
}
