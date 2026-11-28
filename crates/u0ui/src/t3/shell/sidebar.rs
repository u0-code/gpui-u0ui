//! The T3 sidebar (`shell.jsx` "Sidebar"): [`SidebarBrandHeader`],
//! [`ProjectFolderHeader`], [`SidebarShelfHeader`], [`ThreadRow`] with its
//! [`PullRequestMiniBadge`], and the [`UpdatePill`] at the sidebar foot.
use super::super::base::{Spin, Status, fmt_elapsed, ink, stroke_arc, warning_glyph};
use super::super::craft::{mono, press, sans};
use super::super::press::{Handler, focus, hover, icon, inset_frame, rem, ring, track_hover};
use super::{EnvironmentBadge, IdentitySize, ProjectFavicon};
use crate::t3::review::PullRequestState;
use crate::{
    components::{
        button::{Button, ButtonSize, ButtonVariant},
        kbd::Kbd,
    },
    icons::{CarbonIcon, Icon},
    motion::{self, Motion},
    theme::color,
    tokens::{ColorRole, SIZE_LG, SIZE_MD, SIZE_SM},
};
use gpui_kit::{
    AnyElement, App, ElementId, FontWeight, Hsla, ImageSource, InteractiveElement as _,
    IntoElement, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled,
    Window, canvas, div, prelude::FluentBuilder as _, px, radians,
};
use std::f32::consts::{FRAC_PI_2, TAU};
use std::rc::Rc;

fn handler(f: impl Fn(&mut Window, &mut App) + 'static) -> Option<Handler> {
    Some(Rc::new(f))
}

/// A ghost `sm` icon button (`Cb.IconButton kind="ghost" size="sm"`).
fn icon_button(
    id: impl Into<ElementId>,
    label: SharedString,
    glyph: CarbonIcon,
    disabled: bool,
    on_click: Option<Handler>,
) -> Button {
    Button::new(id, label)
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::Sm)
        .icon(glyph)
        .icon_only(true)
        .disabled(disabled)
        .when_some(on_click, |b, f| b.on_click(move |_, w, cx| f(w, cx)))
}

/// The chevron turn of a disclosure: 0 open, −90° closed (`.t3-folder--collapsed
/// .t3-folder-chev { transform: rotate(-90deg) }`, `transition: transform 110ms`).
fn chevron_turn(id: ElementId, expanded: bool, window: &mut Window, cx: &mut App) -> f32 {
    let target = if expanded { 0. } else { -FRAC_PI_2 };
    motion::animate((id, "chev"), target, Motion::TOGGLE, window, cx)
}

// ── SidebarBrandHeader ──────────────────────────────────────────────────

/// The sidebar's top bar (`SidebarBrandHeader`, `.t3-sbhead`): a 48px row
/// (16px start, 8px end padding, `border-subtle-01` bottom rule) with the
/// brand (a 20px `background-inverse` mark holding its first letter, then
/// the 14px semibold name) and four ghost `sm` icon buttons: Back, Forward,
/// Search, New thread.
#[derive(IntoElement)]
pub struct SidebarBrandHeader {
    id: ElementId,
    brand: SharedString,
    can_back: bool,
    can_forward: bool,
    labels: [SharedString; 4],
    on_back: Option<Handler>,
    on_forward: Option<Handler>,
    on_search: Option<Handler>,
    on_new_thread: Option<Handler>,
}

impl SidebarBrandHeader {
    /// The header for `brand` (`brand`, `"Your app"` in the JSX).
    pub fn new(id: impl Into<ElementId>, brand: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            brand: brand.into(),
            can_back: true,
            can_forward: false,
            labels: [
                "Back".into(),
                "Forward".into(),
                "Search (⌘K)".into(),
                "New thread (⌘N)".into(),
            ],
            on_back: None,
            on_forward: None,
            on_search: None,
            on_new_thread: None,
        }
    }

    /// Enable Back (`canBack`, default true).
    pub fn can_back(mut self, can: bool) -> Self {
        self.can_back = can;
        self
    }

    /// Enable Forward (`canForward`, default false).
    pub fn can_forward(mut self, can: bool) -> Self {
        self.can_forward = can;
        self
    }

    /// The four button labels (tooltips): Back, Forward, Search, New thread.
    pub fn labels(mut self, labels: [SharedString; 4]) -> Self {
        self.labels = labels;
        self
    }

    /// `onBack`.
    pub fn on_back(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_back = handler(f);
        self
    }

    /// `onForward`.
    pub fn on_forward(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_forward = handler(f);
        self
    }

    /// `onSearch`.
    pub fn on_search(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_search = handler(f);
        self
    }

    /// `onNewThread`.
    pub fn on_new_thread(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_new_thread = handler(f);
        self
    }
}

