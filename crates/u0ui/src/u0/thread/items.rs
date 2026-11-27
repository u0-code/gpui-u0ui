//! ContinueRow, PlanChecklist and MessageImage (`src/ghost/workspace.jsx`;
//! `.g-cont*`, `.g-plan*`, `.g-mimg*` in `src/ghost.css`, with the d-fix
//! and app-r3 rules of `src/t3.css`).
//!
//! - **ContinueRow**: one "Pick up" row, a full-width button: the agent's
//!   3px accent bar, the title, project · agent · time, an optional status
//!   tag ("Needs you" warm gray, "Failed" red, others gray) and the blue
//!   "Pick up →".
//! - **PlanChecklist**: the agent's plan: a head with progress ("2/5 done"
//!   and a 2px `support-success` bar) that folds the list; items are
//!   pending (dashed circle), in progress (spinner, semibold), done (green
//!   check, struck through), and the app-r3 states needs-you (yellow) and
//!   skipped (struck, `text-helper`). State is told by glyph and hidden text.
//! - **MessageImage**: an image in a message, at most 320×240 with a hairline
//!   ring, opening full size on click (a zoom badge shows on hover or focus);
//!   a placeholder tile without `src`; an optional caption.
use crate::{
    components::{Badge, BadgeSize, BadgeVariant},
    density,
    icons::{CarbonIcon, Icon},
    t3::{
        base::{Spin, visually_hidden},
        craft::{rem, sans},
    },
    theme::color,
    tokens::{self, ColorRole},
    u0::{
        foundations::agent_accent,
        primitives::parts::{focus_ring, focus_visible, keyed_focus},
    },
};
use gpui_kit::{
    AnyElement, App, BoxShadow, CursorStyle, ElementId, Hsla, ImageSource, InteractiveElement as _,
    IntoElement, ObjectFit, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement as _, Styled, StyledImage as _, Transformation, Window, div, img,
    point, prelude::FluentBuilder as _, px, radians, relative,
};
use std::f32::consts::FRAC_PI_2;
use std::rc::Rc;

type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

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

// ── ContinueRow ─────────────────────────────────────────────────────────

/// A paused session's status tag (`status`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContinueStatus {
    /// `needs-you`: "Needs you" in warm gray.
    NeedsYou,
    /// `Failed`: the red tag (d-fix).
    Failed,
    /// Any other status, shown as given in gray.
    Other(SharedString),
}

impl ContinueStatus {
    /// The tag color: `status === 'needs-you' ? 'warm-gray' : 'gray'`, and
    /// red for "Failed" (`.g-cont .cds--tag:has([title="Failed"])`).
    pub fn variant(&self) -> BadgeVariant {
        match self {
            Self::NeedsYou => BadgeVariant::WarmGray,
            Self::Failed => BadgeVariant::Red,
            Self::Other(_) => BadgeVariant::Gray,
        }
    }
}

/// The row's user-facing strings (the reference's defaults).
#[derive(Clone, Debug)]
pub struct ContinueRowTexts {
    /// `"Pick up"`.
    pub pick_up: SharedString,
    /// The needs-you tag: `"Needs you"`.
    pub needs_you: SharedString,
    /// The failed tag: `"Failed"`.
    pub failed: SharedString,
}

impl Default for ContinueRowTexts {
    fn default() -> Self {
        Self {
            pick_up: "Pick up".into(),
            needs_you: "Needs you".into(),
            failed: "Failed".into(),
        }
    }
}

/// A "Pick up" row. See the module docs.
#[derive(IntoElement)]
pub struct ContinueRow {
    id: ElementId,
    title: SharedString,
    project: SharedString,
    agent: SharedString,
    accent: Option<usize>,
    time: SharedString,
    status: Option<ContinueStatus>,
    on_pick_up: Option<Handler>,
    texts: ContinueRowTexts,
}

