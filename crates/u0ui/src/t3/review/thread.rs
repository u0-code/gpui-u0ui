//! T3 · Thread: [`ChatHeader`], [`BranchToolbar`], [`GitActionsControl`],
//! [`ThreadDetailsSection`], [`ContextMeter`] and [`ThreadHoverCard`]
//! (`src/t3.css` "Chat header / branch toolbar / git").
use crate::{
    components::{
        DropdownMenu, DropdownMenuKind, DropdownMenuSize, MenuItem,
        button::{ButtonKind, ButtonSize, ButtonState, ButtonVariant, Geometry, Look},
    },
    density,
    icons::{CarbonIcon, Icon},
    motion::{self, Motion},
    t3::{
        base::{Spin, Status, warning_glyph},
        craft::{self, focus_handle, focus_visible, mono, press, rem, sans, sub},
        shell::{
            BadgeSize, ConnectionStatus, Count, EnvironmentBadge, IconMenu, IdentitySize,
            LayoutPanel, MachineKind, PanelLayoutControls, ProjectFavicon, StatusPill,
        },
    },
    theme::color,
    tokens::ColorRole,
};
use gpui_kit::{
    AnyElement, App, BoxShadow, ElementId, Hsla, InteractiveElement, IntoElement, MouseButton,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _, px, relative,
};
use std::rc::Rc;

type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

// ── ChatHeader ───────────────────────────────────────────────────────────

/// The environment shown in a chat header (`environment`).
#[derive(Clone, Debug)]
pub struct HeaderEnvironment {
    /// `name`.
    pub name: SharedString,
    /// `kind`.
    pub kind: MachineKind,
    /// `status`.
    pub status: ConnectionStatus,
}

impl HeaderEnvironment {
    /// An environment.
    pub fn new(name: impl Into<SharedString>, kind: MachineKind, status: ConnectionStatus) -> Self {
        Self {
            name: name.into(),
            kind,
            status,
        }
    }
}

/// The layout toggles' state (`layout`): which panels are open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeaderLayout {
    /// `sidebar` (default on).
    pub sidebar: bool,
    /// `rightPanel`.
    pub right_panel: bool,
    /// `terminal`.
    pub terminal: bool,
}

impl Default for HeaderLayout {
    fn default() -> Self {
        Self {
            sidebar: true,
            right_panel: false,
            terminal: false,
        }
    }
}

type PanelHandler = Rc<dyn Fn(LayoutPanel, &mut Window, &mut App)>;

/// The thread title bar (`.t3-chathead`, 48px): the title (press to rename),
/// a meta line (project, branch, environment, status), then the caller's
/// actions and the panel layout toggles. `inline` drops the frame to sit in
/// a toolbar.
#[derive(IntoElement)]
pub struct ChatHeader {
    id: ElementId,
    title: SharedString,
    project: Option<SharedString>,
    branch: Option<SharedString>,
    status: Option<Status>,
    status_label: Option<SharedString>,
    environment: Option<HeaderEnvironment>,
    actions: Vec<AnyElement>,
    layout: Option<HeaderLayout>,
    inline: bool,
    on_rename: Option<Handler>,
    on_layout: Option<PanelHandler>,
    rename_label: SharedString,
}

impl ChatHeader {
    /// A header for thread `title`.
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            project: None,
            branch: None,
            status: None,
            status_label: None,
            environment: None,
            actions: Vec::new(),
            layout: Some(HeaderLayout::default()),
            inline: false,
            on_rename: None,
            on_layout: None,
            rename_label: "Rename thread".into(),
        }
    }

    /// The project.
    pub fn project(mut self, project: impl Into<SharedString>) -> Self {
        self.project = Some(project.into());
        self
    }

    /// The branch.
    pub fn branch(mut self, branch: impl Into<SharedString>) -> Self {
        self.branch = Some(branch.into());
        self
    }

    /// The thread status.
    pub fn status(mut self, status: Status) -> Self {
        self.status = Some(status);
        self
    }

    /// The status pill's words in place of the status's English name
    /// (an app's own language).
    pub fn status_label(mut self, label: impl Into<SharedString>) -> Self {
        self.status_label = Some(label.into());
        self
    }

    /// The environment.
    pub fn environment(mut self, environment: HeaderEnvironment) -> Self {
        self.environment = Some(environment);
        self
    }

    /// An action before the layout toggles (`actions`).
    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.actions.push(action.into_any_element());
        self
    }

    /// The layout toggles' state; `None` hides them (`layout={false}`).
    pub fn layout(mut self, layout: Option<HeaderLayout>) -> Self {
        self.layout = layout;
        self
    }

    /// Sit inline in a toolbar (`inline`).
    pub fn inline(mut self, inline: bool) -> Self {
        self.inline = inline;
        self
    }

    /// The title was pressed (`onRename`).
    pub fn on_rename(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_rename = Some(Rc::new(f));
        self
    }

    /// A layout toggle was pressed.
    pub fn on_layout(mut self, f: impl Fn(LayoutPanel, &mut Window, &mut App) + 'static) -> Self {
        self.on_layout = Some(Rc::new(f));
        self
    }

    /// The title's tooltip (`Rename thread`).
    pub fn rename_label(mut self, label: impl Into<SharedString>) -> Self {
        self.rename_label = label.into();
        self
    }
}

