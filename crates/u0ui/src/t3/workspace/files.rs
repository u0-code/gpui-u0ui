//! T3 · Files & preview: [`FileIcon`], [`FileBreadcrumbs`],
//! [`FileBrowserPanel`] and [`FileSurfaceState`] (`src/t3/workspace.jsx`,
//! "Files" in `src/t3.css`).
use super::parts::{child_id, icon_button};
use crate::{
    components::{Spinner, SpinnerSize},
    icons::CarbonIcon,
    motion::{self, Motion},
    t3::{
        base::Spin,
        craft::{mono, sans},
        press::{self, Handler, rem},
        review::FileStatus,
    },
    theme::color,
    tokens::ColorRole,
};
use gpui_kit::{
    AnyElement, App, AppContext as _, CursorStyle, ElementId, Entity, Hsla,
    InteractiveElement as _, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement as _, Styled, Window,
    component::{
        Sizable as _,
        input::{Input, InputState},
    },
    div,
    prelude::FluentBuilder as _,
    relative,
};
use std::{collections::HashSet, rc::Rc};

// ── FileIcon ────────────────────────────────────────────────────────────

/// The glyph of a file or folder (`FileIcon`): folders `Folder` /
/// `FolderOpen`; files by extension (`EXT`: ts/tsx/js/json `Code`, md
/// `Document`, png/jpg `Image`, pdf `DocumentPdf`, css `ColorPalette`),
/// anything else `Document`.
pub fn file_glyph(name: &str, dir: bool, open: bool) -> CarbonIcon {
    if dir {
        return if open {
            CarbonIcon::FolderOpen
        } else {
            CarbonIcon::Folder
        };
    }
    // `name.split('.').pop()`: the whole name when there is no dot.
    match name.rsplit('.').next().unwrap_or(name) {
        "ts" | "tsx" | "js" | "json" => CarbonIcon::Code,
        "md" => CarbonIcon::Document,
        "png" | "jpg" => CarbonIcon::Image,
        "pdf" => CarbonIcon::DocumentPdf,
        "css" => CarbonIcon::ColorPalette,
        _ => CarbonIcon::Document,
    }
}

/// The glyph's fill: `.t3-ficon { fill: var(--icon-secondary) }`,
/// `.t3-ficon--dir { fill: var(--tag-color-blue) }`.
pub fn file_glyph_color(dir: bool) -> Hsla {
    color(if dir {
        ColorRole::TagColorBlue
    } else {
        ColorRole::IconSecondary
    })
}

/// A 16px file-type or folder glyph (`FileIcon`).
#[derive(IntoElement, Clone, Debug)]
pub struct FileIcon {
    name: SharedString,
    dir: bool,
    open: bool,
}

impl FileIcon {
    /// The glyph for the file `name`.
    pub fn new(name: impl Into<SharedString>) -> Self {
        Self {
            name: name.into(),
            dir: false,
            open: false,
        }
    }

    /// A folder (`dir`).
    pub fn dir(mut self, dir: bool) -> Self {
        self.dir = dir;
        self
    }

    /// An expanded folder (`open`).
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }
}

impl RenderOnce for FileIcon {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        press::icon(
            file_glyph(&self.name, self.dir, self.open),
            16.,
            file_glyph_color(self.dir),
        )
        .flex_none()
    }
}

// ── FileBreadcrumbs ─────────────────────────────────────────────────────

/// The segments of a path (`path.split('/').filter(Boolean)`).
pub fn path_segments(path: &str) -> Vec<&str> {
    path.split('/').filter(|s| !s.is_empty()).collect()
}

/// A file path as breadcrumbs (`FileBreadcrumbs`, `.t3-fcrumbs`): folder
/// segments with a 14px folder glyph, 12px chevrons between, the file last
/// in `text-primary` 600. Each segment is a button.
#[derive(IntoElement)]
pub struct FileBreadcrumbs {
    id: ElementId,
    path: SharedString,
    label: SharedString,
    on_select: Option<super::parts::IndexHandler>,
}

