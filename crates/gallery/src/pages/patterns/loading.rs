//! Carbon · Patterns › PatternLoading: skeletons for containers and data,
//! inline loading for one action, a full-area loading over a scrim, and
//! "Load more" at the end of a long list
//! (`components/PatternLoading/preview.html`).
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, Entity, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, component::input::InputState, div, prelude::FluentBuilder as _,
    rems,
};
use std::time::Duration;
use u0ui::components::{
    FieldSize, Input, Skeleton, SkeletonLength, Spinner, SpinnerSize, SpinnerStatus,
};
use u0ui::patterns::{Caption, PatternGrid, PatternRow};
use u0ui::prelude::*;

/// Build the page.
pub fn build(window: &mut Window, cx: &mut App) -> AnyView {
    let email = crate::stand_in::input_state("ben@example.com", window, cx);
    let ref_email = crate::stand_in::input_state("ben@example.com", window, cx);
    cx.new(|_| LoadingPage {
        email,
        ref_email,
        stage: 0,
        invite: 1,
        busy: true,
        rows: ROWS.len(),
    })
    .into()
}

struct LoadingPage {
    email: Entity<InputState>,
    ref_email: Entity<InputState>,
    /// Index into [`STAGES`].
    stage: usize,
    /// Index into [`INVITE`] (the live invite's status).
    invite: usize,
    /// The full-area loading shows.
    busy: bool,
    /// Rows shown in the long list.
    rows: usize,
}

const LIST: [&str; 2] = ["more to load", "all loaded"];
const STAGES: [&str; 3] = ["skeleton", "text first", "loaded"];
const INVITE: [&str; 4] = ["idle", "active", "finished", "error"];
const ROWS: [&str; 3] = [
    "Add dark theme tokens",
    "Fix flaky test",
    "Write migration guide",
];
const MORE: [&str; 6] = [
    "Rename the sidebar groups",
    "Cache the model list",
    "Retry pushes on timeout",
    "Profile the diff view",
    "Pair a second Mac",
    "Trim the changelog",
];

/// `{background: var(--layer-01), padding: 16}`.
fn tile() -> gpui_kit::Div {
    div()
        .flex()
        .flex_col()
        .bg(color(ColorRole::Layer01))
        .p(tokens::SPACING_05)
}

/// One dashboard tile at a loading `stage`: all skeleton (0), text first
/// with the image still a skeleton (1), or loaded (2).
fn dashboard_tile(i: usize, stage: usize) -> AnyElement {
    let (title, body) = [
        ("Threads", "12 running on 3 servers."),
        ("Checkpoints", "48 saved today."),
        ("Usage", "62% of this month's tokens."),
    ][i];
    let heading = if stage == 0 {
        // `SkeletonText heading width="60%"`.
        Skeleton::text()
            .heading(true)
            .lines(1)
            .width(SkeletonLength::Percent(60.))
            .into_any_element()
    } else {
        div()
            .type_style(TypeStyle::Heading02)
            .text_color(color(ColorRole::TextPrimary))
            .child(title)
            .into_any_element()
    };
    let text = if stage == 0 {
        // `SkeletonText paragraph lineCount={2}`.
        Skeleton::text().lines(2).into_any_element()
    } else {
        div()
            .py(tokens::SPACING_03)
            .type_style(TypeStyle::BodyCompact01)
            .text_color(color(ColorRole::TextSecondary))
            .child(body)
            .into_any_element()
    };
    let art = if stage < 2 {
        // `SkeletonPlaceholder style={{width:'100%', height:48}}`.
        Skeleton::block()
            .width(SkeletonLength::Percent(100.))
            .height(SkeletonLength::Px(48.))
            .into_any_element()
    } else {
        div()
            .h(rems(3.))
            .bg(color(ColorRole::LayerAccent01))
            .into_any_element()
    };
    tile()
        .child(heading)
        .child(text)
        .child(art)
        .into_any_element()
}

/// `display: grid; grid-template-columns: repeat(3, 1fr); gap: 16`.
fn skeletons(stage: usize) -> impl IntoElement {
    let mut grid = PatternGrid::new(3);
    for i in 0..3 {
        grid = grid.cell(dashboard_tile(i, stage));
    }
    div().child(grid)
}

