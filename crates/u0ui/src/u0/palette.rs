//! `u0 · Palette` (`src/ghost/workspace.jsx`, "Palette"; `.g-pal*`,
//! `.g-pitem*`, `.g-fres*` in `src/ghost.css`): the layered command palette.
//!
//! - [`PalettePage`]: a palette whose commands can open further pages
//!   ("Switch agent" → the agents). The bar shows a back arrow and a
//!   breadcrumb once a page is pushed; Esc goes back (or closes at the top),
//!   Backspace in an empty input goes back, and the input clears on every
//!   page change. Like the reference's `<input autoFocus>`, the input takes
//!   focus when the palette mounts. The caller renders the results for the current page and
//!   pushes pages through the [`PaletteNav`] it is handed.
//! - [`PaletteItem`]: one result: glyph, title, detail, shortcut keys and a
//!   chevron when it opens a page; the active row is `layer-selected-01`
//!   with a 2px `border-interactive` start bar.
//! - [`FileResultRow`]: a ⌘P result: the file name whole, then its directory
//!   truncated from the start in 12px `text-secondary`; one fuzzy match is
//!   run over name-then-directory so each query character lights once.
use crate::{
    components::{Kbd, KbdGroup},
    density,
    icons::{CarbonIcon, Icon},
    t3::craft::{rem, sans},
    theme::color,
    tokens::{self, ColorRole},
    u0::primitives::{HighlightedText, IconButton, fuzzy_match, parts::editor},
};
use gpui_kit::{
    AnyElement, App, AppContext as _, BoxShadow, CursorStyle, Div, ElementId, Entity, Hsla,
    InteractiveElement as _, IntoElement, KeyDownEvent, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement as _, Styled, Window, component::input::InputState, div, point,
    prelude::FluentBuilder as _, px,
};
use std::rc::Rc;

/// A palette row's (background, start bar): `.g-pitem--on { background:
/// var(--layer-selected-01); box-shadow: inset 2px 0 0
/// var(--border-interactive) }`, `:hover` `layer-hover-01`.
pub fn row_look(active: bool, hovered: bool) -> (Option<Hsla>, bool) {
    if active {
        (Some(color(ColorRole::LayerSelected01)), true)
    } else if hovered {
        (Some(color(ColorRole::LayerHover01)), false)
    } else {
        (None, false)
    }
}

/// `.g-pitem, .g-fres { display: flex; align-items: center; gap:
/// var(--spacing-03); height: var(--density-row-md); padding: 0
/// var(--spacing-04); font-size: 0.875rem; color: var(--text-primary) }`.
fn row(id: ElementId, active: bool, hovered: bool) -> gpui_kit::Stateful<Div> {
    let (bg, bar) = row_look(active, hovered);
    sans(div(), 14., 400)
        .id(id)
        .relative()
        .flex()
        .items_center()
        .gap(tokens::SPACING_03)
        .h(density::current().row())
        .px(tokens::SPACING_04)
        .text_color(color(ColorRole::TextPrimary))
        .cursor(CursorStyle::PointingHand)
        .when_some(bg, |d, c| d.bg(c))
        .when(bar, |d| {
            d.child(
                div()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .left_0()
                    .w(px(2.))
                    .bg(color(ColorRole::BorderInteractive)),
            )
        })
}

/// The 16px `.g-dim-icon` glyph (`icon-secondary`).
fn dim_icon(name: CarbonIcon) -> Icon {
    Icon::carbon(name)
        .size_length(rem(16.))
        .color(color(ColorRole::IconSecondary))
}

/// A keyed hover flag and its updater.
fn hover_state(
    id: ElementId,
    window: &mut Window,
    cx: &mut App,
) -> (bool, impl Fn(&bool, &mut Window, &mut App) + 'static) {
    let store = window.use_keyed_state(id, cx, |_, _| false);
    let over = *store.read(cx);
    (over, move |h: &bool, _: &mut Window, cx: &mut App| {
        store.update(cx, |v, cx| {
            if *v != *h {
                *v = *h;
                cx.notify();
            }
        })
    })
}

type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

