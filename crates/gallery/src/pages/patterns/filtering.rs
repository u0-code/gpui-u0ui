//! Carbon · Patterns › PatternFiltering: filters above the data (instant,
//! applied filters as removable tags with Clear all) and a filter panel at
//! the data's start (batch, Apply filters, the applied count). Live: the
//! filters really filter a thread list, instantly or on Apply.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, Entity, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, component::input::InputState, div, prelude::FluentBuilder as _,
    px, rems,
};
use u0ui::components::form::{self, FieldSize};
use u0ui::components::{Badge, BadgeSize, BadgeVariant, Checkbox, Combobox, Select};
use u0ui::patterns::{Caption, DataTableSkeleton, PatternGrid, PatternRow};
use u0ui::prelude::*;

const MODES: [&str; 2] = ["horizontal · instant", "vertical · batch"];

const CAPTIONS: [&str; 2] = [
    "横向：筛选放在数据上方；已应用的筛选用可移除的标签，并有 Clear all",
    "纵向：左侧筛选面板，多个类别；批量模式用 “Apply filters”，关闭时显示已应用数量",
];

const PROJECTS: [&str; 3] = ["carbon-ui", "api-gateway", "docs"];
const AGENTS: [&str; 3] = ["Any agent", "Claude", "Codex"];
const STATUSES: [&str; 4] = ["Any status", "Working", "Needs you", "Done"];
const STATUS: [&str; 3] = ["Working", "Needs you", "Done"];
/// `defaultChecked` of the panel's boxes.
const STATUS_DEFAULT: [bool; 3] = [true, true, false];

/// The live data: (thread, project, agent, status).
const THREADS: [(&str, &str, &str, &str); 6] = [
    ("Dark tokens", "carbon-ui", "Claude", "Working"),
    ("Gallery thumbnails", "carbon-ui", "Codex", "Done"),
    (
        "Fix the login redirect",
        "api-gateway",
        "Codex",
        "Needs you",
    ),
    ("Rate limits", "api-gateway", "Claude", "Working"),
    ("Write the setup guide", "docs", "Claude", "Done"),
    ("Focus ring audit", "carbon-ui", "Claude", "Needs you"),
];

/// Build the page.
pub fn build(window: &mut Window, cx: &mut App) -> AnyView {
    let project = cx.new(|cx| InputState::new(window, cx));
    let ref_project = cx.new(|cx| InputState::new(window, cx));
    cx.new(|_| Page {
        project,
        ref_project,
        mode: 0,
        projects: vec!["carbon-ui".into()],
        agent: Some("Claude".into()),
        status: None,
        draft: STATUS_DEFAULT,
        applied: STATUS_DEFAULT,
        size: 1,
    })
    .into()
}

struct Page {
    project: Entity<InputState>,
    ref_project: Entity<InputState>,
    mode: usize,
    projects: Vec<SharedString>,
    agent: Option<SharedString>,
    status: Option<SharedString>,
    /// The panel's boxes as edited.
    draft: [bool; 3],
    /// The panel's boxes as last applied.
    applied: [bool; 3],
    size: usize,
}

impl Page {
    /// The threads the active mode's filters keep.
    fn results(&self) -> Vec<(&'static str, &'static str, &'static str, &'static str)> {
        THREADS
            .into_iter()
            .filter(|(_, project, agent, status)| {
                if self.mode == 0 {
                    (self.projects.is_empty() || self.projects.iter().any(|p| p == project))
                        && self.agent.as_ref().is_none_or(|a| a == agent)
                        && self.status.as_ref().is_none_or(|s| s == status)
                } else {
                    let on: Vec<&str> = STATUS
                        .iter()
                        .zip(self.applied)
                        .filter_map(|(s, on)| on.then_some(*s))
                        .collect();
                    on.is_empty() || on.contains(status)
                }
            })
            .collect()
    }
}

fn id(prefix: &str, name: &str) -> SharedString {
    format!("{prefix}-{name}").into()
}

