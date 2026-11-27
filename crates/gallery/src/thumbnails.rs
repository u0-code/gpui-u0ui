//! The overview's specimens: one small live drawing per page (like md3.io's
//! component overview cards), registered with `PageDef::thumbnail`.
//!
//! Each specimen uses the real u0ui component, closed and at rest, sized for
//! a 160px-tall, ~240px-wide area on `background` (so components keep their
//! default `LayerLevel::One`). The overview covers them, so they are inert.
//! Foundations specimens draw tokens only. Ids are prefixed `thumb-<name>-`;
//! components with state keep it in `window.use_keyed_state`.
use gpui_kit::{
    AnyElement, App, Div, Focusable as _, IntoElement, ParentElement, SharedString, Styled, Window,
    div, px, relative, rems,
};
use std::rc::Rc;
use u0ui::components::accordion::{Accordion, AccordionItem};
use u0ui::components::alert::{Alert, AlertVariant, NotificationKind};
use u0ui::components::alert_dialog::AlertDialog;
use u0ui::components::aspect_ratio::AspectRatio;
use u0ui::components::attachment::{
    Attachment, AttachmentAction, AttachmentActions, AttachmentContent, AttachmentDescription,
    AttachmentGroup, AttachmentMedia, AttachmentTitle,
};
use u0ui::components::avatar::{Avatar, AvatarColor, AvatarFallback, AvatarGroup, AvatarSize};
use u0ui::components::badge::{Badge, BadgeVariant};
use u0ui::components::breadcrumb::{Breadcrumb, BreadcrumbItem};
use u0ui::components::bubble::{Bubble, BubbleAlign, BubbleGroup, BubbleVariant};
use u0ui::components::button_group::ButtonGroup;
use u0ui::components::calendar::{Calendar, CalendarDate, Selection};
use u0ui::components::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use u0ui::components::carousel::{
    Carousel, CarouselContent, CarouselItem, CarouselNext, CarouselPrevious,
};
use u0ui::components::chart::{Chart, ChartRow, ChartSeries};
use u0ui::components::checkbox::Checkbox;
use u0ui::components::code_block::CodeBlock;
use u0ui::components::collapsible::Collapsible;
use u0ui::components::combobox::Combobox;
use u0ui::components::command::{Command, CommandGroup, CommandItem};
use u0ui::components::context_menu::ContextMenu;
use u0ui::components::data_table::{DataTable, DataTableColumn, DataTableRow};
use u0ui::components::date_picker::DatePicker;
use u0ui::components::dialog::Dialog;
use u0ui::components::direction::{Direction, DirectionProvider};
use u0ui::components::drawer::Drawer;
use u0ui::components::dropdown_menu::{DropdownMenu, DropdownMenuKind, DropdownMenuSize};
use u0ui::components::empty::{Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle};
use u0ui::components::field::{Field, FieldDescription, FieldLabel};
use u0ui::components::hover_card::HoverCard;
use u0ui::components::input::{Input, InputState};
use u0ui::components::input_group::{AddonAlign, InputGroup, InputGroupAddon, InputGroupInput};
use u0ui::components::input_otp::InputOtp;
use u0ui::components::item::{
    Item, ItemActions, ItemContent, ItemDescription, ItemMedia, ItemMediaVariant, ItemTitle,
    ItemVariant,
};
use u0ui::components::kbd::{Kbd, KbdGroup};
use u0ui::components::label::Label;
use u0ui::components::marker::{Marker, MarkerContent, MarkerIcon, MarkerVariant};
use u0ui::components::menu::MenuItem;
use u0ui::components::menubar::Menubar;
use u0ui::components::message::{
    Message, MessageAvatar, MessageContent, MessageGroup, MessageHeader,
};
use u0ui::components::message_scroller::{MessageScroller, MessageScrollerItem};
use u0ui::components::native_select::NativeSelect;
use u0ui::components::navigation_menu::{NavItem, NavigationMenu};
use u0ui::components::pagination::PaginationNav;
use u0ui::components::popover::Popover;
use u0ui::components::progress::Progress;
use u0ui::components::questionnaire::{
    QuestionKind, Questionnaire, QuestionnaireItem, QuestionnaireTexts,
};
use u0ui::components::radio_group::{RadioGroup, RadioGroupItem};
use u0ui::components::resizable::{ResizableHandle, ResizablePanel, ResizablePanelGroup};
use u0ui::components::scroll_area::ScrollArea;
use u0ui::components::select::Select;
use u0ui::components::separator::Separator;
use u0ui::components::sheet::Sheet;
use u0ui::components::sidebar::{SideNavItem, Sidebar};
use u0ui::components::slider::{Slider, SliderState};
use u0ui::components::spinner::{Spinner, SpinnerSize, SpinnerStatus};
use u0ui::components::switch::Switch;
use u0ui::components::table::{
    Table, TableBody, TableCell, TableHead, TableHeader, TableRow, TableSize,
};
use u0ui::components::tabs::{Tabs, TabsTrigger};
use u0ui::components::textarea::{Textarea, TextareaState};
use u0ui::components::toast::Toast;
use u0ui::components::toggle::{Toggle, ToggleVariant};
use u0ui::components::toggle_group::{ToggleGroup, ToggleGroupItem};
use u0ui::components::tooltip::Tooltip;
use u0ui::components::trace::{Trace, TraceDescription, TraceLine, TraceStatus, TraceTitle};
use u0ui::components::typography::{Inline, TypeBlock, Typography};
use u0ui::prelude::*;

