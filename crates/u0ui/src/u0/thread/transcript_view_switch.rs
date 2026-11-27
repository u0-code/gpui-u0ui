//! TranscriptViewSwitch: Conversation / Detailed / Raw events
//! (`TranscriptViewSwitch`, `.g-tview` in `src/ghost.css` with the later
//! `src/t3.css` overrides): a radio group of segments, then a `⌃ O cycle`
//! hint.
//!
//! Arrows, Home and End move the choice inside the group (`radioKeys`);
//! Control+O cycles it from anywhere in the window while `global_shortcut`
//! is on (the reference's `window` keydown listener; here a keystroke
//! interceptor, filtered to this window, kept alive in keyed state).
//!
//! ```ignore
//! TranscriptViewSwitch::new("tview")
//!     .value(view.clone())
//!     .on_value_change(|v, _, cx| set_view(v, cx))
//! ```
use crate::components::form::controlled;
use crate::components::kbd;
use crate::t3::{base::radio_step, craft, press};
use crate::{
    theme::color,
    tokens::{ColorRole, FontFamilyKey},
    typography::font,
};
use gpui_kit::{
    App, BoxShadow, ElementId, Entity, FocusHandle, FontWeight, Hsla, InteractiveElement as _,
    IntoElement, KeyDownEvent, Keystroke, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement as _, Styled, Subscription, Window, div,
    prelude::FluentBuilder as _, px,
};
use std::{cell::RefCell, rc::Rc};

/// The reference's default `views`.
pub fn default_views() -> Vec<(SharedString, SharedString)> {
    vec![
        ("conversation".into(), "Conversation".into()),
        ("detailed".into(), "Detailed".into()),
        ("raw".into(), "Raw events".into()),
    ]
}

/// Whether `keystroke` is the cycle shortcut: Control+O without Command
/// (`e.ctrlKey && !e.metaKey && e.key.toLowerCase() === 'o'`).
pub fn is_cycle_shortcut(keystroke: &Keystroke) -> bool {
    keystroke.modifiers.control
        && !keystroke.modifiers.platform
        && keystroke.key.eq_ignore_ascii_case("o")
}

/// The view after `current` in `ids`, wrapping (`views[(i + 1) % n]`; a
/// value not in the list counts as −1, so the first view follows it).
pub fn next_view(ids: &[SharedString], current: &SharedString) -> Option<SharedString> {
    if ids.is_empty() {
        return None;
    }
    let i = ids.iter().position(|v| v == current);
    let next = i.map_or(0, |i| (i + 1) % ids.len());
    Some(ids[next].clone())
}

/// A segment's (background, text, weight) when `on` or `hovered`.
pub fn segment_look(on: bool, hovered: bool) -> (Hsla, Hsla, FontWeight) {
    if on {
        // `.g-tview-btn--on { background: layer-selected-01; color: text-primary;
        // font-weight: 600 }` (`t3.css`, overriding `layer-selected-inverse`).
        (
            color(ColorRole::LayerSelected01),
            color(ColorRole::TextPrimary),
            FontWeight::SEMIBOLD,
        )
    } else if hovered {
        // `.g-tview-btn:hover { background: layer-hover-01; color: text-primary }`.
        (
            color(ColorRole::LayerHover01),
            color(ColorRole::TextPrimary),
            FontWeight::NORMAL,
        )
    } else {
        // `background: transparent; color: text-secondary`.
        (
            gpui_kit::transparent_black(),
            color(ColorRole::TextSecondary),
            FontWeight::NORMAL,
        )
    }
}

/// Segment geometry in px.
pub mod geometry {
    /// `.g-tview-btn { height: 2rem }` (`t3.css`, over `density-field-sm`).
    pub const HEIGHT: f32 = 32.;
    /// `padding: 0 var(--spacing-04)`.
    pub const PADDING_X: f32 = 12.;
    /// `border: 1px solid; border-left-width: 0` (`:first-child` 1px).
    pub const BORDER: f32 = 1.;
    /// `.g-tview-hint { margin-left: var(--spacing-04) }`.
    pub const HINT_GAP: f32 = 12.;
}