impl RenderOnce for SidebarBrandHeader {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let id = self.id;
        let [back, forward, search, new] = self.labels;
        let initial: String = self.brand.chars().take(1).collect();
        // `.t3-sbhead-mark { width: 1.25rem; height: 1.25rem; background:
        // var(--background-inverse); color: var(--text-inverse); font-size: 0.6875rem }`.
        let mark = sans(div(), 12., 600)
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .size(rem(20.))
            .bg(color(ColorRole::BackgroundInverse))
            .text_color(color(ColorRole::TextInverse))
            .child(initial);
        // `.t3-sbhead-brand { flex: 1; gap: var(--spacing-03); font-weight: 600; font-size: 0.875rem }`.
        // `flex: 1` keeps `min-width: auto`, so the brand never shrinks
        // below its name: in a narrow sidebar the name stays whole and the
        // actions run on into the 8px end padding (the reference's 255px
        // sidebar does this with "Workbench").
        let brand = sans(div(), 14., 600)
            .flex()
            .flex_1()
            .items_center()
            .gap(rem(8.))
            .text_color(color(ColorRole::TextPrimary))
            .child(mark)
            .child(div().flex_none().whitespace_nowrap().child(self.brand));
        div()
            .flex()
            .items_center()
            .gap(rem(8.))
            .h(SIZE_LG.rems())
            .pl(rem(16.))
            .pr(rem(8.))
            .border_b_1()
            .border_color(color(ColorRole::BorderSubtle01))
            .child(brand)
            .child(
                div()
                    .flex()
                    .flex_none()
                    .child(icon_button(
                        (id.clone(), "back"),
                        back,
                        CarbonIcon::ArrowLeft,
                        !self.can_back,
                        self.on_back,
                    ))
                    .child(icon_button(
                        (id.clone(), "forward"),
                        forward,
                        CarbonIcon::ArrowRight,
                        !self.can_forward,
                        self.on_forward,
                    ))
                    .child(icon_button(
                        (id.clone(), "search"),
                        search,
                        CarbonIcon::Search,
                        false,
                        self.on_search,
                    ))
                    .child(icon_button(
                        (id.clone(), "new"),
                        new,
                        CarbonIcon::Edit,
                        false,
                        self.on_new_thread,
                    )),
            )
    }
}

// ── ProjectFolderHeader ─────────────────────────────────────────────────

/// Per-status thread counts of a project (`counts`); zero counts are hidden.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FolderCounts {
    /// `needs-you`.
    pub needs_you: usize,
    /// `failed`.
    pub failed: usize,
    /// `working`.
    pub working: usize,
    /// `done`.
    pub done: usize,
}

impl FolderCounts {
    /// The visible counts in the reference order (needs you, failed,
    /// working, done), skipping zeros.
    pub fn visible(self) -> Vec<(Status, usize)> {
        [
            (Status::NeedsYou, self.needs_you),
            (Status::Failed, self.failed),
            (Status::Working, self.working),
            (Status::Done, self.done),
        ]
        .into_iter()
        .filter(|(_, n)| *n > 0)
        .collect()
    }
}

/// (background, text) of a `.t3-count--<status>` pill: needs you
/// `support-warning` / `#161616`, failed `support-error` /
/// `text-on-color` (`#161616` on the dark themes, whose `support-error` is
/// a light red white text does not read on), working blue tag, done gray
/// tag.
pub fn count_colors(status: Status) -> (Hsla, Hsla) {
    match status {
        Status::NeedsYou => (color(ColorRole::SupportWarning), ink()),
        Status::Failed => (
            color(ColorRole::SupportError),
            if crate::theme::is_dark() {
                ink()
            } else {
                color(ColorRole::TextOnColor)
            },
        ),
        Status::Working => (
            color(ColorRole::TagBackgroundBlue),
            color(ColorRole::TagColorBlue),
        ),
        _ => (
            color(ColorRole::TagBackgroundGray),
            color(ColorRole::TagColorGray),
        ),
    }
}

// ── Count ───────────────────────────────────────────────────────────────

/// A number on a pill (`.t3-count`, `.t3-shelf-count`): `label-01` (12px:
/// nothing in Carbon's type scale is smaller), 18px tall and at least as
/// wide, 4px side padding, a 16px radius, as Carbon's small tag. Quiet by
/// default (`layer-accent-01` / `text-secondary`, a shelf's), or in a
/// status's colors ([`count_colors`]).
#[derive(IntoElement)]
pub struct Count {
    n: usize,
    status: Option<Status>,
}

impl Count {
    /// A quiet count of `n`.
    pub fn new(n: usize) -> Self {
        Self { n, status: None }
    }

    /// In the colors of `status` (`.t3-count--<status>`).
    pub fn status(mut self, status: Status) -> Self {
        self.status = Some(status);
        self
    }

    /// (background, text) the pill takes.
    fn colors(&self) -> (Hsla, Hsla) {
        self.status.map_or_else(
            || {
                (
                    color(ColorRole::LayerAccent01),
                    color(ColorRole::TextSecondary),
                )
            },
            count_colors,
        )
    }
}