// ── PaletteItem ─────────────────────────────────────────────────────────

/// One palette result. See the module docs.
#[derive(IntoElement)]
pub struct PaletteItem {
    id: ElementId,
    icon: Option<CarbonIcon>,
    title: SharedString,
    detail: Option<SharedString>,
    shortcut: Option<SharedString>,
    chevron: bool,
    active: bool,
    on_select: Option<Handler>,
}

impl PaletteItem {
    /// A result titled `title` (`id` is its `aria-activedescendant` id).
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            icon: None,
            title: title.into(),
            detail: None,
            shortcut: None,
            chevron: false,
            active: false,
            on_select: None,
        }
    }

    /// `icon`.
    pub fn icon(mut self, icon: CarbonIcon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// `detail`.
    pub fn detail(mut self, detail: impl Into<SharedString>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Space-separated keys (`shortcut`, e.g. `"⌘ K"`).
    pub fn shortcut(mut self, shortcut: impl Into<SharedString>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// It opens a further page (`chevron`).
    pub fn chevron(mut self, chevron: bool) -> Self {
        self.chevron = chevron;
        self
    }

    /// The keyboard-highlighted row (`active`).
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// Chosen (`onSelect`).
    pub fn on_select(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for PaletteItem {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (over, on_hover) = hover_state((self.id.clone(), "hover").into(), window, cx);
        let select = self.on_select.clone();
        row(self.id, self.active, over)
            .on_hover(on_hover)
            .on_click(move |_, w, cx| {
                if let Some(f) = &select {
                    f(w, cx);
                }
            })
            .when_some(self.icon, |d, i| d.child(dim_icon(i).flex_none()))
            // `.g-pitem-title { flex: 1; min-width: 0; text-overflow: ellipsis }`.
            .child(div().flex_1().min_w_0().truncate().child(self.title))
            .when_some(self.detail, |d, t| {
                // `.g-pitem-detail { font-size: 0.75rem; color: var(--text-secondary) }`.
                d.child(
                    sans(div(), 12., 400)
                        .flex_none()
                        .text_color(color(ColorRole::TextSecondary))
                        .child(t),
                )
            })
            .when_some(self.shortcut, |d, s| {
                d.child(KbdGroup::new().children(s.split(' ').map(|k| Kbd::new(k.to_string()))))
            })
            .when(self.chevron, |d| {
                d.child(dim_icon(CarbonIcon::ChevronRight).flex_none())
            })
    }
}

// ── FileResultRow ───────────────────────────────────────────────────────

/// Split `path` at its last `/` into (file, directory with its slash), and
/// match `query` once over file-then-directory: (file hits, directory
/// hits), as character indices into each part.
pub fn file_result_hits(path: &str, query: &str) -> (String, String, Vec<usize>, Vec<usize>) {
    let (dir, file) = match path.rfind('/') {
        Some(i) => (&path[..=i], &path[i + 1..]),
        None => ("", path),
    };
    let n = file.chars().count();
    let hits = fuzzy_match(&format!("{file}{dir}"), query).unwrap_or_default();
    let (fh, dh): (Vec<usize>, Vec<usize>) = hits.into_iter().partition(|&x| x < n);
    (
        file.to_string(),
        dir.to_string(),
        fh,
        dh.into_iter().map(|x| x - n).collect(),
    )
}

/// A ⌘P file result. See the module docs.
#[derive(IntoElement)]
pub struct FileResultRow {
    id: ElementId,
    path: SharedString,
    query: SharedString,
    active: bool,
    kind: Option<SharedString>,
    on_select: Option<Handler>,
}

impl FileResultRow {
    /// A result for `path` matched against `query`.
    pub fn new(
        id: impl Into<ElementId>,
        path: impl Into<SharedString>,
        query: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            path: path.into(),
            query: query.into(),
            active: false,
            kind: None,
            on_select: None,
        }
    }

    /// The keyboard-highlighted row (`active`).
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// A note such as "modified" or "new" (`kind`).
    pub fn kind(mut self, kind: impl Into<SharedString>) -> Self {
        self.kind = Some(kind.into());
        self
    }

    /// Chosen (`onSelect`).
    pub fn on_select(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for FileResultRow {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (over, on_hover) = hover_state((self.id.clone(), "hover").into(), window, cx);
        let (file, dir, fh, dh) = file_result_hits(&self.path, &self.query);
        let select = self.on_select.clone();
        row(self.id, self.active, over)
            .on_hover(on_hover)
            .on_click(move |_, w, cx| {
                if let Some(f) = &select {
                    f(w, cx);
                }
            })
            .child(dim_icon(CarbonIcon::Document).flex_none())
            // `.g-fres-file { flex: none; white-space: nowrap }`.
            .child(
                div()
                    .flex_none()
                    .whitespace_nowrap()
                    .child(HighlightedText::new(file).indices(fh)),
            )
            // `.g-fres-dir { flex: 1; min-width: 0; text-overflow: ellipsis;
            // direction: rtl; font-size: 0.75rem; color: var(--text-secondary) }`.
            .child(
                sans(div(), 12., 400)
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_ellipsis_start()
                    .text_color(color(ColorRole::TextSecondary))
                    .child(HighlightedText::new(dir).indices(dh)),
            )
            .when_some(self.kind, |d, k| {
                // `.g-fres-kind { font-size: 0.75rem }` in `.g-dim`.
                d.child(
                    sans(div(), 12., 400)
                        .flex_none()
                        .text_color(color(ColorRole::TextSecondary))
                        .child(k),
                )
            })
    }
}

// ── PalettePage ─────────────────────────────────────────────────────────

/// One palette page (`{ title, placeholder }`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaletteLevel {
    /// `title`: the breadcrumb and the dialog's name.
    pub title: SharedString,
    /// `placeholder` for the input on this page.
    pub placeholder: Option<SharedString>,
}

impl PaletteLevel {
    /// A page titled `title`.
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            placeholder: None,
        }
    }

    /// Its input placeholder.
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }
}