/// The texts of a [`TranscriptViewSwitch`] (hard-coded in the reference).
#[derive(Clone, Debug)]
pub struct TranscriptViewSwitchTexts {
    /// The group's name (`Transcript view (Control+O cycles)`).
    pub label: SharedString,
    /// The two key caps of the hint (`⌃`, `O`).
    pub keys: (SharedString, SharedString),
    /// The hint's word after the keys (`cycle`).
    pub cycle: SharedString,
}

impl Default for TranscriptViewSwitchTexts {
    fn default() -> Self {
        Self {
            label: "Transcript view (Control+O cycles)".into(),
            keys: ("\u{2303}".into(), "O".into()),
            cycle: "cycle".into(),
        }
    }
}

type ChangeHandler = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;

/// What the window-wide shortcut reads at the time of the keystroke.
struct Latest {
    ids: Vec<SharedString>,
    shown: SharedString,
    store: Entity<SharedString>,
    on_change: Option<ChangeHandler>,
    enabled: bool,
}

/// The keyed shortcut listener: the interceptor subscription and the latest
/// props it acts on.
struct Shortcut {
    _subscription: Subscription,
    latest: Rc<RefCell<Option<Latest>>>,
}

/// Set the switch to `value`: remember it when uncontrolled, then report it.
fn choose(
    store: &Entity<SharedString>,
    on_change: &Option<ChangeHandler>,
    value: SharedString,
    window: &mut Window,
    cx: &mut App,
) {
    store.update(cx, |v, cx| {
        *v = value.clone();
        cx.notify();
    });
    if let Some(f) = on_change {
        f(&value, window, cx);
    }
}

/// The transcript view switch. See the module docs.
#[derive(IntoElement)]
pub struct TranscriptViewSwitch {
    id: ElementId,
    value: Option<SharedString>,
    default_value: SharedString,
    views: Vec<(SharedString, SharedString)>,
    global_shortcut: bool,
    on_value_change: Option<ChangeHandler>,
    texts: TranscriptViewSwitchTexts,
}

impl TranscriptViewSwitch {
    /// A switch over the reference's three views, starting at `conversation`.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            value: None,
            default_value: "conversation".into(),
            views: default_views(),
            global_shortcut: true,
            on_value_change: None,
            texts: TranscriptViewSwitchTexts::default(),
        }
    }

    /// `value`: the controlled view id.
    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// `defaultValue` (default `conversation`).
    pub fn default_value(mut self, value: impl Into<SharedString>) -> Self {
        self.default_value = value.into();
        self
    }

    /// `views: [id, label][]`.
    pub fn views<I, L>(mut self, views: impl IntoIterator<Item = (I, L)>) -> Self
    where
        I: Into<SharedString>,
        L: Into<SharedString>,
    {
        self.views = views
            .into_iter()
            .map(|(i, l)| (i.into(), l.into()))
            .collect();
        self
    }

    /// `globalShortcut` (default true): Control+O cycles from anywhere.
    pub fn global_shortcut(mut self, on: bool) -> Self {
        self.global_shortcut = on;
        self
    }

    /// `onValueChange`.
    pub fn on_value_change(
        mut self,
        f: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_value_change = Some(Rc::new(f));
        self
    }

    /// The group name and hint.
    pub fn texts(mut self, texts: TranscriptViewSwitchTexts) -> Self {
        self.texts = texts;
        self
    }
}

