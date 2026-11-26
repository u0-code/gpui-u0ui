//! Carbon components. One module per component; see `docs/ARCHITECTURE.md`
//! for the recipe ([`button`] is the reference implementation).
pub mod alert_dialog;
pub mod button;
pub mod button_group;
pub mod checkbox;
pub mod dialog;
pub mod drawer;
pub mod floating;
pub mod form;
pub mod hover_card;
pub mod input;
pub mod label;
pub mod overlay;
pub mod popover;
pub mod radio_group;
pub mod sheet;
pub mod switch;
pub mod toggle;
pub mod toggle_group;
pub mod tooltip;

pub use alert_dialog::AlertDialog;
pub use button::{Button, ButtonKind, ButtonSize, ButtonVariant};
pub use button_group::{
    ButtonGroup, ButtonGroupOrientation, ButtonGroupSeparator, ButtonGroupText,
};
pub use checkbox::{Checkbox, CheckboxState};
pub use dialog::{Dialog, DialogSize};
pub use drawer::Drawer;
pub use floating::Align;
pub use form::{FieldSize, Validation};
pub use hover_card::HoverCard;
pub use input::{Input, InputState};
pub use label::Label;
pub use popover::Popover;
pub use radio_group::{RadioGroup, RadioGroupItem, RadioOrientation};
pub use sheet::{Sheet, SheetSide};
pub use switch::{Switch, SwitchSize};
pub use toggle::{Toggle, ToggleVariant};
pub use toggle_group::{ToggleGroup, ToggleGroupItem, ToggleGroupType};
pub use tooltip::Tooltip;