impl ContinueRow {
    /// A row for the session `title`.
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            project: SharedString::default(),
            agent: SharedString::default(),
            accent: None,
            time: SharedString::default(),
            status: None,
            on_pick_up: None,
            texts: ContinueRowTexts::default(),
        }
    }

    /// `project`.
    pub fn project(mut self, project: impl Into<SharedString>) -> Self {
        self.project = project.into();
        self
    }

    /// `agent`.
    pub fn agent(mut self, agent: impl Into<SharedString>) -> Self {
        self.agent = agent.into();
        self
    }

    /// The agent's accent slot, 1–8 (`accent`); `border-strong-01` without.
    pub fn accent(mut self, slot: usize) -> Self {
        self.accent = Some(slot);
        self
    }

    /// `time`.
    pub fn time(mut self, time: impl Into<SharedString>) -> Self {
        self.time = time.into();
        self
    }

    /// `status`.
    pub fn status(mut self, status: ContinueStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// The row pressed (`onPickUp`).
    pub fn on_pick_up(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_pick_up = Some(Rc::new(f));
        self
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: ContinueRowTexts) -> Self {
        self.texts = texts;
        self
    }
}

/// `.g-cont-accent { background: var(--agent-accent-N) }`, else
/// `border-strong-01`.
pub fn accent_fill(slot: Option<usize>) -> Hsla {
    color(slot.map_or(ColorRole::BorderStrong01, agent_accent))
}

impl RenderOnce for ContinueRow {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let focus = keyed_focus((id.clone(), "focus"), window, cx);
        let ring = focus_visible(&focus, window);
        let (over, on_hover) = hover_state((id.clone(), "hover").into(), window, cx);
        let on_pick_up = self.on_pick_up.clone();
        let link = color(ColorRole::LinkPrimary);
        let dot = || {
            // `.t3-metadot { color: var(--text-helper); padding: 0 2px }`.
            div()
                .px(px(2.))
                .text_color(color(ColorRole::TextHelper))
                .child("·")
        };
        let status = self.status.clone().map(|s| {
            let label = match &s {
                ContinueStatus::NeedsYou => self.texts.needs_you.clone(),
                ContinueStatus::Failed => self.texts.failed.clone(),
                ContinueStatus::Other(t) => t.clone(),
            };
            Badge::new(label).size(BadgeSize::Sm).variant(s.variant())
        });
        // `.g-cont { display: flex; align-items: center; gap: var(--spacing-04);
        // min-height: var(--density-row-lg); padding: var(--spacing-02)
        // var(--spacing-04) var(--spacing-02) 0; border-bottom: 1px solid
        // var(--border-subtle-01); background: var(--layer-01) }`, hover
        // `layer-hover-01`.
        sans(gpui_kit::base::Button::new(id), 14., 400)
            .track_focus(&focus)
            .relative()
            .flex()
            .items_center()
            .w_full()
            .gap(tokens::SPACING_04)
            .min_h(density::current().row_lg())
            .py(tokens::SPACING_02)
            .pr(tokens::SPACING_04)
            .border_b_1()
            .border_color(color(ColorRole::BorderSubtle01))
            .bg(color(if over {
                ColorRole::LayerHover01
            } else {
                ColorRole::Layer01
            }))
            .text_color(color(ColorRole::TextPrimary))
            .cursor(CursorStyle::PointingHand)
            .on_hover(on_hover)
            .on_click(move |_, w, cx| {
                if let Some(f) = &on_pick_up {
                    f(w, cx);
                }
            })
            // `.g-cont-accent { align-self: stretch; width: 3px; flex: none }`.
            .child(
                div()
                    .flex_none()
                    .self_stretch()
                    .w(px(3.))
                    .bg(accent_fill(self.accent)),
            )
            .child(
                // `.g-cont-main { flex: 1; min-width: 0; flex-direction: column; gap: 2px }`.
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .gap(px(2.))
                    .child(div().min_w_0().truncate().child(self.title.clone()))
                    .child(
                        // `.g-cont-meta { font-size: 0.75rem; color: var(--text-secondary) }`.
                        sans(div(), 12., 400)
                            .flex()
                            .flex_wrap()
                            .text_color(color(ColorRole::TextSecondary))
                            .child(self.project.clone())
                            .child(dot())
                            .child(self.agent.clone())
                            .child(dot())
                            .child(self.time.clone()),
                    ),
            )
            .children(status)
            .child(
                // `.g-cont-go { display: inline-flex; gap: var(--spacing-02);
                // font-size: 0.875rem; color: var(--link-primary) }`.
                div()
                    .flex()
                    .flex_none()
                    .items_center()
                    .gap(tokens::SPACING_02)
                    .text_color(link)
                    .child(self.texts.pick_up.clone())
                    .child(
                        Icon::carbon(CarbonIcon::ArrowRight)
                            .size_length(rem(14.))
                            .color(link),
                    ),
            )
            .when(ring, |b| b.child(focus_ring()))
    }
}

