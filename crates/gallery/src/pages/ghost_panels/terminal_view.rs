//! Ghost · Right column › TerminalView: a live tab strip (pick, ←/→,
//! Delete, close, new) over a terminal whose cursor, selection, exit code
//! and height follow the configuration, and the reference terminal.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render, Styled,
    Window, div, prelude::FluentBuilder as _, rems,
};
use u0ui::ghost::panels::{TerminalTab, TerminalTabs, TerminalView};
use u0ui::prelude::*;

/// `components/TerminalView/preview.html`'s screen.
pub const LINES: [&str; 6] = [
    "$ pnpm dev",
    "  \x1b[32mVITE\x1b[0m v6.0.3  ready in \x1b[1m412\x1b[0m ms",
    "  \x1b[32m➜\x1b[0m  Local:   \x1b[36mhttp://localhost:5173/\x1b[0m",
    "  \x1b[32m➜\x1b[0m  Network: use \x1b[1m--host\x1b[0m to expose",
    "\x1b[90m10:42:03\x1b[0m \x1b[33m[vite]\x1b[0m hmr update /src/theme.ts",
    "$ ",
];

const EXITS: [&str; 3] = ["running", "0", "1"];
const ROWS: [&str; 3] = ["10", "6", "auto"];

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        cursor: true,
        selection: true,
        exit: 0,
        rows: 0,
        tabs: vec![
            TerminalTab::new("dev")
                .running(true)
                .cwd("~/code/carbon-ui"),
            TerminalTab::new("test")
                .exited(true)
                .cwd("~/code/carbon-ui/packages/web"),
        ],
        current: 0,
        made: 0,
    })
    .into()
}

struct Page {
    cursor: bool,
    selection: bool,
    exit: usize,
    rows: usize,
    tabs: Vec<TerminalTab>,
    current: usize,
    made: usize,
}

fn reference_tabs() -> Vec<TerminalTab> {
    vec![
        TerminalTab::new("dev")
            .running(true)
            .cwd("~/code/carbon-ui"),
        TerminalTab::new("test")
            .exited(true)
            .cwd("~/code/carbon-ui/packages/web"),
    ]
}

/// `components/TerminalView/preview.html`.
fn reference() -> impl IntoElement {
    div()
        .w_full()
        .child(TerminalTabs::new("tt-ref", reference_tabs()))
        .child(
            TerminalView::new("term-ref")
                .rows(10)
                .selection(2, 3)
                .lines(LINES),
        )
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(
            TerminalView::new("term-thumb")
                .rows(4)
                .lines(LINES[..4].iter().copied()),
        )
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let e = cx.entity();
        let (pick, close, new) = (e.clone(), e.clone(), e.clone());
        let mut view = TerminalView::new("term-live")
            .title("dev")
            .cursor(self.cursor)
            .lines(LINES);
        if self.selection {
            view = view.selection(2, 3);
        }
        if let Ok(code) = EXITS[self.exit].parse::<i32>() {
            view = view.exited(code);
        }
        if let Ok(rows) = ROWS[self.rows].parse::<usize>() {
            view = view.rows(rows);
        }
        let live = div()
            .w_full()
            .max_w(rems(40.))
            .child(
                TerminalTabs::new("tt-live", self.tabs.clone())
                    .value(self.current.min(self.tabs.len().saturating_sub(1)))
                    .on_value_change(move |i, _, cx| {
                        pick.update(cx, |p, cx| {
                            p.current = i;
                            cx.notify();
                        })
                    })
                    .on_close(move |i, _, cx| {
                        close.update(cx, |p, cx| {
                            if p.tabs.len() > 1 {
                                p.tabs.remove(i);
                                if p.current >= p.tabs.len() || p.current > i {
                                    p.current = p.current.saturating_sub(1);
                                }
                            }
                            cx.notify();
                        })
                    })
                    .on_new(move |_, cx| {
                        new.update(cx, |p, cx| {
                            p.made += 1;
                            p.tabs.push(
                                TerminalTab::new(format!("zsh {}", p.made))
                                    .cwd("~/code/carbon-ui/packages/web/src/components"),
                            );
                            p.current = p.tabs.len() - 1;
                            cx.notify();
                        })
                    }),
            )
            .child(view);
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::toggle(
                "term-cursor",
                "cursor",
                self.cursor,
                controls::setter(&e, |p: &mut Page, v| p.cursor = v),
            ))
            .child(controls::toggle(
                "term-sel",
                "selection [2, 3]",
                self.selection,
                controls::setter(&e, |p: &mut Page, v| p.selection = v),
            ))
            .child(controls::field(
                "exited",
                controls::choices(
                    "term-exit",
                    &EXITS,
                    self.exit,
                    controls::setter(&e, |p: &mut Page, v| p.exit = v),
                ),
            ))
            .child(controls::field(
                "rows",
                controls::choices(
                    "term-rows",
                    &ROWS,
                    self.rows,
                    controls::setter(&e, |p: &mut Page, v| p.rows = v),
                ),
            ))
            .when(self.tabs.len() < 2, |d| {
                d.child(
                    div()
                        .type_style(TypeStyle::Label01)
                        .text_color(color(ColorRole::TextSecondary))
                        .child("The last tab stays open in this demo."),
                )
            });
        Body::new()
            .playground(live)
            .hint("Tab to a terminal tab: ←/→ switch, Delete closes; + opens a new one")
            .config(config)
            .example(controls::block("Reference preview", reference()))
    }
}