/// "Any …" is no filter.
fn pick(value: Option<&SharedString>) -> Option<SharedString> {
    value.filter(|v| !v.starts_with("Any ")).cloned()
}

/// The horizontal bar: Project (multi), Agent, Status, 8px apart, bottoms
/// aligned, wrapping; then the applied tags and Clear all 12px below.
fn horizontal(
    p: &str,
    page: &Page,
    input: &Entity<InputState>,
    size: FieldSize,
    live: Option<&Entity<Page>>,
) -> AnyElement {
    let mut project = Combobox::new(id(p, "project"), input)
        .label("Project")
        .placeholder("Project")
        // `K.MultiSelect label="Project"`: the field text is a label in
        // `text-primary`, not placeholder gray.
        .placeholder_primary(true)
        .items(PROJECTS)
        .multiple(true)
        .size(size)
        .value(page.projects.clone());
    let mut agent = Select::new(id(p, "agent"))
        .label("Agent")
        .placeholder("Any agent")
        .items(AGENTS)
        .size(size)
        .value(Some(page.agent.clone().unwrap_or("Any agent".into())));
    let mut status = Select::new(id(p, "status"))
        .label("Status")
        .placeholder("Any status")
        .items(STATUSES)
        .size(size)
        .value(page.status.clone());
    if let Some(e) = live {
        let (e1, e2, e3) = (e.clone(), e.clone(), e.clone());
        project = project.on_value_change(move |v, _, cx| {
            let v = v.to_vec();
            e1.update(cx, |this, cx| {
                this.projects = v;
                cx.notify();
            })
        });
        agent = agent.on_value_change(move |v, _, cx| {
            let v = pick(v);
            e2.update(cx, |this, cx| {
                this.agent = v;
                cx.notify();
            })
        });
        status = status.on_value_change(move |v, _, cx| {
            let v = pick(v);
            e3.update(cx, |this, cx| {
                this.status = v;
                cx.notify();
            })
        });
    }
    let applied: Vec<SharedString> = page
        .projects
        .iter()
        .cloned()
        .chain(page.agent.clone())
        .chain(page.status.clone())
        .collect();
    let tags = applied.iter().enumerate().map(|(i, label)| {
        let tag = Badge::new(label.clone())
            .id(id(p, &format!("tag{i}")))
            .variant(BadgeVariant::HighContrast)
            .dismiss_label("Remove filter");
        match live {
            Some(e) => {
                let (e, label) = (e.clone(), label.clone());
                tag.on_dismiss(move |_, _, cx| {
                    let label = label.clone();
                    e.update(cx, |this, cx| {
                        this.projects.retain(|p| *p != label);
                        if this.agent.as_ref() == Some(&label) {
                            this.agent = None;
                        }
                        if this.status.as_ref() == Some(&label) {
                            this.status = None;
                        }
                        cx.notify();
                    })
                })
            }
            None => tag.on_dismiss(|_, _, _| {}),
        }
        .into_any_element()
    });
    let mut clear = Button::new(id(p, "clear"), "Clear all").variant(ButtonVariant::Link);
    if let Some(e) = live {
        let e = e.clone();
        clear = clear.on_click(move |_, _, cx| {
            e.update(cx, |this, cx| {
                this.projects.clear();
                this.agent = None;
                this.status = None;
                cx.notify();
            })
        });
    }
    div()
        .flex()
        .flex_col()
        .child(
            // `display: flex; gap: 8; align-items: flex-end; flex-wrap: wrap`,
            // the fields 200 / 180 / 180px wide.
            div()
                .flex()
                .flex_wrap()
                .items_end()
                .gap(tokens::SPACING_03)
                .child(div().w(px(200.)).child(project))
                .child(div().w(px(180.)).child(agent))
                .child(div().w(px(180.)).child(status)),
        )
        .when(!applied.is_empty(), |d| {
            d.child(
                // `display: flex; gap: 8; align-items: center; marginTop: 12`.
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap(tokens::SPACING_03)
                    .mt(tokens::SPACING_04)
                    .children(tags)
                    .child(clear),
            )
        })
        .into_any_element()
}