impl RenderOnce for ChatHeader {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let title_id: ElementId = (id.clone(), "title").into();
        let focus = focus_handle(&title_id, window, cx);
        let ring = focus_visible(&focus, window);
        let on_rename = self.on_rename.clone();
        let secondary = color(ColorRole::TextSecondary);
        // `.t3-chathead-title { 14px 600; ellipsis; cursor: text }`.
        let title = sans(press(title_id, &focus), 14., 600)
            .self_start()
            .max_w_full()
            .overflow_hidden()
            .whitespace_nowrap()
            .text_ellipsis()
            .cursor_text()
            .text_color(color(ColorRole::TextPrimary))
            .tooltip(craft::title_tooltip(self.rename_label.clone()))
            .when_some(on_rename, |b, f| b.on_click(move |_, w, cx| f(w, cx)))
            // The text ellipsizes in its own box: the press's content box
            // does not pass its max width down to a bare string child.
            .child(div().min_w_0().truncate().child(self.title.clone()))
            .when(ring, |b| b.child(craft::focus_ring_offset(2.)));
        let sub_line = sans(div(), 12., 400)
            .flex()
            .items_center()
            .gap(rem(8.))
            .min_w_0()
            .overflow_hidden()
            .whitespace_nowrap()
            .text_color(secondary)
            .when_some(self.project.clone(), |d, p| {
                d.child(
                    div()
                        .flex()
                        .flex_none()
                        .items_center()
                        .gap(rem(4.))
                        .child(ProjectFavicon::new(p.clone()).size(IdentitySize::Xs))
                        .child(p),
                )
            })
            .when_some(self.branch.clone(), |d, b| {
                d.child(
                    mono(div(), 12., 400)
                        .flex()
                        .flex_none()
                        .items_center()
                        .gap(rem(4.))
                        .child(
                            Icon::carbon(CarbonIcon::Branch)
                                .size_length(rem(12.))
                                .color(secondary),
                        )
                        .child(b),
                )
            })
            .when_some(self.environment.clone(), |d, e| {
                d.child(
                    EnvironmentBadge::new(e.name)
                        .kind(e.kind)
                        .status(e.status)
                        .size(BadgeSize::Sm),
                )
            })
            .when_some(self.status, |d, s| {
                d.child(StatusPill::new(s).when_some(self.status_label.clone(), |p, l| p.label(l)))
            });
        let on_layout = self.on_layout.clone();
        let layout = self.layout.map(|l| {
            PanelLayoutControls::new((id.clone(), "layout"))
                .sidebar(l.sidebar)
                .right_panel(l.right_panel)
                .terminal(l.terminal)
                .when_some(on_layout, |c, f| c.on_toggle(move |p, w, cx| f(p, w, cx)))
        });
        div()
            .id(id)
            .flex()
            .items_center()
            .gap(rem(16.))
            .map(|d| {
                if self.inline {
                    // `.t3-chathead--inline { flex: 1 1 auto; min-width: 0; padding-left: 0; no border }`.
                    d.flex_1().min_w_0()
                } else {
                    d.min_h(rem(48.))
                        .pt(rem(4.))
                        .pb(rem(4.))
                        .pr(rem(8.))
                        .pl(rem(16.))
                        .border_b_1()
                        .border_color(color(ColorRole::BorderSubtle01))
                        .bg(color(ColorRole::Background))
                }
            })
            .child(
                // Inline, in a 38px title bar, the title and its meta are
                // one row (`.t3-chathead--inline .t3-chathead-main {
                // flex-direction: row; align-items: center; gap: 12px }`,
                // the title and the meta `flex: 0 1 auto`).
                div()
                    .flex()
                    .flex_1()
                    .min_w_0()
                    .map(|d| {
                        if self.inline {
                            d.flex_row().items_center().gap(rem(12.))
                        } else {
                            d.flex_col().gap(px(2.))
                        }
                    })
                    .child(title.when(self.inline, |t| t.flex_shrink_1().self_center()))
                    .child(sub_line.when(self.inline, |l| l.flex_shrink_1())),
            )
            .child(
                div()
                    .flex()
                    .flex_none()
                    .items_center()
                    .gap(rem(8.))
                    .children(self.actions)
                    .children(layout),
            )
    }
}

// ── BranchToolbar ────────────────────────────────────────────────────────

/// Where a new thread works (`mode`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum WorkspaceMode {
    /// `local`: in the checkout.
    Local,
    /// `worktree`, the default: a new worktree.
    #[default]
    Worktree,
}

impl WorkspaceMode {
    /// Both modes, in order.
    pub const ALL: [WorkspaceMode; 2] = [Self::Local, Self::Worktree];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Worktree => "worktree",
        }
    }
}

/// Texts of a [`BranchToolbar`] (hard-coded in the reference).
#[derive(Clone, Debug)]
pub struct BranchToolbarTexts {
    /// `Run on`.
    pub run_on: SharedString,
    /// The default environment (`This computer`).
    pub this_computer: SharedString,
    /// `Add to another machine`.
    pub add_machine: SharedString,
    /// The segmented control's name (`Workspace`).
    pub workspace: SharedString,
    /// `Local`.
    pub local: SharedString,
    /// `New worktree`.
    pub worktree: SharedString,
    /// The default branch (`main`).
    pub main: SharedString,
}

impl Default for BranchToolbarTexts {
    fn default() -> Self {
        Self {
            run_on: "Run on".into(),
            this_computer: "This computer".into(),
            add_machine: "Add to another machine".into(),
            workspace: "Workspace".into(),
            local: "Local".into(),
            worktree: "New worktree".into(),
            main: "main".into(),
        }
    }
}

type ModeHandler = Rc<dyn Fn(WorkspaceMode, &mut Window, &mut App)>;
type NameHandler = Rc<dyn Fn(SharedString, &mut Window, &mut App)>;

/// The run-settings bar of a new thread (`.t3-branchbar` on `layer-01`):
/// "Run on" + environment menu, the Local / New worktree segmented
/// control, and the base-branch menu, split by 1px rules.
#[derive(IntoElement)]
pub struct BranchToolbar {
    id: ElementId,
    environments: Vec<SharedString>,
    environment: Option<SharedString>,
    mode: Option<WorkspaceMode>,
    default_mode: WorkspaceMode,
    branch: Option<SharedString>,
    branches: Vec<SharedString>,
    on_mode_change: Option<ModeHandler>,
    on_environment: Option<NameHandler>,
    on_branch: Option<NameHandler>,
    on_add_machine: Option<Handler>,
    texts: BranchToolbarTexts,
}