/// The invite row: the email field and the inline loading states.
fn invite(
    id: &'static str,
    email: &Entity<InputState>,
    statuses: Vec<AnyElement>,
) -> impl IntoElement {
    // `display: grid; gap: 12`.
    tile()
        .gap(tokens::SPACING_04)
        .child(
            Input::new(id, email)
                .label("Email")
                .size(FieldSize::Md)
                .layer(LayerLevel::Two),
        )
        // `display: flex; gap: 16; align-items: center`.
        .child(
            div()
                .flex()
                .flex_row()
                .flex_wrap()
                .items_center()
                .gap(tokens::SPACING_05)
                .children(statuses),
        )
}

fn inline(id: &'static str, status: SpinnerStatus, text: &'static str) -> AnyElement {
    Spinner::new(id)
        .size(SpinnerSize::Inline)
        .status(status)
        .description(text)
        .into_any_element()
}

/// `position: relative; height: 150; overflow: hidden` over a paragraph
/// skeleton; the scrim (`position: absolute; inset: 0; background:
/// var(--scrim)`) centers the large spinner.
fn full_area(busy: bool) -> impl IntoElement {
    div()
        .relative()
        .h(rems(150. / 16.))
        .overflow_hidden()
        .bg(color(ColorRole::Layer01))
        .child(div().p(tokens::SPACING_05).child(Skeleton::text().lines(3)))
        .when(busy, |d| {
            d.child(
                div()
                    .absolute()
                    .inset_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(color(ColorRole::Scrim))
                    // `Loading withOverlay={false} description="Restoring checkpoint"`:
                    // the description is the spinner's accessible name.
                    .child(
                        Spinner::new("ld-full")
                            .size(SpinnerSize::Md)
                            .label("Restoring checkpoint"),
                    ),
            )
        })
}

/// The list and its "Load 20 more" ghost button.
fn long_list(rows: Vec<&'static str>, more: Button) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .bg(color(ColorRole::Layer01))
        .children(rows.into_iter().map(|t| PatternRow::new().child(t)))
        .child(div().flex().child(more))
}

fn more_button(id: &'static str, label: SharedString) -> Button {
    // `Button kind="ghost" size="sm" renderIcon={ChevronDown}`.
    Button::new(id, label)
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::Sm)
        .icon(CarbonIcon::ChevronDown)
}

fn layout(
    stage: usize,
    invite_box: impl IntoElement,
    busy: bool,
    list: impl IntoElement,
) -> impl IntoElement {
    PatternGrid::new(2)
        .align_start(true)
        .full(
            div()
                .child(Caption::new(
                    "骨架屏：只用于容器（Tile、列表）和数据（表格、卡片）；仪表板先出容器和文字，再出图片和交互",
                ))
                .child(skeletons(stage)),
        )
        .cell(
            div()
                .child(Caption::new("Inline loading：单个组件的操作（邀请成员）"))
                .child(invite_box),
        )
        .cell(
            div()
                .child(Caption::new("全屏加载：整页处理中，内容上加遮罩"))
                .child(full_area(busy)),
        )
        .full(
            div()
                .child(Caption::new("长列表末尾：“Load more”"))
                .child(list),
        )
}

/// `components/PatternLoading/preview.html`.
pub fn reference(email: &Entity<InputState>) -> impl IntoElement {
    layout(
        0,
        invite(
            "ld-ref-email",
            email,
            vec![
                inline("ld-ref-active", SpinnerStatus::Active, "Inviting…"),
                inline("ld-ref-done", SpinnerStatus::Finished, "Invited"),
            ],
        ),
        true,
        long_list(
            ROWS.to_vec(),
            more_button("ld-ref-more", "Load 20 more".into()),
        ),
    )
}

/// The overview tile: one tile's skeleton and an inline loading.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .flex()
        .flex_col()
        .gap(tokens::SPACING_03)
        .child(dashboard_tile(0, 0))
        .child(inline("ld-thumb", SpinnerStatus::Active, "Inviting…"))
        .into_any_element()
}