impl FileBreadcrumbs {
    /// Breadcrumbs for `path`.
    pub fn new(id: impl Into<ElementId>, path: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            path: path.into(),
            label: "Path".into(),
            on_select: None,
        }
    }

    /// The navigation's accessible name (`aria-label`, default `Path`).
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    /// A segment pressed, with its index.
    pub fn on_select(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for FileBreadcrumbs {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let segments = path_segments(&self.path);
        let last = segments.len().saturating_sub(1);
        let mut parts: Vec<AnyElement> = Vec::new();
        for (i, seg) in segments.iter().enumerate() {
            if i > 0 {
                parts.push(
                    press::icon(
                        CarbonIcon::ChevronRight,
                        12.,
                        color(ColorRole::IconSecondary),
                    )
                    .flex_none()
                    .into_any_element(),
                );
            }
            let current = i == last;
            let seg_id = child_id(&id, "crumb", i);
            let (hovered, store) = press::hover((seg_id.clone(), "hover"), window, cx);
            let (focus, ring) = press::focus((seg_id.clone(), "focus"), window, cx);
            // `.t3-fcrumb { color: text-secondary }`, `:hover { background:
            // background-hover; color: text-primary }`, `--cur { color: text-primary; 600 }`.
            let ink = color(if current || hovered {
                ColorRole::TextPrimary
            } else {
                ColorRole::TextSecondary
            });
            let on_select = self.on_select.clone();
            parts.push(
                sans(
                    gpui_kit::base::Button::new(seg_id)
                        .track_focus(&focus)
                        .accessibility_label(SharedString::from(seg.to_string())),
                    12.,
                    if current { 600 } else { 400 },
                )
                // `.t3-fcrumb { display: inline-flex; align-items: center; gap: spacing-02;
                // height: 1.5rem; padding: 0 spacing-02 }`.
                .relative()
                .flex()
                .flex_row()
                .items_center()
                .gap(rem(4.))
                .h(rem(24.))
                .px(rem(4.))
                .text_color(ink)
                .when(hovered, |this| this.bg(color(ColorRole::BackgroundHover)))
                .cursor(CursorStyle::PointingHand)
                .on_hover(press::track_hover(store))
                .when_some(on_select, |this, f| {
                    this.on_click(move |_, w, cx| f(i, w, cx))
                })
                // `svg { fill: currentColor }`.
                .when(!current, |this| {
                    this.child(press::icon(CarbonIcon::Folder, 14., ink).flex_none())
                })
                .child(SharedString::from(seg.to_string()))
                .when(ring, |this| this.child(press::ring()))
                .into_any_element(),
            );
        }
        // `.t3-fcrumbs { display: flex; flex-wrap: wrap; align-items: center; gap: 2px }`.
        div()
            .id(id)
            .flex()
            .flex_row()
            .flex_wrap()
            .items_center()
            .gap(rem(2.))
            .children(parts)
    }
}

// ── FileBrowserPanel ────────────────────────────────────────────────────

/// One entry of a [`FileBrowserPanel`] tree (`{ path, name, dir, open,
/// status, children }`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FileNode {
    /// `path`: unique, the selection key.
    pub path: SharedString,
    /// `name`.
    pub name: SharedString,
    /// `dir`.
    pub dir: bool,
    /// `open`: expanded at first.
    pub open: bool,
    /// `status`: the Git status letter.
    pub status: Option<FileStatus>,
    /// `children`.
    pub children: Vec<FileNode>,
}

impl FileNode {
    /// A file.
    pub fn file(path: impl Into<SharedString>, name: impl Into<SharedString>) -> Self {
        Self {
            path: path.into(),
            name: name.into(),
            ..Default::default()
        }
    }

    /// A folder with `children`.
    pub fn dir(
        path: impl Into<SharedString>,
        name: impl Into<SharedString>,
        children: impl IntoIterator<Item = FileNode>,
    ) -> Self {
        Self {
            path: path.into(),
            name: name.into(),
            dir: true,
            children: children.into_iter().collect(),
            ..Default::default()
        }
    }

