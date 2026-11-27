//! ProcessTree: every process the app started, as a tree with CPU and memory
//! (`ProcessTree` in `src/ghost/admin.jsx`, `.g-ptree` in `src/ghost.css`).
//!
//! - **Behavior**: branches start open; each chevron collapses or expands
//!   its children (the chevron turns 90° in 110ms). Every row has its own
//!   Stop button.
//! - **Appearance**: a `density-row-md` header on `layer-accent-01`
//!   (Process · PID · CPU · Memory · Stop), columns `minmax(0, 1fr) 4rem
//!   7.5rem 8.5rem 2.5rem`; rows indent 16px per level from 8px; agent
//!   processes show their provider mark, the rest a terminal glyph; CPU and
//!   memory pair a 2.5rem bar (`charts-1`, `support-error` past 70%) with
//!   the figure.
use crate::{
    components::{Button, ButtonSize, ButtonVariant},
    ghost::primitives::parts::{focus_ring, focus_visible, keyed_focus},
    icons::{CarbonIcon, Icon},
    motion::{self, Motion},
    t3::{
        IdentitySize, ProviderIcon,
        craft::{mono, rem, sans},
    },
    theme::color,
    tokens::{self, ColorRole},
};
use gpui_kit::{
    AnyElement, App, CursorStyle, Div, ElementId, Hsla, InteractiveElement as _, IntoElement,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement as _, Styled, Window, div,
    prelude::FluentBuilder as _, px, relative,
};
use std::{collections::HashSet, rc::Rc};

/// One process (`{pid, name, kind, cpu, mem, children}`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Process {
    /// `pid`.
    pub pid: u32,
    /// `name`.
    pub name: SharedString,
    /// `kind`: the agent behind it (draws its `ProviderIcon`).
    pub kind: Option<SharedString>,
    /// `cpu` in percent.
    pub cpu: f32,
    /// `mem` in MB.
    pub mem: f32,
    /// `children`.
    pub children: Option<Vec<Process>>,
}

impl Process {
    /// A leaf process.
    pub fn new(pid: u32, name: impl Into<SharedString>, cpu: f32, mem: f32) -> Self {
        Self {
            pid,
            name: name.into(),
            kind: None,
            cpu,
            mem,
            children: None,
        }
    }

    /// The agent behind it (`kind`).
    pub fn kind(mut self, kind: impl Into<SharedString>) -> Self {
        self.kind = Some(kind.into());
        self
    }

    /// Its children (makes it a branch).
    pub fn children(mut self, children: impl IntoIterator<Item = Process>) -> Self {
        self.children = Some(children.into_iter().collect());
        self
    }
}

/// The visible rows with their depth: children of open branches only.
pub fn visible_rows(processes: &[Process], open: &HashSet<u32>) -> Vec<(Process, usize)> {
    fn walk(ps: &[Process], d: usize, open: &HashSet<u32>, out: &mut Vec<(Process, usize)>) {
        for p in ps {
            out.push((p.clone(), d));
            if let Some(c) = &p.children
                && open.contains(&p.pid)
            {
                walk(c, d + 1, open, out);
            }
        }
    }
    let mut out = Vec::new();
    walk(processes, 0, open, &mut out);
    out
}

/// Every branch's pid (the initial open set).
pub fn branch_pids(processes: &[Process]) -> HashSet<u32> {
    fn walk(ps: &[Process], s: &mut HashSet<u32>) {
        for p in ps {
            if let Some(c) = &p.children {
                s.insert(p.pid);
                walk(c, s);
            }
        }
    }
    let mut s = HashSet::new();
    walk(processes, &mut s);
    s
}

/// `mem >= 1024 ? (mem / 1024).toFixed(1) + ' GB' : mem + ' MB'`.
pub fn format_mem(mem: f32) -> String {
    if mem >= 1024. {
        format!("{:.1} GB", mem / 1024.)
    } else {
        format!("{} MB", fmt_num(mem))
    }
}