impl RenderOnce for Count {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let (bg, fg) = self.colors();
        sans(div(), 12., 400)
            .flex_none()
            .flex()
            .justify_center()
            .h(rem(18.))
            .line_height(rem(18.))
            .min_w(rem(18.))
            .px(rem(4.))
            .rounded(rem(16.))
            .bg(bg)
            .text_color(fg)
            .child(self.n.to_string())
    }
}

/// A project group header in the sidebar (`ProjectFolderHeader`,
/// `.t3-folder`): a 32px row that takes `layer-hover-01` on hover. Its
/// main button (chevron, favicon, semibold name, optional environment
/// badge) toggles the group; then the status counts, a pin and a ghost
/// "New thread in …" icon button.
#[derive(IntoElement)]
pub struct ProjectFolderHeader {
    id: ElementId,
    name: SharedString,
    favicon: Option<ImageSource>,
    counts: FolderCounts,
    pinned: bool,
    expanded: bool,
    environment: Option<EnvironmentBadge>,
    new_thread_label: Option<SharedString>,
    on_toggle: Option<Handler>,
    on_new_thread: Option<Handler>,
}

impl ProjectFolderHeader {
    /// The header of project `name`.
    pub fn new(id: impl Into<ElementId>, name: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            favicon: None,
            counts: FolderCounts::default(),
            pinned: false,
            expanded: true,
            environment: None,
            new_thread_label: None,
            on_toggle: None,
            on_new_thread: None,
        }
    }

    /// The favicon image (`favicon`); the monogram otherwise.
    pub fn favicon(mut self, src: impl Into<ImageSource>) -> Self {
        self.favicon = Some(src.into());
        self
    }

    /// The thread counts (`counts`).
    pub fn counts(mut self, counts: FolderCounts) -> Self {
        self.counts = counts;
        self
    }

    /// Show the pin (`pinned`).
    pub fn pinned(mut self, pinned: bool) -> Self {
        self.pinned = pinned;
        self
    }

    /// Whether the group is open (`expanded`, default true).
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    /// The remote environment badge (`environment`), drawn `sm`.
    pub fn environment(mut self, badge: EnvironmentBadge) -> Self {
        self.environment = Some(badge);
        self
    }

    /// The add button's label (default `"New thread in <name>"`).
    pub fn new_thread_label(mut self, label: impl Into<SharedString>) -> Self {
        self.new_thread_label = Some(label.into());
        self
    }

    /// `onToggle`.
    pub fn on_toggle(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = handler(f);
        self
    }

    /// `onNewThread`.
    pub fn on_new_thread(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_new_thread = handler(f);
        self
    }
}

impl RenderOnce for ProjectFolderHeader {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id;
        let (hovered, hover_store) = hover((id.clone(), "hover"), window, cx);
        let (focus_handle, focused) = focus((id.clone(), "focus"), window, cx);
        let turn = chevron_turn(id.clone(), self.expanded, window, cx);
        let mut favicon = ProjectFavicon::new(self.name.clone()).size(IdentitySize::Sm);
        if let Some(src) = self.favicon {
            favicon = favicon.src(src);
        }
        let on_toggle = self.on_toggle;
        // `.t3-folder-main { flex: 1; gap: var(--spacing-03); padding: 0
        // var(--spacing-03); color: var(--text-primary); font-size: 0.875rem;
        // font-weight: 600 }`.
        let main = sans(press((id.clone(), "main"), &focus_handle), 14., 600)
            .flex()
            .flex_1()
            .min_w_0()
            .h_full()
            .items_center()
            .gap(rem(8.))
            .px(rem(8.))
            .text_color(color(ColorRole::TextPrimary))
            .when_some(on_toggle, |b, f| b.on_click(move |_, w, cx| f(w, cx)))
            .child(
                icon(
                    CarbonIcon::ChevronDown,
                    16.,
                    color(ColorRole::IconSecondary),
                )
                .rotate(radians(turn))
                .flex_none(),
            )
            .child(favicon)
            // `.t3-folder-name { flex: 0 1 auto; min-width: 2.5rem }`, ellipsis.
            .child(
                div()
                    .flex_shrink(1.)
                    .min_w(rem(40.))
                    .truncate()
                    .child(self.name.clone()),
            )
            .children(self.environment.map(|e| {
                div()
                    .flex_shrink(1.)
                    .min_w_0()
                    .overflow_hidden()
                    .child(e.size(super::BadgeSize::Sm))
            }))
            .when(focused, |b| b.child(ring()));
        // `.t3-folder-counts { display: flex; gap: 2px }`.
        let counts = div().flex().flex_none().gap(rem(2.)).children(
            (self.counts.visible().into_iter()).map(|(status, n)| Count::new(n).status(status)),
        );
        let new_label = self
            .new_thread_label
            .unwrap_or_else(|| format!("New thread in {}", self.name).into());
        div()
            .id((id.clone(), "row"))
            .flex()
            .items_center()
            .gap(rem(4.))
            .h(SIZE_SM.rems())
            .pr(rem(4.))
            .when(hovered, |d| d.bg(color(ColorRole::LayerHover01)))
            .on_hover(track_hover(hover_store))
            .child(main)
            .child(counts)
            .when(self.pinned, |d| {
                d.child(icon(
                    CarbonIcon::PinFilled,
                    14.,
                    color(ColorRole::IconSecondary),
                ))
            })
            .child(icon_button(
                (id.clone(), "add"),
                new_label,
                CarbonIcon::Add,
                false,
                self.on_new_thread,
            ))
    }
}