    /// Expanded at first (`open`).
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// The Git status (`status`).
    pub fn status(mut self, status: FileStatus) -> Self {
        self.status = Some(status);
        self
    }
}

/// One visible row of the tree: the node and its depth.
#[derive(Clone, Debug, PartialEq)]
pub struct FileRow<'a> {
    /// The node.
    pub node: &'a FileNode,
    /// Nesting depth (0 at the root).
    pub depth: usize,
}

/// The paths expanded at first (every node with `open`).
pub fn initially_open(tree: &[FileNode]) -> HashSet<SharedString> {
    fn walk(nodes: &[FileNode], set: &mut HashSet<SharedString>) {
        for n in nodes {
            if n.open {
                set.insert(n.path.clone());
            }
            walk(&n.children, set);
        }
    }
    let mut set = HashSet::new();
    walk(tree, &mut set);
    set
}

/// The visible rows (`walk`): with a query, files whose path does not
/// contain it (case-insensitively) drop out and every folder shows its
/// children; without one, a folder's children show when it is open.
pub fn visible_rows<'a>(
    tree: &'a [FileNode],
    open: &HashSet<SharedString>,
    query: &str,
) -> Vec<FileRow<'a>> {
    let q = query.to_lowercase();
    let mut rows = Vec::new();
    fn walk<'a>(
        nodes: &'a [FileNode],
        depth: usize,
        open: &HashSet<SharedString>,
        q: &str,
        rows: &mut Vec<FileRow<'a>>,
    ) {
        for n in nodes {
            if !q.is_empty() && !n.dir && !n.path.to_lowercase().contains(q) {
                continue;
            }
            rows.push(FileRow { node: n, depth });
            if n.dir && (open.contains(&n.path) || !q.is_empty()) {
                walk(&n.children, depth + 1, open, q, rows);
            }
        }
    }
    walk(tree, 0, open, &q, &mut rows);
    rows
}

/// `.t3-frow { padding-left: 8 + depth × 16 }` (inline style).
pub fn row_indent(depth: usize) -> f32 {
    8. + depth as f32 * 16.
}

/// Texts of a [`FileBrowserPanel`] (hard-coded in the reference).
#[derive(Clone, Debug)]
pub struct FileBrowserTexts {
    /// Search placeholder and name (`Search files`).
    pub search: SharedString,
    /// Refresh tooltip (`Refresh files`).
    pub refresh: SharedString,
    /// Loading (`Loading files…`).
    pub loading: SharedString,
    /// No rows (`No matching files.`).
    pub empty: SharedString,
}

impl Default for FileBrowserTexts {
    fn default() -> Self {
        Self {
            search: "Search files".into(),
            refresh: "Refresh files".into(),
            loading: "Loading files\u{2026}".into(),
            empty: "No matching files.".into(),
        }
    }
}

struct BrowserState {
    query: Entity<InputState>,
    open: HashSet<SharedString>,
}

/// A file browser (`FileBrowserPanel`, `.t3-files`): a search field and
/// refresh button over a tree of folders and files with type glyphs and Git
/// status letters. Folders toggle on click, Enter or Space (the chevron
/// turns 90° in 110ms); files call `on_select`. `layer-01`, at least 16rem
/// tall, filling its parent's height.
#[derive(IntoElement)]
pub struct FileBrowserPanel {
    id: ElementId,
    tree: Vec<FileNode>,
    selected: Option<SharedString>,
    loading: bool,
    query: SharedString,
    on_select: Option<super::parts::TextHandler>,
    on_refresh: Option<Handler>,
    texts: FileBrowserTexts,
}

impl FileBrowserPanel {
    /// A panel over `tree`.
    pub fn new(id: impl Into<ElementId>, tree: impl IntoIterator<Item = FileNode>) -> Self {
        Self {
            id: id.into(),
            tree: tree.into_iter().collect(),
            selected: None,
            loading: false,
            query: SharedString::default(),
            on_select: None,
            on_refresh: None,
            texts: FileBrowserTexts::default(),
        }
    }

    /// The selected file's path (`selected`).
    pub fn selected(mut self, path: impl Into<SharedString>) -> Self {
        self.selected = Some(path.into());
        self
    }