/// A column with `spacing-03` between its children.
fn column() -> Div {
    div().flex().flex_col().gap(tokens::SPACING_03)
}

/// A row with `spacing-03` between its children.
fn row() -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(tokens::SPACING_03)
}

/// Secondary text beside a specimen.
fn text(s: impl Into<SharedString>) -> Div {
    div()
        .type_style(TypeStyle::BodyCompact01)
        .text_color(color(ColorRole::TextPrimary))
        .child(s.into())
}

/// A specimen taller than the area, clipped to show its top.
fn clipped(width: f32) -> Div {
    div()
        .w(rems(width))
        .h(px(160.))
        .flex()
        .flex_col()
        .justify_start()
        .overflow_hidden()
}

// Foundations.

pub fn colors(_: &mut Window, _: &mut App) -> AnyElement {
    const ROLES: [ColorRole; 12] = [
        ColorRole::Background,
        ColorRole::Layer01,
        ColorRole::Layer02,
        ColorRole::Layer03,
        ColorRole::Interactive,
        ColorRole::Focus,
        ColorRole::SupportError,
        ColorRole::SupportSuccess,
        ColorRole::SupportWarning,
        ColorRole::SupportInfo,
        ColorRole::BorderStrong01,
        ColorRole::TextPrimary,
    ];
    div()
        .flex()
        .flex_row()
        .flex_wrap()
        .w(rems(13.5))
        .gap(tokens::SPACING_02)
        .children(ROLES.map(|role| {
            div()
                .size(tokens::SPACING_09)
                .bg(color(role))
                .border_1()
                .border_color(color(ColorRole::BorderSubtle01))
        }))
        .into_any_element()
}

pub fn type_styles(_: &mut Window, _: &mut App) -> AnyElement {
    let sample = |style: TypeStyle, s: &'static str| {
        div()
            .type_style(style)
            .text_color(color(ColorRole::TextPrimary))
            .child(s)
    };
    row()
        .gap(tokens::SPACING_05)
        .child(sample(TypeStyle::Heading06, "Aa"))
        .child(
            column()
                .gap(tokens::SPACING_02)
                .child(sample(TypeStyle::Heading03, "Heading"))
                .child(sample(TypeStyle::BodyCompact01, "Body compact"))
                .child(sample(TypeStyle::Code01, "code-01"))
                .child(
                    div()
                        .type_style(TypeStyle::Label01)
                        .text_color(color(ColorRole::TextSecondary))
                        .child("label-01"),
                ),
        )
        .into_any_element()
}

pub fn layout(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .items_end()
        .gap(tokens::SPACING_02)
        .children(tokens::SPACING_TOKENS.iter().take(10).map(|(_, value)| {
            div()
                .w(tokens::SPACING_05)
                .h(*value)
                .bg(color(ColorRole::Interactive))
        }))
        .into_any_element()
}

pub fn motion(_: &mut Window, _: &mut App) -> AnyElement {
    let longest = Speed::ALL.iter().map(|s| s.millis()).max().unwrap_or(1) as f32;
    column()
        .w(rems(13.))
        .children(Speed::ALL.map(|speed| {
            row()
                .child(
                    div()
                        .w(rems(5.))
                        .flex_none()
                        .type_style(TypeStyle::Code01)
                        .text_color(color(ColorRole::TextSecondary))
                        .child(speed.name()),
                )
                .child(
                    div().flex_1().child(
                        div()
                            .h(tokens::SPACING_03)
                            .w(relative(speed.millis() as f32 / longest))
                            .bg(color(ColorRole::Interactive)),
                    ),
                )
        }))
        .into_any_element()
}

pub fn icons(_: &mut Window, _: &mut App) -> AnyElement {
    const ICONS: [CarbonIcon; 12] = [
        CarbonIcon::Add,
        CarbonIcon::Search,
        CarbonIcon::Settings,
        CarbonIcon::User,
        CarbonIcon::Folder,
        CarbonIcon::Dashboard,
        CarbonIcon::Notification,
        CarbonIcon::Download,
        CarbonIcon::Chat,
        CarbonIcon::Code,
        CarbonIcon::Home,
        CarbonIcon::Star,
    ];
    div()
        .flex()
        .flex_row()
        .flex_wrap()
        .w(rems(13.))
        .gap(tokens::SPACING_05)
        .children(ICONS.map(|icon| {
            Icon::carbon(icon)
                .size(IconSize::S24)
                .color(color(ColorRole::IconPrimary))
        }))
        .into_any_element()
}

// Actions.

pub fn button(_: &mut Window, _: &mut App) -> AnyElement {
    column()
        .items_start()
        .child(Button::new("thumb-button-create", "Create").icon(CarbonIcon::Add))
        .child(Button::new("thumb-button-secondary", "Secondary").variant(ButtonVariant::Secondary))
        .into_any_element()
}

pub fn button_group(_: &mut Window, _: &mut App) -> AnyElement {
    ButtonGroup::new()
        .button(
            Button::new("thumb-button-group-cancel", "Cancel").variant(ButtonVariant::Secondary),
        )
        .button(Button::new("thumb-button-group-save", "Save"))
        .into_any_element()
}