// ── PlanChecklist ───────────────────────────────────────────────────────

/// A plan step's state (`status`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PlanStatus {
    /// `pending` (the default): a dashed circle.
    #[default]
    Pending,
    /// `in-progress`: a spinner, semibold.
    InProgress,
    /// `done`: a green check, struck through.
    Done,
    /// `needs-you` (app-r3): waiting on the user, yellow.
    NeedsYou,
    /// `skipped` (app-r3): struck through in `text-helper`.
    Skipped,
}

impl PlanStatus {
    /// Every state.
    pub const ALL: [Self; 5] = [
        Self::Pending,
        Self::InProgress,
        Self::Done,
        Self::NeedsYou,
        Self::Skipped,
    ];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::InProgress => "in-progress",
            Self::Done => "done",
            Self::NeedsYou => "needs-you",
            Self::Skipped => "skipped",
        }
    }

    /// (text ink, glyph ink, struck, semibold): `.g-plan--done { color:
    /// text-secondary } > span { line-through } svg { support-success }`,
    /// `--in-progress { font-weight: 600 }`, `--pending svg { icon-secondary
    /// }`, app-r3 `--needs-you > svg { support-warning } > span {
    /// text-primary }`, `--skipped { color: text-helper } > span { line-through }`.
    pub fn look(self) -> (Hsla, Hsla, bool, bool) {
        use ColorRole::*;
        let (text, glyph, struck, bold) = match self {
            Self::Pending => (TextPrimary, IconSecondary, false, false),
            Self::InProgress => (TextPrimary, Interactive, false, true),
            Self::Done => (TextSecondary, SupportSuccess, true, false),
            Self::NeedsYou => (TextPrimary, SupportWarning, false, false),
            Self::Skipped => (TextHelper, TextHelper, true, false),
        };
        (color(text), color(glyph), struck, bold)
    }
}

/// One plan step (`items[i]`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanItem {
    /// `text`.
    pub text: SharedString,
    /// `status`.
    pub status: PlanStatus,
}

impl PlanItem {
    /// A step.
    pub fn new(text: impl Into<SharedString>, status: PlanStatus) -> Self {
        Self {
            text: text.into(),
            status,
        }
    }
}

/// `{done}/{n}`: the count of done steps and the bar's fill fraction
/// (`done / Math.max(items.length, 1)`).
pub fn plan_progress(items: &[PlanItem]) -> (usize, f32) {
    let done = items
        .iter()
        .filter(|i| i.status == PlanStatus::Done)
        .count();
    (done, done as f32 / items.len().max(1) as f32)
}

/// The checklist's user-facing strings (the reference's defaults).
#[derive(Clone)]
pub struct PlanChecklistTexts {
    /// `"{done}/{n} done"`.
    pub progress: Rc<dyn Fn(usize, usize) -> SharedString>,
    /// Hidden state after each step: `" (pending)"`, `" (in progress)"`, …
    pub state: Rc<dyn Fn(PlanStatus) -> SharedString>,
}

impl Default for PlanChecklistTexts {
    fn default() -> Self {
        Self {
            progress: Rc::new(|d, n| format!("{d}/{n} done").into()),
            state: Rc::new(|s| {
                let name = match s {
                    PlanStatus::InProgress => "in progress",
                    other => other.name(),
                };
                format!(" ({name})").into()
            }),
        }
    }
}

/// The agent's plan. See the module docs.
#[derive(IntoElement)]
pub struct PlanChecklist {
    id: ElementId,
    title: SharedString,
    items: Vec<PlanItem>,
    collapsed: bool,
    texts: PlanChecklistTexts,
}

impl PlanChecklist {
    /// A plan titled `title` (`title`, the reference's "Plan").
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            items: Vec::new(),
            collapsed: false,
            texts: PlanChecklistTexts::default(),
        }
    }

    /// The steps (`items`).
    pub fn items(mut self, items: impl IntoIterator<Item = PlanItem>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    /// Start folded (`collapsed`).
    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: PlanChecklistTexts) -> Self {
        self.texts = texts;
        self
    }
}