    /// Show the loading message instead of the tree (`loading`).
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    /// The initial search text (`query`).
    pub fn query(mut self, query: impl Into<SharedString>) -> Self {
        self.query = query.into();
        self
    }

    /// A file pressed (`onSelect(path)`).
    pub fn on_select(mut self, f: impl Fn(SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }

    /// Refresh pressed.
    pub fn on_refresh(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_refresh = Some(Rc::new(f));
        self
    }

    /// Replace the default texts.
    pub fn texts(mut self, texts: FileBrowserTexts) -> Self {
        self.texts = texts;
        self
    }
}

impl RenderOnce for FileBrowserPanel {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let state = window.use_keyed_state((id.clone(), "state"), cx, {
            let query = self.query.clone();
            let open = initially_open(&self.tree);
            move |window, cx| BrowserState {
                query: cx.new(|cx| InputState::new(window, cx).default_value(query)),
                open,
            }
        });
        let input = state.read(cx).query.clone();
        let query = input.read(cx).value().to_string();
        let open = state.read(cx).open.clone();
        let secondary = color(ColorRole::IconSecondary);

        // `.t3-files-search { display: flex; flex: 1; align-items: center; gap: spacing-03;
        // height: size-md; padding: 0 spacing-04; min-width: 0 }`.
        let search = div()
            .relative()
            .flex()
            .flex_row()
            .flex_1()
            .min_w_0()
            .items_center()
            .gap(rem(8.))
            .h(rem(40.))
            .px(rem(12.))
            .child(press::icon(CarbonIcon::Search, 16., secondary).flex_none())
            .child(
                // `input { flex: 1; border: 0; background: none; font: inherit; outline: none }`.
                sans(div(), 14., 400)
                    .relative()
                    .flex()
                    .flex_1()
                    .min_w_0()
                    .items_center()
                    .h_full()
                    .text_color(color(ColorRole::TextPrimary))
                    .when(query.is_empty(), |this| {
                        this.child(
                            div()
                                .absolute()
                                .left_0()
                                .text_color(color(ColorRole::TextPlaceholder))
                                .child(self.texts.search.clone()),
                        )
                    })
                    .child(
                        Input::new(&input)
                            .appearance(false)
                            .with_size(gpui_kit::component::Size::XSmall)
                            .aria_label(self.texts.search.clone())
                            .p_0()
                            .h_full()
                            .w_full()
                            .text_size(rem(14.))
                            .line_height(relative(crate::t3::craft::BODY_LINE)),
                    ),
            );
        let on_refresh = self.on_refresh.clone();
        // `.t3-files-bar { display: flex; align-items: center; gap: spacing-02;
        // padding-right: spacing-02; border-bottom: 1px solid border-subtle-01 }`.
        let bar = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(rem(4.))
            .pr(rem(4.))
            .border_b_1()
            .border_color(color(ColorRole::BorderSubtle01))
            .child(search)
            .child(
                icon_button(
                    (id.clone(), "refresh"),
                    self.texts.refresh.clone(),
                    CarbonIcon::Renew,
                )
                .when_some(on_refresh, |b, f| b.on_click(move |_, w, cx| f(w, cx))),
            );