pub fn toggle(_: &mut Window, _: &mut App) -> AnyElement {
    row()
        .child(
            Toggle::new("thumb-toggle-bold")
                .aria_label("Bold")
                .default_pressed(true)
                // `h('b', null, 'B')`: the Carbon reset (`b, i { font: inherit }`) keeps
                // it at the toggle's normal weight, upright.
                .child("B"),
        )
        .child(
            Toggle::new("thumb-toggle-italic")
                .aria_label("Italic")
                .child("I"),
        )
        .child(
            Toggle::new("thumb-toggle-pin")
                .variant(ToggleVariant::Outline)
                .aria_label("Pin")
                .child("Pin"),
        )
        .into_any_element()
}

pub fn toggle_group(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(13.))
        .child(
            ToggleGroup::new("thumb-toggle-group-range")
                .default_value(["week"])
                .item(ToggleGroupItem::new("day").child("Day"))
                .item(ToggleGroupItem::new("week").child("Week"))
                .item(ToggleGroupItem::new("month").child("Month")),
        )
        .into_any_element()
}

// Forms.

pub fn input(window: &mut Window, cx: &mut App) -> AnyElement {
    let state = window.use_keyed_state("thumb-input-state", cx, |window, cx| {
        InputState::new(window, cx).default_value("prod-us-east")
    });
    div()
        .w(rems(13.))
        .child(
            Input::new("thumb-input-name", &state)
                .label("Cluster name")
                .placeholder("e.g. prod-us-east")
                .helper_text("Lowercase letters and hyphens."),
        )
        .into_any_element()
}

pub fn textarea(window: &mut Window, cx: &mut App) -> AnyElement {
    let state = window.use_keyed_state("thumb-textarea-state", cx, |window, cx| {
        TextareaState::new(window, cx)
    });
    div()
        .w(rems(13.))
        .child(
            Textarea::new("thumb-textarea-desc", &state)
                .label("Description")
                .placeholder("What is this service for?")
                .rows(2),
        )
        .into_any_element()
}

pub fn label(window: &mut Window, cx: &mut App) -> AnyElement {
    let state = window.use_keyed_state("thumb-label-state", cx, |window, cx| {
        InputState::new(window, cx)
    });
    let focus = state.read(cx).focus_handle(cx);
    column()
        .gap(px(0.))
        .w(rems(13.))
        .child(Label::new("Region").target(&focus))
        .child(
            Input::new("thumb-label-input", &state)
                .label("Region")
                .hide_label(true)
                .placeholder("us-south"),
        )
        .into_any_element()
}

pub fn checkbox(_: &mut Window, _: &mut App) -> AnyElement {
    column()
        .child(
            Checkbox::new("thumb-checkbox-notes")
                .label("Send release notes")
                .default_checked(true),
        )
        .child(Checkbox::new("thumb-checkbox-audit").label("Enable audit logs"))
        .child(
            Checkbox::new("thumb-checkbox-all")
                .label("Select all")
                .indeterminate(true),
        )
        .into_any_element()
}

pub fn radio_group(_: &mut Window, _: &mut App) -> AnyElement {
    RadioGroup::new("thumb-radio-group-plan")
        .legend("Plan")
        .default_value("std")
        .item(RadioGroupItem::new("lite").label("Lite"))
        .item(RadioGroupItem::new("std").label("Standard"))
        .item(
            RadioGroupItem::new("ent")
                .label("Enterprise")
                .disabled(true),
        )
        .into_any_element()
}

pub fn select(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(13.))
        .child(
            Select::new("thumb-select-region")
                .label("Region")
                .items([("fra", "Frankfurt"), ("lon", "London"), ("dal", "Dallas")])
                .default_value("fra")
                .placeholder("Choose an option"),
        )
        .into_any_element()
}

pub fn native_select(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(13.))
        .child(
            NativeSelect::new("thumb-native-select-region")
                .label("Region")
                .group("Europe", [("fra", "Frankfurt"), ("lon", "London")])
                .group("Americas", [("dal", "Dallas")])
                .default_value("fra"),
        )
        .into_any_element()
}

pub fn combobox(window: &mut Window, cx: &mut App) -> AnyElement {
    let state = window.use_keyed_state("thumb-combobox-state", cx, |window, cx| {
        InputState::new(window, cx)
    });
    div()
        .w(rems(13.))
        .child(
            Combobox::new("thumb-combobox-framework", &state)
                .label("Framework")
                .items(["React", "Angular", "Vue"])
                .default_value(["React"])
                .placeholder("Filter…")
                .clear_label("Clear selected item"),
        )
        .into_any_element()
}

pub fn calendar(_: &mut Window, _: &mut App) -> AnyElement {
    // The 288px panel is wider than the area: centered, its edges clip.
    clipped(18.)
        .flex_none()
        .child(
            Calendar::new("thumb-calendar-month")
                .default_value(Selection::single(CalendarDate::new(2026, 11, 26)))
                .nav_labels("Previous month", "Next month"),
        )
        .into_any_element()
}

pub fn date_picker(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(13.))
        .child(
            DatePicker::new("thumb-date-picker-start")
                .label("Start date")
                .placeholder("mm/dd/yyyy"),
        )
        .into_any_element()
}

