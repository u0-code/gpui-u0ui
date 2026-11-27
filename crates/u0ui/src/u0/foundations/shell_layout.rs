//! ShellLayout: sidebar · thread · right column, where the side columns float
//! over the thread when they no longer fit (`ShellLayout` in
//! `src/ghost/foundations.jsx`, the LayoutRules specimen).
//!
//! Like the reference's `@container` queries, it follows its own width, not
//! the window's, measured in rems so the breakpoints hold at every zoom:
//!
//! - wider than 67.5rem (1080px): sidebar (`sidebar-width`), thread (at least
//!   `thread-min-width`) and right column (`right-column-width`) side by side;
//! - at most 67.5rem: the right column floats over the thread's end on
//!   `layer-01` with a `-2px 0 6px shadow`;
//! - at most 45rem (720px): the sidebar floats over its start too
//!   (`2px 0 6px shadow`). Floating columns never squeeze the thread.
use crate::{
    theme::color,
    tokens::{ColorRole, RIGHT_COLUMN_WIDTH, SIDEBAR_WIDTH, THREAD_MIN_WIDTH},
};
use gpui_kit::{
    AnyElement, App, BoxShadow, ElementId, IntoElement, ParentElement, Rems, RenderOnce,
    SharedString, Styled, Window, canvas, div, prelude::FluentBuilder as _, px,
};

/// `@container (max-width: 67.5rem)`: the right column floats.
pub const RIGHT_FLOATS_AT: f32 = 67.5;
/// `@container (max-width: 45rem)`: the sidebar floats too.
pub const SIDEBAR_FLOATS_AT: f32 = 45.;

/// How the columns sit at a container width.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ShellMode {
    /// The sidebar floats over the thread.
    pub sidebar_floats: bool,
    /// The right column floats over the thread.
    pub right_floats: bool,
}

/// The mode for a container `width_rem` wide.
pub fn mode(width_rem: f32) -> ShellMode {
    ShellMode {
        sidebar_floats: width_rem <= SIDEBAR_FLOATS_AT,
        right_floats: width_rem <= RIGHT_FLOATS_AT,
    }
}

fn shadow(x: f32) -> Vec<BoxShadow> {
    vec![BoxShadow {
        color: color(ColorRole::Shadow),
        offset: gpui_kit::point(px(x), px(0.)),
        blur_radius: px(6.),
        spread_radius: px(0.),
        inset: false,
    }]
}

/// The three-column shell. See the module docs.
#[derive(IntoElement)]
pub struct ShellLayout {
    id: ElementId,
    sidebar: Option<AnyElement>,
    thread: Option<AnyElement>,
    right: Option<AnyElement>,
    sidebar_open: bool,
    right_open: bool,
    width: Option<Rems>,
    height: Rems,
    label: Option<SharedString>,
}