        // `.t3-files-msg { padding: spacing-05; font-size: 0.875rem; color: text-secondary }`.
        let message = |content: AnyElement| {
            sans(div(), 14., 400)
                .p(rem(16.))
                .text_color(color(ColorRole::TextSecondary))
                .child(content)
                .into_any_element()
        };
        let body: Vec<AnyElement> = if self.loading {
            vec![message(
                Spinner::new((id.clone(), "loading"))
                    .size(SpinnerSize::Inline)
                    .description(self.texts.loading.clone())
                    .into_any_element(),
            )]
        } else {
            let rows = visible_rows(&self.tree, &open, &query);
            if rows.is_empty() {
                vec![message(self.texts.empty.clone().into_any_element())]
            } else {
                rows.iter()
                    .map(|row| {
                        file_row(
                            &id,
                            row,
                            open.contains(&row.node.path),
                            self.selected.as_ref() == Some(&row.node.path),
                            &state,
                            self.on_select.clone(),
                            window,
                            cx,
                        )
                    })
                    .collect()
            }
        };
        // `.t3-files-tree { flex: 1; overflow-y: auto; padding: spacing-02 0 }`.
        let tree = crate::components::ScrollArea::new((id.clone(), "tree"))
            .axes(crate::components::ScrollAxes::Vertical)
            .focusable(false)
            .flex_1()
            .min_h_0()
            .child(div().flex().flex_col().py(rem(4.)).children(body));
        // `.t3-files { display: flex; flex-direction: column; height: 100%; min-height: 16rem;
        // background: layer-01 }`.
        div()
            .id(id)
            .flex()
            .flex_col()
            .size_full()
            .min_h(rem(256.))
            .bg(color(ColorRole::Layer01))
            .child(bar)
            .child(tree)
    }
}

#[allow(clippy::too_many_arguments)]
fn file_row(
    id: &ElementId,
    row: &FileRow<'_>,
    is_open: bool,
    selected: bool,
    state: &Entity<BrowserState>,
    on_select: Option<super::parts::TextHandler>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let node = row.node;
    let row_id = ElementId::from((id.clone(), SharedString::from(format!("row:{}", node.path))));
    let (hovered, store) = press::hover((row_id.clone(), "hover"), window, cx);
    let (focus, ring) = press::focus((row_id.clone(), "focus"), window, cx);
    let bg = if selected {
        // `.t3-frow--on { background: layer-selected-01 }`.
        Some(color(ColorRole::LayerSelected01))
    } else if hovered {
        // `.t3-frow:hover { background: layer-hover-01 }`.
        Some(color(ColorRole::LayerHover01))
    } else {
        None
    };
    // `.t3-chev { transition: transform 110ms productive } .t3-chev--open { rotate(90deg) }`.
    let turn = motion::animate(
        (row_id.clone(), "chev"),
        if is_open {
            std::f32::consts::FRAC_PI_2
        } else {
            0.
        },
        Motion::TOGGLE,
        window,
        cx,
    );
    let lead = if node.dir {
        press::icon(
            CarbonIcon::ChevronRight,
            12.,
            color(ColorRole::IconSecondary),
        )
        .rotate(gpui_kit::radians(turn))
        .flex_none()
        .into_any_element()
    } else {
        // `.t3-frow-sp { width: 12px; flex: none }`.
        div().w(rem(12.)).flex_none().into_any_element()
    };
    let path = node.path.clone();
    let dir = node.dir;
    let state = state.clone();
    let status = node.status.map(|s| {
        let (fill, ink) = s.colors();
        // `.t3-frow-git { font-family: mono; font-size: 0.6875rem; font-weight: 600;
        // padding: 0 4px }` with the `.t3-fstat--<s>` pair.
        mono(div(), 11., 600)
            .flex_none()
            .px(rem(4.))
            .bg(fill)
            .text_color(ink)
            .child(s.letter())
    });
    sans(
        gpui_kit::base::Button::new(row_id)
            .track_focus(&focus)
            .accessibility_label(node.name.clone()),
        14.,
        400,
    )
    // `.t3-frow { display: flex; align-items: center; gap: spacing-02; height: 1.75rem;
    // padding-right: spacing-04; font-size: 0.875rem; cursor: pointer }`.
    .relative()
    .flex()
    .flex_row()
    .items_center()
    .justify_start()
    .gap(rem(4.))
    .w_full()
    .h(rem(28.))
    .pl(rem(row_indent(row.depth)))
    .pr(rem(12.))
    .text_color(color(ColorRole::TextPrimary))
    .when_some(bg, |this, bg| this.bg(bg))
    .cursor(CursorStyle::PointingHand)
    .on_hover(press::track_hover(store))
    .on_click(move |_, window, cx| {
        if dir {
            let path = path.clone();
            state.update(cx, |s, cx| {
                if !s.open.remove(&path) {
                    s.open.insert(path);
                }
                cx.notify();
            });
        } else if let Some(f) = on_select.as_ref() {
            f(path.clone(), window, cx);
        }
    })
    .child(lead)
    .child(FileIcon::new(node.name.clone()).dir(node.dir).open(is_open))
    .child(
        // `.t3-frow-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap }`.
        div()
            .flex_1()
            .min_w_0()
            .overflow_hidden()
            .text_ellipsis()
            .whitespace_nowrap()
            .text_left()
            .child(node.name.clone()),
    )
    .children(status)
    .when(ring, |this| this.child(press::ring()))
    .into_any_element()
}