pub fn input_group(window: &mut Window, cx: &mut App) -> AnyElement {
    let state = window.use_keyed_state("thumb-input-group-state", cx, |window, cx| {
        InputState::new(window, cx)
    });
    div()
        .w(rems(13.))
        .child(
            InputGroup::new("thumb-input-group-search")
                .addon(
                    InputGroupAddon::new()
                        .child(Icon::carbon(CarbonIcon::Search).size(IconSize::S16)),
                )
                .input(InputGroupInput::new(&state).placeholder("Search"))
                .addon(
                    InputGroupAddon::new()
                        .align(AddonAlign::InlineEnd)
                        .child(Kbd::new("⌘K")),
                ),
        )
        .into_any_element()
}

pub fn field(window: &mut Window, cx: &mut App) -> AnyElement {
    let state = window.use_keyed_state("thumb-field-state", cx, |window, cx| {
        InputState::new(window, cx)
    });
    div()
        .w(rems(13.))
        .child(
            Field::new()
                .child(FieldLabel::new("Full name"))
                .child(
                    InputGroup::new("thumb-field-name")
                        .input(InputGroupInput::new(&state).placeholder("Jane Doe")),
                )
                .child(FieldDescription::new("As shown on the invoice.")),
        )
        .into_any_element()
}

pub fn input_otp(window: &mut Window, cx: &mut App) -> AnyElement {
    let state = window.use_keyed_state("thumb-input-otp-state", cx, |window, cx| {
        InputState::new(window, cx).default_value("12")
    });
    InputOtp::new("thumb-input-otp-code", &state)
        .length(4)
        .label("One-time code")
        .into_any_element()
}

pub fn switch(_: &mut Window, _: &mut App) -> AnyElement {
    column()
        .gap(tokens::SPACING_05)
        .child(
            Switch::new("thumb-switch-autoscale")
                .label("Auto-scaling")
                .label_on("On")
                .label_off("Off")
                .default_checked(true),
        )
        .child(
            Switch::new("thumb-switch-public")
                .label("Public endpoint")
                .label_on("On")
                .label_off("Off"),
        )
        .into_any_element()
}

pub fn slider(window: &mut Window, cx: &mut App) -> AnyElement {
    let state = window.use_keyed_state("thumb-slider-state", cx, |_, _| {
        SliderState::new()
            .min(0.)
            .max(64.)
            .step(1.)
            .default_value(24.)
    });
    div()
        .w(rems(13.))
        .child(
            Slider::new("thumb-slider-memory", &state)
                .label("Memory (GB)")
                .hide_input(true),
        )
        .into_any_element()
}

// Overlays: the closed component with its real trigger.

pub fn dialog(_: &mut Window, _: &mut App) -> AnyElement {
    Dialog::new("thumb-dialog-rename")
        .trigger(Button::new("thumb-dialog-open", "Open dialog"))
        .label("Settings")
        .title("Rename project")
        .primary_text("Save")
        .secondary_text("Cancel")
        .into_any_element()
}

pub fn alert_dialog(_: &mut Window, _: &mut App) -> AnyElement {
    AlertDialog::new("thumb-alert-dialog-delete")
        .trigger(
            Button::new("thumb-alert-dialog-open", "Delete bucket")
                .variant(ButtonVariant::Destructive),
        )
        .label("Storage")
        .title("Delete logs-2026?")
        .description("This cannot be undone.")
        .action_text("Delete bucket")
        .cancel_text("Cancel")
        .into_any_element()
}

pub fn sheet(_: &mut Window, _: &mut App) -> AnyElement {
    Sheet::new("thumb-sheet-edit")
        .trigger(Button::new("thumb-sheet-open", "Open sheet").variant(ButtonVariant::Tertiary))
        .title("Edit instance")
        .description("Changes apply after restart.")
        .into_any_element()
}

pub fn drawer(_: &mut Window, _: &mut App) -> AnyElement {
    Drawer::new("thumb-drawer-filters")
        .trigger(Button::new("thumb-drawer-open", "Open drawer").variant(ButtonVariant::Tertiary))
        .title("Filters")
        .into_any_element()
}

pub fn tooltip(_: &mut Window, _: &mut App) -> AnyElement {
    Tooltip::new(
        "thumb-tooltip-copy",
        Button::new("thumb-tooltip-button", "Copy")
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Md)
            .icon(CarbonIcon::Copy)
            .icon_only(true)
            .tooltip(false),
    )
    .content("Copy to clipboard")
    .into_any_element()
}

pub fn popover(_: &mut Window, _: &mut App) -> AnyElement {
    Popover::new(
        "thumb-popover-dimensions",
        Button::new("thumb-popover-button", "Dimensions")
            .variant(ButtonVariant::Outline)
            .size(ButtonSize::Md),
    )
    .child(text("Width 100%"))
    .into_any_element()
}

pub fn hover_card(_: &mut Window, _: &mut App) -> AnyElement {
    HoverCard::new(
        "thumb-hover-card-profile",
        Button::new("thumb-hover-card-link", "@carbon-design").variant(ButtonVariant::Link),
    )
    .child(text("Carbon Design System"))
    .into_any_element()
}

pub fn dropdown_menu(_: &mut Window, _: &mut App) -> AnyElement {
    DropdownMenu::new("thumb-dropdown-menu-actions")
        .label("Actions")
        .kind(DropdownMenuKind::Primary)
        .size(DropdownMenuSize::Md)
        .items(vec![
            MenuItem::item("Restart"),
            MenuItem::item("Resize").shortcut("⌘R"),
            MenuItem::divider(),
            MenuItem::item("Delete").danger(true),
        ])
        .into_any_element()
}

