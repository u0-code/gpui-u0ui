//! u0 · Settings › SettingsDialog: the reference settings window, live
//! (search filters the navigation, items switch the section, the server menu
//! picks a Mac), showing the Agents section with its scope bar and rows.
use crate::controls;
use crate::page::Body;
use gpui_kit::{
    AnyElement, AnyView, App, AppContext as _, Context, ElementId, IntoElement, ParentElement,
    Render, SharedString, Styled, Window, div, prelude::FluentBuilder as _, rems,
};
use u0ui::prelude::*;
use u0ui::u0::settings::{
    AgentRow, AgentState, ScopeHeader, SettingsDialog, SettingsSection, settings_heading,
};

/// Build the page.
pub fn build(_: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| Page {
        value: "agents".into(),
        servers: true,
    })
    .into()
}

struct Page {
    value: SharedString,
    servers: bool,
}

/// The reference sections.
fn sections() -> Vec<SettingsSection> {
    vec![
        SettingsSection::new("")
            .item("general", "General", CarbonIcon::Settings)
            .item("agents", "Agents", CarbonIcon::Bot)
            .item("accounts", "Accounts", CarbonIcon::User),
        SettingsSection::new("Workspace")
            .item("scripts", "Scripts", CarbonIcon::Terminal)
            .item("theme", "Theme", CarbonIcon::ColorPalette)
            .item("voice", "Voice", CarbonIcon::Microphone),
        SettingsSection::new("This Mac")
            .item("devices", "Devices", CarbonIcon::Mobile)
            .item("privacy", "Privacy", CarbonIcon::Locked)
            .item("diag", "Diagnostics", CarbonIcon::Dashboard),
    ]
}

/// The section title for an item id.
fn title_of(value: &str) -> SharedString {
    sections()
        .into_iter()
        .flat_map(|s| s.items)
        .find(|i| i.id.as_ref() == value)
        .map(|i| i.label)
        .unwrap_or_default()
}

/// The Agents section from `components/SettingsDialog/preview.html`.
fn agents(prefix: &'static str) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .child(settings_heading("Agents"))
        .child(
            ScopeHeader::new((ElementId::from(prefix), "scope"))
                .scope(Some("carbon-ui".into()))
                .projects(["carbon-ui", "api-gateway"]),
        )
        .child(
            AgentRow::new((ElementId::from(prefix), "claude"), "Claude")
                .version("2.1.0")
                .signed_in(true)
                .accounts(["work"]),
        )
        .child(
            AgentRow::new((ElementId::from(prefix), "codex"), "Codex")
                .version("0.9.2")
                .latest("0.10.0"),
        )
        .child(
            AgentRow::new((ElementId::from(prefix), "gemini"), "Gemini")
                .state(AgentState::Installing)
                .progress(64.),
        )
        .child(
            AgentRow::new((ElementId::from(prefix), "cursor"), "Cursor").state(AgentState::Missing),
        )
}

fn dialog(prefix: &'static str, value: Option<SharedString>, servers: bool) -> SettingsDialog {
    let mut d = SettingsDialog::new(prefix);
    for s in sections() {
        d = d.section(s);
    }
    if let Some(v) = value {
        d = d.value(v);
    }
    d.when(servers, |d| d.servers(["MacBook Pro", "build-box"]))
}

/// The overview tile.
pub fn thumbnail(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .h(rems(9.))
        .overflow_hidden()
        .child(dialog("sdlg-thumb", Some("agents".into()), false))
        .into_any_element()
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let pick = entity.clone();
        let content = if self.value.as_ref() == "agents" {
            agents("sdlg-live-agents").into_any_element()
        } else {
            div()
                .child(settings_heading(title_of(&self.value)))
                .into_any_element()
        };
        let live = div().w_full().h(rems(26.25)).child(
            dialog("sdlg-live", Some(self.value.clone()), self.servers)
                .on_value_change(move |v, _, cx| {
                    pick.update(cx, |p, cx| {
                        p.value = v;
                        cx.notify();
                    })
                })
                .child(content),
        );
        let config = controls::toggle(
            "sdlg-servers",
            "servers",
            self.servers,
            controls::setter(&entity, |p: &mut Page, v| p.servers = v),
        );
        Body::new()
            .playground(live)
            .hint("Type in Search settings to filter; press an item to open it")
            .config(config)
            .example(controls::block(
                "Reference preview",
                div().w_full().h(rems(26.25)).child(
                    dialog("sdlg-ref", Some("agents".into()), true)
                        .child(agents("sdlg-ref-agents")),
                ),
            ))
    }
}