/// The crumb text: the pushed pages' titles (`stack.slice(1)`) joined by
/// " › "; `None` at the top.
pub fn crumb(stack: &[PaletteLevel]) -> Option<String> {
    (stack.len() > 1).then(|| {
        stack[1..]
            .iter()
            .map(|p| p.title.to_string())
            .collect::<Vec<_>>()
            .join(" › ")
    })
}

/// What the palette does on a key: `Esc` goes back, or closes at the top;
/// Backspace in an empty input goes back when it can.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaletteKey {
    /// Pop a page.
    Back,
    /// Call `onClose`.
    Close,
    /// Let the input have it.
    Pass,
}

/// [`PaletteKey`] for `key` at stack `depth` with an `empty` input.
pub fn palette_key(key: &str, depth: usize, empty: bool) -> PaletteKey {
    match key {
        "escape" if depth > 1 => PaletteKey::Back,
        "escape" => PaletteKey::Close,
        "backspace" if empty && depth > 1 => PaletteKey::Back,
        _ => PaletteKey::Pass,
    }
}

/// Navigation handed to the page content: the current page, the query and
/// `push` to open another page.
#[derive(Clone)]
pub struct PaletteNav {
    /// The current page (`page`).
    pub page: PaletteLevel,
    /// How many pages deep (1 at the top).
    pub depth: usize,
    /// The input's text.
    pub query: SharedString,
    stack: Entity<Vec<PaletteLevel>>,
    input: Entity<InputState>,
}

impl PaletteNav {
    /// Open `page` on top (`push`); the input clears and takes focus.
    pub fn push(&self, page: PaletteLevel, window: &mut Window, cx: &mut App) {
        self.stack.update(cx, |s, cx| {
            s.push(page);
            cx.notify();
        });
        fresh_input(&self.input, window, cx);
    }
}

/// A page change gives the new page an empty, focused input: the reference
/// recreates it (`<input key={stack.length} autoFocus>`), so typing reaches
/// the new page and focus is not left on a Back arrow that just went away.
fn fresh_input(input: &Entity<InputState>, window: &mut Window, cx: &mut App) {
    input.update(cx, |s, cx| s.set_value("", window, cx));
    let focus = gpui_kit::Focusable::focus_handle(input.read(cx), cx);
    focus.focus(window, cx);
}