impl BranchToolbar {
    /// A toolbar.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            environments: Vec::new(),
            environment: None,
            mode: None,
            default_mode: WorkspaceMode::Worktree,
            branch: None,
            branches: Vec::new(),
            on_mode_change: None,
            on_environment: None,
            on_branch: None,
            on_add_machine: None,
            texts: BranchToolbarTexts::default(),
        }
    }

    /// The machines in the environment menu.
    pub fn environments<S: Into<SharedString>>(
        mut self,
        names: impl IntoIterator<Item = S>,
    ) -> Self {
        self.environments = names.into_iter().map(Into::into).collect();
        self
    }

    /// The chosen environment (default "This computer").
    pub fn environment(mut self, name: impl Into<SharedString>) -> Self {
        self.environment = Some(name.into());
        self
    }

    /// Controlled mode.
    pub fn mode(mut self, mode: WorkspaceMode) -> Self {
        self.mode = Some(mode);
        self
    }

    /// Initial mode when uncontrolled (default worktree).
    pub fn default_mode(mut self, mode: WorkspaceMode) -> Self {
        self.default_mode = mode;
        self
    }

    /// The base branch (default "main").
    pub fn branch(mut self, branch: impl Into<SharedString>) -> Self {
        self.branch = Some(branch.into());
        self
    }

    /// The branches in the branch menu.
    pub fn branches<S: Into<SharedString>>(mut self, names: impl IntoIterator<Item = S>) -> Self {
        self.branches = names.into_iter().map(Into::into).collect();
        self
    }

    /// The mode changed (`onModeChange`).
    pub fn on_mode_change(
        mut self,
        f: impl Fn(WorkspaceMode, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_mode_change = Some(Rc::new(f));
        self
    }

    /// An environment was chosen.
    pub fn on_environment(
        mut self,
        f: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_environment = Some(Rc::new(f));
        self
    }

    /// A branch was chosen.
    pub fn on_branch(mut self, f: impl Fn(SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_branch = Some(Rc::new(f));
        self
    }

    /// "Add to another machine" was chosen.
    pub fn on_add_machine(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_add_machine = Some(Rc::new(f));
        self
    }

    /// Replace the default texts.
    pub fn texts(mut self, texts: BranchToolbarTexts) -> Self {
        self.texts = texts;
        self
    }
}

/// `.t3-sep-v`: a 1px `border-subtle-01` rule, 4px short of each edge.
fn separator() -> AnyElement {
    div()
        .flex_none()
        .w(px(1.))
        .self_stretch()
        .my(rem(4.))
        .bg(color(ColorRole::BorderSubtle01))
        .into_any_element()
}

fn picks(names: &[SharedString], icon: CarbonIcon, handler: &Option<NameHandler>) -> Vec<MenuItem> {
    names
        .iter()
        .map(|n| {
            let item = MenuItem::item(n.clone()).icon(icon);
            match handler.clone() {
                Some(f) => {
                    let n = n.clone();
                    item.on_select(move |w, cx| f(n.clone(), w, cx))
                }
                None => item,
            }
        })
        .collect()
}

impl RenderOnce for BranchToolbar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let (mode, store) = crate::components::form::controlled(
            (id.clone(), "mode"),
            self.mode,
            self.default_mode,
            window,
            cx,
        );
        let mut env_items = picks(&self.environments, CarbonIcon::Laptop, &self.on_environment);
        env_items.push(MenuItem::divider());
        let mut add = MenuItem::item(self.texts.add_machine.clone()).icon(CarbonIcon::Add);
        if let Some(f) = self.on_add_machine.clone() {
            add = add.on_select(move |w, cx| f(w, cx));
        }
        env_items.push(add);
        let segs = [
            (
                WorkspaceMode::Local,
                self.texts.local.clone(),
                CarbonIcon::Folder,
            ),
            (
                WorkspaceMode::Worktree,
                self.texts.worktree.clone(),
                CarbonIcon::Branch,
            ),
        ]
        .into_iter()
        .enumerate()
        .map(|(i, (m, label, icon))| {
            let on = m == mode;
            let sid = sub(&id, "seg", i);
            let focus = focus_handle(&sid, window, cx);
            let ring = focus_visible(&focus, window);
            let store = store.clone();
            let on_change = self.on_mode_change.clone();
            let ink = color(if on {
                ColorRole::TextPrimary
            } else {
                ColorRole::TextSecondary
            });
            // `.t3-seg-btn { height: 24px; padding: 0 8px; 12px }`; selected in the
            // branch bar (`app-r3`): `layer-02`, inset 1px `border-inverse`, 600.
            sans(press(sid, &focus), 12., if on { 600 } else { 400 })
                .role(gpui_kit::Role::RadioButton)
                .accessibility_label(label.clone())
                .gap(rem(4.))
                .h(rem(24.))
                .px(rem(8.))
                .text_color(ink)
                .when(on, |b| {
                    b.bg(color(ColorRole::Layer02)).child(
                        div()
                            .absolute()
                            .top_0()
                            .left_0()
                            .right_0()
                            .bottom_0()
                            .border_1()
                            .border_color(color(ColorRole::BorderInverse)),
                    )
                })
                .on_click(move |_, window, cx| {
                    store.update(cx, |v, cx| {
                        *v = m;
                        cx.notify();
                    });
                    if let Some(f) = &on_change {
                        f(m, window, cx);
                    }
                })
                .child(Icon::carbon(icon).size_length(rem(14.)).color(ink))
                .child(label)
                .when(ring, |b| b.child(craft::focus_ring()))
        });
        sans(div(), 12., 400)
            .id(id.clone())
            .flex()
            .flex_wrap()
            .items_center()
            .gap(rem(4.))
            .px(rem(8.))
            .bg(color(ColorRole::Layer01))
            .child(
                craft::text(self.texts.run_on.clone(), 12., ColorRole::TextSecondary).pl(rem(8.)),
            )
            .child(
                DropdownMenu::new((id.clone(), "env"))
                    .label(
                        self.environment
                            .clone()
                            .unwrap_or_else(|| self.texts.this_computer.clone()),
                    )
                    .kind(DropdownMenuKind::Ghost)
                    .size(DropdownMenuSize::Sm)
                    .items(env_items),
            )
            .child(separator())
            .child(
                // `.t3-seg { padding: 2px; background: layer-accent-01 }`.
                div()
                    .flex()
                    .p(px(2.))
                    .bg(color(ColorRole::LayerAccent01))
                    .children(segs),
            )
            .child(separator())
            .child(
                DropdownMenu::new((id, "branch"))
                    .label(
                        self.branch
                            .clone()
                            .unwrap_or_else(|| self.texts.main.clone()),
                    )
                    .kind(DropdownMenuKind::Ghost)
                    .size(DropdownMenuSize::Sm)
                    .items(picks(&self.branches, CarbonIcon::Branch, &self.on_branch)),
            )
    }
}