pub fn context_menu(_: &mut Window, _: &mut App) -> AnyElement {
    // The trigger area, as on the page (`preview.html`'s dashed box).
    let area = div()
        .w(rems(13.))
        .h(rems(7.))
        .flex()
        .items_center()
        .justify_center()
        .border_1()
        .border_dashed()
        .border_color(color(ColorRole::BorderStrong01))
        .type_style(TypeStyle::BodyCompact01)
        .text_color(color(ColorRole::TextSecondary))
        .child("Right-click here");
    ContextMenu::new("thumb-context-menu-area", area)
        .label("Context menu")
        .items(vec![
            MenuItem::item("Copy").shortcut("⌘C"),
            MenuItem::item("Rename"),
            MenuItem::divider(),
            MenuItem::item("Delete").danger(true),
        ])
        .into_any_element()
}

pub fn menubar(_: &mut Window, _: &mut App) -> AnyElement {
    Menubar::new("thumb-menubar-bar")
        .menu(
            "File",
            vec![
                MenuItem::item("New tab").shortcut("⌘T"),
                MenuItem::item("Open…").shortcut("⌘O"),
            ],
        )
        .menu(
            "View",
            vec![MenuItem::checkbox("Show grid").default_selected(true)],
        )
        .menu("Help", vec![MenuItem::item("Documentation")])
        .into_any_element()
}

pub fn command(_: &mut Window, _: &mut App) -> AnyElement {
    // The inline palette (`components/Command/preview.html`).
    clipped(14.)
        .child(
            Command::new("thumb-command-palette").group(
                CommandGroup::new("Suggestions")
                    .item(CommandItem::new("Open dashboard").icon(CarbonIcon::Dashboard))
                    .item(
                        CommandItem::new("Create resource")
                            .icon(CarbonIcon::Add)
                            .shortcut("⌘N"),
                    )
                    .item(CommandItem::new("Search docs").icon(CarbonIcon::Search)),
            ),
        )
        .into_any_element()
}

// Navigation.

pub fn tabs(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(14.))
        .child(
            Tabs::new("thumb-tabs-tabs")
                .default_value("ov")
                .trigger(TabsTrigger::new("ov", "Overview"))
                .trigger(TabsTrigger::new("met", "Metrics"))
                .content("ov", text("3 of 3 replicas ready."))
                .content("met", text("CPU 34%")),
        )
        .into_any_element()
}

pub fn accordion(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(14.))
        .child(
            Accordion::new("thumb-accordion-faq")
                .item(AccordionItem::new("Resource groups").child("Organize access and billing."))
                .item(AccordionItem::new("Billing").child("Per second."))
                .item(AccordionItem::new("Unavailable").disabled(true)),
        )
        .into_any_element()
}

pub fn collapsible(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(14.))
        .child(
            Collapsible::new("thumb-collapsible-advanced", "Advanced options")
                .default_open(true)
                .child(Checkbox::new("thumb-collapsible-ipv6").label("Enable IPv6"))
                .child(
                    Checkbox::new("thumb-collapsible-encrypt")
                        .label("Encrypt at rest")
                        .default_checked(true),
                ),
        )
        .into_any_element()
}

pub fn breadcrumb(_: &mut Window, _: &mut App) -> AnyElement {
    Breadcrumb::new("thumb-breadcrumb-path")
        .item(BreadcrumbItem::new("Resources"))
        .item(BreadcrumbItem::new("Storage"))
        .item(BreadcrumbItem::new("logs").current(true))
        .into_any_element()
}

pub fn pagination(_: &mut Window, _: &mut App) -> AnyElement {
    PaginationNav::new("thumb-pagination-nav", 12)
        .items_shown(5)
        .page(2)
        .into_any_element()
}

pub fn navigation_menu(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(15.))
        .overflow_hidden()
        .child(
            NavigationMenu::new("thumb-navigation-menu-header")
                .brand_prefix("IBM")
                .brand("[Platform]")
                .item(NavItem::link("Catalog").active(true)),
        )
        .into_any_element()
}

pub fn sidebar(_: &mut Window, _: &mut App) -> AnyElement {
    clipped(15.)
        .flex_row()
        .child(
            Sidebar::new("thumb-sidebar-nav")
                .header("Cloud console")
                .item(
                    SideNavItem::link("Dashboard")
                        .icon(CarbonIcon::Dashboard)
                        .active(true),
                )
                .item(SideNavItem::link("Resources").icon(CarbonIcon::Folder))
                .item(SideNavItem::link("Settings").icon(CarbonIcon::Settings)),
        )
        .into_any_element()
}

// Layout.

pub fn aspect_ratio(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(12.))
        .child(
            AspectRatio::new(16. / 9.).child(
                div()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(color(ColorRole::Layer01))
                    .type_style(TypeStyle::BodyCompact01)
                    .text_color(color(ColorRole::TextSecondary))
                    .child("16 : 9"),
            ),
        )
        .into_any_element()
}

