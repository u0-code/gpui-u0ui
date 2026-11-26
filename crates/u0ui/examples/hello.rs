//! The README's quick start: one window with a Carbon button and switch.
//!
//! ```text
//! cargo run -p u0ui --example hello
//! ```
use gpui_kit::*;
use u0ui::components::Switch;
use u0ui::prelude::*;

struct Hello {
    deployed: usize,
}

impl Render for Hello {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity();
        div()
            .size_full()
            .p(tokens::SPACING_07)
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            // Paint with color roles so the UI follows the theme.
            .bg(color(ColorRole::Background))
            .text_color(color(ColorRole::TextPrimary))
            .type_style(TypeStyle::BodyCompact01)
            .child(
                Button::new("deploy", "Deploy")
                    .icon(CarbonIcon::Launch)
                    .on_click(move |_, _, cx| {
                        view.update(cx, |this, cx| {
                            this.deployed += 1;
                            cx.notify();
                        })
                    }),
            )
            .child(format!("Deployed {} times", self.deployed))
            .child(
                Switch::new("dark")
                    .label("Dark theme")
                    .label_on("On")
                    .label_off("Off")
                    .checked(theme::is_dark())
                    .on_checked_change(|dark, _, cx| {
                        let theme = if dark {
                            CarbonTheme::G100
                        } else {
                            CarbonTheme::White
                        };
                        theme::set_theme(cx, theme);
                    }),
            )
    }
}

fn main() {
    gpui_kit::application()
        .with_assets(u0ui::Assets::new())
        .run(|cx| {
            gpui_kit::init(cx);
            u0ui::init(cx);
            gpui_kit::open_window(WindowOptions::default(), cx, |_, cx| {
                cx.new(|_| Hello { deployed: 0 })
            })
            .expect("open the window");
            cx.activate(true);
        });
}
