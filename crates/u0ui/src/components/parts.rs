//! `part!`: declares a shadcn-style layout part (`EmptyHeader`, `ItemContent`,
//! …): a struct that collects children and renders them in a styled `div`,
//! like the reference's `part('div', 'cu-…')` helper.

/// `part!(Name, "doc", |children| div()…children(children))`.
macro_rules! part {
    ($(#[$doc:meta])* $name:ident, $render:expr) => {
        $(#[$doc])*
        #[derive(gpui_kit::IntoElement, Default)]
        pub struct $name {
            children: Vec<gpui_kit::AnyElement>,
        }

        impl $name {
            /// An empty part.
            pub fn new() -> Self {
                Self::default()
            }
        }

        impl gpui_kit::ParentElement for $name {
            fn extend(&mut self, elements: impl IntoIterator<Item = gpui_kit::AnyElement>) {
                self.children.extend(elements);
            }
        }

        impl gpui_kit::RenderOnce for $name {
            fn render(
                self,
                _: &mut gpui_kit::Window,
                _: &mut gpui_kit::App,
            ) -> impl gpui_kit::IntoElement {
                let render: fn(Vec<gpui_kit::AnyElement>) -> gpui_kit::Div = $render;
                render(self.children)
            }
        }
    };
}

pub(crate) use part;