impl LoadingPage {
    /// Run the invite: active for 1.5s, then finished.
    fn start_invite(entity: &Entity<Self>, window: &mut Window, cx: &mut App) {
        entity.update(cx, |this, cx| {
            this.invite = 1;
            cx.notify();
        });
        let entity = entity.clone();
        window
            .spawn(cx, async move |cx| {
                cx.background_executor()
                    .timer(Duration::from_millis(1500))
                    .await;
                entity.update(cx, |this, cx| {
                    if this.invite == 1 {
                        this.invite = 2;
                        cx.notify();
                    }
                });
            })
            .detach();
    }

    /// Show the full-area loading for 2s.
    fn restore(entity: &Entity<Self>, window: &mut Window, cx: &mut App) {
        entity.update(cx, |this, cx| {
            this.busy = true;
            cx.notify();
        });
        let entity = entity.clone();
        window
            .spawn(cx, async move |cx| {
                cx.background_executor().timer(Duration::from_secs(2)).await;
                entity.update(cx, |this, cx| {
                    this.busy = false;
                    cx.notify();
                });
            })
            .detach();
    }
}

impl Render for LoadingPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let status = match self.invite {
            1 => Some((SpinnerStatus::Active, "Inviting…")),
            2 => Some((SpinnerStatus::Finished, "Invited")),
            3 => Some((SpinnerStatus::Error, "Invite failed")),
            _ => None,
        };
        let mut statuses = vec![{
            let entity = entity.clone();
            Button::new("ld-live-invite", "Invite")
                .variant(ButtonVariant::Primary)
                .size(ButtonSize::Md)
                .disabled(self.invite == 1)
                .on_click(move |_, window, cx| Self::start_invite(&entity, window, cx))
                .into_any_element()
        }];
        if let Some((s, text)) = status {
            statuses.push(inline("ld-live-status", s, text));
        }

        let all: Vec<&'static str> = ROWS.iter().chain(MORE.iter()).copied().collect();
        let shown = all[..self.rows.min(all.len())].to_vec();
        let left = all.len() - shown.len();
        let more = {
            let entity = entity.clone();
            more_button("ld-live-more", format!("Load {} more", left.min(3)).into())
                .disabled(left == 0)
                .on_click(move |_, _, cx| {
                    entity.update(cx, |this, cx| {
                        this.rows += 3;
                        cx.notify();
                    })
                })
        };

        let restore = {
            let entity = entity.clone();
            Button::new("ld-live-restore", "Restore checkpoint")
                .variant(ButtonVariant::Tertiary)
                .size(ButtonSize::Sm)
                .disabled(self.busy)
                .on_click(move |_, window, cx| Self::restore(&entity, window, cx))
        };

        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(layout(
                self.stage,
                invite("ld-live-email", &self.email, statuses),
                self.busy,
                long_list(shown, more),
            ))
            .child(div().flex().child(restore));

        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "dashboard stage",
                controls::choices(
                    "ld-stage",
                    &STAGES,
                    self.stage,
                    controls::setter(&entity, |this: &mut Self, i| this.stage = i),
                ),
            ))
            .child(controls::field(
                "inline loading",
                controls::choices(
                    "ld-invite",
                    &INVITE,
                    self.invite,
                    controls::setter(&entity, |this: &mut Self, i| this.invite = i),
                ),
            ))
            .child(controls::toggle(
                "ld-busy",
                "full-area loading",
                self.busy,
                controls::setter(&entity, |this: &mut Self, on| this.busy = on),
            ))
            .child(controls::field(
                "long list",
                controls::choices(
                    "ld-list",
                    &LIST,
                    usize::from(self.rows >= ROWS.len() + MORE.len()),
                    controls::setter(&entity, |this: &mut Self, i: usize| {
                        this.rows = if i == 1 {
                            ROWS.len() + MORE.len()
                        } else {
                            ROWS.len()
                        }
                    }),
                ),
            ));

        Body::new()
            .playground(live)
            .hint("Invite runs the inline loading; Load more appends rows; reduced motion turns spinners into a pulse")
            .config(config)
            .example(controls::block("Reference preview", reference(&self.ref_email)))
    }
}
