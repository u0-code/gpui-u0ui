//! `Ghost · Devices` (`src/ghost/admin.jsx`, "Devices"): pairing, paired
//! devices, Macs seen from elsewhere and the tunnel. Everything is
//! re-exported here, so callers write `u0ui::ghost::devices::PairingCard`.
mod pairing;
mod rows;

pub use pairing::{
    PairConfirmSheet, PairConfirmTexts, PairingCard, PairingCardTexts, QR_SIZE, countdown,
};
pub use rows::{
    DeviceRow, DeviceRowTexts, MacIcon, MacRow, MacRowTexts, MacState, TunnelButton, TunnelRow,
    TunnelRowTexts, TunnelState,
};