// ── GitActionsControl ────────────────────────────────────────────────────

/// The repository state that picks the primary git action (`state`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum GitState {
    /// `changes`, the default: uncommitted files.
    #[default]
    Changes,
    /// `ahead`: local commits to push.
    Ahead,
    /// `pushed`: a branch without a PR.
    Pushed,
    /// `pr`: an open PR.
    Pr,
    /// `behind`: upstream has new commits.
    Behind,
    /// `none`: nothing to do.
    None,
    /// `init`: not a repository.
    Init,
    /// `publish`: no remote.
    Publish,
}

impl GitState {
    /// Every state.
    pub const ALL: [GitState; 8] = [
        Self::Changes,
        Self::Ahead,
        Self::Pushed,
        Self::Pr,
        Self::Behind,
        Self::None,
        Self::Init,
        Self::Publish,
    ];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Changes => "changes",
            Self::Ahead => "ahead",
            Self::Pushed => "pushed",
            Self::Pr => "pr",
            Self::Behind => "behind",
            Self::None => "none",
            Self::Init => "init",
            Self::Publish => "publish",
        }
    }

    /// `Launch` for a PR, `PullRequest` when pushed, `Commit` otherwise.
    pub const fn icon(self) -> CarbonIcon {
        match self {
            Self::Pr => CarbonIcon::Launch,
            Self::Pushed => CarbonIcon::PullRequest,
            _ => CarbonIcon::Commit,
        }
    }
}

fn plural(n: usize, one: &str) -> String {
    format!("{n} {one}{}", if n == 1 { "" } else { "s" })
}

/// The primary label (`GitActionsControl`): "Commit N files" (never a
/// one-click push), "Push N commits", "Create PR", "View PR", "Pull",
/// "No changes", "Initialize Git", "Publish repository...".
pub fn git_label(state: GitState, files: usize, ahead: usize) -> String {
    match state {
        GitState::Changes if files > 0 => format!("Commit {}", plural(files, "file")),
        GitState::Changes => "Commit\u{2026}".into(),
        GitState::Ahead => format!("Push {}", plural(ahead, "commit")),
        GitState::Pushed => "Create PR".into(),
        GitState::Pr => "View PR".into(),
        GitState::Behind => "Pull".into(),
        GitState::None => "No changes".into(),
        GitState::Init => "Initialize Git".into(),
        GitState::Publish => "Publish repository...".into(),
    }
}

/// A git menu action (`onAction(key)`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GitAction {
    /// `commit`.
    Commit,
    /// `commit-branch`.
    CommitBranch,
    /// `push`.
    Push,
    /// `pr`.
    Pr,
    /// `discard`.
    Discard,
}

/// Which menu actions are enabled for `state` with `files` changed:
/// (commit, commit-branch, push, pr, discard).
pub fn git_menu_enabled(state: GitState, files: usize) -> [bool; 5] {
    let f = files > 0;
    [
        f,
        f,
        state == GitState::Ahead,
        matches!(state, GitState::Pushed | GitState::Ahead),
        f,
    ]
}

type ActionHandler = Rc<dyn Fn(GitAction, &mut Window, &mut App)>;

/// The git split button (`.t3-gitctl`): a secondary sm button whose label
/// follows the repository state (ghost and disabled when there is nothing
/// to do), a 1px-separated [`IconMenu`] of the other actions, and a warning
/// note (or "Behind upstream…"). `progress` swaps in a spinner and text.
#[derive(IntoElement)]
pub struct GitActionsControl {
    id: ElementId,
    state: GitState,
    ahead: usize,
    files: usize,
    progress: Option<SharedString>,
    disabled: bool,
    approval_pending: bool,
    note: Option<SharedString>,
    label: Option<SharedString>,
    on_primary: Option<Handler>,
    on_action: Option<ActionHandler>,
    menu_label: SharedString,
    menu_items: [SharedString; 5],
    behind_note: SharedString,
}

impl GitActionsControl {
    /// A control for repository `state`.
    pub fn new(id: impl Into<ElementId>, state: GitState) -> Self {
        Self {
            id: id.into(),
            state,
            ahead: 0,
            files: 0,
            progress: None,
            disabled: false,
            approval_pending: false,
            note: None,
            label: None,
            on_primary: None,
            on_action: None,
            menu_label: "Git action options".into(),
            menu_items: [
                "Commit".into(),
                "Commit on new branch".into(),
                "Push".into(),
                "Create pull request".into(),
                "Discard all changes".into(),
            ],
            behind_note: "Behind upstream. Pull/rebase first.".into(),
        }
    }

