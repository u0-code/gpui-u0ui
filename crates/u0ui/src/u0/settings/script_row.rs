//! u0 · Settings › [`ScriptRow`] (`.g-script`): one project script: its
//! kind (run / test / setup…) in an uppercase chip, name over the mono
//! command, the shortcut's key caps, Run (or Stop while running) and Edit.
use super::row::{self, Handler, ghost_button, icon_button};
use crate::{
    components::kbd::Kbd,
    icons::CarbonIcon,
    t3::craft::{mono, rem, sans},
    theme::color,
    tokens::{self, ColorRole},
};
use gpui_kit::{
    App, ElementId, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
    prelude::FluentBuilder as _,
};
use std::rc::Rc;

/// `.g-script-kind { width: 3.5rem }`.
const KIND_WIDTH: f32 = 56.;
/// `.g-script-kind { line-height: 1.25rem }` (the chip's height).
const KIND_LINE: f32 = 20.;
/// `.g-kbds { gap: 2px }`.
const KBD_GAP: f32 = 2.;

/// The chip text: `.g-script-kind { text-transform: uppercase }`.
pub fn kind_label(kind: &str) -> String {
    kind.to_uppercase()
}

/// The shortcut's key caps: `shortcut.split(' ')` (empty parts dropped).
pub fn shortcut_keys(shortcut: &str) -> Vec<&str> {
    shortcut.split(' ').filter(|k| !k.is_empty()).collect()
}

/// What the row's run slot shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunSlot {
    /// `Cb.IconButton label="Run {name}"` with a play glyph.
    Run,
    /// `Button variant="ghost" size="sm" icon={StopFilled}` "Stop".
    Stop,
}

impl RunSlot {
    /// The slot for `running`.
    pub fn of(running: bool) -> Self {
        if running { Self::Stop } else { Self::Run }
    }
}

/// The row's user-facing strings (the reference's English defaults).
#[derive(Clone)]
pub struct ScriptRowTexts {
    /// The run button's tooltip: `"Run {name}"`.
    pub run: Rc<dyn Fn(&str) -> SharedString>,
    /// The stop button: `"Stop"`.
    pub stop: SharedString,
    /// The edit button's tooltip: `"Edit {name}"`.
    pub edit: Rc<dyn Fn(&str) -> SharedString>,
}

impl Default for ScriptRowTexts {
    fn default() -> Self {
        Self {
            run: Rc::new(|name| format!("Run {name}").into()),
            stop: "Stop".into(),
            edit: Rc::new(|name| format!("Edit {name}").into()),
        }
    }
}

/// A project script row (`ScriptRow`). See the module docs.
#[derive(IntoElement)]
pub struct ScriptRow {
    id: ElementId,
    name: SharedString,
    command: SharedString,
    kind: SharedString,
    shortcut: Option<SharedString>,
    running: bool,
    texts: ScriptRowTexts,
    on_run: Option<Handler>,
    on_stop: Option<Handler>,
    on_edit: Option<Handler>,
}