/// A number as JavaScript prints it (no trailing `.0`).
fn fmt_num(n: f32) -> String {
    if n.fract() == 0. {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}

/// The bar's fill fraction and ink: `min(100, v / max × 100)%`,
/// `.g-ptree-bar i { background: var(--charts-1) }`, `.g-hot { background:
/// var(--support-error) }` past 70%.
pub fn bar_look(v: f32, max: f32) -> (f32, Hsla) {
    let f = v / max;
    (
        f.clamp(0., 1.),
        color(if f > 0.7 {
            ColorRole::SupportError
        } else {
            ColorRole::Charts1
        }),
    )
}

/// `.g-ptree-head, .g-ptree-row { grid-template-columns: minmax(0, 1fr) 4rem
/// 7.5rem 8.5rem 2.5rem }`: the fixed columns in px.
pub const COLUMNS: [f32; 4] = [64., 120., 136., 40.];

/// A toggle's name for (open, process name).
pub type ToggleText = Rc<dyn Fn(bool, &str) -> SharedString>;

/// The tree's user-facing strings (the reference's English defaults).
#[derive(Clone)]
pub struct ProcessTreeTexts {
    /// The headers: Process, PID, CPU, Memory, and the stop column's `Stop`.
    pub headers: [SharedString; 5],
    /// A toggle's name: `"Expand {name}"` / `"Collapse {name}"`.
    pub toggle: ToggleText,
    /// A stop button's name: `"Stop {name}"`.
    pub stop: Rc<dyn Fn(&str) -> SharedString>,
}

impl Default for ProcessTreeTexts {
    fn default() -> Self {
        Self {
            headers: [
                "Process".into(),
                "PID".into(),
                "CPU".into(),
                "Memory".into(),
                "Stop".into(),
            ],
            toggle: Rc::new(|open, n| {
                format!("{} {n}", if open { "Collapse" } else { "Expand" }).into()
            }),
            stop: Rc::new(|n| format!("Stop {n}").into()),
        }
    }
}

type StopHandler = Rc<dyn Fn(u32, &mut Window, &mut App)>;

/// The process tree. See the module docs.
#[derive(IntoElement)]
pub struct ProcessTree {
    id: ElementId,
    processes: Vec<Process>,
    texts: ProcessTreeTexts,
    on_stop: Option<StopHandler>,
}

impl ProcessTree {
    /// A tree of `processes` (`processes`).
    pub fn new(id: impl Into<ElementId>, processes: impl IntoIterator<Item = Process>) -> Self {
        Self {
            id: id.into(),
            processes: processes.into_iter().collect(),
            texts: ProcessTreeTexts::default(),
            on_stop: None,
        }
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: ProcessTreeTexts) -> Self {
        self.texts = texts;
        self
    }

    /// A Stop button was pressed, with its pid (the reference leaves it
    /// unwired; confirm before stopping).
    pub fn on_stop(mut self, f: impl Fn(u32, &mut Window, &mut App) + 'static) -> Self {
        self.on_stop = Some(Rc::new(f));
        self
    }
}

/// A grid row's five cells: the flexible name column, then the fixed ones.
fn grid(cells: [AnyElement; 5]) -> Div {
    let [a, b, c, d, e] = cells;
    let fixed = |el: AnyElement, w: f32| div().flex_none().w(rem(w)).min_w_0().child(el);
    div()
        .flex()
        .items_center()
        .w_full()
        .h(crate::density::current().row())
        .child(div().flex_1().min_w_0().child(a))
        .child(fixed(b, COLUMNS[0]))
        .child(fixed(c, COLUMNS[1]))
        .child(fixed(d, COLUMNS[2]))
        .child(fixed(e, COLUMNS[3]))
}

/// `.g-ptree-head > span, .g-ptree-row > span { padding: 0 var(--spacing-04) }`.
fn cell() -> Div {
    div()
        .px(tokens::SPACING_04)
        .overflow_hidden()
        .whitespace_nowrap()
}

/// A figure with its bar (`.g-ptree-row .g-tabnum { display: flex;
/// align-items: center; gap: var(--spacing-03); font-size: 0.75rem }`).
fn metric(v: f32, max: f32, label: String) -> AnyElement {
    let (f, ink) = bar_look(v, max);
    sans(cell(), 12., 400)
        .flex()
        .items_center()
        .gap(tokens::SPACING_03)
        .child(
            // `.g-ptree-bar { width: 2.5rem; height: 4px; background:
            // var(--layer-accent-01); flex: none }`.
            div()
                .flex_none()
                .w(rem(40.))
                .h(px(4.))
                .bg(color(ColorRole::LayerAccent01))
                .child(div().h_full().w(relative(f)).bg(ink)),
        )
        .child(label)
        .into_any_element()
}

impl RenderOnce for ProcessTree {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let seed = branch_pids(&self.processes);
        let open = window.use_keyed_state((id.clone(), "open"), cx, |_, _| seed);
        let opened = open.read(cx).clone();
        let rows = visible_rows(&self.processes, &opened);
        let texts = self.texts.clone();

        let header = {
            let h = |t: &SharedString| cell().child(t.clone()).into_any_element();
            // `.g-ptree-head { background: var(--layer-accent-01); font-weight:
            // 600; font-size: 0.75rem }`; the last header is centered with no
            // padding (`::before { content: 'Stop' }`).
            sans(
                grid([
                    h(&texts.headers[0]),
                    h(&texts.headers[1]),
                    h(&texts.headers[2]),
                    h(&texts.headers[3]),
                    div()
                        .flex()
                        .justify_center()
                        .child(texts.headers[4].clone())
                        .into_any_element(),
                ]),
                12.,
                600,
            )
            .bg(color(ColorRole::LayerAccent01))
            .text_color(color(ColorRole::TextPrimary))
        };

        let body = rows.into_iter().map(|(p, d)| {
            let pid = p.pid;
            let hover = window.use_keyed_state(
                (id.clone(), SharedString::from(format!("hover-{pid}"))),
                cx,
                |_, _| false,
            );
            let hovered = *hover.read(cx);
            let is_open = opened.contains(&pid);
            let toggle = if p.children.is_some() {
                let chevron = motion::animate(
                    (id.clone(), SharedString::from(format!("chev-{pid}"))),
                    if is_open {
                        std::f32::consts::FRAC_PI_2
                    } else {
                        0.
                    },
                    Motion::TOGGLE,
                    window,
                    cx,
                );
                let open = open.clone();
                let focus = keyed_focus(
                    (id.clone(), SharedString::from(format!("tog-focus-{pid}"))),
                    window,
                    cx,
                );
                // `.g-ttable-tog:focus-visible { outline: 2px solid var(--focus);
                // outline-offset: -2px }`.
                let ring = focus_visible(&focus, window);
                // `.g-ttable-tog { width/height: 1.25rem }` with the 14px
                // `.t3-chev` (`icon-secondary`, rotate 90° open).
                gpui_kit::base::Button::new((id.clone(), SharedString::from(format!("tog-{pid}"))))
                    .track_focus(&focus)
                    .accessibility_label((texts.toggle)(is_open, &p.name))
                    .relative()
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .size(rem(20.))
                    .cursor(CursorStyle::PointingHand)
                    .child(
                        Icon::carbon(CarbonIcon::ChevronRight)
                            .size_length(px(14.))
                            .color(color(ColorRole::IconSecondary))
                            .rotate(gpui_kit::radians(chevron)),
                    )
                    .when(ring, |b| b.child(focus_ring()))
                    .on_click(move |_, _, cx| {
                        open.update(cx, |s, cx| {
                            if !s.remove(&pid) {
                                s.insert(pid);
                            }
                            cx.notify();
                        })
                    })
                    .into_any_element()
            } else {
                // `.g-trow-sp { width: 14px }`.
                div().flex_none().w(px(14.)).into_any_element()
            };
            let mark = match &p.kind {
                Some(k) => ProviderIcon::new(k.clone())
                    .id((id.clone(), SharedString::from(format!("mark-{pid}"))))
                    .size(IdentitySize::Xs)
                    .into_any_element(),
                None => Icon::carbon(CarbonIcon::Terminal)
                    .size_length(px(14.))
                    .color(color(ColorRole::IconSecondary))
                    .into_any_element(),
            };
            // `.g-ptree-name { display: flex; align-items: center; gap:
            // var(--spacing-02) }`, `paddingLeft: 8 + d * 16`.
            let name = div()
                .flex()
                .items_center()
                .gap(tokens::SPACING_02)
                .pl(px(8. + d as f32 * 16.))
                .pr(tokens::SPACING_04)
                .overflow_hidden()
                .whitespace_nowrap()
                .child(toggle)
                .child(div().flex_none().child(mark))
                .child(mono(div(), 12., 400).child(p.name.clone()));
            let stop = {
                let f = self.on_stop.clone();
                Button::new(
                    (id.clone(), SharedString::from(format!("stop-{pid}"))),
                    (texts.stop)(&p.name),
                )
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .icon(CarbonIcon::StopFilled)
                // `.g-ptree-row .cds--btn--ghost svg { fill: var(--icon-secondary) }`.
                .icon_color(color(ColorRole::IconSecondary))
                .icon_only(true)
                .on_click(move |_, w, cx| {
                    if let Some(f) = &f {
                        f(pid, w, cx);
                    }
                })
            };
            // `.g-ptree-row { height: var(--density-row-md); background:
            // var(--layer-01); border-bottom: 1px solid var(--border-subtle-01) }`,
            // `:hover { background: var(--layer-hover-01) }`.
            sans(
                grid([
                    name.into_any_element(),
                    mono(cell(), 12., 400)
                        .text_color(color(ColorRole::TextSecondary))
                        .child(format!("{pid}"))
                        .into_any_element(),
                    metric(p.cpu, 100., format!("{:.1}%", p.cpu)),
                    metric(p.mem, 2048., format_mem(p.mem)),
                    div().flex().justify_center().child(stop).into_any_element(),
                ]),
                14.,
                400,
            )
            .id((id.clone(), SharedString::from(format!("row-{pid}"))))
            .bg(color(if hovered {
                ColorRole::LayerHover01
            } else {
                ColorRole::Layer01
            }))
            .border_b_1()
            .border_color(color(ColorRole::BorderSubtle01))
            .text_color(color(ColorRole::TextPrimary))
            .on_hover(move |h, _, cx| {
                hover.update(cx, |v, cx| {
                    *v = *h;
                    cx.notify();
                })
            })
        });

        div()
            .flex()
            .flex_col()
            .w_full()
            .child(header)
            .children(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CarbonTheme;

    fn sample() -> Vec<Process> {
        vec![
            Process::new(4120, "ghost-server", 3.2, 180.).children([
                Process::new(4188, "claude", 42.5, 610.)
                    .kind("Claude")
                    .children([Process::new(4301, "pnpm test", 88.1, 1540.)]),
                Process::new(4190, "codex", 0.4, 220.).kind("Codex"),
            ]),
            Process::new(5011, "cloudflared", 0.8, 38.),
        ]
    }

    #[test]
    fn branches_start_open_and_collapse() {
        let open = branch_pids(&sample());
        assert_eq!(open, HashSet::from([4120, 4188]));
        let rows = visible_rows(&sample(), &open);
        let flat: Vec<_> = rows.iter().map(|(p, d)| (p.pid, *d)).collect();
        assert_eq!(
            flat,
            vec![(4120, 0), (4188, 1), (4301, 2), (4190, 1), (5011, 0)]
        );
        let rows = visible_rows(&sample(), &HashSet::from([4120]));
        assert_eq!(rows.len(), 4);
    }

    #[test]
    fn memory_and_bars_follow_the_reference() {
        crate::theme::tests_set(CarbonTheme::White);
        assert_eq!(format_mem(610.), "610 MB");
        assert_eq!(format_mem(1540.), "1.5 GB");
        assert_eq!(bar_look(88.1, 100.).1, color(ColorRole::SupportError));
        assert_eq!(bar_look(42.5, 100.).1, color(ColorRole::Charts1));
        // 1.4 GB of 2048 MB is the 70% line.
        assert_eq!(bar_look(1540., 2048.).1, color(ColorRole::SupportError));
        assert_eq!(bar_look(3000., 2048.).0, 1.);
        assert_eq!(COLUMNS, [64., 120., 136., 40.]);
    }
}