/// The vertical panel (220px, `layer-01`, 16px padding and gaps, a
/// `border-subtle-01` end rule) beside the data, 230px tall.
fn vertical(p: &str, page: &Page, data: AnyElement, live: Option<&Entity<Page>>) -> AnyElement {
    let count = page.applied.iter().filter(|on| **on).count();
    let count_label: SharedString = if live.is_some() {
        count.to_string().into()
    } else {
        // The reference shows a fixed "3".
        "3".into()
    };
    let boxes = STATUS.iter().enumerate().map(|(i, label)| {
        let b = Checkbox::new(id(p, &format!("v{i}"))).label(*label);
        let b = match live {
            Some(e) => {
                let e = e.clone();
                b.checked(page.draft[i]).on_checked_change(move |v, _, cx| {
                    e.update(cx, |this, cx| {
                        this.draft[i] = v;
                        cx.notify();
                    })
                })
            }
            None => b.default_checked(STATUS_DEFAULT[i]),
        };
        // The checkbox brings `.cds--checkbox-wrapper`'s 6px bottom margin;
        // `.cds--label + .cds--form-item.cds--checkbox-wrapper {
        // margin-block-start: -0.125rem }` tucks the first one 2px under the
        // legend, as in Disclosures.
        div().when(i == 0, |d| d.mt(px(-2.))).child(b)
    });
    // `style: {flex: 1}`: a zero basis floored at each button's padding,
    // so Reset 94 / Apply 93 in the 187px panel; "Apply filters" wraps and
    // both buttons grow to two lines (50px).
    let mut reset = Button::new(id(p, "reset"), "Reset")
        .variant(ButtonVariant::Secondary)
        .size(ButtonSize::Sm)
        .flex_share(1.)
        .wrap_label(true);
    let mut apply = Button::new(id(p, "apply"), "Apply filters")
        .variant(ButtonVariant::Primary)
        .size(ButtonSize::Sm)
        .flex_share(1.)
        .wrap_label(true)
        .disabled(live.is_some() && page.draft == page.applied);
    if let Some(e) = live {
        let (r, a) = (e.clone(), e.clone());
        reset = reset.on_click(move |_, _, cx| {
            r.update(cx, |this, cx| {
                this.draft = STATUS_DEFAULT;
                this.applied = STATUS_DEFAULT;
                cx.notify();
            })
        });
        apply = apply.on_click(move |_, _, cx| {
            a.update(cx, |this, cx| {
                this.applied = this.draft;
                cx.notify();
            })
        });
    }
    div()
        .flex()
        .h(px(230.))
        .child(
            div()
                .flex()
                .flex_col()
                .flex_none()
                .w(px(220.))
                .gap(tokens::SPACING_05)
                // `display: grid; align-content: start` in a 230px box: the
                // rows keep their heights, so the 232px content (the first
                // box's −2px included) overflows the 16px bottom padding by
                // 2px and the buttons end 14px above the panel's edge
                // (every row is `flex_none`).
                .p(tokens::SPACING_05)
                .bg(color(ColorRole::Layer01))
                .border_r_1()
                .border_color(color(ColorRole::BorderSubtle01))
                .child(
                    div()
                        .flex_none()
                        .flex()
                        .justify_between()
                        .items_center()
                        .child(
                            div()
                                .type_style(TypeStyle::HeadingCompact01)
                                .text_color(color(ColorRole::TextPrimary))
                                .child("Filters"),
                        )
                        .child(
                            Badge::new(count_label)
                                .variant(BadgeVariant::HighContrast)
                                .size(BadgeSize::Sm),
                        ),
                )
                .child(
                    div()
                        .flex_none()
                        .flex()
                        .flex_col()
                        .child(form::label("Status", false))
                        .children(boxes),
                )
                .child(
                    // The row stretches both buttons to the taller.
                    div().flex_none().flex().child(reset).child(apply),
                ),
        )
        .child(div().flex_1().min_w_0().overflow_hidden().child(data))
        .into_any_element()
}

