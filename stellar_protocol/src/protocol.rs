
use serde::{Serialize, Deserialize};
use strum_macros::{EnumString, Display, EnumIter, VariantArray};

#[derive(Serialize, Deserialize, PartialEq, Debug, EnumString, Display, EnumIter, VariantArray, Hash, Eq)]
pub enum StellarChannel {
    Frame,
    WindowChanges,
    Signaling
}

use strum::IntoEnumIterator;
use strum::VariantArray;
pub fn get_all_channels() -> Vec<StellarChannel> {
    let vec: Vec<StellarChannel> = StellarChannel::iter().collect();
    vec
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum StellarMessage {
    Test,
    Version,
    NewFrame,
    ToggleDebugOverlay,
    ToggleDebugOverlayResponse(bool),
    RequestShImgPath,
    ShImgPathResponse(String),
    RequestResolutionBroadcast,
    ResolutionBroadcastResponse(Option<(u32, u32)>),
}