// ── SidebarShelfHeader ──────────────────────────────────────────────────

/// A collapsible shelf title in the thread list (`SidebarShelfHeader`,
/// `.t3-shelf`): a 32px row whose button holds a 14px chevron, the
/// `label-01` `text-secondary` label and a pill count; then `action`.
#[derive(IntoElement)]
pub struct SidebarShelfHeader {
    id: ElementId,
    label: SharedString,
    count: Option<usize>,
    expanded: bool,
    on_toggle: Option<Handler>,
    action: Option<AnyElement>,
}

impl SidebarShelfHeader {
    /// A shelf titled `label`.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            count: None,
            expanded: true,
            on_toggle: None,
            action: None,
        }
    }

    /// The count pill (`count`).
    pub fn count(mut self, count: usize) -> Self {
        self.count = Some(count);
        self
    }

    /// Whether the shelf is open (`expanded`, default true).
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    /// `onToggle`.
    pub fn on_toggle(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = handler(f);
        self
    }

    /// A trailing action (`action`).
    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.action = Some(action.into_any_element());
        self
    }
}

impl RenderOnce for SidebarShelfHeader {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id;
        let (focus_handle, focused) = focus((id.clone(), "focus"), window, cx);
        let turn = chevron_turn(id.clone(), self.expanded, window, cx);
        let ink = color(ColorRole::TextSecondary);
        // `.t3-shelf-btn { gap: var(--spacing-02); padding: 0 var(--spacing-02);
        // color: var(--text-secondary); font-size: 0.75rem }`.
        let button = sans(press((id.clone(), "btn"), &focus_handle), 12., 400)
            .flex()
            .items_center()
            .gap(rem(4.))
            .px(rem(4.))
            .text_color(ink)
            .when_some(self.on_toggle, |b, f| b.on_click(move |_, w, cx| f(w, cx)))
            .child(icon(CarbonIcon::ChevronDown, 14., ink).rotate(radians(turn)))
            .child(self.label)
            .children(self.count.map(Count::new))
            .when(focused, |b| b.child(ring()));
        div()
            .flex()
            .items_center()
            .h(SIZE_SM.rems())
            .px(rem(8.))
            .child(button)
            .children(self.action)
    }
}

// ── PullRequestMiniBadge ────────────────────────────────────────────────

/// A compact PR badge (`PullRequestMiniBadge`, `.t3-prmini`): 12px
/// `PullRequest` glyph and `#<n>` on a tag-colored pill, titled
/// `PR #<n> - <State>`.
#[derive(IntoElement)]
pub struct PullRequestMiniBadge {
    number: u32,
    state: PullRequestState,
}

impl PullRequestMiniBadge {
    /// The badge for PR `number` in `state`.
    pub fn new(number: u32, state: PullRequestState) -> Self {
        Self { number, state }
    }
}

impl RenderOnce for PullRequestMiniBadge {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let (bg, fg) = self.state.tag_colors();
        let title: SharedString = format!("PR #{} - {}", self.number, self.state.label()).into();
        // `.t3-prmini { gap: 2px; padding: 0 var(--spacing-02); font-size:
        // 0.6875rem; line-height: 1rem; border-radius: var(--border-radius-16) }`.
        sans(div(), 12., 400)
            .id(ElementId::Name(format!("t3-prmini-{}", self.number).into()))
            .line_height(rem(16.))
            .flex()
            .flex_none()
            .items_center()
            .gap(px(2.))
            .px(rem(4.))
            .rounded(px(16.))
            .bg(bg)
            .text_color(fg)
            .tooltip(super::title_tooltip(title))
            .child(icon(CarbonIcon::PullRequest, 12., fg))
            // `bundle.js` prints `"#", number` (the `src/t3/shell.jsx` source
            // still shows the bare number).
            .child(format!("#{}", self.number))
    }
}

// ── ThreadRow ───────────────────────────────────────────────────────────

/// One thread in the sidebar or a list (`ThreadRow`, `.t3-thread`): a
/// status glyph, the title (with an "Unsent draft · " prefix for drafts),
/// a meta line (provider, branch, snooze), and at the end the PR badge,
/// pin and either the jump hint key or the time (the elapsed time while
/// working).
///
/// States: `text-secondary` at rest; hover `layer-hover-01` and
/// `text-primary`; active `layer-selected-01` with a 3px
/// `border-interactive` start bar; unread titles turn `text-primary`
/// semibold; keyboard focus draws the inset ring.
#[derive(IntoElement)]
pub struct ThreadRow {
    id: ElementId,
    title: SharedString,
    status: Status,
    time: Option<SharedString>,
    elapsed: Option<u64>,
    unread: bool,
    pinned: bool,
    snoozed_until: Option<SharedString>,
    draft: bool,
    draft_label: SharedString,
    active: bool,
    pull_request: Option<(u32, PullRequestState)>,
    provider: Option<SharedString>,
    jump_hint: Option<SharedString>,
    branch: Option<SharedString>,
    on_click: Option<Handler>,
}