// ── FileSurfaceState ────────────────────────────────────────────────────

/// `FileSurfaceState` `kind`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FileSurfaceKind {
    /// `loading` (the default): a 24px spinner.
    #[default]
    Loading,
    /// `notice`: `InformationFilled`.
    Notice,
    /// `failure`: `ErrorFilled` in `support-error`.
    Failure,
    /// `binary`: `Document`.
    Binary,
    /// `large`: `WarningAltFilled` in `support-warning`.
    Large,
}

impl FileSurfaceKind {
    /// Every kind.
    pub const ALL: [FileSurfaceKind; 5] = [
        Self::Loading,
        Self::Notice,
        Self::Failure,
        Self::Binary,
        Self::Large,
    ];

    /// The reference name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Loading => "loading",
            Self::Notice => "notice",
            Self::Failure => "failure",
            Self::Binary => "binary",
            Self::Large => "large",
        }
    }

    /// The glyph and its fill; `None` for `loading` (a spinner).
    /// `.t3-fsurf > svg { fill: icon-secondary }`, `--failure` `support-error`,
    /// `--large` `support-warning`.
    pub fn glyph(self) -> Option<(CarbonIcon, ColorRole)> {
        match self {
            Self::Loading => None,
            Self::Notice => Some((CarbonIcon::InformationFilled, ColorRole::IconSecondary)),
            Self::Failure => Some((CarbonIcon::ErrorFilled, ColorRole::SupportError)),
            Self::Binary => Some((CarbonIcon::Document, ColorRole::IconSecondary)),
            Self::Large => Some((CarbonIcon::WarningAltFilled, ColorRole::SupportWarning)),
        }
    }
}

/// A file preview's state placeholder (`FileSurfaceState`, `.t3-fsurf`): a
/// centred 24px glyph or spinner, a 600 title (selectable), an optional note and
/// actions, on `layer-01`, at least 12rem tall.
#[derive(IntoElement)]
pub struct FileSurfaceState {
    id: ElementId,
    kind: FileSurfaceKind,
    title: Option<SharedString>,
    loading_title: SharedString,
    body: Option<AnyElement>,
    actions: Vec<AnyElement>,
}

impl FileSurfaceState {
    /// A `kind` placeholder.
    pub fn new(id: impl Into<ElementId>, kind: FileSurfaceKind) -> Self {
        Self {
            id: id.into(),
            kind,
            title: None,
            loading_title: "Loading preview\u{2026}".into(),
            body: None,
            actions: Vec::new(),
        }
    }

    /// The title (`title`; `loading` defaults to `Loading preview…`).
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Replace `loading`'s default title.
    pub fn loading_title(mut self, title: impl Into<SharedString>) -> Self {
        self.loading_title = title.into();
        self
    }

    /// The note under the title (`children`).
    pub fn body(mut self, body: impl IntoElement) -> Self {
        self.body = Some(body.into_any_element());
        self
    }

    /// Actions under the note (`actions`).
    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.actions.push(action.into_any_element());
        self
    }
}

