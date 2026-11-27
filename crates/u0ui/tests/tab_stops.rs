//! Tab reaches every interactive element of a representative set of
//! components, one stop each, as in the reference (design review FG3).
//!
//! A tracked `FocusHandle` carries its own tab-stop flag: a div's
//! `.tab_stop()` / `.tab_index()` does not reach it, so an element that set
//! only the div's flag was silently skipped. Each section below sits between
//! two sentinel stops; the walk from the first sentinel to the second counts
//! the distinct stops in between. The expected counts are the reference
//! previews' own Tab walks in headless Chromium (refrender, `--density
//! compact`), with the same demo content:
//!
//! | Card | Stops |
//! | --- | --- |
//! | CodeViewer | copy, code body |
//! | CommitComposer | 3 file checks, message, "Write it for me" (Commit is disabled) |
//! | TerminalTabs + TerminalView | the shown tab, New terminal, the terminal |
//! | ProcessTree | 2 expand toggles, 5 Stop buttons |
//! | ImageViewer | zoom out, the zoom value, zoom in, background |
//! | ConsentSheet | slider, copy JSON, the JSON, Share nothing, Continue |
//! | PairConfirmSheet | the control check, Don't pair, Pair |
//! | UserInputCard | the radio group (first option), the "other" field |
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{
    AnyElement, AnyWindowHandle, AppContext as _, Bounds, Context, FocusHandle,
    InteractiveElement as _, IntoElement, ParentElement as _, Point, Render, Styled as _,
    TestAppContext, Window, WindowBounds, WindowOptions, div, px, size,
};
use std::time::Duration;
use u0ui::t3::review::FileStatus;
use u0ui::t3::timeline::{InputOption, InputQuestion, UserInputCard};
use u0ui::u0::devices::PairConfirmSheet;
use u0ui::u0::diagnostics::{Process, ProcessTree};
use u0ui::u0::panels::{
    ChangedFile, CodeViewer, CommitComposer, ImageViewer, TerminalTab, TerminalTabs, TerminalView,
};
use u0ui::u0::settings::ConsentSheet;

const CODE: &str = "import { tokens } from './tokens';\n\n// Resolve a token for the active theme\nexport function resolve(name: string, theme = 'g100') {\n  const value = tokens[theme]?.[name];\n  if (value === undefined) throw new Error(`Unknown token ${name}`);\n  return value;\n}\n\nexport const DEFAULT_THEME = 'white';";

/// The sections in page order: (name, reference stop count, element).
fn sections() -> Vec<(&'static str, usize, AnyElement)> {
    vec![
        (
            "CodeViewer",
            2,
            CodeViewer::new("cv", CODE)
                .path("src/theme/resolve.ts")
                .language("TypeScript")
                .start_line(12)
                .highlight_lines([15, 16])
                .max_height(250.)
                .into_any_element(),
        ),
        (
            "CommitComposer",
            5,
            CommitComposer::new(
                "cc",
                vec![
                    ChangedFile::new("src/theme/g90.ts", FileStatus::Modified, 42, 3),
                    ChangedFile::new("src/theme/tokens.ts", FileStatus::Modified, 6, 1),
                    ChangedFile::new("docs/migration.md", FileStatus::Added, 58, 0),
                ],
            )
            .branch("feat/dark-tokens")
            .ahead(2)
            .into_any_element(),
        ),
        (
            "TerminalTabs + TerminalView",
            3,
            div()
                .flex()
                .flex_col()
                .child(TerminalTabs::new(
                    "tt",
                    vec![
                        TerminalTab::new("dev")
                            .running(true)
                            .cwd("~/code/carbon-ui"),
                        TerminalTab::new("test")
                            .exited(true)
                            .cwd("~/code/carbon-ui/packages/web"),
                    ],
                ))
                .child(TerminalView::new("term").rows(10).selection(2, 3).lines([
                    "$ pnpm dev",
                    "  VITE v6.0.3  ready in 412 ms",
                    "$ ",
                ]))
                .into_any_element(),
        ),
        (
            "ProcessTree",
            7,
            ProcessTree::new(
                "pt",
                vec![
                    Process::new(4120, "u0-server", 3.2, 180.).children([
                        Process::new(4188, "claude", 42.5, 610.)
                            .kind("Claude")
                            .children([Process::new(4301, "pnpm test", 88.1, 1540.)]),
                        Process::new(4190, "codex", 0.4, 220.).kind("Codex"),
                    ]),
                    Process::new(5011, "cloudflared", 0.8, 38.),
                ],
            )
            .into_any_element(),
        ),
        (
            "ImageViewer",
            4,
            ImageViewer::new("iv", "hero@2x.png")
                .dimensions(1440, 900)
                .alt("Hero illustration")
                .into_any_element(),
        ),
        (
            "ConsentSheet",
            5,
            ConsentSheet::new("consent")
                .app_name("u0")
                .into_any_element(),
        ),
        (
            "PairConfirmSheet",
            3,
            PairConfirmSheet::new("pair")
                .device("iPhone 16")
                .fingerprint("A1F3 9C2E 7B04 51D8")
                .into_any_element(),
        ),
        (
            "UserInputCard",
            2,
            UserInputCard::new(
                "input",
                vec![
                    InputQuestion::new(
                        "Which theme should be the default for new users?",
                        vec![
                            InputOption::new("White").description("Matches the marketing site"),
                            InputOption::new("G10").description("Slightly softer page background"),
                            InputOption::new("Follow the OS")
                                .description("Light or dark based on system setting"),
                        ],
                    )
                    .allow_other(true),
                    InputQuestion::new(
                        "Ship behind a flag?",
                        vec![InputOption::new("Yes"), InputOption::new("No")],
                    ),
                ],
            )
            .into_any_element(),
        ),
    ]
}