impl ThreadRow {
    /// A row for thread `title`.
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            status: Status::Idle,
            time: None,
            elapsed: None,
            unread: false,
            pinned: false,
            snoozed_until: None,
            draft: false,
            draft_label: "Unsent draft · ".into(),
            active: false,
            pull_request: None,
            provider: None,
            jump_hint: None,
            branch: None,
            on_click: None,
        }
    }

    /// The status (`status`, default idle).
    pub fn status(mut self, status: Status) -> Self {
        self.status = status;
        self
    }

    /// The relative time (`time`).
    pub fn time(mut self, time: impl Into<SharedString>) -> Self {
        self.time = Some(time.into());
        self
    }

    /// Seconds elapsed, shown instead of `time` while working (`elapsed`).
    pub fn elapsed(mut self, secs: u64) -> Self {
        self.elapsed = Some(secs);
        self
    }

    /// Unread (`unread`).
    pub fn unread(mut self, unread: bool) -> Self {
        self.unread = unread;
        self
    }

    /// Pinned (`pinned`).
    pub fn pinned(mut self, pinned: bool) -> Self {
        self.pinned = pinned;
        self
    }

    /// Snoozed until (`snoozedUntil`).
    pub fn snoozed_until(mut self, until: impl Into<SharedString>) -> Self {
        self.snoozed_until = Some(until.into());
        self
    }

    /// An unsent draft (`draft`).
    pub fn draft(mut self, draft: bool) -> Self {
        self.draft = draft;
        self
    }

    /// The draft prefix (default `"Unsent draft · "`).
    pub fn draft_label(mut self, label: impl Into<SharedString>) -> Self {
        self.draft_label = label.into();
        self
    }

    /// The current thread (`active`).
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// The thread's pull request (`pullRequest`).
    pub fn pull_request(mut self, number: u32, state: PullRequestState) -> Self {
        self.pull_request = Some((number, state));
        self
    }

    /// The agent provider (`provider`).
    pub fn provider(mut self, provider: impl Into<SharedString>) -> Self {
        self.provider = Some(provider.into());
        self
    }

    /// A jump shortcut shown instead of the time (`jumpHint`, e.g. `⌘3`).
    pub fn jump_hint(mut self, hint: impl Into<SharedString>) -> Self {
        self.jump_hint = Some(hint.into());
        self
    }

    /// The branch (`branch`).
    pub fn branch(mut self, branch: impl Into<SharedString>) -> Self {
        self.branch = Some(branch.into());
        self
    }

    /// `onClick`.
    pub fn on_click(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = handler(f);
        self
    }

    /// The status slot glyph: working spins; needs-you, failed and done
    /// take their filled status glyphs; otherwise a draft pencil, the
    /// unread dot or a chat bubble.
    fn status_glyph(&self) -> AnyElement {
        let secondary = color(ColorRole::IconSecondary);
        match self.status {
            Status::Working => Spin::new((self.id.clone(), "spin")).into_any_element(),
            Status::NeedsYou => warning_glyph(16.).into_any_element(),
            // `.t3-thread--failed .t3-thread-status svg { fill: var(--support-error) }`.
            Status::Failed => icon(CarbonIcon::ErrorFilled, 16., color(ColorRole::SupportError))
                .into_any_element(),
            // `.t3-thread--done .t3-thread-status svg { fill: var(--support-success) }`.
            Status::Done => icon(
                CarbonIcon::CheckmarkFilled,
                16.,
                color(ColorRole::SupportSuccess),
            )
            .into_any_element(),
            _ if self.draft => icon(CarbonIcon::Edit, 16., secondary).into_any_element(),
            // `.t3-unread { width: 8px; height: 8px; border-radius: 50%; background: var(--interactive) }`.
            _ if self.unread => div()
                .size(rem(8.))
                .rounded_full()
                .bg(color(ColorRole::Interactive))
                .into_any_element(),
            _ => icon(CarbonIcon::Chat, 16., secondary).into_any_element(),
        }
    }
}

/// (background, text) of a thread row: active `layer-selected-01`, hover
/// `layer-hover-01` (both `text-primary`), else transparent `text-secondary`.
pub fn thread_row_colors(active: bool, hovered: bool) -> (Hsla, Hsla) {
    if active {
        (
            color(ColorRole::LayerSelected01),
            color(ColorRole::TextPrimary),
        )
    } else if hovered {
        (
            color(ColorRole::LayerHover01),
            color(ColorRole::TextPrimary),
        )
    } else {
        (
            gpui_kit::transparent_black(),
            color(ColorRole::TextSecondary),
        )
    }
}