impl ShellLayout {
    /// An empty shell (320px high).
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            sidebar: None,
            thread: None,
            right: None,
            sidebar_open: true,
            right_open: true,
            width: None,
            height: gpui_kit::rems(20.),
            label: None,
        }
    }

    /// The sidebar column.
    pub fn sidebar(mut self, sidebar: impl IntoElement) -> Self {
        self.sidebar = Some(sidebar.into_any_element());
        self
    }

    /// The thread column.
    pub fn thread(mut self, thread: impl IntoElement) -> Self {
        self.thread = Some(thread.into_any_element());
        self
    }

    /// The right column.
    pub fn right(mut self, right: impl IntoElement) -> Self {
        self.right = Some(right.into_any_element());
        self
    }

    /// Show the sidebar (default true).
    pub fn sidebar_open(mut self, open: bool) -> Self {
        self.sidebar_open = open;
        self
    }

    /// Show the right column (default true).
    pub fn right_open(mut self, open: bool) -> Self {
        self.right_open = open;
        self
    }

    /// A fixed width (default: the parent's).
    pub fn width(mut self, width: Rems) -> Self {
        self.width = Some(width);
        self
    }

    /// The height (default 20rem).
    pub fn height(mut self, height: Rems) -> Self {
        self.height = height;
        self
    }

    /// An accessible name.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl RenderOnce for ShellLayout {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let width = window.use_keyed_state((self.id.clone(), "width"), cx, |_, _| 0f32);
        // Before the first measure, a fixed width answers directly.
        let measured = *width.read(cx);
        let w_rem = if measured > 0. {
            measured
        } else {
            self.width.map_or(100., |w| w.0)
        };
        let m = mode(w_rem);
        let sidebar = self.sidebar.filter(|_| self.sidebar_open);
        let right = self.right.filter(|_| self.right_open);
        let docked_sidebar = sidebar.is_some() && !m.sidebar_floats;
        let docked_right = right.is_some() && !m.right_floats;
        // `minmax(var(--thread-min-width), 1fr)` until a column floats, then
        // `minmax(0, 1fr)`.
        let thread_min = if m.right_floats || m.sidebar_floats {
            gpui_kit::rems(0.)
        } else {
            gpui_kit::rems(THREAD_MIN_WIDTH.px_value() / crate::tokens::REM_PX)
        };
        let measure = {
            let width = width.clone();
            canvas(
                move |bounds, window, cx| {
                    let rem = f32::from(bounds.size.width) / f32::from(window.rem_size());
                    if (rem - *width.read(cx)).abs() > 0.01 {
                        width.update(cx, |w, cx| {
                            *w = rem;
                            cx.notify();
                        });
                    }
                },
                |_, _, _, _| {},
            )
            .absolute()
            .size_full()
        };
        let sb = sidebar.map(|s| {
            let col = div()
                .flex_none()
                .w(SIDEBAR_WIDTH)
                .h_full()
                .overflow_hidden();
            if docked_sidebar {
                // `.g-shell-sb { background: var(--sidebar-material); border-right: 1px }`.
                col.bg(color(ColorRole::SidebarMaterial))
                    .border_r(px(1.))
                    .border_color(color(ColorRole::BorderSubtle01))
                    .child(s)
            } else {
                col.absolute()
                    .top_0()
                    .left_0()
                    .bottom_0()
                    .bg(color(ColorRole::Layer01))
                    .shadow(shadow(2.))
                    .border_r(px(1.))
                    .border_color(color(ColorRole::BorderSubtle01))
                    .child(s)
            }
        });
        let rc = right.map(|r| {
            let col = div()
                .flex_none()
                .w(RIGHT_COLUMN_WIDTH)
                .h_full()
                .overflow_hidden()
                .bg(color(ColorRole::Layer01))
                .border_l(px(1.))
                .border_color(color(ColorRole::BorderSubtle01));
            if docked_right {
                col.child(r)
            } else {
                col.absolute()
                    .top_0()
                    .right_0()
                    .bottom_0()
                    .shadow(shadow(-2.))
                    .child(r)
            }
        });
        let thread = div()
            .flex_1()
            .min_w(thread_min)
            .h_full()
            .overflow_hidden()
            .children(self.thread);
        // Docked columns go in the row; floating ones paint after the thread
        // (the sidebar, z-index 6, above the right column, 5).
        let (sb_docked, sb_float) = if docked_sidebar {
            (sb, None)
        } else {
            (None, sb)
        };
        let (rc_docked, rc_float) = if docked_right { (rc, None) } else { (None, rc) };
        div()
            .relative()
            .overflow_hidden()
            .bg(color(ColorRole::Background))
            // `.g-shell-wrap { box-shadow: 0 0 0 1px var(--border-subtle-01) }`.
            .shadow(vec![BoxShadow {
                color: color(ColorRole::BorderSubtle01),
                offset: gpui_kit::point(px(0.), px(0.)),
                blur_radius: px(0.),
                spread_radius: px(1.),
                inset: false,
            }])
            .map(|d| match self.width {
                Some(w) => d.w(w),
                None => d.w_full(),
            })
            .h(self.height)
            .child(measure)
            .child(
                div()
                    .flex()
                    .size_full()
                    .children(sb_docked)
                    .child(thread)
                    .children(rc_docked),
            )
            .children(rc_float)
            .children(sb_float)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breakpoints_follow_the_container_queries() {
        // 1120px: three columns; 960 and 800: the right floats; 600: both.
        assert_eq!(mode(70.), ShellMode::default());
        assert_eq!(
            mode(60.),
            ShellMode {
                sidebar_floats: false,
                right_floats: true
            }
        );
        assert_eq!(mode(50.), mode(60.));
        assert_eq!(
            mode(37.5),
            ShellMode {
                sidebar_floats: true,
                right_floats: true
            }
        );
        // sidebar 240 + thread 480 + right 360 = 1080 = 67.5rem.
        assert_eq!(
            SIDEBAR_WIDTH.px_value() + THREAD_MIN_WIDTH.px_value() + RIGHT_COLUMN_WIDTH.px_value(),
            RIGHT_FLOATS_AT * 16.
        );
        assert_eq!(
            SIDEBAR_WIDTH.px_value() + THREAD_MIN_WIDTH.px_value(),
            SIDEBAR_FLOATS_AT * 16.
        );
    }
}