impl ScriptRow {
    /// The script `name` running `command` (kind `run`).
    pub fn new(
        id: impl Into<ElementId>,
        name: impl Into<SharedString>,
        command: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            command: command.into(),
            kind: "run".into(),
            shortcut: None,
            running: false,
            texts: ScriptRowTexts::default(),
            on_run: None,
            on_stop: None,
            on_edit: None,
        }
    }

    /// The kind (`kind`: run, test, setup…), shown upper-cased.
    pub fn kind(mut self, kind: impl Into<SharedString>) -> Self {
        self.kind = kind.into();
        self
    }

    /// The shortcut, keys separated by spaces (`shortcut`, e.g. `"⌘ R"`).
    pub fn shortcut(mut self, shortcut: impl Into<SharedString>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// Running: Stop in place of Run (`running`).
    pub fn running(mut self, running: bool) -> Self {
        self.running = running;
        self
    }

    /// The user-facing strings.
    pub fn texts(mut self, texts: ScriptRowTexts) -> Self {
        self.texts = texts;
        self
    }

    /// Run (`onRun`).
    pub fn on_run(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_run = Some(Rc::new(f));
        self
    }

    /// Stop, while running.
    pub fn on_stop(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_stop = Some(Rc::new(f));
        self
    }

    /// Edit.
    pub fn on_edit(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_edit = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for ScriptRow {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let id = self.id;
        // `.g-script-kind { flex: none; width: 3.5rem; padding: 0
        // var(--spacing-02); background: var(--layer-accent-01); color:
        // var(--text-secondary); font-size: 0.6875rem; line-height: 1.25rem;
        // text-align: center }`.
        let kind = sans(div(), 11., 400)
            .flex()
            .flex_none()
            .justify_center()
            .items_center()
            .w(rem(KIND_WIDTH))
            .h(rem(KIND_LINE))
            .px(tokens::SPACING_02)
            .overflow_hidden()
            .whitespace_nowrap()
            .bg(color(ColorRole::LayerAccent01))
            .text_color(color(ColorRole::TextSecondary))
            .child(kind_label(&self.kind));
        // `.g-mono.g-dim.g-script-cmd`: 12px mono `text-secondary`, one line.
        let command = mono(div(), 12., 400)
            .min_w_0()
            .overflow_hidden()
            .whitespace_nowrap()
            .text_ellipsis()
            .text_color(color(ColorRole::TextSecondary))
            .child(self.command);
        let keys = self.shortcut.as_ref().map(|s| {
            // `.g-kbds { display: inline-flex; gap: 2px; flex: none }`.
            div().flex().flex_none().gap(rem(KBD_GAP)).children(
                shortcut_keys(s)
                    .into_iter()
                    .map(|k| Kbd::new(k.to_string())),
            )
        });
        let run = match RunSlot::of(self.running) {
            RunSlot::Stop => ghost_button(
                (id.clone(), "stop"),
                self.texts.stop.clone(),
                Some(CarbonIcon::StopFilled),
                self.on_stop,
            )
            .into_any_element(),
            RunSlot::Run => icon_button(
                (id.clone(), "run"),
                (self.texts.run)(&self.name),
                CarbonIcon::Play,
                self.on_run,
            )
            .into_any_element(),
        };
        let edit = icon_button(
            (id.clone(), "edit"),
            (self.texts.edit)(&self.name),
            CarbonIcon::Edit,
            self.on_edit,
        );
        row::frame()
            .child(kind)
            .child(row::main(self.name, Some(command)))
            .when_some(keys, |this, k| this.child(k))
            .child(run)
            .child(edit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_is_upper_cased() {
        assert_eq!(kind_label("run"), "RUN");
        assert_eq!(kind_label("setup"), "SETUP");
    }

    #[test]
    fn shortcut_splits_on_spaces() {
        assert_eq!(shortcut_keys("⌘ R"), ["⌘", "R"]);
        assert_eq!(shortcut_keys("⌘ ⇧  T"), ["⌘", "⇧", "T"]);
        assert!(shortcut_keys("").is_empty());
    }

    #[test]
    fn running_swaps_run_for_stop() {
        assert_eq!(RunSlot::of(true), RunSlot::Stop);
        assert_eq!(RunSlot::of(false), RunSlot::Run);
        let t = ScriptRowTexts::default();
        assert_eq!((t.run)("Test").as_ref(), "Run Test");
        assert_eq!((t.edit)("Test").as_ref(), "Edit Test");
    }

    #[test]
    fn chip_matches_g_script_kind() {
        // `width: 3.5rem; line-height: 1.25rem; padding: 0 var(--spacing-02)`.
        assert_eq!(KIND_WIDTH, 3.5 * tokens::REM_PX);
        assert_eq!(KIND_LINE, 1.25 * tokens::REM_PX);
        assert_eq!(tokens::SPACING_02.px_value(), 4.);
    }
}