impl RenderOnce for ThreadRow {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let (hovered, hover_store) = hover((id.clone(), "hover"), window, cx);
        let (focus_handle, focused) = focus((id.clone(), "focus"), window, cx);
        let (bg, fg) = thread_row_colors(self.active, hovered);
        let helper = color(ColorRole::TextHelper);
        let status = self.status_glyph();
        // `.t3-thread-title { font-size: 0.875rem; line-height: 1.25rem; ellipsis }`;
        // `.t3-thread--unread .t3-thread-title { color: text-primary; font-weight: 600 }`.
        let title = sans(div(), 14., if self.unread { 600 } else { 400 })
            .line_height(rem(20.))
            .min_w_0()
            .truncate()
            .when(self.unread, |d| {
                d.text_color(color(ColorRole::TextPrimary))
                    .font_weight(FontWeight::SEMIBOLD)
            })
            .when(self.draft, |d| {
                // `.t3-thread-title em { font-style: normal; color: var(--text-helper) }`.
                d.child(
                    gpui_kit::StyledText::new(format!("{}{}", self.draft_label, self.title))
                        .with_highlights([(
                            0..self.draft_label.len(),
                            gpui_kit::HighlightStyle {
                                color: Some(helper),
                                ..Default::default()
                            },
                        )]),
                )
            })
            .when(!self.draft, |d| d.child(self.title.clone()));
        // `.t3-thread-meta { gap: var(--spacing-03); font-size: 0.75rem; color:
        // var(--text-helper); nowrap }`; spans shrink and ellipsize, the first
        // keeps its width up to 70%.
        let mut meta: Vec<AnyElement> = Vec::new();
        if let Some(p) = self.provider {
            meta.push(
                div()
                    .flex_none()
                    .max_w(gpui_kit::relative(0.7))
                    .truncate()
                    .child(p)
                    .into_any_element(),
            );
        }
        if let Some(b) = self.branch {
            // `.t3-thread-branch { gap: 2px; font-family: var(--font-mono) }`.
            meta.push(
                mono(div(), 12., 400)
                    .flex()
                    .flex_shrink(1.)
                    .min_w_0()
                    .items_center()
                    .gap(px(2.))
                    .child(icon(CarbonIcon::Branch, 12., helper).flex_none())
                    .child(div().min_w_0().truncate().child(b))
                    .into_any_element(),
            );
        }
        if let Some(s) = self.snoozed_until {
            meta.push(
                div()
                    .flex()
                    .flex_shrink(1.)
                    .min_w_0()
                    .items_center()
                    .child(icon(CarbonIcon::Asleep, 12., helper).flex_none())
                    .child(div().min_w_0().truncate().child(format!(" {s}")))
                    .into_any_element(),
            );
        }
        let meta_line = (!meta.is_empty()).then(|| {
            sans(div(), 12., 400)
                .flex()
                .min_w_0()
                .overflow_hidden()
                .whitespace_nowrap()
                .gap(rem(8.))
                .text_color(helper)
                .children(meta)
        });
        let time_text = if self.status == Status::Working && self.elapsed.is_some() {
            self.elapsed.map(|e| SharedString::from(fmt_elapsed(e)))
        } else {
            self.time
        };
        // `.t3-thread-side { gap: var(--spacing-02); height: 1.25rem;
        // font-size: 0.75rem; color: var(--text-helper) }`.
        let side = sans(div(), 12., 400)
            .flex()
            .flex_none()
            .items_center()
            .gap(rem(4.))
            .h(rem(20.))
            .text_color(helper)
            .children(
                self.pull_request
                    .map(|(n, s)| PullRequestMiniBadge::new(n, s)),
            )
            .when(self.pinned, |d| {
                d.child(icon(
                    CarbonIcon::PinFilled,
                    12.,
                    color(ColorRole::IconSecondary),
                ))
            })
            .map(|d| match self.jump_hint {
                Some(k) => d.child(Kbd::new(k)),
                None => d.children(time_text),
            });
        let on_click = self.on_click;
        // `.t3-thread { gap: var(--spacing-03); min-height: var(--size-lg);
        // padding: var(--spacing-03) var(--spacing-03) var(--spacing-03) var(--spacing-05) }`.
        press((id.clone(), "row"), &focus_handle)
            .flex()
            .w_full()
            .items_start()
            .gap(rem(8.))
            .min_h(SIZE_LG.rems())
            .pt(rem(8.))
            .pb(rem(8.))
            .pr(rem(8.))
            .pl(rem(16.))
            .bg(bg)
            .text_color(fg)
            .on_hover(track_hover(hover_store))
            .when_some(on_click, |b, f| b.on_click(move |_, w, cx| f(w, cx)))
            // `.t3-thread--active { box-shadow: inset 3px 0 0 var(--border-interactive) }`.
            .when(self.active, |b| {
                b.child(
                    div()
                        .absolute()
                        .top_0()
                        .bottom_0()
                        .left_0()
                        .w(rem(3.))
                        .bg(color(ColorRole::BorderInteractive)),
                )
            })
            .child(
                // `.t3-thread-status { width: 1rem; height: 1.25rem }`.
                div()
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .w(rem(16.))
                    .h(rem(20.))
                    .child(status),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .gap(px(2.))
                    .child(title)
                    .children(meta_line),
            )
            .child(side)
            .when(focused, |b| b.child(ring()))
    }
}