    /// Commits ahead of upstream.
    pub fn ahead(mut self, ahead: usize) -> Self {
        self.ahead = ahead;
        self
    }

    /// Changed files.
    pub fn files(mut self, files: usize) -> Self {
        self.files = files;
        self
    }

    /// An action in progress (replaces the buttons).
    pub fn progress(mut self, progress: impl Into<SharedString>) -> Self {
        self.progress = Some(progress.into());
        self
    }

    /// Disable both halves.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// An approval is waiting elsewhere on the page (`body:has(.t3-approval--pending)`):
    /// the control goes quiet so Approve is the only heavy button. See
    /// [`GitQuiet`].
    pub fn approval_pending(mut self, pending: bool) -> Self {
        self.approval_pending = pending;
        self
    }

    /// A warning note after the buttons.
    pub fn note(mut self, note: impl Into<SharedString>) -> Self {
        self.note = Some(note.into());
        self
    }

    /// Replace the state's label (`label`).
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// The primary button was pressed (`onPrimary`).
    pub fn on_primary(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_primary = Some(Rc::new(f));
        self
    }

    /// A menu action was chosen (`onAction`).
    pub fn on_action(mut self, f: impl Fn(GitAction, &mut Window, &mut App) + 'static) -> Self {
        self.on_action = Some(Rc::new(f));
        self
    }

    /// The menu's name and item texts, and the behind-upstream note.
    pub fn texts(
        mut self,
        menu_label: impl Into<SharedString>,
        items: [SharedString; 5],
        behind_note: impl Into<SharedString>,
    ) -> Self {
        self.menu_label = menu_label.into();
        self.menu_items = items;
        self.behind_note = behind_note.into();
        self
    }
}

/// The quiet colors of a [`GitActionsControl`] while an approval is pending
/// (`t3.css` "r2-desk: approval hierarchy"):
/// `body:has(.t3-approval--pending) .t3-gitctl .cds--btn.cds--btn--secondary
/// { background: transparent; color: var(--text-secondary); box-shadow:
/// none; border-color: transparent }`, `… .cds--btn--secondary svg { fill:
/// var(--icon-secondary) }` and `… .t3-gitctl-n { background:
/// var(--layer-accent-01); color: var(--text-secondary) }`. The rule
/// outranks the secondary hover and active fills, so the buttons stay
/// transparent while hovered or pressed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GitQuiet {
    /// The secondary buttons' label ink.
    pub text: Hsla,
    /// Their glyph ink.
    pub icon: Hsla,
    /// The count chip's (background, text).
    pub chip: (Hsla, Hsla),
}

impl GitQuiet {
    /// The quiet colors on the active theme.
    pub fn current() -> Self {
        Self {
            text: color(ColorRole::TextSecondary),
            icon: color(ColorRole::IconSecondary),
            chip: (
                color(ColorRole::LayerAccent01),
                color(ColorRole::TextSecondary),
            ),
        }
    }
}

/// The count chip's (background, text): `.t3-gitctl-n { background:
/// var(--layer-02); color: var(--text-primary) }`, or the quiet colors
/// while an approval is pending (whatever the button kind).
pub fn git_chip_colors(approval_pending: bool) -> (Hsla, Hsla) {
    if approval_pending {
        GitQuiet::current().chip
    } else {
        (color(ColorRole::Layer02), color(ColorRole::TextPrimary))
    }
}

/// `.t3-gitctl-warn`: a 14px warning glyph and 12px `text-secondary` text,
/// 8px after the buttons.
fn warn(text: SharedString) -> AnyElement {
    sans(div(), 12., 400)
        .flex()
        .flex_none()
        .items_center()
        .gap(rem(4.))
        .ml(rem(8.))
        .text_color(color(ColorRole::TextSecondary))
        .child(warning_glyph(14.))
        .child(text)
        .into_any_element()
}

#[derive(Default)]
struct Pointer {
    hovered: bool,
    pressed: bool,
}

