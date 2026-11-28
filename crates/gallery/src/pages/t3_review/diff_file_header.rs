//! T3 · Diff & review › DiffFileHeader: the three reference headers
//! (modified, added + viewed, deleted + collapsed), a live header whose
//! chevron toggles its collapsed state, the bare content a host's own
//! header draws (no bar, a rename, folded comments), and the parts on their
//! own.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, rems,
};
use u0ui::prelude::*;
use u0ui::t3::review::{DiffFileHeader, DiffFileStatus, DiffStat, FileStatus, StartTruncatedPath};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        status: 1,
        path: 0,
        stat: 0,
        collapsed: false,
        viewed: false,
    })
    .into()
}

struct Page {
    status: usize,
    path: usize,
    stat: usize,
    collapsed: bool,
    viewed: bool,
}

const PATHS: [&str; 3] = [
    "packages/ui/src/components/date-picker/DatePicker.tsx",
    "src/tokens/g90.json",
    "README.md",
];
const STATS: [(usize, usize); 4] = [(24, 9), (217, 0), (0, 88), (3, 1)];

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(
            DiffFileHeader::new("dfh-thumb", "src/tokens/g90.json")
                .status(FileStatus::Added)
                .stat(217, 0),
        )
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        // `components/DiffFileHeader/preview.html` (max-width 760).
        let reference = div()
            .flex()
            .flex_col()
            .w_full()
            .max_w(rems(47.5))
            .child(
                DiffFileHeader::new(
                    "dfh-ref-0",
                    "packages/ui/src/components/date-picker/DatePicker.tsx",
                )
                .status(FileStatus::Modified)
                .stat(24, 9),
            )
            .child(
                DiffFileHeader::new("dfh-ref-1", "src/tokens/g90.json")
                    .status(FileStatus::Added)
                    .stat(217, 0)
                    .default_viewed(true),
            )
            .child(
                DiffFileHeader::new("dfh-ref-2", "src/legacy/theme.css")
                    .status(FileStatus::Deleted)
                    .stat(0, 88)
                    .collapsed(true),
            );

        let (added, removed) = STATS[self.stat];
        let live = div().flex().flex_col().w_full().max_w(rems(47.5)).child(
            DiffFileHeader::new("dfh-live", PATHS[self.path])
                .status(FileStatus::ALL[self.status])
                .stat(added, removed)
                .collapsed(self.collapsed)
                .viewed(self.viewed)
                .on_toggle({
                    let entity = entity.clone();
                    move |_, cx| {
                        entity.update(cx, |this, cx| {
                            this.collapsed = !this.collapsed;
                            cx.notify();
                        })
                    }
                })
                .on_viewed(controls::setter(&entity, |this: &mut Page, v| {
                    this.viewed = v
                })),
        );

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::field(
                "status",
                controls::choices(
                    "dfh-status",
                    &["A", "M", "D", "R"],
                    self.status,
                    controls::setter(&entity, |this: &mut Page, i| this.status = i),
                ),
            ))
            .child(controls::field(
                "path",
                controls::choices(
                    "dfh-path",
                    &["Long", "Nested", "Root"],
                    self.path,
                    controls::setter(&entity, |this: &mut Page, i| this.path = i),
                ),
            ))
            .child(controls::field(
                "added / removed",
                controls::choices(
                    "dfh-stat",
                    &["+24 −9", "+217 −0", "+0 −88", "+3 −1"],
                    self.stat,
                    controls::setter(&entity, |this: &mut Page, i| this.stat = i),
                ),
            ))
            .child(controls::toggle(
                "dfh-collapsed",
                "collapsed",
                self.collapsed,
                controls::setter(&entity, |this: &mut Page, v| this.collapsed = v),
            ))
            .child(controls::toggle(
                "dfh-viewed",
                "viewed",
                self.viewed,
                controls::setter(&entity, |this: &mut Page, v| this.viewed = v),
            ));

        let parts = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap(tokens::SPACING_05)
                    .children(FileStatus::ALL.map(DiffFileStatus::new))
                    .child(DiffStat::new(312, 14))
                    .child(DiffStat::new(312, 14).bar(true))
                    .child(DiffStat::new(0, 0).bar(true)),
            )
            .child(div().w(rems(12.5)).flex().child(StartTruncatedPath::new(
                "packages/ui/src/components/date-picker/DatePicker.tsx",
            )));

        Body::new()
            .playground(live)
            .hint("Press the chevron to collapse; Viewed and the copy button work")
            .config(config)
            .example(controls::block("Reference preview", reference))
            .example(controls::block(
                "Bare, in a host's header (u0's column)",
                div()
                    .flex()
                    .flex_col()
                    .w_full()
                    .max_w(rems(30.))
                    .gap(tokens::SPACING_03)
                    .child(
                        div()
                            .h(rems(2.5))
                            .px(rems(0.75))
                            .flex()
                            .items_center()
                            .bg(theme::color(ColorRole::Layer01))
                            .child(
                                DiffFileHeader::new("dfh-bare-0", "src/tokens/themes.ts")
                                    .bare(true)
                                    .bar(false)
                                    .stat(2, 1),
                            ),
                    )
                    .child(
                        div()
                            .h(rems(2.5))
                            .px(rems(0.75))
                            .flex()
                            .items_center()
                            .bg(theme::color(ColorRole::Layer01))
                            .child(
                                DiffFileHeader::new("dfh-bare-1", "test/tokens.test.ts")
                                    .bare(true)
                                    .bar(false)
                                    .stat(3, 1)
                                    .collapsed(true)
                                    .comments(1),
                            ),
                    )
                    .child(
                        div()
                            .h(rems(2.5))
                            .px(rems(0.75))
                            .flex()
                            .items_center()
                            .bg(theme::color(ColorRole::Layer01))
                            .child(
                                DiffFileHeader::new("dfh-bare-2", "src/cover/Cover.tsx")
                                    .bare(true)
                                    .bar(false)
                                    .status(FileStatus::Renamed)
                                    .renamed_from("src/Cover.tsx")
                                    .stat(1, 1)
                                    .default_viewed(true),
                            ),
                    ),
            ))
            .example(controls::block(
                "DiffFileStatus, DiffStat and StartTruncatedPath",
                parts,
            ))
    }
}
