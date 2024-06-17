
use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use strum_macros::{EnumString, Display, EnumIter, VariantArray};

#[derive(Serialize, Deserialize, PartialEq, Debug, EnumString, Display, EnumIter, VariantArray, Hash, Eq)]
pub enum StellarChannel {
    Frame,
    WindowChanges,
    Signaling
}

#[derive(Serialize, Deserialize, PartialEq, Debug, EnumString, Display, EnumIter, VariantArray, Hash, Eq)]
pub enum GraphicsAPI {
    Unknown,
    OpenGL,
    Vulkan,
    DirectX, // ewww I hope not
    Metal, // idk if i'll ever be able to
}

use strum::IntoEnumIterator;
use strum::VariantArray;
pub fn get_all_channels() -> Vec<StellarChannel> {
    let vec: Vec<StellarChannel> = StellarChannel::iter().collect();
    vec
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Handshake {
    pub resolution: (u32, u32),
    pub shimg_path: PathBuf,
}


// things that can change but all optional
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Synchornization {
    pub resolution: Option<(u32, u32)>,
}


#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum StellarMessage {
    Test,
    Hello,
    HelloName(String),
    Version,
    NewFrame,
    ToggleDebugOverlay,
    ToggleDebugOverlayResponse(bool),
    ShImgPathRequest,
    ShImgPathResponse(String),
    ShImgPathResponseStruct(PathBuf),
    ResolutionRequest,
    ResolutionBroadcastResponse(Option<(u32, u32)>),
    HandshakeRequest,
    HandshakeResponse(Handshake),
    SynchronizationEvent(Synchornization),
    SubscribeChannel(StellarChannel),
    UnsubscribeChannel(StellarChannel),
}

pub fn should_flip_buffers_for_graphics_api(gapi: GraphicsAPI) -> bool {
    match gapi {
        GraphicsAPI::OpenGL => true,
        GraphicsAPI::Vulkan => false,
        GraphicsAPI::DirectX => false,
        GraphicsAPI::Metal => false,
        GraphicsAPI::Unknown => false,
    }
}