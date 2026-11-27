//! T3 · Composer › ComposerBannerStack: usage warning with an action and
//! dismiss, a clone in progress, a held message and a provider error
//! (`components/ComposerBannerStack/preview.html`), plus a live stack that
//! folds to the first banner and dismisses.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div,
};
use u0ui::prelude::*;
use u0ui::t3::composer::{BannerKind, ComposerBanner, ComposerBannerStack};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| StackPage {
        collapsed: true,
        dismissed: false,
        generation: 0,
    })
    .into()
}

struct StackPage {
    collapsed: bool,
    dismissed: bool,
    generation: usize,
}

fn ghost(id: SharedString, label: &'static str) -> Button {
    Button::new(id, label)
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::Sm)
}

type Dismiss = std::rc::Rc<dyn Fn(&mut Window, &mut App)>;

fn stack(
    prefix: &str,
    collapsed: bool,
    usage: bool,
    on_dismiss: Option<Dismiss>,
) -> ComposerBannerStack {
    let key = |s: &str| SharedString::from(format!("{prefix}-{s}"));
    let mut usage_banner = ComposerBanner::new(key("usage"), BannerKind::Warning)
        .title("Usage at 92%")
        .body("Resets in 2h 14m.")
        .action(ghost(key("view"), "View usage"));
    usage_banner = match on_dismiss {
        Some(f) => usage_banner.on_dismiss(move |w, cx| f(w, cx)),
        None => usage_banner.on_dismiss(|_, _| {}),
    };
    ComposerBannerStack::new(key("stack"))
        .collapsed(collapsed)
        .when_usage(usage, usage_banner)
        .child(
            ComposerBanner::new(key("clone"), BannerKind::Progress)
                .title("Cloning repository")
                .body("github.com/acme/web")
                .progress(48.),
        )
        .child(
            ComposerBanner::new(key("held"), BannerKind::Info)
                .title("Message held")
                .body("Will send when the current run finishes.")
                .action(ghost(key("send"), "Send now")),
        )
        .child(
            ComposerBanner::new(key("error"), BannerKind::Error)
                .title("Provider is not responding")
                .body("Restart the run to continue."),
        )
}

/// `usage` banner first when shown.
trait WhenUsage {
    fn when_usage(self, show: bool, banner: ComposerBanner) -> Self;
}

impl WhenUsage for ComposerBannerStack {
    fn when_usage(self, show: bool, banner: ComposerBanner) -> Self {
        if show { self.child(banner) } else { self }
    }
}

impl Render for StackPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let dismiss = {
            let entity = entity.clone();
            std::rc::Rc::new(move |_: &mut Window, cx: &mut App| {
                entity.update(cx, |this, cx| {
                    this.dismissed = true;
                    cx.notify();
                })
            }) as std::rc::Rc<dyn Fn(&mut Window, &mut App)>
        };
        let live = div().w_full().max_w(gpui_kit::rems(42.5)).child(stack(
            &format!("bs-live-{}-{}", self.generation, self.collapsed),
            self.collapsed,
            !self.dismissed,
            Some(dismiss),
        ));
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::toggle(
                "bs-collapsed",
                "collapsedCount",
                self.collapsed,
                controls::setter(&entity, |this: &mut Self, on| this.collapsed = on),
            ))
            .child(
                Button::new("bs-reset", "Restore banners")
                    .variant(ButtonVariant::Tertiary)
                    .size(ButtonSize::Sm)
                    .on_click({
                        let entity = entity.clone();
                        move |_, _, cx| {
                            entity.update(cx, |this, cx| {
                                this.dismissed = false;
                                this.generation += 1;
                                cx.notify();
                            })
                        }
                    }),
            );
        Body::new()
            .playground(live)
            .config(config)
            .example(controls::block(
                "Reference preview",
                div()
                    .w_full()
                    .max_w(gpui_kit::rems(42.5))
                    .child(stack("bs-ref", false, true, None)),
            ))
    }
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(gpui_kit::rems(15.))
        .child(
            ComposerBanner::new("bs-th", BannerKind::Warning)
                .title("Usage at 92%")
                .body("Resets in 2h 14m."),
        )
        .into_any_element()
}
