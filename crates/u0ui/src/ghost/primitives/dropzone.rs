//! Dropzone: drop files or a folder, or press to choose (`Dropzone` in
//! `src/ghost/primitives.jsx`; Carbon `FileUploaderDropContainer`).
//!
//! - **Behavior**: the whole box is focusable; a press, Enter or Space opens
//!   the system picker (files, or a folder with [`Dropzone::folder`]);
//!   dropping files from the OS hands their paths to `on_files`. While a drag
//!   is over it the label reads "Release to add". `accept` filters dropped
//!   paths by extension. `busy` shows a [`Spin`] and "Reading…".
//! - **Appearance** (`.g-drop`): 8rem min height, 16px padding, a 1px dashed
//!   `border-strong-01` box on `layer-01` (hover `layer-hover-01`), 24px
//!   `icon-secondary` glyph, label `text-primary`, hint 12px
//!   `text-secondary`; drag-over a 2px solid `border-interactive` border on
//!   `highlight`; an error turns the border `support-error` and adds the
//!   message; 2px inset `focus` ring on keyboard focus.
use super::parts::{self, rem};
use crate::{
    icons::{CarbonIcon, Icon, IconSize},
    t3::base::Spin,
    theme::color,
    tokens::ColorRole,
};
use gpui_kit::{
    App, CursorStyle, ElementId, ExternalPaths, Hsla, InteractiveElement, IntoElement,
    KeyDownEvent, ParentElement, PathPromptOptions, RenderOnce, SharedString,
    StatefulInteractiveElement as _, Styled, Window, div, prelude::FluentBuilder as _, px,
};
use std::{path::PathBuf, rc::Rc};

/// The interaction state of a drop zone.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DropState {
    /// Pointer over it.
    pub hovered: bool,
    /// A drag is over it (`.g-drop--over`).
    pub over: bool,
    /// An error is shown (`.g-drop--err`).
    pub error: bool,
}

/// (background, border color, border width) for a state, in cascade order:
/// `.g-drop`, `:hover`, `--over`, `--err` (the error keeps the over width).
pub fn look(state: DropState) -> (Hsla, Hsla, f32) {
    let mut bg = color(ColorRole::Layer01);
    let mut border = color(ColorRole::BorderStrong01);
    let mut width = 1.;
    if state.hovered {
        bg = color(ColorRole::LayerHover01);
    }
    if state.over {
        bg = color(ColorRole::Highlight);
        border = color(ColorRole::BorderInteractive);
        width = 2.;
    }
    if state.error {
        border = color(ColorRole::SupportError);
    }
    (bg, border, width)
}

/// Whether `path` passes an `accept` list of extensions (`.json`, `vsix`;
/// empty accepts everything; folders always pass).
pub fn accepts(accept: &[SharedString], path: &std::path::Path) -> bool {
    if accept.is_empty() || path.is_dir() {
        return true;
    }
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    accept
        .iter()
        .any(|a| a.trim_start_matches('.').eq_ignore_ascii_case(&ext))
}

type FilesHandler = Rc<dyn Fn(Vec<PathBuf>, &mut Window, &mut App)>;
type Choose = Rc<dyn Fn(&mut Window, &mut App)>;

/// A file/folder drop zone. See the module docs.
#[derive(IntoElement)]
pub struct Dropzone {
    id: ElementId,
    label: SharedString,
    hint: SharedString,
    over_label: SharedString,
    busy_label: SharedString,
    accept: Vec<SharedString>,
    folder: bool,
    busy: bool,
    error: Option<SharedString>,
    on_files: Option<FilesHandler>,
}

