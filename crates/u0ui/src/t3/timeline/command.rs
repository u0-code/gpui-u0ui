//! [`CommandOutputBlock`] (`.t3-cmd`, "Command output" in `t3.css`): a
//! command and its output. The 40px head (terminal icon, plain mono
//! `$ command` per r3-gal, meta, duration, then a spinner, a `stopped` tag
//! or the exit tag, and the chevron) folds the output; a non-zero exit adds
//! a 3px `support-error` rail.
use super::card;
use crate::components::badge::{Badge, BadgeSize, BadgeVariant};
use crate::t3::base::{Spin, fmt_elapsed};
use crate::t3::press::{self, rem};
use crate::{
    icons::CarbonIcon,
    theme::color,
    tokens::{ColorRole, FontFamilyKey},
    typography::font,
};
use gpui_kit::{
    App, ElementId, InteractiveElement as _, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement as _, Styled, Window, div, prelude::FluentBuilder as _,
};

/// `duration`: seconds (formatted) or text.
#[derive(Clone, Debug, PartialEq)]
pub enum CommandDuration {
    /// Seconds, shown with `fmtElapsed`.
    Seconds(u64),
    /// Preformatted.
    Text(SharedString),
}

/// The exit tag: (label, variant) for `exitCode`.
pub fn exit_tag(code: i32) -> (SharedString, BadgeVariant) {
    if code != 0 {
        (format!("exit {code}").into(), BadgeVariant::Red)
    } else {
        ("exit 0".into(), BadgeVariant::Green)
    }
}

/// A command and its output. See the module docs.
#[derive(IntoElement)]
pub struct CommandOutputBlock {
    id: ElementId,
    command: SharedString,
    output: SharedString,
    exit_code: Option<i32>,
    running: bool,
    stopped: bool,
    cwd: Option<SharedString>,
    meta: Option<SharedString>,
    duration: Option<CommandDuration>,
    running_label: SharedString,
    stopped_label: SharedString,
}

impl CommandOutputBlock {
    /// `command` with its `output` (open by default when there is output).
    pub fn new(
        id: impl Into<ElementId>,
        command: impl Into<SharedString>,
        output: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            command: command.into(),
            output: output.into(),
            exit_code: None,
            running: false,
            stopped: false,
            cwd: None,
            meta: None,
            duration: None,
            running_label: "Running".into(),
            stopped_label: "stopped".into(),
        }
    }

    /// `exitCode`.
    pub fn exit_code(mut self, code: i32) -> Self {
        self.exit_code = Some(code);
        self
    }

    /// `running`: a spinner instead of the exit tag.
    pub fn running(mut self, running: bool) -> Self {
        self.running = running;
        self
    }

    /// `stopped`: a gray `stopped` tag.
    pub fn stopped(mut self, stopped: bool) -> Self {
        self.stopped = stopped;
        self
    }

    /// `cwd`: printed above the output.
    pub fn cwd(mut self, cwd: impl Into<SharedString>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    /// `meta`: a quiet note after the command.
    pub fn meta(mut self, meta: impl Into<SharedString>) -> Self {
        self.meta = Some(meta.into());
        self
    }

    /// `duration`.
    pub fn duration(mut self, duration: CommandDuration) -> Self {
        self.duration = Some(duration);
        self
    }

    /// The spinner's hidden label and the stopped tag's text (defaults
    /// "Running", "stopped").
    pub fn labels(
        mut self,
        running: impl Into<SharedString>,
        stopped: impl Into<SharedString>,
    ) -> Self {
        self.running_label = running.into();
        self.stopped_label = stopped.into();
        self
    }
}

