//! u0 · Settings › SettingsListRow: the reference project rows, live
//! (actions report what was pressed), with every optional part switchable.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, rems,
};
use u0ui::prelude::*;
use u0ui::t3::{MachineKind, ProjectMonogram};
use u0ui::u0::settings::{SettingsAction, SettingsListRow};
use u0ui::u0::shell::{ServerChip, ServerChipSize};

const LEADING: [&str; 3] = ["icon", "leading", "none"];
const ACTIONS: [&str; 3] = ["icons", "text", "none"];

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| SettingsListRowPage {
        leading: 0,
        subtitle: true,
        status: true,
        actions: 0,
        last: "Nothing pressed yet".into(),
    })
    .into()
}

struct SettingsListRowPage {
    leading: usize,
    subtitle: bool,
    status: bool,
    actions: usize,
    last: SharedString,
}

/// `ServerChip name kind="server" size="sm"`.
fn server_chip(id: &'static str, name: &'static str) -> impl IntoElement {
    ServerChip::new(id, name)
        .kind(MachineKind::Server)
        .size(ServerChipSize::Sm)
}

/// `components/SettingsListRow/preview.html`: two rows, 560px on `layer-01`.
pub fn reference(prefix: &'static str) -> impl IntoElement {
    div()
        .w(rems(35.))
        .max_w_full()
        .bg(color(ColorRole::Layer01))
        .child(
            SettingsListRow::new((prefix, 0usize), "carbon-ui")
                .icon(CarbonIcon::Folder)
                .subtitle("~/code/carbon-ui")
                .action(SettingsAction::icon("Open", CarbonIcon::Launch))
                .action(SettingsAction::icon("Remove", CarbonIcon::TrashCan)),
        )
        .child(
            SettingsListRow::new((prefix, 1usize), "api-gateway")
                .icon(CarbonIcon::Folder)
                .subtitle("build-box:~/srv/api")
                .status(server_chip(prefix, "build-box"))
                .action(SettingsAction::icon("Open", CarbonIcon::Launch)),
        )
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .bg(color(ColorRole::Layer01))
        .child(
            SettingsListRow::new("thumb-slrow", "carbon-ui")
                .icon(CarbonIcon::Folder)
                .subtitle("~/code/carbon-ui")
                .action(SettingsAction::icon("Open", CarbonIcon::Launch)),
        )
        .into_any_element()
}

impl Render for SettingsListRowPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let report = |label: &'static str| {
            let e = entity.clone();
            move |_: &mut Window, cx: &mut App| {
                e.update(cx, |this, cx| {
                    this.last = format!("Pressed {label}").into();
                    cx.notify();
                })
            }
        };
        let mut row = SettingsListRow::new("slrow-live", "api-gateway");
        row = match self.leading {
            0 => row.icon(CarbonIcon::Folder),
            1 => row.leading(ProjectMonogram::new("api-gateway")),
            _ => row,
        };
        if self.subtitle {
            row = row.subtitle("build-box:~/srv/api");
        }
        if self.status {
            row = row.status(server_chip("slrow-live-chip", "build-box"));
        }
        row = match self.actions {
            0 => row
                .action(SettingsAction::icon("Open", CarbonIcon::Launch).on_click(report("Open")))
                .action(
                    SettingsAction::icon("Remove", CarbonIcon::TrashCan).on_click(report("Remove")),
                ),
            1 => row
                .action(SettingsAction::new("Open").on_click(report("Open")))
                .action(SettingsAction::new("Remove").on_click(report("Remove"))),
            _ => row,
        };
        let live = div()
            .w(rems(35.))
            .max_w_full()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_03)
            .child(div().bg(color(ColorRole::Layer01)).child(row))
            .child(
                div()
                    .type_style(TypeStyle::Label01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child(self.last.clone()),
            );
        let config = div()
            .flex()
            .flex_col()
            .gap(tokens::SPACING_05)
            .child(controls::field(
                "icon / leading",
                controls::choices(
                    "slrow-leading",
                    &LEADING,
                    self.leading,
                    controls::setter(&entity, |this: &mut Self, i| this.leading = i),
                ),
            ))
            .child(controls::toggle(
                "slrow-subtitle",
                "subtitle",
                self.subtitle,
                controls::setter(&entity, |this: &mut Self, on| this.subtitle = on),
            ))
            .child(controls::toggle(
                "slrow-status",
                "status",
                self.status,
                controls::setter(&entity, |this: &mut Self, on| this.status = on),
            ))
            .child(controls::field(
                "actions",
                controls::choices(
                    "slrow-actions",
                    &ACTIONS,
                    self.actions,
                    controls::setter(&entity, |this: &mut Self, i| this.actions = i),
                ),
            ));
        Body::new()
            .playground(live)
            .hint("Icon actions show their label as a tooltip")
            .config(config)
            .example(controls::block("Reference preview", reference("slrow-ref")))
    }
}