/// The head chevron's turn: `.t3-rot { transform: rotate(-90deg) }` while
/// folded. It has no transition (computed `transition: all 0s`), so it snaps
/// instead of animating.
pub fn plan_chevron_angle(open: bool) -> f32 {
    if open { 0. } else { -FRAC_PI_2 }
}

impl RenderOnce for PlanChecklist {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let collapsed = self.collapsed;
        let store = window.use_keyed_state((id.clone(), "open"), cx, |_, _| !collapsed);
        let open = *store.read(cx);
        let focus = keyed_focus((id.clone(), "focus"), window, cx);
        let ring = focus_visible(&focus, window);
        let (done, fraction) = plan_progress(&self.items);
        let secondary = color(ColorRole::IconSecondary);
        let angle = plan_chevron_angle(open);
        // `.g-plan-head { display: flex; align-items: center; gap:
        // var(--spacing-03); height: var(--density-row-lg); padding: 0
        // var(--spacing-04); font-size: 0.875rem; color: var(--text-primary) }`.
        let head = sans(gpui_kit::base::Button::new((id.clone(), "head")), 14., 400)
            .track_focus(&focus)
            .relative()
            .flex()
            .items_center()
            .w_full()
            .gap(tokens::SPACING_03)
            .h(density::current().row_lg())
            .px(tokens::SPACING_04)
            .text_color(color(ColorRole::TextPrimary))
            .cursor(CursorStyle::PointingHand)
            .on_click(move |_, _, cx| {
                store.update(cx, |o, cx| {
                    *o = !*o;
                    cx.notify();
                })
            })
            .child(
                Icon::carbon(CarbonIcon::ListChecked)
                    .size_length(rem(16.))
                    .color(secondary),
            )
            .child(sans(div(), 14., 600).flex_none().child(self.title.clone()))
            .child(
                sans(div(), 12., 400)
                    .flex_none()
                    .text_color(color(ColorRole::TextSecondary))
                    .child((self.texts.progress)(done, self.items.len())),
            )
            .child(
                // `.g-plan-bar { flex: 1; height: 2px; background:
                // var(--border-subtle-01) } i { background: var(--support-success) }`.
                div()
                    .flex_1()
                    .h(px(2.))
                    .bg(color(ColorRole::BorderSubtle01))
                    .child(
                        div()
                            .h_full()
                            .w(relative(fraction))
                            .bg(color(ColorRole::SupportSuccess)),
                    ),
            )
            .child(
                Icon::carbon(CarbonIcon::ChevronDown)
                    .size_length(rem(14.))
                    .color(secondary)
                    .transform(Transformation::rotate(radians(angle))),
            )
            .when(ring, |h| h.child(focus_ring()));
        let list = open.then(|| {
            // `.g-plan-list { padding: 0 var(--spacing-04) var(--spacing-03) }`.
            div()
                .flex()
                .flex_col()
                .px(tokens::SPACING_04)
                .pb(tokens::SPACING_03)
                .children(self.items.iter().enumerate().map(|(i, it)| {
                    let (ink, glyph_ink, struck, bold) = it.status.look();
                    let glyph: AnyElement = if it.status == PlanStatus::InProgress {
                        Spin::new((id.clone(), SharedString::from(format!("spin-{i}"))))
                            .into_any_element()
                    } else {
                        let name = match it.status {
                            PlanStatus::Done => CarbonIcon::CheckmarkFilled,
                            _ => CarbonIcon::CircleDash,
                        };
                        Icon::carbon(name)
                            .size_length(rem(16.))
                            .color(glyph_ink)
                            .into_any_element()
                    };
                    // `.g-plan-list li { display: flex; align-items: flex-start;
                    // gap: var(--spacing-03); padding: var(--spacing-02) 0;
                    // font-size: 0.875rem; line-height: 1.25rem }`; the glyph
                    // sits 2px down.
                    sans(div(), 14., if bold { 600 } else { 400 })
                        .relative()
                        .flex()
                        .items_start()
                        .gap(tokens::SPACING_03)
                        .py(tokens::SPACING_02)
                        .line_height(rem(20.))
                        .text_color(ink)
                        .child(div().flex_none().mt(px(2.)).child(glyph))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .when(struck, |d| d.line_through())
                                .child(it.text.clone()),
                        )
                        .child(visually_hidden((self.texts.state)(it.status)))
                }))
        });
        // `.g-plan { background: var(--layer-01) }`.
        div()
            .id(id)
            .flex()
            .flex_col()
            .w_full()
            .bg(color(ColorRole::Layer01))
            .child(head)
            .children(list)
    }
}