struct Page {
    /// One sentinel before each section and one after the last.
    sentinels: Vec<FocusHandle>,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let sentinel = |i: usize, h: &FocusHandle| {
            div()
                .id(("sentinel", i))
                .track_focus(&h.clone().tab_index(0).tab_stop(true))
                .size(px(4.))
        };
        let mut page = div().w(px(900.)).flex().flex_col().gap(px(8.));
        for (i, (_, _, element)) in sections().into_iter().enumerate() {
            page = page.child(sentinel(i, &self.sentinels[i])).child(element);
        }
        let last = self.sentinels.len() - 1;
        page.child(sentinel(last, &self.sentinels[last]))
    }
}

fn open(cx: &mut TestAppContext) -> (AnyWindowHandle, Vec<FocusHandle>) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        u0ui::init(cx);
    });
    let count = sections().len() + 1;
    let sentinels: Vec<FocusHandle> =
        cx.update(|cx| (0..count).map(|_| cx.focus_handle()).collect());
    let page = Page {
        sentinels: sentinels.clone(),
    };
    let window = cx.update(|cx| {
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(1000.), px(4000.)),
                })),
                ..Default::default()
            },
            cx,
            move |_, cx| cx.new(|_| page),
        )
        .expect("open test window")
        .0
    });
    (window, sentinels)
}

fn settle(window: AnyWindowHandle, cx: &mut TestAppContext) {
    cx.executor().advance_clock(Duration::from_millis(300));
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| window.render_frame(cx))
        .unwrap();
}

fn press(key: &str, window: AnyWindowHandle, cx: &mut TestAppContext) {
    cx.update_window(window, |_, window, cx| window.press(key, cx))
        .unwrap();
    settle(window, cx);
}

fn focused(window: AnyWindowHandle, cx: &mut TestAppContext) -> Option<FocusHandle> {
    cx.update_window(window, |_, window, cx| window.focused(cx))
        .unwrap()
}

#[gpui_kit::test]
fn tab_reaches_every_interactive_element(cx: &mut TestAppContext) {
    let (window, sentinels) = open(cx);
    settle(window, cx);
    let mut failures = Vec::new();
    for (i, (name, expected, _)) in sections().into_iter().enumerate() {
        cx.update_window(window, |_, window, cx| sentinels[i].focus(window, cx))
            .unwrap();
        settle(window, cx);
        let mut stops: Vec<FocusHandle> = Vec::new();
        loop {
            press("tab", window, cx);
            let Some(now) = focused(window, cx) else {
                failures.push(format!("{name}: Tab left the window"));
                break;
            };
            if now == sentinels[i + 1] {
                break;
            }
            if stops.contains(&now) || sentinels.contains(&now) || stops.len() > 40 {
                failures.push(format!("{name}: Tab went in a circle"));
                break;
            }
            stops.push(now);
        }
        if stops.len() != expected {
            failures.push(format!(
                "{name}: {} tab stops, the reference has {expected}",
                stops.len()
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[gpui_kit::test]
fn shift_tab_walks_the_same_stops_backwards(cx: &mut TestAppContext) {
    let (window, sentinels) = open(cx);
    settle(window, cx);
    let total: usize = sections().iter().map(|(_, n, _)| n).sum();
    let last = sentinels.len() - 1;
    cx.update_window(window, |_, window, cx| sentinels[last].focus(window, cx))
        .unwrap();
    settle(window, cx);
    let mut reached = 0;
    loop {
        press("shift-tab", window, cx);
        let now = focused(window, cx).expect("focus stays in the window");
        if now == sentinels[0] {
            break;
        }
        if !sentinels.contains(&now) {
            reached += 1;
        }
        assert!(reached <= total + 1, "Shift+Tab went in a circle");
    }
    assert_eq!(reached, total);
}
