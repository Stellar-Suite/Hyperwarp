
use serde::{Serialize, Deserialize};
use strum_macros::{EnumString, Display, EnumIter, VariantArray};

#[derive(Serialize, Deserialize, PartialEq, Debug, EnumString, Display, EnumIter, VariantArray)]
pub enum StellarChannel {
    Frame,
    WindowChanges,
    Signaling
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum StellarMessage {
    Test,
    Version,
    NewFrame,
    ToggleDebugOverlay,
    ToggleDebugOverlayResponse(bool),
}