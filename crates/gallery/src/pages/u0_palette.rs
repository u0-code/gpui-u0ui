//! u0 · Palette pages (`src/ghost/workspace.jsx`, "Palette"):
//! PalettePage (+ PaletteItem) and FileResultRow, live plus the reference
//! previews.
use crate::page::Body;
use crate::{controls, stand_in};
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, Entity, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, component::input::InputState, div, rems,
};
use u0ui::icons::CarbonIcon;
use u0ui::prelude::*;
use u0ui::u0::palette::{FileResultRow, PaletteItem, PaletteLevel, PaletteNav, PalettePage};
use u0ui::u0::primitives::fuzzy_match;

fn log(text: impl Into<SharedString>) -> impl IntoElement {
    div()
        .type_style(TypeStyle::Label01)
        .text_color(color(ColorRole::TextSecondary))
        .child(text.into())
}

// ── PalettePage ─────────────────────────────────────────────────────────

/// `components/PalettePage/preview.html`: already on "Switch agent".
fn agents(prefix: &'static str) -> Vec<AnyElement> {
    vec![
        PaletteItem::new((prefix, 0usize), "Claude")
            .icon(CarbonIcon::Bot)
            .detail("Default")
            .active(true)
            .into_any_element(),
        PaletteItem::new((prefix, 1usize), "Codex")
            .icon(CarbonIcon::Bot)
            .detail("Signed in")
            .into_any_element(),
        PaletteItem::new((prefix, 2usize), "Gemini")
            .icon(CarbonIcon::Bot)
            .detail("Not signed in")
            .into_any_element(),
        PaletteItem::new((prefix, 3usize), "Add an agent…")
            .icon(CarbonIcon::Add)
            .chevron(true)
            .into_any_element(),
    ]
}

fn reference() -> impl IntoElement {
    div().w_full().max_w(rems(35.)).child(
        PalettePage::new("pal-ref")
            .pages([
                PaletteLevel::new("Commands"),
                PaletteLevel::new("Switch agent").placeholder("Choose an agent"),
            ])
            .content(|_, _, _| agents("pal-ref-item")),
    )
}

/// The PalettePage overview tile.
pub fn palette_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(
            PalettePage::new("pal-thumb")
                .pages([
                    PaletteLevel::new("Commands"),
                    PaletteLevel::new("Switch agent"),
                ])
                .content(|_, _, _| agents("pal-thumb-item").into_iter().take(2).collect()),
        )
        .into_any_element()
}

/// The commands on the live palette's first page: (title, icon, shortcut,
/// opens a page).
const COMMANDS: [(&str, CarbonIcon, Option<&str>, bool); 4] = [
    ("New thread", CarbonIcon::Add, Some("⌘ N"), false),
    ("Switch agent", CarbonIcon::Bot, None, true),
    ("Open settings", CarbonIcon::Settings, Some("⌘ ,"), false),
    ("Toggle terminal", CarbonIcon::Terminal, Some("⌃ `"), false),
];

struct PalettePageDemo {
    log: SharedString,
}

/// Build the PalettePage page.
pub fn build_palette_page(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| PalettePageDemo { log: "—".into() }).into()
}

/// The live page's results: commands at the top, agents below "Switch
/// agent", filtered by the query.
fn live_results(nav: &PaletteNav, e: &Entity<PalettePageDemo>) -> Vec<AnyElement> {
    let q = nav.query.to_string();
    let say = |text: String| {
        let e = e.clone();
        move |_: &mut Window, cx: &mut App| {
            e.update(cx, |p, cx| {
                p.log = text.clone().into();
                cx.notify();
            })
        }
    };
    if nav.depth == 1 {
        COMMANDS
            .iter()
            .enumerate()
            .filter(|(_, c)| fuzzy_match(c.0, &q).is_some())
            .map(|(i, &(title, icon, keys, opens))| {
                let nav = nav.clone();
                let mut item = PaletteItem::new(("pal-cmd", i), title)
                    .icon(icon)
                    .chevron(opens)
                    .active(i == 0);
                if let Some(k) = keys {
                    item = item.shortcut(k);
                }
                let picked = say(format!("ran “{title}”"));
                item.on_select(move |w, cx| {
                    if opens {
                        nav.push(
                            PaletteLevel::new("Switch agent").placeholder("Choose an agent"),
                            w,
                            cx,
                        );
                    } else {
                        picked(w, cx);
                    }
                })
                .into_any_element()
            })
            .collect()
    } else {
        ["Claude", "Codex", "Gemini"]
            .iter()
            .enumerate()
            .filter(|(_, a)| fuzzy_match(a, &q).is_some())
            .map(|(i, a)| {
                PaletteItem::new(("pal-agent", i), *a)
                    .icon(CarbonIcon::Bot)
                    .active(i == 0)
                    .on_select(say(format!("switched to {a}")))
                    .into_any_element()
            })
            .collect()
    }
}