/// The palette's user-facing strings (the reference's defaults).
#[derive(Clone)]
pub struct PalettePageTexts {
    /// The first page when none is given: `"Commands"`.
    pub root: SharedString,
    /// The default placeholder: `"Type a command or search"`.
    pub placeholder: SharedString,
    /// The back button: `"Back to {title}"`.
    pub back_to: Rc<dyn Fn(&str) -> SharedString>,
    /// Footer: `"Open"`.
    pub open: SharedString,
    /// Footer: `"Back"`.
    pub back: SharedString,
    /// Footer: `"Close"`.
    pub close: SharedString,
}

impl Default for PalettePageTexts {
    fn default() -> Self {
        Self {
            root: "Commands".into(),
            placeholder: "Type a command or search".into(),
            back_to: Rc::new(|t| format!("Back to {t}").into()),
            open: "Open".into(),
            back: "Back".into(),
            close: "Close".into(),
        }
    }
}

/// Pops a page; `false` at the top.
type BackHandler = Rc<dyn Fn(&mut Window, &mut App) -> bool>;
type Content = Rc<dyn Fn(&PaletteNav, &mut Window, &mut App) -> Vec<AnyElement>>;

/// A layered command palette. See the module docs.
#[derive(IntoElement)]
pub struct PalettePage {
    id: ElementId,
    pages: Vec<PaletteLevel>,
    content: Option<Content>,
    on_back: Option<Handler>,
    on_close: Option<Handler>,
    texts: PalettePageTexts,
    auto_focus: bool,
}

impl PalettePage {
    /// A palette starting on the "Commands" page.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            pages: Vec::new(),
            content: None,
            on_back: None,
            on_close: None,
            texts: PalettePageTexts::default(),
            auto_focus: true,
        }
    }

    /// Focus the input when the palette mounts (the reference's `<input
    /// autoFocus>`; on by default). Turn it off for a second palette on the
    /// same screen so it does not take focus from the first.
    pub fn auto_focus(mut self, auto_focus: bool) -> Self {
        self.auto_focus = auto_focus;
        self
    }

    /// The initial stack (`pages`).
    pub fn pages(mut self, pages: impl IntoIterator<Item = PaletteLevel>) -> Self {
        self.pages = pages.into_iter().collect();
        self
    }

    /// The results for the current page (`children({ page, push })`).
    pub fn content(
        mut self,
        f: impl Fn(&PaletteNav, &mut Window, &mut App) -> Vec<AnyElement> + 'static,
    ) -> Self {
        self.content = Some(Rc::new(f));
        self
    }

    /// A page was popped (`onBack`).
    pub fn on_back(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_back = Some(Rc::new(f));
        self
    }

    /// Esc at the top (`onClose`).
    pub fn on_close(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(f));
        self
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: PalettePageTexts) -> Self {
        self.texts = texts;
        self
    }
}

/// `.g-pal { box-shadow: 0 2px 6px var(--shadow), 0 0 0 1px var(--border-subtle-01) }`.
fn palette_shadow() -> Vec<BoxShadow> {
    vec![
        BoxShadow::new(px(0.), px(2.), color(ColorRole::Shadow)).blur_radius(px(6.)),
        BoxShadow {
            color: color(ColorRole::BorderSubtle01),
            offset: point(px(0.), px(0.)),
            blur_radius: px(0.),
            spread_radius: px(1.),
            inset: false,
        },
    ]
}

/// A footer hint: key cap then label (`.g-pal-foot span { gap: var(--spacing-02) }`).
fn hint(key: &'static str, label: SharedString) -> Div {
    div()
        .flex()
        .items_center()
        .gap(tokens::SPACING_02)
        .child(Kbd::new(key))
        .child(label)
}