/// The live data: the kept threads as rows, or a note when none are left.
fn results(page: &Page) -> AnyElement {
    let rows = page.results();
    div()
        .flex()
        .flex_col()
        .w_full()
        .bg(color(ColorRole::Layer01))
        .px(tokens::SPACING_05)
        .when(rows.is_empty(), |d| {
            d.py(tokens::SPACING_05)
                .text_color(color(ColorRole::TextSecondary))
                .type_style(TypeStyle::BodyCompact01)
                .child("No threads match these filters")
        })
        .children(rows.into_iter().map(|(name, project, agent, status)| {
            // Title at the start, meta at the end; when the column is too
            // narrow for both (the batch panel at 940px), the meta wraps
            // under the title instead of drawing over it.
            PatternRow::new().child(
                div()
                    .flex()
                    .flex_wrap()
                    .justify_between()
                    .gap_x(tokens::SPACING_05)
                    .flex_1()
                    .min_w_0()
                    .child(
                        div()
                            .min_w_0()
                            .overflow_hidden()
                            .text_ellipsis()
                            .whitespace_nowrap()
                            .child(name),
                    )
                    .child(
                        div()
                            .min_w_0()
                            .overflow_hidden()
                            .text_ellipsis()
                            .whitespace_nowrap()
                            .text_color(color(ColorRole::TextSecondary))
                            .child(format!("{project} · {agent} · {status}")),
                    ),
            )
        }))
        .into_any_element()
}

fn captioned(i: usize, body: AnyElement) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .min_w_0()
        .child(Caption::new(CAPTIONS[i]))
        .child(body)
        .into_any_element()
}

/// The overview tile: the data-table skeleton the panel filters.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .child(DataTableSkeleton::new(2, 3))
        .into_any_element()
}

/// The reference state of the examples.
fn reference(page: &Page) -> Page {
    Page {
        project: page.ref_project.clone(),
        ref_project: page.ref_project.clone(),
        mode: 0,
        projects: vec!["carbon-ui".into()],
        agent: Some("Claude".into()),
        status: None,
        draft: STATUS_DEFAULT,
        applied: STATUS_DEFAULT,
        size: 1,
    }
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let size = FieldSize::ALL[self.size];
        let live = if self.mode == 0 {
            div()
                .flex()
                .flex_col()
                .gap(tokens::SPACING_05)
                .child(captioned(
                    0,
                    horizontal("filt-live", self, &self.project, size, Some(&entity)),
                ))
                .child(results(self))
                .into_any_element()
        } else {
            captioned(1, vertical("filt-live", self, results(self), Some(&entity)))
        };
        let live = div().w_full().child(live);

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::field(
                "placement",
                controls::choices(
                    "filt-mode",
                    &MODES,
                    self.mode,
                    controls::setter(&entity, |this: &mut Page, i| this.mode = i),
                ),
            ))
            .child(controls::field(
                "field size",
                controls::choices(
                    "filt-size",
                    &FieldSize::ALL.map(FieldSize::name),
                    self.size,
                    controls::setter(&entity, |this: &mut Page, i| this.size = i),
                ),
            ));

        let r = reference(self);
        let grid = PatternGrid::tracks(vec![1.])
            .cell(captioned(
                0,
                horizontal("filt-ref0", &r, &self.ref_project, FieldSize::Md, None),
            ))
            .cell(captioned(
                1,
                vertical(
                    "filt-ref1",
                    &r,
                    DataTableSkeleton::new(3, 4).into_any_element(),
                    None,
                ),
            ));

        Body::new()
            .playground(live)
            .hint("Horizontal filters apply at once; the panel applies on Apply filters")
            .config(config)
            .example(controls::block("Reference preview", grid))
    }
}