impl RenderOnce for GitActionsControl {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        if let Some(progress) = self.progress.clone() {
            // `.t3-gitctl--busy { gap: 8px; height: 32px; padding: 0 12px; layer-01; 12px }`.
            return sans(div(), 12., 400)
                .flex()
                .flex_none()
                .items_center()
                .gap(rem(8.))
                .h(rem(32.))
                .px(rem(12.))
                .bg(color(ColorRole::Layer01))
                .text_color(color(ColorRole::TextPrimary))
                .child(Spin::new((id, "spin")))
                .child(progress)
                .into_any_element();
        }
        let off = self.state == GitState::None || self.disabled;
        let count_in_label =
            self.state == GitState::Changes && self.label.is_none() && self.files > 0;
        let label: SharedString = self
            .label
            .clone()
            .unwrap_or_else(|| git_label(self.state, self.files, self.ahead).into());
        let kind = if off {
            ButtonKind::Ghost
        } else {
            ButtonKind::Secondary
        };
        // The primary half is a Carbon `Button` (secondary sm, icon at the end)
        // that also carries the count chip (`.t3-gitctl-n`), so it is drawn
        // here from the Button's own Look and Geometry.
        let main_id: ElementId = (id.clone(), "primary").into();
        let focus = focus_handle(&main_id, window, cx);
        let pointer =
            window.use_keyed_state((id.clone(), "pointer"), cx, |_, _| Pointer::default());
        let state = {
            let p = pointer.read(cx);
            ButtonState {
                hovered: p.hovered && !off,
                pressed: p.pressed && !off,
                focused: focus.is_focused(window),
                disabled: off,
            }
        };
        let mut look = Look::of(kind, state, false);
        // Only a `secondary` button goes quiet (an `off` control is ghost).
        let quiet = (self.approval_pending && !off).then(GitQuiet::current);
        if let Some(q) = quiet {
            look.background = Hsla::transparent_black();
            look.text = q.text;
            look.icon = q.icon;
            // `border-color: transparent; box-shadow: none` would hide the
            // focus ring too; u0ui keeps it on every focusable control.
            if !state.focused {
                look.border = Hsla::transparent_black();
            }
        }
        let geometry = Geometry::of(ButtonSize::Sm, density::current(), kind, false);
        let bg = motion::animate_color(
            (id.clone(), "bg"),
            look.background,
            Motion::HOVER,
            window,
            cx,
        );
        let ink = motion::animate_color((id.clone(), "ink"), look.text, Motion::HOVER, window, cx);
        let on_primary = self.on_primary.clone();
        let chip = (self.files > 0 && !count_in_label).then(|| {
            // `.t3-gitctl-n { margin-left: 8px; padding: 0 4px; layer-02; 11px/16px; pill }`.
            let (chip_bg, chip_ink) = git_chip_colors(self.approval_pending);
            sans(div(), 12., 400)
                .ml(rem(8.))
                .px(rem(4.))
                .line_height(rem(16.))
                .rounded(rem(16.))
                .bg(chip_bg)
                .text_color(chip_ink)
                .child(self.files.to_string())
        });
        let icon = Icon::carbon(self.state.icon())
            .size_length(rem(16.))
            .color(look.icon);
        let mut primary = gpui_kit::base::Button::new(main_id)
            .track_focus(&focus)
            .disabled(off)
            .accessibility_label(label.clone())
            .relative()
            .flex()
            .flex_none()
            .items_start()
            .justify_start()
            .whitespace_nowrap()
            .type_style_body()
            .h(rem(geometry.height))
            .pt(rem(geometry.padding_top))
            .pl(rem(geometry.padding_left))
            .pr(rem(geometry.padding_right))
            .bg(bg)
            .border_1()
            .border_color(look.border)
            .text_color(ink)
            .cursor(if off {
                gpui_kit::CursorStyle::OperationNotAllowed
            } else {
                gpui_kit::CursorStyle::PointingHand
            })
            .on_hover({
                let pointer = pointer.clone();
                move |h, _, cx| {
                    pointer.update(cx, |p, cx| {
                        p.hovered = *h;
                        if !h {
                            p.pressed = false;
                        }
                        cx.notify();
                    })
                }
            })
            .on_mouse_down(MouseButton::Left, {
                let pointer = pointer.clone();
                move |_, _, cx| {
                    pointer.update(cx, |p, cx| {
                        p.pressed = true;
                        cx.notify();
                    })
                }
            })
            .on_mouse_up(MouseButton::Left, {
                let pointer = pointer.clone();
                move |_, _, cx| {
                    pointer.update(cx, |p, cx| {
                        p.pressed = false;
                        cx.notify();
                    })
                }
            })
            .when_some(on_primary, |b, f| b.on_click(move |_, w, cx| f(w, cx)))
            .child(label)
            .children(chip);
        primary = match geometry.icon_top {
            Some(top) => primary.child(
                div()
                    .absolute()
                    .top(rem(top))
                    .right(rem(geometry.icon_right))
                    .size(rem(16.))
                    .child(icon),
            ),
            None => primary.child(
                div()
                    .self_center()
                    .ml(rem(8.))
                    .mt(rem(-geometry.padding_top))
                    .size(rem(16.))
                    .child(icon),
            ),
        };
        let primary = primary.children(
            [
                look.focus_ring.map(|c| (0., c)),
                look.focus_inset.map(|c| (1., c)),
            ]
            .into_iter()
            .flatten()
            .map(|(inset, c)| {
                div()
                    .absolute()
                    .top(px(inset))
                    .left(px(inset))
                    .right(px(inset))
                    .bottom(px(inset))
                    .border_1()
                    .border_color(c)
            }),
        );
        let enabled = git_menu_enabled(self.state, self.files);
        let keys = [
            GitAction::Commit,
            GitAction::CommitBranch,
            GitAction::Push,
            GitAction::Pr,
            GitAction::Discard,
        ];
        let mut items: Vec<MenuItem> = Vec::new();
        for i in 0..5 {
            if i == 4 {
                items.push(MenuItem::divider());
            }
            let mut item = MenuItem::item(self.menu_items[i].clone())
                .disabled(!enabled[i])
                .danger(i == 4);
            if let Some(f) = self.on_action.clone() {
                let key = keys[i];
                item = item.on_select(move |w, cx| f(key, w, cx));
            }
            items.push(item);
        }
        // The quiet secondary trigger is drawn as a ghost one with an
        // `icon-secondary` glyph, and stays transparent on hover and press
        // like the main button: the pending rule outranks
        // `.cds--btn--secondary:hover` (bundle.css). It keeps its focus ring.
        let variant = if off || quiet.is_some() {
            ButtonVariant::Ghost
        } else {
            ButtonVariant::Secondary
        };
        div()
            .flex()
            .flex_none()
            .items_center()
            .gap(px(1.))
            .child(primary)
            .child(
                IconMenu::new((id, "menu"), self.menu_label.clone())
                    .variant(variant)
                    .size(ButtonSize::Sm)
                    .disabled(off)
                    .when_some(quiet, |m, q| {
                        m.icon_color(q.icon).fill(Hsla::transparent_black())
                    })
                    .items(items),
            )
            .when_some(self.note.clone(), |d, n| d.child(warn(n)))
            .when(self.state == GitState::Behind, |d| {
                d.child(warn(self.behind_note.clone()))
            })
            .into_any_element()
    }
}

trait BodyType: Styled + Sized {
    fn type_style_body(self) -> Self {
        use crate::typography::TypeStyled as _;
        self.type_style(crate::tokens::TypeStyle::BodyCompact01)
    }
}

