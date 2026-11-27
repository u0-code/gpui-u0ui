//! LogTroubleRow: one problem from the logs on the Diagnostics page
//! (`LogTroubleRow` in `src/ghost/admin.jsx`, `.g-log` in `src/ghost.css`):
//! the level glyph, the message on one line, the source in mono, the time,
//! a "×N" count when it repeated, "Open log" and a copy button.
use crate::{
    components::{Button, ButtonSize, ButtonVariant},
    ghost::rows as row,
    icons::{CarbonIcon, Icon},
    t3::{
        base::{visually_hidden, warning_glyph},
        craft::{rem, sans},
    },
    theme::color,
    tokens::{self, ColorRole},
};
use gpui_kit::{
    AnyElement, App, ElementId, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    Window, div, prelude::FluentBuilder as _,
};
use std::rc::Rc;

/// The problem's level (`level`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum LogLevel {
    /// `error` (default): `ErrorFilled` in `support-error`.
    #[default]
    Error,
    /// `warning`: the `.t3-warnicon` glyph.
    Warning,
    /// `info`: `InformationFilled` in `support-info`.
    Info,
}

impl LogLevel {
    /// Every level.
    pub const ALL: [Self; 3] = [Self::Error, Self::Warning, Self::Info];

    /// The `index.d.ts` name (also the hidden label).
    pub const fn name(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Info => "info",
        }
    }

    /// The glyph fill: `.g-log--error .g-log-lvl svg { fill:
    /// var(--support-error) }`, `.t3-warnicon` `support-warning`,
    /// `.g-log--info … { fill: var(--support-info) }`.
    pub fn ink(self) -> Hsla {
        color(match self {
            Self::Error => ColorRole::SupportError,
            Self::Warning => ColorRole::SupportWarning,
            Self::Info => ColorRole::SupportInfo,
        })
    }
}

/// The row's user-facing strings (the reference's English defaults).
#[derive(Clone)]
pub struct LogTroubleRowTexts {
    /// `"Open log"`.
    pub open: SharedString,
    /// `"Copy details"`.
    pub copy: SharedString,
}

impl Default for LogTroubleRowTexts {
    fn default() -> Self {
        Self {
            open: "Open log".into(),
            copy: "Copy details".into(),
        }
    }
}

/// `count > 1 ? '×' + count : null`.
pub fn count_badge(count: u32) -> Option<String> {
    (count > 1).then(|| format!("×{count}"))
}

type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

/// A log problem row. See the module docs.
#[derive(IntoElement)]
pub struct LogTroubleRow {
    id: ElementId,
    level: LogLevel,
    source: Option<SharedString>,
    message: SharedString,
    count: u32,
    time: Option<SharedString>,
    texts: LogTroubleRowTexts,
    on_open: Option<Handler>,
    on_copy: Option<Handler>,
}

impl LogTroubleRow {
    /// An error row for `message`.
    pub fn new(id: impl Into<ElementId>, message: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            level: LogLevel::Error,
            source: None,
            message: message.into(),
            count: 1,
            time: None,
            texts: LogTroubleRowTexts::default(),
            on_open: None,
            on_copy: None,
        }
    }

    /// The level (`level`).
    pub fn level(mut self, level: LogLevel) -> Self {
        self.level = level;
        self
    }

    /// Where it came from (`source`), mono.
    pub fn source(mut self, source: impl Into<SharedString>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// How many times it happened (`count`, default 1).
    pub fn count(mut self, count: u32) -> Self {
        self.count = count;
        self
    }

    /// When (`time`).
    pub fn time(mut self, time: impl Into<SharedString>) -> Self {
        self.time = Some(time.into());
        self
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: LogTroubleRowTexts) -> Self {
        self.texts = texts;
        self
    }

    /// "Open log" pressed (`onOpen`).
    pub fn on_open(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_open = Some(Rc::new(f));
        self
    }

    /// The copy button pressed (`onCopy`).
    pub fn on_copy(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_copy = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for LogTroubleRow {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let glyph: AnyElement = match self.level {
            LogLevel::Warning => warning_glyph(16.).into_any_element(),
            LogLevel::Error => Icon::carbon(CarbonIcon::ErrorFilled)
                .size_length(rem(16.))
                .color(self.level.ink())
                .into_any_element(),
            LogLevel::Info => Icon::carbon(CarbonIcon::InformationFilled)
                .size_length(rem(16.))
                .color(self.level.ink())
                .into_any_element(),
        };
        let mut meta: Vec<AnyElement> = Vec::new();
        if let Some(s) = &self.source {
            meta.push(row::mono_text(s.clone()).into_any_element());
        }
        if let Some(t) = &self.time {
            meta.push(div().child(t.clone()).into_any_element());
        }
        if let Some(c) = count_badge(self.count) {
            // `.g-log-count { padding: 0 var(--spacing-02); background:
            // var(--layer-accent-01); border-radius: var(--border-radius-16) }`.
            meta.push(
                div()
                    .px(tokens::SPACING_02)
                    .rounded(rem(16.))
                    .bg(color(ColorRole::LayerAccent01))
                    .child(c)
                    .into_any_element(),
            );
        }
        let (on_open, on_copy) = (self.on_open.clone(), self.on_copy.clone());
        row::frame()
            .child(
                // `.g-log-lvl { display: inline-flex; flex: none }`.
                div()
                    .relative()
                    .flex()
                    .flex_none()
                    .child(glyph)
                    .child(visually_hidden(self.level.name())),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .gap(gpui_kit::px(2.))
                    .child(
                        // `.g-log-msg { font-size: 0.875rem; overflow: hidden;
                        // white-space: nowrap; text-overflow: ellipsis }`.
                        sans(div(), 14., 400)
                            .min_w_0()
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .text_ellipsis()
                            .child(self.message.clone()),
                    )
                    .child(row::meta(meta)),
            )
            .child(row::tail(
                Button::new((id.clone(), "open"), self.texts.open.clone())
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm)
                    .when_some(on_open, |b, f| b.on_click(move |_, w, cx| f(w, cx))),
            ))
            .child(row::tail(
                Button::new((id.clone(), "copy"), self.texts.copy.clone())
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm)
                    .icon(CarbonIcon::Copy)
                    .icon_only(true)
                    .when_some(on_copy, |b, f| b.on_click(move |_, w, cx| f(w, cx))),
            ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CarbonTheme;

    #[test]
    fn levels_follow_g_log() {
        crate::theme::tests_set(CarbonTheme::G100);
        assert_eq!(LogLevel::Error.ink(), color(ColorRole::SupportError));
        assert_eq!(LogLevel::Info.ink(), color(ColorRole::SupportInfo));
        assert_eq!(LogLevel::Warning.ink(), color(ColorRole::SupportWarning));
    }

    #[test]
    fn count_shows_past_one() {
        assert_eq!(count_badge(1), None);
        assert_eq!(count_badge(12).as_deref(), Some("×12"));
    }
}