// ── UpdatePill ──────────────────────────────────────────────────────────

/// `UpdatePill` `state`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum UpdateState {
    /// `available` (the default).
    #[default]
    Available,
    /// `downloading`: a progress ring.
    Downloading,
    /// `ready`: a primary button.
    Ready,
    /// `error`.
    Error,
    /// `installing`.
    Installing,
}

impl UpdateState {
    /// Every state.
    pub const ALL: [UpdateState; 5] = [
        Self::Available,
        Self::Downloading,
        Self::Ready,
        Self::Error,
        Self::Installing,
    ];

    /// The reference name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Downloading => "downloading",
            Self::Ready => "ready",
            Self::Error => "error",
            Self::Installing => "installing",
        }
    }

    /// The default English text; callers may pass their own `label`.
    pub fn default_label(self, version: &str, progress: u8) -> String {
        match self {
            Self::Available => format!("Update to {version}"),
            Self::Downloading => format!("Downloading {progress}%"),
            Self::Ready => "Restart to update".into(),
            Self::Error => "Could not download update".into(),
            Self::Installing => "Installing…".into(),
        }
    }
}

/// (background, text, glyph) of the update pill: `.t3-update` `layer-02`
/// (`layer-hover-02` on hover), `text-primary`, `interactive` glyph;
/// `--ready` `button-primary` (`button-primary-hover`), `text-on-color`,
/// `icon-on-color`.
pub fn update_colors(state: UpdateState, hovered: bool) -> (Hsla, Hsla, Hsla) {
    if state == UpdateState::Ready {
        (
            color(if hovered {
                ColorRole::ButtonPrimaryHover
            } else {
                ColorRole::ButtonPrimary
            }),
            color(ColorRole::TextOnColor),
            color(ColorRole::IconOnColor),
        )
    } else {
        (
            color(if hovered {
                ColorRole::LayerHover02
            } else {
                ColorRole::Layer02
            }),
            color(ColorRole::TextPrimary),
            color(ColorRole::Interactive),
        )
    }
}

/// The download ring: `conic-gradient(interactive p%, layer-accent-01 0)`
/// masked to a ring from r 5.25 to 8 (`radial-gradient(circle, transparent
/// 5px, #000 5.5px)`), filling clockwise from 12 o'clock.
fn progress_ring(progress: u8) -> impl IntoElement {
    let p = progress.min(100) as f32 / 100.;
    canvas(
        |_, _, _| {},
        move |bounds, _, window, _| {
            let k = f32::from(bounds.size.width) / 16.;
            let (inner, outer) = (5.25 * k, 8. * k);
            let r = (inner + outer) / 2.;
            let w = outer - inner;
            let start = -FRAC_PI_2;
            let center = bounds.center();
            stroke_arc(
                window,
                center,
                r,
                w,
                start,
                TAU,
                color(ColorRole::LayerAccent01),
            );
            stroke_arc(
                window,
                center,
                r,
                w,
                start,
                TAU * p,
                color(ColorRole::Interactive),
            );
        },
    )
    .flex_none()
    .size(rem(16.))
}

/// The sidebar-foot update prompt (`UpdatePill`, `.t3-update`): a 40px
/// full-width button with a 1px `border-subtle-01` inset frame, 16px
/// inline padding, an `interactive` glyph (Upgrade; a progress ring while
/// downloading; the warning glyph on error; Restart when ready, on a
/// primary fill) and the state's text.
#[derive(IntoElement)]
pub struct UpdatePill {
    id: ElementId,
    state: UpdateState,
    version: SharedString,
    progress: u8,
    label: Option<SharedString>,
    on_click: Option<Handler>,
}

impl UpdatePill {
    /// An update pill in `state`.
    pub fn new(id: impl Into<ElementId>, state: UpdateState) -> Self {
        Self {
            id: id.into(),
            state,
            version: SharedString::default(),
            progress: 0,
            label: None,
            on_click: None,
        }
    }

    /// The new version (`version`).
    pub fn version(mut self, version: impl Into<SharedString>) -> Self {
        self.version = version.into();
        self
    }

    /// Download progress 0–100 (`progress`).
    pub fn progress(mut self, progress: u8) -> Self {
        self.progress = progress;
        self
    }

    /// Replace the text (`label`).
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// `onClick`.
    pub fn on_click(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = handler(f);
        self
    }
}

