//! u0 · Primitives › ColorPicker: a live picker reporting its value, plus
//! the reference "Accent colour" picker.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div,
};
use u0ui::prelude::*;
use u0ui::u0::primitives::ColorPicker;

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| ColorPickerPage {
        value: "#8a3ffc".into(),
        swatches: true,
        full_width: false,
    })
    .into()
}

struct ColorPickerPage {
    value: SharedString,
    swatches: bool,
    full_width: bool,
}

/// `components/ColorPicker/preview.html`.
pub fn reference(id: &'static str) -> impl IntoElement {
    ColorPicker::new(id)
        .label("Accent colour")
        .default_value("#8a3ffc")
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    reference("thumb-color").into_any_element()
}

impl Render for ColorPickerPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let mut picker = ColorPicker::new("color-live")
            .label("Accent colour")
            .value(self.value.clone())
            .full_width(self.full_width)
            .on_value_change(controls::setter(&entity, |this: &mut Self, v| {
                this.value = v
            }));
        if !self.swatches {
            picker = picker.swatches(Vec::<SharedString>::new());
        }
        // A 22rem column, so `full_width` shows against the 15rem default.
        let live = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .w(gpui_kit::rems(22.))
            .max_w_full()
            .child(picker)
            .child(
                div()
                    .type_style(TypeStyle::Code01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(format!("value: \"{}\"", self.value)),
            );
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_06)
            .child(controls::toggle(
                "color-swatches",
                "swatches",
                self.swatches,
                controls::setter(&entity, |this: &mut Self, on| this.swatches = on),
            ))
            .child(controls::toggle(
                "color-full-width",
                "full_width",
                self.full_width,
                controls::setter(&entity, |this: &mut Self, on| this.full_width = on),
            ));
        Body::new()
            .playground(live)
            .hint(
                "Drag on the plane or the hue bar; arrows step 2% / 1°; type a hex and press Enter",
            )
            .config(config)
            .example(controls::block("Reference preview", reference("color-ref")))
    }
}