pub fn separator(_: &mut Window, _: &mut App) -> AnyElement {
    column()
        .gap(px(0.))
        .w(rems(13.))
        .child(text("Account"))
        .child(div().my(rems(0.75)).child(Separator::new()))
        .child(
            row()
                .gap(rems(0.75))
                .h(rems(1.25))
                .child(text("Profile"))
                .child(Separator::vertical())
                .child(text("Billing")),
        )
        .into_any_element()
}

pub fn scroll_area(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .flex_none()
        .border_1()
        .border_color(color(ColorRole::BorderSubtle01))
        .child(
            ScrollArea::new("thumb-scroll-area-releases")
                .h(rems(8.5))
                .w(rems(13.))
                .child(div().p(tokens::SPACING_05).children((0..12).map(|i| {
                    div()
                        .py(rems(0.375))
                        .border_b_1()
                        .border_color(color(ColorRole::BorderSubtle01))
                        .type_style(TypeStyle::BodyCompact01)
                        .text_color(color(ColorRole::TextPrimary))
                        .child(format!("v1.{}.0", 12 - i))
                }))),
        )
        .into_any_element()
}

pub fn resizable(_: &mut Window, _: &mut App) -> AnyElement {
    let pane = |label: &'static str| text(label).p(tokens::SPACING_05);
    div()
        .w(rems(14.))
        .h(rems(8.))
        .border_1()
        .border_color(color(ColorRole::BorderSubtle01))
        .child(
            ResizablePanelGroup::new("thumb-resizable-group")
                .panel(
                    ResizablePanel::new()
                        .default_size(40.)
                        .child(pane("Sidebar")),
                )
                .handle(ResizableHandle::new().with_handle(true))
                .panel(
                    ResizablePanel::new()
                        .default_size(60.)
                        .child(pane("Editor")),
                ),
        )
        .into_any_element()
}

pub fn direction(window: &mut Window, cx: &mut App) -> AnyElement {
    let state = window.use_keyed_state("thumb-direction-state", cx, |window, cx| {
        InputState::new(window, cx)
    });
    div()
        .w(rems(13.))
        .child(
            DirectionProvider::new(Direction::Rtl).child(
                column()
                    .child(
                        Breadcrumb::new("thumb-direction-crumb")
                            .item(BreadcrumbItem::new("الرئيسية"))
                            .item(BreadcrumbItem::new("الإعدادات").current(true)),
                    )
                    .child(
                        Input::new("thumb-direction-input", &state)
                            .label("الاسم")
                            .hide_label(true),
                    ),
            ),
        )
        .into_any_element()
}

pub fn carousel(_: &mut Window, _: &mut App) -> AnyElement {
    const TITLES: [&str; 3] = ["Compute", "Storage", "Networking"];
    div()
        .w(rems(14.))
        .child(
            Carousel::new("thumb-carousel-slides")
                .label("Carousel")
                .content(
                    CarouselContent::new()
                        .items((0..TITLES.len()).map(|i| {
                            CarouselItem::new().child(
                                Card::new(SharedString::from(format!("thumb-carousel-card-{i}")))
                                    .h(rems(5.5))
                                    .child(
                                        CardHeader::new().child(CardTitle::new(TITLES[i])).child(
                                            CardDescription::new(format!(
                                                "Slide {} of {}",
                                                i + 1,
                                                TITLES.len()
                                            )),
                                        ),
                                    ),
                            )
                        }))
                        .dot_labels(|i| format!("Slide {}", i + 1).into()),
                )
                .previous(CarouselPrevious::new("Previous slide"))
                .next(CarouselNext::new("Next slide")),
        )
        .into_any_element()
}

// Data display.

pub fn kbd(_: &mut Window, _: &mut App) -> AnyElement {
    column()
        .items_center()
        .gap(tokens::SPACING_05)
        .child(KbdGroup::new().child(Kbd::new("⌘")).child(Kbd::new("K")))
        .child(
            row()
                .gap(px(0.))
                .child(text("Press\u{a0}"))
                .child(Kbd::new("Esc"))
                .child(text("\u{a0}to close")),
        )
        .into_any_element()
}

pub fn badge(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .flex_wrap()
        .w(rems(14.))
        .gap(tokens::SPACING_03)
        .child(Badge::new("Default"))
        .child(Badge::new("Secondary").variant(BadgeVariant::Secondary))
        .child(Badge::new("Outline").variant(BadgeVariant::Outline))
        .child(Badge::new("Blue").variant(BadgeVariant::Blue))
        .child(Badge::new("Green").variant(BadgeVariant::Green))
        .child(Badge::new("Red").variant(BadgeVariant::Red))
        .into_any_element()
}

pub fn card(_: &mut Window, _: &mut App) -> AnyElement {
    Card::new("thumb-card-storage")
        .w(rems(13.))
        .child(
            CardHeader::new()
                .child(CardTitle::new("Object storage"))
                .child(CardDescription::new("Store unstructured data.")),
        )
        .child(CardContent::new().child("3 buckets · 1.2 TB"))
        .into_any_element()
}

pub fn chart(_: &mut Window, _: &mut App) -> AnyElement {
    let data = [
        ("Jan", 42., 30.),
        ("Feb", 51., 38.),
        ("Mar", 47., 45.),
        ("Apr", 60., 52.),
    ]
    .map(|(m, us, eu)| ChartRow::new().text("m", m).num("us", us).num("eu", eu))
    .to_vec();
    div()
        .w(rems(14.))
        .child(
            Chart::new("thumb-chart-bar")
                .x_key("m")
                .data(data)
                .series(vec![
                    ChartSeries::new("us").label("US"),
                    ChartSeries::new("eu").label("EU"),
                ])
                .height(120.),
        )
        .into_any_element()
}