impl RenderOnce for PalettePage {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let initial = if self.pages.is_empty() {
            vec![PaletteLevel::new(self.texts.root.clone())]
        } else {
            self.pages.clone()
        };
        let stack = window.use_keyed_state((id.clone(), "stack"), cx, move |_, _| initial);
        let auto_focus = self.auto_focus;
        let input = window.use_keyed_state((id.clone(), "input"), cx, move |window, cx| {
            let input = cx.new(|cx| InputState::new(window, cx));
            if auto_focus {
                // `<input autoFocus>`: focus once the first frame has the
                // input's focus handle in its dispatch tree.
                let focus = gpui_kit::Focusable::focus_handle(input.read(cx), cx);
                window.defer(cx, move |window, cx| focus.focus(window, cx));
            }
            input
        });
        let input = input.read(cx).clone();
        let pages = stack.read(cx).clone();
        let depth = pages.len();
        let current = pages
            .last()
            .cloned()
            .unwrap_or(PaletteLevel::new(self.texts.root.clone()));
        let query = input.read(cx).value();
        let nav = PaletteNav {
            page: current.clone(),
            depth,
            query: query.clone(),
            stack: stack.clone(),
            input: input.clone(),
        };
        let results = self
            .content
            .as_ref()
            .map(|f| f(&nav, window, cx))
            .unwrap_or_default();
        let back: BackHandler = {
            let stack = stack.clone();
            let input = input.clone();
            let on_back = self.on_back.clone();
            Rc::new(move |window, cx| {
                if stack.read(cx).len() <= 1 {
                    return false;
                }
                stack.update(cx, |s, cx| {
                    s.pop();
                    cx.notify();
                });
                fresh_input(&input, window, cx);
                if let Some(f) = &on_back {
                    f(window, cx);
                }
                true
            })
        };
        let lead: AnyElement = if depth > 1 {
            let back = back.clone();
            IconButton::new(
                (id.clone(), "back"),
                CarbonIcon::ArrowLeft,
                (self.texts.back_to)(&pages[depth - 2].title),
            )
            .on_click(move |_, w, cx| {
                back(w, cx);
            })
            .into_any_element()
        } else {
            dim_icon(CarbonIcon::Search).flex_none().into_any_element()
        };
        let placeholder = query.is_empty().then(|| {
            sans(div(), 16., 400)
                .absolute()
                .left_0()
                .top_0()
                .bottom_0()
                .flex()
                .items_center()
                .text_color(color(ColorRole::TextPlaceholder))
                .child(
                    current
                        .placeholder
                        .clone()
                        .unwrap_or(self.texts.placeholder.clone()),
                )
        });
        let keys = {
            let back = back.clone();
            let input = input.clone();
            let close = self.on_close.clone();
            move |e: &KeyDownEvent, window: &mut Window, cx: &mut App| {
                let empty = input.read(cx).value().is_empty();
                let depth = stack.read(cx).len();
                match palette_key(e.keystroke.key.as_str(), depth, empty) {
                    PaletteKey::Back => {
                        back(window, cx);
                        cx.stop_propagation();
                    }
                    PaletteKey::Close => {
                        if let Some(f) = &close {
                            f(window, cx);
                        }
                        cx.stop_propagation();
                    }
                    PaletteKey::Pass => {}
                }
            }
        };
        // `.g-pal-bar { display: flex; align-items: center; gap:
        // var(--spacing-03); height: var(--palette-input-height); padding: 0
        // var(--spacing-04); border-bottom: 1px solid var(--border-subtle-01) }`.
        let bar = div()
            .relative()
            .flex()
            .items_center()
            .gap(tokens::SPACING_03)
            .h(tokens::PALETTE_INPUT_HEIGHT)
            .px(tokens::SPACING_04)
            .border_b_1()
            .border_color(color(ColorRole::BorderSubtle01))
            .child(lead)
            .when_some(crumb(&pages), |d, c| {
                // `.g-pal-crumb { padding: 0 var(--spacing-03); background:
                // var(--layer-accent-01); font-size: 0.75rem; line-height:
                // 1.25rem; color: var(--text-secondary) }`.
                d.child(
                    sans(div(), 12., 400)
                        .flex_none()
                        .line_height(rem(20.))
                        .px(tokens::SPACING_03)
                        .bg(color(ColorRole::LayerAccent01))
                        .text_color(color(ColorRole::TextSecondary))
                        .child(c),
                )
            })
            .child(
                // `.g-pal-input { flex: 1; font-size: 1rem }`.
                div()
                    .relative()
                    .flex()
                    .items_center()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .children(placeholder)
                    .child(editor(&input).text_size(rem(16.)).line_height(rem(22.))),
            )
            .child(Kbd::new("Esc"));
        // `.g-pal-foot { display: flex; gap: var(--spacing-05); padding:
        // var(--spacing-02) var(--spacing-04); border-top: 1px solid
        // var(--border-subtle-01); font-size: 0.75rem; color: var(--text-secondary) }`.
        let foot = sans(div(), 12., 400)
            .flex()
            .gap(tokens::SPACING_05)
            .py(tokens::SPACING_02)
            .px(tokens::SPACING_04)
            .border_t_1()
            .border_color(color(ColorRole::BorderSubtle01))
            .text_color(color(ColorRole::TextSecondary))
            .child(hint("↵", self.texts.open.clone()))
            .child(if depth > 1 {
                hint("⌫", self.texts.back.clone())
            } else {
                hint("Esc", self.texts.close.clone())
            });
        // `.g-pal { display: flex; flex-direction: column; width: 100%;
        // max-width: 40rem; background: var(--layer-01) }`.
        div()
            .id(id.clone())
            .flex()
            .flex_col()
            .w_full()
            .max_w(rem(640.))
            .bg(color(ColorRole::Layer01))
            .shadow(palette_shadow())
            .capture_key_down(keys)
            .child(bar)
            .child(
                // `.g-pal-body { max-height: 20rem; overflow: auto; padding:
                // var(--spacing-02) 0 }`.
                div()
                    .id((id, "body"))
                    .flex()
                    .flex_col()
                    .max_h(rem(320.))
                    .overflow_y_scroll()
                    .py(tokens::SPACING_02)
                    .children(results),
            )
            .child(foot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CarbonTheme, theme::color_in};