impl<T: Styled> BodyType for T {}

// ── ThreadDetailsPanel ───────────────────────────────────────────────────

/// A collapsible thread-details section (`.t3-dsec`): a 40px head with a
/// chevron, the semibold title and a count chip, an optional action at its
/// end, and the body (12px inset, 8px gaps).
#[derive(IntoElement)]
pub struct ThreadDetailsSection {
    id: ElementId,
    title: SharedString,
    count: Option<usize>,
    action: Option<AnyElement>,
    default_open: bool,
    children: Vec<AnyElement>,
}

impl ThreadDetailsSection {
    /// A section titled `title`.
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            count: None,
            action: None,
            default_open: true,
            children: Vec::new(),
        }
    }

    /// The count chip.
    pub fn count(mut self, count: usize) -> Self {
        self.count = Some(count);
        self
    }

    /// An action at the head's end.
    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.action = Some(action.into_any_element());
        self
    }

    /// Open at first (`defaultOpen`, default true).
    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }
}

impl ParentElement for ThreadDetailsSection {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for ThreadDetailsSection {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let open_state = window.use_keyed_state((id.clone(), "open"), cx, {
            let d = self.default_open;
            move |_, _| d
        });
        let open = *open_state.read(cx);
        let head_id: ElementId = (id.clone(), "head").into();
        let focus = focus_handle(&head_id, window, cx);
        let ring = focus_visible(&focus, window);
        let head = sans(press(head_id, &focus), 14., 400)
            .accessibility_label(self.title.clone())
            .flex_1()
            .w_full()
            .gap(rem(8.))
            .h(rem(40.))
            .pl(rem(12.))
            .pr(rem(8.))
            .text_color(color(ColorRole::TextPrimary))
            .on_click(move |_, _, cx| {
                open_state.update(cx, |o, cx| {
                    *o = !*o;
                    cx.notify();
                })
            })
            .child(
                Icon::carbon(CarbonIcon::ChevronDown)
                    .size_length(rem(14.))
                    .color(color(ColorRole::IconSecondary))
                    .rotate(gpui_kit::radians(if open {
                        0.
                    } else {
                        -std::f32::consts::FRAC_PI_2
                    })),
            )
            // `.t3-dsec-head b`: no weight rule, so Carbon's `b { font: inherit }`
            // keeps it 400.
            .child(sans(div(), 14., 400).child(self.title.clone()))
            .when_some(self.count, |b, n| b.child(Count::new(n)))
            .when(ring, |b| b.child(craft::focus_ring()));
        div()
            .id(id)
            .flex()
            .flex_col()
            .border_b_1()
            .border_color(color(ColorRole::BorderSubtle01))
            .child(
                div()
                    .flex()
                    .items_center()
                    .pr(rem(8.))
                    .child(head)
                    .children(self.action),
            )
            .when(open, |d| {
                d.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(rem(8.))
                        .px(rem(12.))
                        .pb(rem(12.))
                        .children(self.children),
                )
            })
    }
}

/// The context meter's tone: over 90% error, over 70% warning, else ok.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MeterTone {
    /// Up to 70%: a neutral `icon-secondary` fill (`gemini-A`).
    Ok,
    /// Over 70%: `support-warning`.
    Warning,
    /// Over 90%: `support-error`.
    Error,
}

impl MeterTone {
    /// The tone of `pct` percent.
    pub fn of(pct: u32) -> Self {
        if pct > 90 {
            Self::Error
        } else if pct > 70 {
            Self::Warning
        } else {
            Self::Ok
        }
    }

    /// The fill color.
    pub fn fill(self) -> Hsla {
        color(match self {
            Self::Ok => ColorRole::IconSecondary,
            Self::Warning => ColorRole::SupportWarning,
            Self::Error => ColorRole::SupportError,
        })
    }
}

/// `Math.round(used / total * 100)`.
pub fn meter_percent(used: u64, total: u64) -> u32 {
    if total == 0 {
        return 0;
    }
    ((used as f64 / total as f64) * 100.).round() as u32
}

/// Context window usage (`.t3-meter`): label and percent, a 4px track, and
/// "142k of 200k tokens".
#[derive(IntoElement, Clone, Debug)]
pub struct ContextMeter {
    used: u64,
    total: u64,
    label: SharedString,
    tokens_label: SharedString,
}

impl ContextMeter {
    /// `used` of `total` tokens (default total 200 000).
    pub fn new(used: u64, total: u64) -> Self {
        Self {
            used,
            total,
            label: "Context window".into(),
            tokens_label: "tokens".into(),
        }
    }

    /// The label (`Context window`).
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    /// The unit word (`tokens`).
    pub fn tokens_label(mut self, label: impl Into<SharedString>) -> Self {
        self.tokens_label = label.into();
        self
    }
}

impl RenderOnce for ContextMeter {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let pct = meter_percent(self.used, self.total);
        let tone = MeterTone::of(pct);
        let k = |n: u64| (n as f64 / 1000.).round() as u64;
        sans(div(), 12., 400)
            .flex()
            .flex_col()
            .gap(rem(4.))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .text_color(color(ColorRole::TextSecondary))
                    .child(self.label.clone())
                    .child(
                        // `.t3-meter-top b { color: text-primary }`, no weight:
                        // Carbon's `b { font: inherit }` keeps it 400.
                        sans(div(), 12., 400)
                            .text_color(color(ColorRole::TextPrimary))
                            .child(format!("{pct}%")),
                    ),
            )
            .child(
                div().h(px(4.)).bg(color(ColorRole::LayerAccent01)).child(
                    div()
                        .h_full()
                        .w(relative((pct.min(100) as f32) / 100.))
                        .bg(tone.fill()),
                ),
            )
            .child(craft::text(
                format!(
                    "{}k of {}k {}",
                    k(self.used),
                    k(self.total),
                    self.tokens_label
                ),
                12.,
                ColorRole::TextHelper,
            ))
    }
}

