//! Ghost · Thread › TranscriptViewSwitch: the reference switch
//! (`components/TranscriptViewSwitch/preview.html`), live and controlled:
//! click a segment, use the arrows inside the group, or press Control+O
//! anywhere in the window while the global shortcut is on.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div,
};
use u0ui::ghost::thread::TranscriptViewSwitch;
use u0ui::prelude::*;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| TranscriptViewSwitchPage {
        value: "conversation".into(),
        global_shortcut: true,
        two_views: false,
    })
    .into()
}

struct TranscriptViewSwitchPage {
    value: SharedString,
    global_shortcut: bool,
    two_views: bool,
}

/// The view ids the config panel offers (`views` of the reference).
const VIEWS: [&str; 3] = ["conversation", "detailed", "raw"];

impl Render for TranscriptViewSwitchPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let mut switch = TranscriptViewSwitch::new("tview-live")
            .value(self.value.clone())
            .global_shortcut(self.global_shortcut)
            .on_value_change({
                let entity = entity.clone();
                move |v: &SharedString, _: &mut Window, cx: &mut App| {
                    entity.update(cx, |this, cx| {
                        this.value = v.clone();
                        cx.notify();
                    })
                }
            });
        if self.two_views {
            switch = switch.views([("conversation", "Conversation"), ("raw", "Raw events")]);
        }
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(switch)
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(format!("value: {}", self.value)),
            );
        let selected = VIEWS
            .iter()
            .position(|v| *v == self.value.as_ref())
            .unwrap_or(0);
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::field(
                "value",
                controls::choices(
                    "tview-value",
                    &VIEWS,
                    selected,
                    controls::setter(&entity, |this: &mut Self, i: usize| {
                        this.value = VIEWS[i].into()
                    }),
                ),
            ))
            .child(controls::toggle(
                "tview-global",
                "globalShortcut",
                self.global_shortcut,
                controls::setter(&entity, |this: &mut Self, on| this.global_shortcut = on),
            ))
            .child(controls::toggle(
                "tview-two",
                "views: Conversation, Raw events",
                self.two_views,
                controls::setter(&entity, |this: &mut Self, on| this.two_views = on),
            ));
        Body::new()
            .playground(live)
            .hint("←/→ move inside the group; Control+O cycles from anywhere.")
            .config(config)
            .example(controls::block(
                "Reference preview",
                // `h(C.TranscriptViewSwitch, null)`: uncontrolled, no global
                // shortcut so it does not fight the live one.
                TranscriptViewSwitch::new("tview-reference").global_shortcut(false),
            ))
    }
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    TranscriptViewSwitch::new("tview-thumb")
        .global_shortcut(false)
        .into_any_element()
}