impl Render for PalettePageDemo {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let e = cx.entity();
        let (content, back, close) = (e.clone(), e.clone(), e.clone());
        let live = div()
            .w_full()
            .max_w(rems(35.))
            .flex()
            .flex_col()
            .gap(tokens::SPACING_03)
            .child(
                PalettePage::new("pal-live")
                    .content(move |nav, _, _| live_results(nav, &content))
                    .on_back(move |_, cx| {
                        back.update(cx, |p, cx| {
                            p.log = "onBack".into();
                            cx.notify();
                        })
                    })
                    .on_close(move |_, cx| {
                        close.update(cx, |p, cx| {
                            p.log = "onClose".into();
                            cx.notify();
                        })
                    }),
            )
            .child(log(format!("last: {}", self.log)));
        Body::new()
            .playground(live)
            .hint("Pick “Switch agent”, then go back with the arrow, Esc, or ⌫ in the empty input")
            .example(controls::block("Reference preview", reference()))
    }
}

// ── FileResultRow ───────────────────────────────────────────────────────

const PATHS: [&str; 6] = [
    "packages/web/src/components/sidebar/SidebarThreadRow.tsx",
    "packages/web/src/components/sidebar/SidebarBrand.tsx",
    "apps/mobile/src/thread/ThreadRow.tsx",
    "README.md",
    "packages/web/src/theme/tokens.ts",
    "scripts/build-tokens.py",
];

/// `components/FileResultRow/preview.html`: four rows, 520px on `layer-01`.
fn file_reference() -> impl IntoElement {
    div()
        .w_full()
        .max_w(rems(32.5))
        .bg(color(ColorRole::Layer01))
        .child(FileResultRow::new("fr-0", PATHS[0], "sbtr").active(true))
        .child(FileResultRow::new("fr-1", PATHS[1], "sbtr"))
        .child(FileResultRow::new("fr-2", PATHS[2], "thr").kind("modified"))
        .child(FileResultRow::new("fr-3", PATHS[3], "rd"))
}

/// The FileResultRow overview tile.
pub fn file_thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .bg(color(ColorRole::Layer01))
        .child(FileResultRow::new("fr-thumb", "src/sidebar/ThreadRow.tsx", "thr").active(true))
        .into_any_element()
}

struct FilePage {
    query: Entity<InputState>,
    active: usize,
}

/// Build the FileResultRow page.
pub fn build_file_result_row(window: &mut Window, cx: &mut App) -> AnyView {
    let query = stand_in::input_state("sbtr", window, cx);
    cx.new(|cx| {
        cx.observe(&query, |_, _, cx| cx.notify()).detach();
        FilePage { query, active: 0 }
    })
    .into()
}

impl Render for FilePage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let e = cx.entity();
        let q = self.query.read(cx).value().to_string();
        let shown: Vec<&str> = PATHS
            .iter()
            .copied()
            .filter(|p| fuzzy_match(p, &q).is_some())
            .collect();
        let rows: Vec<AnyElement> = shown
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let pick = e.clone();
                FileResultRow::new(("fr-live", i), *p, q.clone())
                    .active(i == self.active)
                    .when_modified(i == 2)
                    .on_select(move |_, cx| {
                        pick.update(cx, |page, cx| {
                            page.active = i;
                            cx.notify();
                        })
                    })
                    .into_any_element()
            })
            .collect();
        let live = div()
            .w_full()
            .max_w(rems(32.5))
            .bg(color(ColorRole::Layer01))
            .children(rows);
        let config = div().max_w(rems(18.)).child(stand_in::text_field(
            "fr-query",
            "query",
            &self.query,
            LayerLevel::Two,
        ));
        Body::new()
            .playground(live)
            .hint("Type a query; click a row to make it active")
            .config(config)
            .example(controls::block("Reference preview", file_reference()))
    }
}

trait WhenModified {
    fn when_modified(self, on: bool) -> Self;
}

impl WhenModified for FileResultRow {
    fn when_modified(self, on: bool) -> Self {
        if on { self.kind("modified") } else { self }
    }
}