impl RenderOnce for UpdatePill {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id;
        let (hovered, hover_store) = hover((id.clone(), "hover"), window, cx);
        let (focus_handle, focused) = focus((id.clone(), "focus"), window, cx);
        let (bg, fg, glyph) = update_colors(self.state, hovered);
        let text = self.label.unwrap_or_else(|| {
            self.state
                .default_label(&self.version, self.progress)
                .into()
        });
        let lead: AnyElement = match self.state {
            UpdateState::Downloading => progress_ring(self.progress).into_any_element(),
            UpdateState::Error => warning_glyph(16.).into_any_element(),
            UpdateState::Ready => Icon::carbon(CarbonIcon::Restart)
                .size_length(rem(16.))
                .color(glyph)
                .flex_none()
                .into_any_element(),
            _ => Icon::carbon(CarbonIcon::Upgrade)
                .size_length(rem(16.))
                .color(glyph)
                .flex_none()
                .into_any_element(),
        };
        let framed = self.state != UpdateState::Ready;
        sans(press((id.clone(), "btn"), &focus_handle), 14., 400)
            .flex()
            .w_full()
            .items_center()
            .gap(rem(8.))
            .h(SIZE_MD.rems())
            .px(rem(16.))
            .bg(bg)
            .text_color(fg)
            // `box-shadow: inset 0 0 0 1px var(--border-subtle-01)`; `--ready { box-shadow: none }`.
            .when(framed, |b| {
                b.shadow(inset_frame(color(ColorRole::BorderSubtle01)))
            })
            .on_hover(track_hover(hover_store))
            .when_some(self.on_click, |b, f| b.on_click(move |_, w, cx| f(w, cx)))
            .child(lead)
            .child(div().min_w_0().truncate().child(text))
            .when(focused, |b| b.child(ring()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    fn hex(c: Hsla) -> String {
        let c = c.to_rgb();
        let b = |v: f32| (v * 255.).round() as u8;
        format!("#{:02x}{:02x}{:02x}", b(c.r), b(c.g), b(c.b))
    }

    #[test]
    fn counts_follow_the_reference_order_and_colors() {
        crate::theme::tests_set(CarbonTheme::White);
        let counts = FolderCounts {
            needs_you: 1,
            failed: 0,
            working: 2,
            done: 3,
        };
        assert_eq!(
            counts.visible(),
            vec![
                (Status::NeedsYou, 1),
                (Status::Working, 2),
                (Status::Done, 3)
            ]
        );
        let (bg, fg) = count_colors(Status::NeedsYou);
        assert_eq!((hex(bg), hex(fg)), ("#f1c21b".into(), "#161616".into()));
        let (bg, fg) = count_colors(Status::Failed);
        assert_eq!((hex(bg), hex(fg)), ("#da1e28".into(), "#ffffff".into()));
    }

    #[test]
    fn a_failed_count_on_a_dark_theme_is_dark_ink_on_its_light_red() {
        crate::theme::tests_set(CarbonTheme::G100);
        let (bg, fg) = count_colors(Status::Failed);
        assert_eq!((hex(bg), hex(fg)), ("#fa4d56".into(), "#161616".into()));
    }

    #[test]
    fn a_count_is_quiet_unless_given_a_status() {
        crate::theme::tests_set(CarbonTheme::White);
        let (bg, fg) = Count::new(3).colors();
        assert_eq!((hex(bg), hex(fg)), ("#e0e0e0".into(), "#525252".into()));
        assert_eq!(
            Count::new(3).status(Status::Working).colors(),
            count_colors(Status::Working)
        );
    }

    #[test]
    fn thread_rows_follow_t3_css() {
        crate::theme::tests_set(CarbonTheme::White);
        let (bg, fg) = thread_row_colors(false, false);
        assert_eq!((bg.a, hex(fg)), (0., "#525252".into()));
        let (bg, fg) = thread_row_colors(false, true);
        assert_eq!((hex(bg), hex(fg)), ("#e8e8e8".into(), "#161616".into()));
        let (bg, _) = thread_row_colors(true, true);
        assert_eq!(hex(bg), "#e0e0e0");
        let (bg, fg) = PullRequestState::Merged.tag_colors();
        assert_eq!((hex(bg), hex(fg)), ("#e8daff".into(), "#6929c4".into()));
    }

    #[test]
    fn update_pill_follows_t3_css() {
        crate::theme::tests_set(CarbonTheme::White);
        assert_eq!(
            UpdateState::Available.default_label("1.4.2", 0),
            "Update to 1.4.2"
        );
        assert_eq!(
            UpdateState::Downloading.default_label("", 62),
            "Downloading 62%"
        );
        let (bg, fg, glyph) = update_colors(UpdateState::Available, false);
        assert_eq!(
            (hex(bg), hex(fg), hex(glyph)),
            ("#ffffff".into(), "#161616".into(), "#0f62fe".into())
        );
        let (bg, _, _) = update_colors(UpdateState::Ready, true);
        assert_eq!(hex(bg), "#0050e6");
    }
}