// ── MessageImage ────────────────────────────────────────────────────────

/// `.g-mimg-btn img { max-width: 20rem; max-height: 15rem }`.
pub const MAX_SIZE: (f32, f32) = (320., 240.);

/// The drawn size of a `w`×`h` image inside 320×240, keeping its ratio.
pub fn fit_size(w: f32, h: f32) -> (f32, f32) {
    let k = (MAX_SIZE.0 / w).min(MAX_SIZE.1 / h).min(1.);
    (w * k, h * k)
}

/// The image's user-facing strings (the reference's defaults).
#[derive(Clone)]
pub struct MessageImageTexts {
    /// The button's name: `"Open {alt} full size"` (`alt || 'image'`).
    pub open: Rc<dyn Fn(&str) -> SharedString>,
}

impl Default for MessageImageTexts {
    fn default() -> Self {
        Self {
            open: Rc::new(|alt| {
                let alt = if alt.is_empty() { "image" } else { alt };
                format!("Open {alt} full size").into()
            }),
        }
    }
}

/// An image in a message. See the module docs.
#[derive(IntoElement)]
pub struct MessageImage {
    id: ElementId,
    src: Option<ImageSource>,
    alt: SharedString,
    size: (f32, f32),
    caption: Option<SharedString>,
    on_open: Option<Handler>,
    texts: MessageImageTexts,
}

impl MessageImage {
    /// An image described by `alt` (required, for screen readers).
    pub fn new(id: impl Into<ElementId>, alt: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            src: None,
            alt: alt.into(),
            size: MAX_SIZE,
            caption: None,
            on_open: None,
            texts: MessageImageTexts::default(),
        }
    }

    /// The image (`src`); a placeholder tile without it.
    pub fn src(mut self, src: impl Into<ImageSource>) -> Self {
        self.src = Some(src.into());
        self
    }

    /// Its size in px (`width`, `height`), fitted into 320×240.
    pub fn dimensions(mut self, width: f32, height: f32) -> Self {
        self.size = (width, height);
        self
    }

    /// A caption under it (`caption`).
    pub fn caption(mut self, caption: impl Into<SharedString>) -> Self {
        self.caption = Some(caption.into());
        self
    }

    /// Open it full size (`onOpen`), e.g. in an `ImageViewer`.
    pub fn on_open(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_open = Some(Rc::new(f));
        self
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: MessageImageTexts) -> Self {
        self.texts = texts;
        self
    }
}

