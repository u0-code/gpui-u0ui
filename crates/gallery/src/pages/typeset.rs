//! Data display › Typography (typeset): the reference article (Getting
//! started, a paragraph with a link, a list with inline code, a quote, Next
//! steps) and every block type.
use crate::controls;
use crate::page::Body;
use gpui_kit::{AnyView, App, AppContext as _, Context, IntoElement, Render, Window};
use u0ui::components::typography::{Inline, TypeBlock, Typography};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| TypesetPage).into()
}

struct TypesetPage;

impl Render for TypesetPage {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        // `components/Typography/preview.html` (the h2 has `marginTop: 0`).
        let reference = Typography::new("ref").blocks([
            TypeBlock::Heading(2, "Getting started".into()),
            TypeBlock::Paragraph(vec![
                "Carbon UI pairs ".into(),
                Inline::Link(
                    "IBM Carbon".into(),
                    "https://carbondesignsystem.com".into(),
                ),
                " styling with the shadcn/ui component API. Install the package and wrap your app in a theme.".into(),
            ]),
            TypeBlock::List {
                ordered: false,
                items: vec![
                    vec!["Four themes: White, G10, G90, G100".into()],
                    vec![
                        "Use ".into(),
                        Inline::Code("body-01".into()),
                        " for product copy".into(),
                    ],
                ],
            },
            TypeBlock::Quote(
                "Design is the intermediary between information and understanding.".into(),
            ),
            TypeBlock::Heading(3, "Next steps".into()),
            TypeBlock::Paragraph(vec!["Read the component guidelines.".into()]),
        ]);

        let all = Typography::new("all").blocks([
            TypeBlock::Heading(1, "Heading 1 · heading-06".into()),
            TypeBlock::Paragraph(vec!["Body copy in body-02, 16/24.".into()]),
            TypeBlock::Heading(2, "Heading 2 · heading-05".into()),
            TypeBlock::Heading(3, "Heading 3 · heading-03".into()),
            TypeBlock::Heading(4, "Heading 4 · heading-02".into()),
            TypeBlock::List {
                ordered: true,
                items: vec![
                    vec!["Create a project".into()],
                    vec!["Add a bucket".into()],
                    vec!["Upload objects".into()],
                ],
            },
            TypeBlock::Table {
                header: vec!["Token".into(), "Size".into(), "Use".into()],
                rows: vec![
                    vec!["body-01".into(), "14/20".into(), "Product copy".into()],
                    vec!["body-02".into(), "16/24".into(), "Long-form text".into()],
                ],
            },
        ]);

        Body::new()
            .playground(controls::block("Reference preview", reference))
            .example(controls::block("Every block", all))
    }
}