impl Dropzone {
    /// A drop zone with a stable `id`.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            // The reference's defaults; override per locale.
            label: "Drop a file or folder".into(),
            hint: "or click to choose".into(),
            over_label: "Release to add".into(),
            busy_label: "Reading…".into(),
            accept: Vec::new(),
            folder: false,
            busy: false,
            error: None,
            on_files: None,
        }
    }

    /// The main line.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    /// The hint under it.
    pub fn hint(mut self, hint: impl Into<SharedString>) -> Self {
        self.hint = hint.into();
        self
    }

    /// The drag-over and busy texts (default "Release to add", "Reading…").
    pub fn state_labels(
        mut self,
        over: impl Into<SharedString>,
        busy: impl Into<SharedString>,
    ) -> Self {
        self.over_label = over.into();
        self.busy_label = busy.into();
        self
    }

    /// Accepted extensions (`accept`, e.g. `[".json", ".vsix"]`).
    pub fn accept(mut self, exts: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.accept = exts.into_iter().map(Into::into).collect();
        self
    }

    /// Choose a folder instead of files (`folder`; the FolderAdd glyph).
    pub fn folder(mut self, folder: bool) -> Self {
        self.folder = folder;
        self
    }

    /// Show the busy state (`state="busy"`).
    pub fn busy(mut self, busy: bool) -> Self {
        self.busy = busy;
        self
    }

    /// An error message (red border plus the message).
    pub fn error(mut self, error: impl Into<SharedString>) -> Self {
        self.error = Some(error.into());
        self
    }

    /// Called with the dropped or chosen paths.
    pub fn on_files(mut self, f: impl Fn(Vec<PathBuf>, &mut Window, &mut App) + 'static) -> Self {
        self.on_files = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for Dropzone {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let focus = parts::keyed_focus((id.clone(), "focus"), window, cx);
        let ring = parts::focus_visible(&focus, window);
        let hovered = window.use_keyed_state((id.clone(), "hover"), cx, |_, _| false);
        let over = window.use_keyed_state((id.clone(), "over"), cx, |_, _| false);
        // A drag that ended elsewhere leaves no "over" behind.
        if *over.read(cx) && !cx.has_active_drag() {
            over.update(cx, |o, _| *o = false);
        }
        let state = DropState {
            hovered: *hovered.read(cx),
            over: *over.read(cx),
            error: self.error.is_some(),
        };
        let (bg, border, width) = look(state);

        let choose: Choose = {
            let on_files = self.on_files.clone();
            let folder = self.folder;
            let prompt = self.label.clone();
            Rc::new(move |window, cx| {
                let rx = cx.prompt_for_paths(PathPromptOptions {
                    files: !folder,
                    directories: folder,
                    multiple: !folder,
                    prompt: Some(prompt.clone()),
                });
                let on_files = on_files.clone();
                window
                    .spawn(cx, async move |cx| {
                        if let Ok(Ok(Some(paths))) = rx.await
                            && let Some(f) = on_files
                        {
                            let _ = cx.update(|window, cx| f(paths, window, cx));
                        }
                    })
                    .detach();
            })
        };

        let content: Vec<gpui_kit::AnyElement> = if self.busy {
            vec![
                // `.g-drop-busy { display: inline-flex; gap: spacing-03; color: text-secondary }`.
                div()
                    .flex()
                    .items_center()
                    .gap(rem(8.))
                    .text_color(color(ColorRole::TextSecondary))
                    .child(Spin::new((id.clone(), "spin")).size(20.))
                    .child(self.busy_label.clone())
                    .into_any_element(),
            ]
        } else {
            vec![
                Icon::carbon(if self.folder {
                    CarbonIcon::FolderAdd
                } else {
                    CarbonIcon::Upload
                })
                .size(IconSize::S24)
                .color(color(ColorRole::IconSecondary))
                .into_any_element(),
                div()
                    .text_color(color(ColorRole::TextPrimary))
                    .child(if state.over {
                        self.over_label.clone()
                    } else {
                        self.label.clone()
                    })
                    .into_any_element(),
                div()
                    .text_size(rem(12.))
                    .line_height(rem(16.))
                    .child(self.hint.clone())
                    .into_any_element(),
            ]
        };

        let accept = self.accept.clone();
        let on_files = self.on_files.clone();
        let label = self.label.clone();
        div()
            .id(id.clone())
            .track_focus(&focus)
            .tab_index(0)
            .role(gpui_kit::Role::Button)
            .aria_label(label)
            .relative()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(rem(4.))
            .min_h(rem(128.))
            .p(rem(16.))
            .bg(bg)
            .border(px(width))
            .when(!state.over, |d| d.border_dashed())
            .border_color(border)
            .text_size(rem(14.))
            .line_height(rem(18.))
            .text_color(color(ColorRole::TextSecondary))
            .text_center()
            .cursor(CursorStyle::PointingHand)
            .on_hover(move |h, _, cx| {
                hovered.update(cx, |v, cx| {
                    *v = *h;
                    cx.notify();
                })
            })
            .on_click({
                let choose = choose.clone();
                move |_, window, cx| choose(window, cx)
            })
            .on_key_down({
                let choose = choose.clone();
                move |e: &KeyDownEvent, window, cx| {
                    if matches!(e.keystroke.key.as_str(), "enter" | "space") {
                        cx.stop_propagation();
                        choose(window, cx);
                    }
                }
            })
            .on_drag_move::<ExternalPaths>({
                let over = over.clone();
                move |e, _, cx| {
                    let inside = e.bounds.contains(&e.event.position);
                    over.update(cx, |o, cx| {
                        if *o != inside {
                            *o = inside;
                            cx.notify();
                        }
                    })
                }
            })
            .on_drop({
                let over = over.clone();
                move |paths: &ExternalPaths, window, cx| {
                    over.update(cx, |o, cx| {
                        *o = false;
                        cx.notify();
                    });
                    let picked: Vec<PathBuf> = paths
                        .paths()
                        .iter()
                        .filter(|p| accepts(&accept, p))
                        .cloned()
                        .collect();
                    if let Some(f) = &on_files {
                        f(picked, window, cx);
                    }
                }
            })
            .children(content)
            .children(self.error.clone().map(parts::error))
            .when(ring, |d| d.child(parts::focus_ring()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CarbonTheme, theme::color_in};

    #[test]
    fn states_follow_the_cascade() {
        crate::theme::tests_set(CarbonTheme::White);
        let c = |r| color_in(CarbonTheme::White, r);
        assert_eq!(
            look(DropState::default()),
            (c(ColorRole::Layer01), c(ColorRole::BorderStrong01), 1.)
        );
        let over = look(DropState {
            hovered: true,
            over: true,
            error: false,
        });
        assert_eq!(
            over,
            (c(ColorRole::Highlight), c(ColorRole::BorderInteractive), 2.)
        );
        let err = look(DropState {
            error: true,
            ..Default::default()
        });
        assert_eq!(err.1, c(ColorRole::SupportError));
    }

    #[test]
    fn accept_filters_by_extension() {
        let accept: Vec<SharedString> = vec![".json".into(), "vsix".into()];
        assert!(accepts(&accept, std::path::Path::new("theme.JSON")));
        assert!(accepts(&accept, std::path::Path::new("a.vsix")));
        assert!(!accepts(&accept, std::path::Path::new("a.txt")));
        assert!(accepts(&[], std::path::Path::new("a.txt")));
    }
}