/// The hint's joined key cap: `.cu-kbd` with the corners split between the
/// two keys (`.g-tview-hint .cu-kbd:first-child`, `.cu-kbd + .cu-kbd`).
fn hint_key(label: SharedString, first: bool) -> impl IntoElement {
    let (background, rule, ink) = kbd::colors();
    let r = px(kbd::geometry::RADIUS);
    let pad = press::rem(kbd::geometry::PADDING_X);
    let size = press::rem(kbd::geometry::SIZE);
    let fs = press::rem(kbd::geometry::FONT_SIZE);
    div()
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        // `.g-tview-hint .cu-kbd { min-width: 0 }`.
        .h(size)
        .bg(background)
        .shadow(vec![BoxShadow::new(px(0.), px(-1.), rule).inset()])
        .text_color(ink)
        .text_size(fs)
        .line_height(fs)
        .whitespace_nowrap()
        .map(|this| {
            if first {
                // `border-radius: r 0 0 r; padding-right: 1px`, mono.
                this.rounded_tl(r)
                    .rounded_bl(r)
                    .pl(pad)
                    .pr(px(1.))
                    .font(font(FontFamilyKey::Mono, 400))
            } else {
                // `border-radius: 0 r r 0; padding-left: 1px; font-family: sans;
                // margin-right: spacing-02`.
                this.rounded_tr(r)
                    .rounded_br(r)
                    .pr(pad)
                    .pl(px(1.))
                    .mr(press::rem(4.))
                    .font(font(FontFamilyKey::Sans, 400))
            }
        })
        .child(label)
}

impl RenderOnce for TranscriptViewSwitch {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id;
        let (shown, store) = controlled(
            (id.clone(), "value"),
            self.value.clone(),
            self.default_value.clone(),
            window,
            cx,
        );
        let ids: Vec<SharedString> = self.views.iter().map(|(k, _)| k.clone()).collect();
        let on_change = self.on_value_change.clone();

        // The window-wide Control+O listener, created once and fed the
        // latest props every render.
        let handle = window.window_handle();
        let shortcut = window.use_keyed_state((id.clone(), "shortcut"), cx, |_, cx| {
            let latest: Rc<RefCell<Option<Latest>>> = Rc::new(RefCell::new(None));
            let read = latest.clone();
            let subscription = cx.intercept_keystrokes(move |event, window, cx| {
                if window.window_handle() != handle || !is_cycle_shortcut(&event.keystroke) {
                    return;
                }
                let Some((store, on_change, next)) = read.borrow().as_ref().and_then(|l| {
                    (l.enabled)
                        .then(|| next_view(&l.ids, &l.shown))
                        .flatten()
                        .map(|n| (l.store.clone(), l.on_change.clone(), n))
                }) else {
                    return;
                };
                cx.stop_propagation();
                choose(&store, &on_change, next, window, cx);
            });
            Shortcut {
                _subscription: subscription,
                latest,
            }
        });
        *shortcut.read(cx).latest.borrow_mut() = Some(Latest {
            ids: ids.clone(),
            shown: shown.clone(),
            store: store.clone(),
            on_change: on_change.clone(),
            enabled: self.global_shortcut,
        });