impl RenderOnce for MessageImage {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let focus = keyed_focus((id.clone(), "focus"), window, cx);
        let ring = focus_visible(&focus, window);
        let (over, on_hover) = hover_state((id.clone(), "hover").into(), window, cx);
        let (w, h) = match self.src {
            Some(_) => fit_size(self.size.0, self.size.1),
            // `.g-mimg-ph { width, height }` as given.
            None => self.size,
        };
        let picture: AnyElement = match &self.src {
            Some(src) => img(src.clone())
                .w(rem(w))
                .h(rem(h))
                .object_fit(ObjectFit::Contain)
                // `box-shadow: 0 0 0 1px var(--border-subtle-01)`.
                .shadow(vec![BoxShadow {
                    color: color(ColorRole::BorderSubtle01),
                    offset: point(px(0.), px(0.)),
                    blur_radius: px(0.),
                    spread_radius: px(1.),
                    inset: false,
                }])
                .into_any_element(),
            // `.g-mimg-ph { display: flex; align-items: center; justify-content:
            // center; background: var(--layer-accent-01) }`, a 24px Image glyph.
            None => div()
                .flex()
                .items_center()
                .justify_center()
                .w(rem(w))
                .h(rem(h))
                .bg(color(ColorRole::LayerAccent01))
                .child(
                    Icon::carbon(CarbonIcon::Image)
                        .size_length(rem(24.))
                        .color(color(ColorRole::IconSecondary)),
                )
                .into_any_element(),
        };
        let on_open = self.on_open.clone();
        let zoom = over || ring;
        // `.g-mimg { display: inline-flex; flex-direction: column; gap: var(--spacing-02) }`.
        div()
            .flex()
            .flex_col()
            .flex_none()
            .items_start()
            .gap(tokens::SPACING_02)
            .child(
                // `.g-mimg-btn { position: relative; cursor: zoom-in }`.
                gpui_kit::base::Button::new(id)
                    .track_focus(&focus)
                    .accessibility_label((self.texts.open)(&self.alt))
                    .relative()
                    .flex_none()
                    .cursor(CursorStyle::PointingHand)
                    .on_hover(on_hover)
                    .on_click(move |_, w, cx| {
                        if let Some(f) = &on_open {
                            f(w, cx);
                        }
                    })
                    .child(picture)
                    .child(
                        // `.g-mimg-zoom { position: absolute; top: 4px; right: 4px;
                        // padding: 4px; background: var(--background-inverse);
                        // opacity: 0 }`, `.9` on hover / focus.
                        div()
                            .absolute()
                            .top(px(4.))
                            .right(px(4.))
                            .flex()
                            .p(px(4.))
                            .bg(color(ColorRole::BackgroundInverse))
                            .opacity(if zoom { 0.9 } else { 0. })
                            .child(
                                Icon::carbon(CarbonIcon::Maximize)
                                    .size_length(rem(14.))
                                    .color(color(ColorRole::IconInverse)),
                            ),
                    )
                    .when(ring, |b| b.child(focus_ring())),
            )
            .when_some(self.caption, |d, c| {
                // `.g-mimg figcaption { font-size: 0.75rem; color: var(--text-secondary) }`.
                d.child(
                    sans(div(), 12., 400)
                        .text_color(color(ColorRole::TextSecondary))
                        .child(c),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CarbonTheme, theme::color_in};

    #[test]
    fn continue_rows() {
        assert_eq!(ContinueStatus::NeedsYou.variant(), BadgeVariant::WarmGray);
        assert_eq!(ContinueStatus::Failed.variant(), BadgeVariant::Red);
        assert_eq!(
            ContinueStatus::Other("paused".into()).variant(),
            BadgeVariant::Gray
        );
        crate::theme::tests_set(CarbonTheme::White);
        let c = |r| color_in(CarbonTheme::White, r);
        assert_eq!(accent_fill(Some(2)), c(ColorRole::AgentAccent2));
        assert_eq!(accent_fill(None), c(ColorRole::BorderStrong01));
    }

    #[test]
    fn plan_progress_and_looks() {
        let items = [
            PlanItem::new("a", PlanStatus::Done),
            PlanItem::new("b", PlanStatus::Done),
            PlanItem::new("c", PlanStatus::InProgress),
            PlanItem::new("d", PlanStatus::Pending),
            PlanItem::new("e", PlanStatus::Pending),
        ];
        assert_eq!(plan_progress(&items), (2, 0.4));
        assert_eq!(plan_progress(&[]), (0, 0.));
        crate::theme::tests_set(CarbonTheme::White);
        let c = |r| color_in(CarbonTheme::White, r);
        let (ink, glyph, struck, _) = PlanStatus::Done.look();
        assert_eq!(
            (ink, glyph, struck),
            (
                c(ColorRole::TextSecondary),
                c(ColorRole::SupportSuccess),
                true
            )
        );
        assert!(PlanStatus::InProgress.look().3);
        assert_eq!(PlanStatus::NeedsYou.look().1, c(ColorRole::SupportWarning));
        assert_eq!(PlanStatus::Skipped.look().0, c(ColorRole::TextHelper));
        let t = PlanChecklistTexts::default();
        assert_eq!((t.state)(PlanStatus::InProgress), " (in progress)");
        assert_eq!((t.progress)(2, 5), "2/5 done");
    }

    #[test]
    fn images_fit_320_by_240() {
        assert_eq!(fit_size(1440., 900.), (320., 200.));
        assert_eq!(fit_size(120., 150.), (120., 150.));
        assert_eq!(fit_size(240., 480.), (120., 240.));
        assert_eq!(
            (MessageImageTexts::default().open)(""),
            "Open image full size"
        );
    }

    #[test]
    fn plan_chevron_snaps_to_the_t3_rot_angle() {
        // A pure function of the state: no animation in between.
        assert_eq!(plan_chevron_angle(true), 0.);
        assert_eq!(plan_chevron_angle(false), -FRAC_PI_2);
    }
}