    #[test]
    fn rows_follow_g_pitem() {
        crate::theme::tests_set(CarbonTheme::White);
        let c = |r| color_in(CarbonTheme::White, r);
        assert_eq!(
            row_look(true, false),
            (Some(c(ColorRole::LayerSelected01)), true)
        );
        assert_eq!(
            row_look(false, true),
            (Some(c(ColorRole::LayerHover01)), false)
        );
        assert_eq!(row_look(false, false), (None, false));
    }

    #[test]
    fn file_hits_light_each_query_char_once() {
        let (file, dir, fh, dh) = file_result_hits(
            "packages/web/src/components/sidebar/SidebarThreadRow.tsx",
            "sbtr",
        );
        assert_eq!(file, "SidebarThreadRow.tsx");
        assert_eq!(dir, "packages/web/src/components/sidebar/");
        // S·b in "Sidebar", T·r in "ThreadRow": all in the file name.
        assert_eq!(fh, vec![0, 4, 7, 9]);
        assert!(dh.is_empty());
        // A match spilling into the directory.
        let (_, _, fh, dh) = file_result_hits("README.md", "rd");
        assert_eq!((fh, dh), (vec![0, 3], vec![]));
        let (_, _, fh, dh) = file_result_hits("apps/x.ts", "xa");
        assert_eq!((fh, dh), (vec![0], vec![0]));
    }

    #[test]
    fn keys_and_crumbs() {
        assert_eq!(palette_key("escape", 2, false), PaletteKey::Back);
        assert_eq!(palette_key("escape", 1, true), PaletteKey::Close);
        assert_eq!(palette_key("backspace", 2, true), PaletteKey::Back);
        assert_eq!(palette_key("backspace", 2, false), PaletteKey::Pass);
        assert_eq!(palette_key("backspace", 1, true), PaletteKey::Pass);
        let stack = [
            PaletteLevel::new("Commands"),
            PaletteLevel::new("Switch agent"),
            PaletteLevel::new("Claude"),
        ];
        assert_eq!(crumb(&stack).as_deref(), Some("Switch agent › Claude"));
        assert_eq!(crumb(&stack[..1]), None);
    }
}
