//! Trace: a column of steps (reasoning, tool calls, sources, a plan) on a
//! vertical rail (`.cu-trace*` in `custom.css`; the vertical Carbon
//! `ProgressIndicator` anatomy). Layout only; the disclosure is u0ui's
//! [`Collapsible`] restyled ([`Collapsible::trace`]).
//!
//! - [`TraceLine`]: `position: relative; display: flex; gap: spacing-04;
//!   padding-bottom: spacing-05`; the rail `left: 7.5px; top: 1.25rem; bottom:
//!   0; width: 1px; background: border-subtle-01`, hidden on the last line.
//! - Status icon (`.cu-trace-icon { width: 1rem; height: 1.25rem; background:
//!   background }`): done `CheckmarkFilled` in `support-success`, error
//!   `ErrorFilled` in `support-error`, pending `CircleDash` in
//!   `icon-secondary`, running the small Carbon `Loading` (1rem).
//! - Body: [`TraceTitle`] `body-01` in `text-primary` (`text-secondary` when
//!   pending), [`TraceDescription`] 12px `text-secondary` 2px below, then any
//!   children (e.g. a [`TraceCollapsible`] with a small code block).
//! - [`TraceCollapsible`]: a [`Collapsible`] whose trigger is a bare 12px
//!   `link-primary` row (chevron in `link-primary`, 4px gap) and whose content
//!   is padded 8px on top only. Its chevron turns in 110ms like Collapsible's.
use super::collapsible::Collapsible;
use super::form::rem;
use super::spinner::{Spinner, SpinnerSize};
use crate::{
    icons::{CarbonIcon, Icon, IconSize},
    theme::color,
    tokens::{ColorRole, SPACING_04, SPACING_05, TypeStyle},
    typography::TypeStyled as _,
};
use gpui_kit::{
    AnyElement, App, ElementId, Hsla, IntoElement, ParentElement, RenderOnce, SharedString,
    StyleRefinement, Styled, Window, div, prelude::FluentBuilder as _, px,
};

/// `index.d.ts` `TraceLine.status`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TraceStatus {
    /// Finished (default).
    #[default]
    Done,
    /// In progress: the small spinner.
    Running,
    /// Failed.
    Error,
    /// Not started.
    Pending,
}

impl TraceStatus {
    /// Every status.
    pub const ALL: [TraceStatus; 4] = [Self::Done, Self::Running, Self::Error, Self::Pending];

    /// The `index.d.ts` name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Done => "done",
            Self::Running => "running",
            Self::Error => "error",
            Self::Pending => "pending",
        }
    }

    /// The icon and its color (`None` for running, which shows the spinner).
    pub fn icon(self) -> Option<(CarbonIcon, Hsla)> {
        match self {
            Self::Done => Some((
                CarbonIcon::CheckmarkFilled,
                color(ColorRole::SupportSuccess),
            )),
            Self::Error => Some((CarbonIcon::ErrorFilled, color(ColorRole::SupportError))),
            Self::Pending => Some((CarbonIcon::CircleDash, color(ColorRole::IconSecondary))),
            Self::Running => None,
        }
    }

    /// The title color (`.cu-trace-line--pending .cu-trace-title { color: text-secondary }`).
    pub fn title_color(self) -> Hsla {
        color(if self == Self::Pending {
            ColorRole::TextSecondary
        } else {
            ColorRole::TextPrimary
        })
    }
}

/// `.cu-trace-rail { left: 7.5px }`.
pub const RAIL_LEFT: f32 = 7.5;
/// `.cu-trace-rail { top: 1.25rem }` and the icon box height, in px.
pub const ICON_HEIGHT: f32 = 20.;
/// `.cu-trace-icon { width: 1rem }` in px.
pub const ICON_WIDTH: f32 = 16.;

/// `TraceTitle`.
pub struct TraceTitle(SharedString);

impl TraceTitle {
    /// A title.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self(text.into())
    }
}

/// `TraceDescription`.
pub struct TraceDescription(SharedString);

impl TraceDescription {
    /// A description.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self(text.into())
    }
}

/// `TraceIcon`: an extra inline icon in the body (`.cu-trace-custom-icon`,
/// unstyled in the reference).
#[derive(IntoElement, Default)]
pub struct TraceIcon {
    children: Vec<AnyElement>,
}

impl TraceIcon {
    /// An empty slot.
    pub fn new() -> Self {
        Self::default()
    }
}

impl ParentElement for TraceIcon {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for TraceIcon {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div().flex().flex_row().children(self.children)
    }
}