        let handles: Vec<FocusHandle> = (0..ids.len())
            .map(|i| craft::focus_handle(&craft::sub(&id, "seg", i), window, cx))
            .collect();
        let selected = ids.iter().position(|v| *v == shown);
        let segments: Vec<_> = self
            .views
            .into_iter()
            .enumerate()
            .map(|(i, (key, label))| {
                let seg = craft::sub(&id, "seg", i);
                let on = key == shown;
                let focus = handles[i].clone();
                let ring = craft::focus_visible(&focus, window);
                let (hovered, hover) = press::hover((seg.clone(), "hover"), window, cx);
                let (bg, ink, weight) = segment_look(on, hovered);
                let store = store.clone();
                let on_change = on_change.clone();
                gpui_kit::base::Button::new(seg)
                    .track_focus(&focus)
                    .role(gpui_kit::Role::RadioButton)
                    .accessibility_label(label.clone())
                    // `tabIndex={v === k ? 0 : -1}`; with no match the first is the stop.
                    .tab_stop(on || (selected.is_none() && i == 0))
                    .relative()
                    .flex()
                    .flex_none()
                    .items_center()
                    .h(press::rem(geometry::HEIGHT))
                    .px(press::rem(geometry::PADDING_X))
                    // `.g-tview-btn { border: 1px solid border-subtle-01 }` (`t3.css`),
                    // `border-left-width: 0` but on the first.
                    .border_t(px(geometry::BORDER))
                    .border_b(px(geometry::BORDER))
                    .border_r(px(geometry::BORDER))
                    .when(i == 0, |b| b.border_l(px(geometry::BORDER)))
                    .border_color(color(ColorRole::BorderSubtle01))
                    .bg(bg)
                    .text_color(ink)
                    .font_weight(weight)
                    .text_size(press::rem(12.))
                    .line_height(press::rem(16.))
                    .whitespace_nowrap()
                    .cursor_pointer()
                    .on_hover(press::track_hover(hover))
                    .on_click(move |_, window, cx| {
                        choose(&store, &on_change, key.clone(), window, cx)
                    })
                    .child(label)
                    .when(ring, |b| b.child(press::ring()))
            })
            .collect();
        let keys = {
            let store = store.clone();
            let on_change = on_change.clone();
            let ids = ids.clone();
            move |e: &KeyDownEvent, window: &mut Window, cx: &mut App| {
                let current = ids.iter().position(|v| *v == *store.read(cx));
                if let Some(n) = radio_step(e.keystroke.key.as_str(), ids.len(), current) {
                    cx.stop_propagation();
                    choose(&store, &on_change, ids[n].clone(), window, cx);
                    handles[n].focus(window, cx);
                }
            }
        };
        let (k1, k2) = self.texts.keys;
        // `.g-tview { display: inline-flex; align-items: center; gap: 0 }`.
        div()
            .id(id)
            .flex()
            .flex_row()
            .flex_none()
            .items_center()
            .role(gpui_kit::Role::RadioGroup)
            .aria_label(self.texts.label)
            .on_key_down(keys)
            .children(segments)
            // `.g-tview-hint { display: inline-flex; gap: 0; align-items: center;
            // margin-left: spacing-04; font-size: 0.75rem; color: text-helper }`.
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_none()
                    .items_center()
                    .ml(press::rem(geometry::HINT_GAP))
                    .text_size(press::rem(12.))
                    .line_height(press::rem(16.))
                    .text_color(color(ColorRole::TextHelper))
                    .whitespace_nowrap()
                    .child(hint_key(k1, true))
                    .child(hint_key(k2, false))
                    .child(self.texts.cycle),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::CarbonTheme;

    #[test]
    fn control_o_cycles_and_wraps() {
        let ids: Vec<SharedString> = default_views().into_iter().map(|(k, _)| k).collect();
        assert_eq!(next_view(&ids, &"conversation".into()).unwrap(), "detailed");
        assert_eq!(next_view(&ids, &"raw".into()).unwrap(), "conversation");
        assert_eq!(next_view(&ids, &"bogus".into()).unwrap(), "conversation");
        assert!(next_view(&[], &"raw".into()).is_none());
        assert!(is_cycle_shortcut(&Keystroke::parse("ctrl-o").unwrap()));
        assert!(!is_cycle_shortcut(&Keystroke::parse("cmd-ctrl-o").unwrap()));
        assert!(!is_cycle_shortcut(&Keystroke::parse("o").unwrap()));
    }

    #[test]
    fn segments_follow_the_override_rules() {
        crate::theme::tests_set(CarbonTheme::White);
        let (bg, ink, w) = segment_look(true, false);
        assert_eq!(bg, color(ColorRole::LayerSelected01));
        assert_eq!(ink, color(ColorRole::TextPrimary));
        assert_eq!(w, FontWeight::SEMIBOLD);
        let (bg, ink, _) = segment_look(false, true);
        assert_eq!(bg, color(ColorRole::LayerHover01));
        assert_eq!(ink, color(ColorRole::TextPrimary));
        let (bg, ink, w) = segment_look(false, false);
        assert_eq!(bg.a, 0.);
        assert_eq!(ink, color(ColorRole::TextSecondary));
        assert_eq!(w, FontWeight::NORMAL);
        assert_eq!(geometry::HEIGHT, 32.);
    }
}