// ── ThreadHoverCard ──────────────────────────────────────────────────────

/// The preview card shown on hovering a thread reference
/// (`.t3-hovercard`, 20rem on `layer-01` with the floating shadow):
/// project and status, the title, the last message (3 lines) and
/// "provider · time".
#[derive(IntoElement, Clone, Debug)]
pub struct ThreadHoverCard {
    title: SharedString,
    project: SharedString,
    status: Status,
    provider: SharedString,
    last_message: SharedString,
    time: SharedString,
}

impl ThreadHoverCard {
    /// A card for thread `title` in `project`.
    pub fn new(title: impl Into<SharedString>, project: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            project: project.into(),
            status: Status::Idle,
            provider: SharedString::default(),
            last_message: SharedString::default(),
            time: SharedString::default(),
        }
    }

    /// The status.
    pub fn status(mut self, status: Status) -> Self {
        self.status = status;
        self
    }

    /// The provider line (`Codex · GPT`).
    pub fn provider(mut self, provider: impl Into<SharedString>) -> Self {
        self.provider = provider.into();
        self
    }

    /// The last message.
    pub fn last_message(mut self, message: impl Into<SharedString>) -> Self {
        self.last_message = message.into();
        self
    }

    /// The relative time.
    pub fn time(mut self, time: impl Into<SharedString>) -> Self {
        self.time = time.into();
        self
    }
}

impl RenderOnce for ThreadHoverCard {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        sans(div(), 14., 400)
            .flex()
            .flex_col()
            .gap(rem(8.))
            .w(rem(320.))
            .max_w_full()
            .p(rem(16.))
            .bg(color(ColorRole::Layer01))
            .shadow(vec![
                BoxShadow::new(px(0.), px(2.), color(ColorRole::Shadow)).blur_radius(px(6.)),
            ])
            .text_color(color(ColorRole::TextPrimary))
            .child(
                sans(div(), 12., 400)
                    .flex()
                    .items_center()
                    .gap(rem(8.))
                    .text_color(color(ColorRole::TextSecondary))
                    .child(ProjectFavicon::new(self.project.clone()).size(IdentitySize::Sm))
                    .child(self.project.clone())
                    .child(div().flex_1())
                    .child(StatusPill::new(self.status)),
            )
            .child(sans(div(), 14., 600).child(self.title.clone()))
            // `.t3-hovercard-last` is a `<p>` with no size rule of its own,
            // so Carbon's bare `p` rule makes it 16px/24px (not the card's
            // 14px); `{ color: text-secondary; -webkit-line-clamp: 3 }`.
            .child(
                crate::typography::paragraph()
                    .line_clamp(3)
                    .overflow_hidden()
                    .text_color(color(ColorRole::TextSecondary))
                    .child(self.last_message.clone()),
            )
            .child(
                sans(div(), 12., 400)
                    .flex()
                    .text_color(color(ColorRole::TextHelper))
                    .child(self.provider.clone())
                    // `.t3-metadot { color: text-helper; padding: 0 2px }`.
                    .child(div().px(px(2.)).child("\u{b7}"))
                    .child(self.time.clone()),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_header_says_its_status_in_the_words_given() {
        let h = ChatHeader::new("h", "t").status(Status::NeedsYou);
        assert_eq!(h.status_label, None);
        let h = h.status_label("需要你");
        assert_eq!(h.status_label.as_deref(), Some("需要你"));
    }

    #[test]
    fn git_labels_follow_the_reference() {
        assert_eq!(git_label(GitState::Changes, 6, 0), "Commit 6 files");
        assert_eq!(git_label(GitState::Changes, 1, 0), "Commit 1 file");
        assert_eq!(git_label(GitState::Changes, 0, 0), "Commit\u{2026}");
        assert_eq!(git_label(GitState::Ahead, 0, 2), "Push 2 commits");
        assert_eq!(git_label(GitState::Publish, 0, 0), "Publish repository...");
        assert_eq!(GitState::Pr.icon(), CarbonIcon::Launch);
        assert_eq!(
            git_menu_enabled(GitState::Ahead, 0),
            [false, false, true, true, false]
        );
        assert_eq!(
            git_menu_enabled(GitState::Changes, 6),
            [true, true, false, false, true]
        );
    }

    #[test]
    fn approval_pending_quiets_the_control() {
        // `t3.css` 1096–1098: text-secondary ink, icon-secondary glyphs and a
        // layer-accent-01 / text-secondary count chip.
        let q = GitQuiet::current();
        assert_eq!(q.text, color(ColorRole::TextSecondary));
        assert_eq!(q.icon, color(ColorRole::IconSecondary));
        assert_eq!(
            git_chip_colors(true),
            (
                color(ColorRole::LayerAccent01),
                color(ColorRole::TextSecondary)
            )
        );
        // `.t3-gitctl-n { background: var(--layer-02); color: var(--text-primary) }`.
        assert_eq!(
            git_chip_colors(false),
            (color(ColorRole::Layer02), color(ColorRole::TextPrimary))
        );
        assert!(!GitActionsControl::new("g", GitState::Changes).approval_pending);
        assert!(
            GitActionsControl::new("g", GitState::Changes)
                .approval_pending(true)
                .approval_pending
        );
    }

    #[test]
    fn meter_tones() {
        assert_eq!(meter_percent(142_000, 200_000), 71);
        assert_eq!(MeterTone::of(71), MeterTone::Warning);
        assert_eq!(MeterTone::of(70), MeterTone::Ok);
        assert_eq!(MeterTone::of(91), MeterTone::Error);
        assert_eq!(meter_percent(1, 0), 0);
    }
}