pub fn avatar(_: &mut Window, _: &mut App) -> AnyElement {
    let fallback =
        |s: &'static str, c: AvatarColor| Avatar::new().fallback(AvatarFallback::new(s).color(c));
    row()
        .gap(tokens::SPACING_05)
        .child(fallback("JD", AvatarColor::Blue))
        .child(fallback("AK", AvatarColor::Teal).size(AvatarSize::Lg))
        .child(
            AvatarGroup::new()
                .avatar(fallback("LS", AvatarColor::Magenta))
                .avatar(fallback("TP", AvatarColor::Purple))
                .avatar(fallback("+3", AvatarColor::Gray)),
        )
        .into_any_element()
}

pub fn table(_: &mut Window, _: &mut App) -> AnyElement {
    const ROWS: [[&str; 2]; 3] = [
        ["api-gateway", "Running"],
        ["auth-service", "Running"],
        ["billing", "Failed"],
    ];
    div()
        .w(rems(14.))
        .child(
            Table::new("thumb-table-services")
                .size(TableSize::Sm)
                .header(
                    TableHeader::new()
                        .child(TableRow::new().children(["Name", "Status"].map(TableHead::new))),
                )
                .body(TableBody::new().children(ROWS.map(|r| {
                    let tone = if r[1] == "Running" {
                        BadgeVariant::Green
                    } else {
                        BadgeVariant::Red
                    };
                    TableRow::new()
                        .child(TableCell::new(r[0]))
                        .child(TableCell::new(Badge::new(r[1]).variant(tone)))
                }))),
        )
        .into_any_element()
}

pub fn data_table(_: &mut Window, _: &mut App) -> AnyElement {
    const ZONE: [&str; 3] = ["us-south-1", "eu-de-2", "jp-tok-3"];
    div()
        .w(rems(14.))
        .child(
            DataTable::new("thumb-data-table-servers")
                .size(TableSize::Sm)
                .selectable(true)
                .search(false)
                .columns(vec![
                    DataTableColumn::new("name", "Name"),
                    DataTableColumn::new("zone", "Zone"),
                ])
                .rows(
                    (1..=3)
                        .map(|i| {
                            DataTableRow::new(i.to_string())
                                .cell("name", format!("vm-{}", 100 + i))
                                .cell("zone", ZONE[i % 3])
                        })
                        .collect(),
                ),
        )
        .into_any_element()
}

pub fn typography(_: &mut Window, _: &mut App) -> AnyElement {
    clipped(14.)
        .child(Typography::new("thumb-typography-doc").blocks([
            TypeBlock::Heading(3, "Getting started".into()),
            TypeBlock::Paragraph(vec![
                "Use ".into(),
                Inline::Code("body-01".into()),
                " for product copy.".into(),
            ]),
            TypeBlock::Quote("Design is the intermediary.".into()),
        ]))
        .into_any_element()
}

pub fn code_block(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(14.))
        .child(
            CodeBlock::new("thumb-code-block-install", "npm install carbon-ui")
                .title("install.sh")
                .language("bash")
                .copy_texts("Copy to clipboard", "Copied!"),
        )
        .into_any_element()
}

pub fn empty(_: &mut Window, _: &mut App) -> AnyElement {
    let media = EmptyMedia::new();
    let icon = Icon::carbon(CarbonIcon::Folder)
        .size(IconSize::S32)
        .color(media.icon_color());
    div()
        .w(rems(14.))
        .child(
            Empty::new().child(
                EmptyHeader::new()
                    .child(media.child(icon))
                    .child(EmptyTitle::new("No buckets yet"))
                    .child(EmptyDescription::new("Create one to start uploading.")),
            ),
        )
        .into_any_element()
}

pub fn item(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(14.))
        .child(
            Item::new("thumb-item-logs")
                .variant(ItemVariant::Outline)
                .child(
                    ItemMedia::new().variant(ItemMediaVariant::Icon).child(
                        Icon::carbon(CarbonIcon::Folder)
                            .size(IconSize::S16)
                            .color(ItemMedia::icon_color()),
                    ),
                )
                .child(
                    ItemContent::new()
                        .child(ItemTitle::new("logs-2026"))
                        .child(ItemDescription::new("12,408 objects")),
                )
                .child(
                    ItemActions::new().child(
                        Button::new("thumb-item-open", "Open")
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::Sm),
                    ),
                ),
        )
        .into_any_element()
}

// Feedback.

pub fn spinner(_: &mut Window, _: &mut App) -> AnyElement {
    column()
        .items_center()
        .gap(tokens::SPACING_05)
        // Still specimens: the overview repeats no animation.
        .child(
            Spinner::new("thumb-spinner-md")
                .label("Loading")
                .status(SpinnerStatus::Inactive),
        )
        .child(
            row()
                .gap(tokens::SPACING_06)
                .child(
                    Spinner::new("thumb-spinner-inline")
                        .size(SpinnerSize::Sm)
                        .description("Paused")
                        .status(SpinnerStatus::Inactive),
                )
                .child(
                    Spinner::new("thumb-spinner-saved")
                        .description("Saved")
                        .status(SpinnerStatus::Finished),
                ),
        )
        .into_any_element()
}