impl RenderOnce for CommandOutputBlock {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let open_state = window.use_keyed_state((id.clone(), "open"), cx, {
            let has_output = !self.output.is_empty();
            move |_, _| has_output
        });
        let open = *open_state.read(cx);
        let (hovered, hover) = press::hover((id.clone(), "hover"), window, cx);
        let (focus, ring) = press::focus((id.clone(), "focus"), window, cx);
        let failed = self.exit_code.is_some_and(|c| c != 0);
        let secondary_icon = color(ColorRole::IconSecondary);
        let helper = color(ColorRole::TextHelper);
        let status = if self.running {
            Some(
                Spin::new((id.clone(), "spin"))
                    .label(self.running_label.clone())
                    .into_any_element(),
            )
        } else if self.stopped {
            Some(
                Badge::new(self.stopped_label.clone())
                    .size(BadgeSize::Sm)
                    .variant(BadgeVariant::Gray)
                    .into_any_element(),
            )
        } else {
            self.exit_code.map(|code| {
                let (label, variant) = exit_tag(code);
                Badge::new(label)
                    .size(BadgeSize::Sm)
                    .variant(variant)
                    .into_any_element()
            })
        };
        let duration = self.duration.clone().map(|d| match d {
            CommandDuration::Seconds(s) => SharedString::from(fmt_elapsed(s)),
            CommandDuration::Text(t) => t,
        });
        // `.t3-cmd-head { display: flex; align-items: center; gap: spacing-03; width: 100%;
        // min-height: size-md; padding: 0 spacing-04; color: text-primary }`, `:hover { background: layer-hover-01 }`.
        let head = gpui_kit::base::Button::new((id.clone(), "head"))
            .track_focus(&focus)
            .accessibility_label(self.command.clone())
            .relative()
            .flex()
            .flex_row()
            .items_center()
            .justify_start()
            .w_full()
            .gap(rem(8.))
            .min_h(rem(40.))
            .px(rem(12.))
            .text_color(color(ColorRole::TextPrimary))
            .cursor_pointer()
            .when(hovered, |this| this.bg(color(ColorRole::LayerHover01)))
            .on_hover(press::track_hover(hover))
            .on_click(move |_, _, cx| {
                open_state.update(cx, |o, cx| {
                    *o = !*o;
                    cx.notify();
                })
            })
            .child(press::icon(CarbonIcon::Terminal, 16., secondary_icon))
            // `.t3-cmd-line { flex: 1; text-overflow: ellipsis; font-family: mono }`, 13px (r3-gal);
            // `.t3-cmd-prompt { color: text-helper; margin-right: spacing-03 }`.
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .font(font(FontFamilyKey::Mono, 400))
                    .text_size(rem(13.))
                    .child(div().flex_none().mr(rem(8.)).text_color(helper).child("$"))
                    .child(
                        div()
                            .min_w_0()
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(self.command.clone()),
                    ),
            )
            // `.t3-cmd-meta { flex: none; font-size: 0.75rem; color: text-helper }`.
            .when_some(self.meta.clone(), |this, m| {
                this.child(
                    div()
                        .flex_none()
                        .text_size(rem(12.))
                        .text_color(helper)
                        .child(m),
                )
            })
            // `.t3-cmd-dur { flex: none; font-size: 0.75rem; color: text-helper; tabular-nums }`.
            .when_some(duration, |this, d| {
                this.child(
                    div()
                        .flex_none()
                        .text_size(rem(12.))
                        .text_color(helper)
                        .child(d),
                )
            })
            .children(status)
            .child(
                press::icon(CarbonIcon::ChevronDown, 14., secondary_icon).rotate(
                    gpui_kit::radians(if open {
                        0.
                    } else {
                        -std::f32::consts::FRAC_PI_2
                    }),
                ),
            )
            .when(ring, |this| this.child(press::ring()));
        // `.t3-cmd { background: layer-01 }`, `.t3-cmd--fail { box-shadow: inset 3px 0 0 support-error }`.
        div()
            .flex()
            .flex_col()
            .w_full()
            .min_w_0()
            .bg(color(ColorRole::Layer01))
            .when(failed, |this| {
                this.shadow(card::rail(color(ColorRole::SupportError)))
            })
            .text_size(rem(14.))
            .line_height(rem(18.))
            .child(head)
            .when(open, |this| {
                // `.t3-cmd-out { max-height: 12rem; overflow: auto; padding: spacing-04;
                // border-top: 1px solid border-subtle-01; font-family: mono; font-size: 0.75rem;
                // line-height: 1.33333; color: text-secondary; white-space: pre-wrap }`.
                this.child(
                    div()
                        .id((id.clone(), "out"))
                        .max_h(rem(192.))
                        .overflow_y_scroll()
                        .p(rem(12.))
                        .border_t_1()
                        .border_color(color(ColorRole::BorderSubtle01))
                        .font(font(FontFamilyKey::Mono, 400))
                        .text_size(rem(12.))
                        .line_height(rem(16.))
                        .text_color(color(ColorRole::TextSecondary))
                        // `.t3-cmd-cwd { color: text-helper }`.
                        .when_some(self.cwd.clone(), |this, cwd| {
                            this.child(div().text_color(helper).child(cwd))
                        })
                        .child(self.output.clone()),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_tags_follow_the_code() {
        assert_eq!(exit_tag(1), ("exit 1".into(), BadgeVariant::Red));
        assert_eq!(exit_tag(0), ("exit 0".into(), BadgeVariant::Green));
    }
}