impl RenderOnce for FileSurfaceState {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let mark = match self.kind.glyph() {
            None => Spin::new((self.id.clone(), "spin"))
                .size(24.)
                .into_any_element(),
            Some((glyph, role)) => press::icon(glyph, 24., color(role))
                .flex_none()
                .into_any_element(),
        };
        let title = self.title.clone().or_else(|| {
            (self.kind == FileSurfaceKind::Loading).then(|| self.loading_title.clone())
        });
        // `.t3-fsurf { display: flex; flex-direction: column; align-items: center;
        // justify-content: center; gap: spacing-03; min-height: 12rem; padding: spacing-07;
        // text-align: center; background: layer-01; font-size: 0.875rem }`.
        let id = self.id.clone();
        sans(div(), 14., 400)
            .id(self.id)
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(rem(8.))
            .min_h(rem(192.))
            .p(rem(32.))
            .text_center()
            .bg(color(ColorRole::Layer01))
            .text_color(color(ColorRole::TextPrimary))
            .child(mark)
            // `.t3-fsurf b { font-size: 0.875rem; font-weight: 600 }`,
            // selectable, so what a failure says can be copied.
            .child(
                sans(div(), 14., 600).child(gpui_kit::base::SelectableText::new(
                    (id.clone(), "title"),
                    title.unwrap_or_default(),
                )),
            )
            .when_some(self.body, |this, body| {
                // `.t3-fsurf p { margin: 0; max-width: 24rem; font-size: 0.875rem;
                // line-height: 1.42857; color: text-secondary }`.
                this.child(
                    sans(div(), 14., 400)
                        .max_w(rem(384.))
                        .line_height(relative(1.42857))
                        .text_color(color(ColorRole::TextSecondary))
                        .child(body),
                )
            })
            .when(!self.actions.is_empty(), |this| {
                // `.t3-card-actions { display: flex; flex-wrap: wrap; justify-content: flex-end;
                // gap: spacing-03 }`.
                this.child(
                    div()
                        .flex()
                        .flex_row()
                        .flex_wrap()
                        .justify_end()
                        .items_center()
                        .gap(rem(8.))
                        .children(self.actions),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glyphs_follow_ext() {
        assert_eq!(file_glyph("Button.tsx", false, false), CarbonIcon::Code);
        assert_eq!(file_glyph("tokens.json", false, false), CarbonIcon::Code);
        assert_eq!(file_glyph("README.md", false, false), CarbonIcon::Document);
        assert_eq!(file_glyph("logo.png", false, false), CarbonIcon::Image);
        assert_eq!(file_glyph("a.pdf", false, false), CarbonIcon::DocumentPdf);
        assert_eq!(file_glyph("x.css", false, false), CarbonIcon::ColorPalette);
        assert_eq!(file_glyph("Makefile", false, false), CarbonIcon::Document);
        assert_eq!(file_glyph("src", true, false), CarbonIcon::Folder);
        assert_eq!(file_glyph("src", true, true), CarbonIcon::FolderOpen);
    }

    fn tree() -> Vec<FileNode> {
        vec![
            FileNode::dir(
                "src",
                "src",
                [
                    FileNode::dir(
                        "src/components",
                        "components",
                        [FileNode::file("src/components/Button.tsx", "Button.tsx")],
                    ),
                    FileNode::file("src/tokens.json", "tokens.json").status(FileStatus::Modified),
                ],
            )
            .open(true),
            FileNode::file("README.md", "README.md"),
        ]
    }

    #[test]
    fn rows_walk_open_folders_and_filter_files() {
        let tree = tree();
        let open = initially_open(&tree);
        let names = |rows: Vec<FileRow<'_>>| {
            rows.iter()
                .map(|r| (r.node.name.to_string(), r.depth))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            names(visible_rows(&tree, &open, "")),
            vec![
                ("src".into(), 0),
                ("components".into(), 1),
                ("tokens.json".into(), 1),
                ("README.md".into(), 0)
            ]
        );
        // A query opens every folder and keeps only matching files.
        assert_eq!(
            names(visible_rows(&tree, &open, "BUTTON")),
            vec![
                ("src".into(), 0),
                ("components".into(), 1),
                ("Button.tsx".into(), 2)
            ]
        );
    }

    #[test]
    fn rows_indent_by_depth_and_crumbs_split_the_path() {
        assert_eq!(row_indent(0), 8.);
        assert_eq!(row_indent(2), 40.);
        assert_eq!(path_segments("/a//b/c.tsx"), vec!["a", "b", "c.tsx"]);
    }
}