pub fn progress(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(13.))
        .child(
            Progress::new("thumb-progress-upload")
                .label("Uploading backup.tar")
                .value(62.)
                .helper_text("620 MB of 1 GB"),
        )
        .into_any_element()
}

pub fn alert(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(14.))
        .child(
            Alert::new("thumb-alert-quota")
                .variant(AlertVariant::Warning)
                .title("Quota at 90%")
                .description("Remove unused volumes."),
        )
        .into_any_element()
}

pub fn toast(_: &mut Window, _: &mut App) -> AnyElement {
    // The inline toast element (no timeout); its 18rem minimum clips at the edges.
    Toast::new("thumb-toast-backup", "Backup complete")
        .variant(NotificationKind::Success)
        .description("Snapshot saved.")
        .caption("Just now")
        .into_any_element()
}

// Chat & AI.

pub fn bubble(_: &mut Window, _: &mut App) -> AnyElement {
    BubbleGroup::new()
        .w(rems(14.))
        .child(
            Bubble::new()
                .align(BubbleAlign::End)
                .child("Summarize last week?"),
        )
        .child(
            Bubble::new()
                .variant(BubbleVariant::Secondary)
                .child("Two incidents, both resolved."),
        )
        .into_any_element()
}

pub fn message(_: &mut Window, _: &mut App) -> AnyElement {
    MessageGroup::new()
        .w(rems(14.))
        .child(
            Message::new().avatar(MessageAvatar::agent("AI")).content(
                MessageContent::new()
                    .child(MessageHeader::new().name("Assistant").child("10:42"))
                    .child(
                        Bubble::new()
                            .variant(BubbleVariant::Secondary)
                            .fill_message(true)
                            .child("eu-de has 14 idle VMs."),
                    ),
            ),
        )
        .into_any_element()
}

pub fn marker(_: &mut Window, _: &mut App) -> AnyElement {
    column()
        .gap(tokens::SPACING_05)
        .w(rems(14.))
        .child(
            Marker::new()
                .variant(MarkerVariant::Border)
                .child(
                    MarkerIcon::new()
                        .child(Icon::carbon(CarbonIcon::CheckmarkFilled).size(IconSize::S16)),
                )
                .child(MarkerContent::new().child("Conversation exported")),
        )
        .child(
            Marker::new()
                .variant(MarkerVariant::Separator)
                .child("Today"),
        )
        .into_any_element()
}

pub fn message_scroller(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(14.))
        .child(
            MessageScroller::new("thumb-message-scroller-list")
                .height(px(140.))
                .jump_label("Jump to latest")
                .children((0..6usize).map(|i| {
                    let reply = i % 2 == 1;
                    MessageScrollerItem::new(("thumb-message-scroller-item", i)).child(
                        Bubble::new()
                            .align(if reply {
                                BubbleAlign::Start
                            } else {
                                BubbleAlign::End
                            })
                            .variant(if reply {
                                BubbleVariant::Secondary
                            } else {
                                BubbleVariant::Default
                            })
                            .child(if reply {
                                format!("Reply {i}")
                            } else {
                                format!("Question {i}")
                            }),
                    )
                })),
        )
        .into_any_element()
}

pub fn attachment(_: &mut Window, _: &mut App) -> AnyElement {
    div()
        .w(rems(14.))
        .child(
            AttachmentGroup::new().child(
                Attachment::new()
                    .media(AttachmentMedia::new())
                    .content(
                        AttachmentContent::new()
                            .child(AttachmentTitle::new("report.pdf"))
                            .child(AttachmentDescription::new("2.4 MB")),
                    )
                    .actions(
                        AttachmentActions::new()
                            .child(AttachmentAction::new("thumb-attachment-remove", "Remove")),
                    ),
            ),
        )
        .into_any_element()
}

pub fn questionnaire(_: &mut Window, _: &mut App) -> AnyElement {
    let texts = QuestionnaireTexts {
        progress: Rc::new(|i, n| SharedString::from(format!("Question {i} of {n}"))),
        previous: "Previous".into(),
        skip: "Skip".into(),
        next: "Next".into(),
        submit: "Submit".into(),
        required: "Answer this question to continue.".into(),
    };
    let items = vec![
        QuestionnaireItem::new("team", "How big is your team?", QuestionKind::Single)
            .choices(["Just me", "2–10", "11–50"]),
    ];
    clipped(14.)
        .child(
            Questionnaire::new("thumb-questionnaire-setup", items, texts).title("Workspace setup"),
        )
        .into_any_element()
}

pub fn trace(_: &mut Window, _: &mut App) -> AnyElement {
    Trace::new()
        .w(rems(14.))
        .line(
            TraceLine::new("thumb-trace-search", TraceStatus::Done)
                .title(TraceTitle::new("Searched the docs"))
                .description(TraceDescription::new("4 documents matched")),
        )
        .line(
            TraceLine::new("thumb-trace-summary", TraceStatus::Pending)
                .title(TraceTitle::new("Summarizing")),
        )
        .line(
            TraceLine::new("thumb-trace-draft", TraceStatus::Pending)
                .title(TraceTitle::new("Draft answer")),
        )
        .into_any_element()
}