/// A [`Collapsible`] styled for a trace (`TraceCollapsible` with its trigger
/// label and content children).
pub struct TraceCollapsible;

impl TraceCollapsible {
    /// A trace disclosure labelled `trigger` (`TraceCollapsibleTrigger`).
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<ElementId>, trigger: impl Into<SharedString>) -> Collapsible {
        Collapsible::new(id, trigger).trace(true)
    }
}

/// One step. See the module docs.
pub struct TraceLine {
    id: ElementId,
    status: TraceStatus,
    title: Option<TraceTitle>,
    description: Option<TraceDescription>,
    children: Vec<AnyElement>,
}

impl TraceLine {
    /// A step keyed `id` (the running spinner keeps its phase under it).
    pub fn new(id: impl Into<ElementId>, status: TraceStatus) -> Self {
        Self {
            id: id.into(),
            status,
            title: None,
            description: None,
            children: Vec::new(),
        }
    }

    /// The title.
    pub fn title(mut self, title: TraceTitle) -> Self {
        self.title = Some(title);
        self
    }

    /// The description.
    pub fn description(mut self, description: TraceDescription) -> Self {
        self.description = Some(description);
        self
    }

    fn render(self, last: bool) -> AnyElement {
        let icon: AnyElement = match self.status.icon() {
            Some((icon, fill)) => Icon::carbon(icon)
                .size(IconSize::S16)
                .color(fill)
                .into_any_element(),
            None => Spinner::new((self.id.clone(), "spinner"))
                .size(SpinnerSize::Sm)
                .into_any_element(),
        };
        div()
            .relative()
            .flex()
            .flex_row()
            .gap(SPACING_04)
            .pb(SPACING_05)
            .when(!last, |this| {
                this.child(
                    div()
                        .absolute()
                        .left(px(RAIL_LEFT))
                        .top(rem(ICON_HEIGHT))
                        .bottom_0()
                        .w(px(1.))
                        .bg(color(ColorRole::BorderSubtle01)),
                )
            })
            .child(
                div()
                    .relative()
                    .flex()
                    .flex_none()
                    .w(rem(ICON_WIDTH))
                    .h(rem(ICON_HEIGHT))
                    .items_center()
                    .bg(color(ColorRole::Background))
                    .child(icon),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .when_some(self.title, |this, title| {
                        this.child(
                            div()
                                .type_style(TypeStyle::Body01)
                                .text_color(self.status.title_color())
                                .child(title.0),
                        )
                    })
                    .when_some(self.description, |this, d| {
                        this.child(
                            div()
                                .mt(px(2.))
                                .type_style(TypeStyle::Label01)
                                .text_color(color(ColorRole::TextSecondary))
                                .child(d.0),
                        )
                    })
                    .children(self.children),
            )
            .into_any_element()
    }
}

impl ParentElement for TraceLine {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

/// The list of steps.
#[derive(IntoElement, Default)]
pub struct Trace {
    style: StyleRefinement,
    lines: Vec<TraceLine>,
}

impl Trace {
    /// An empty trace.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a step.
    pub fn line(mut self, line: TraceLine) -> Self {
        self.lines.push(line);
        self
    }
}

impl Styled for Trace {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for Trace {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let count = self.lines.len();
        let mut list = div().flex().flex_col().children(
            self.lines
                .into_iter()
                .enumerate()
                .map(|(i, line)| line.render(i + 1 == count)),
        );
        gpui_kit::Refineable::refine(list.style(), &self.style);
        list
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    #[test]
    fn status_icons_follow_custom_css() {
        crate::theme::tests_set(CarbonTheme::White);
        assert_eq!(
            TraceStatus::Done.icon(),
            Some((
                CarbonIcon::CheckmarkFilled,
                color(ColorRole::SupportSuccess)
            ))
        );
        assert_eq!(
            TraceStatus::Error.icon(),
            Some((CarbonIcon::ErrorFilled, color(ColorRole::SupportError)))
        );
        assert_eq!(
            TraceStatus::Pending.icon(),
            Some((CarbonIcon::CircleDash, color(ColorRole::IconSecondary)))
        );
        assert_eq!(TraceStatus::Running.icon(), None);
        assert_eq!(
            TraceStatus::Pending.title_color(),
            color(ColorRole::TextSecondary)
        );
        assert_eq!(
            TraceStatus::Done.title_color(),
            color(ColorRole::TextPrimary)
        );
        assert_eq!((RAIL_LEFT, ICON_HEIGHT, ICON_WIDTH), (7.5, 20., 16.));
    }
}
