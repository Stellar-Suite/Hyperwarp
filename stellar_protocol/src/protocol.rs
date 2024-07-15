
use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use serde_repr::Deserialize_repr;
use serde_repr::Serialize_repr;
use strum_macros::{EnumString, Display, EnumIter, VariantArray};

#[derive(Serialize, Deserialize, PartialEq, Debug, EnumString, Display, EnumIter, VariantArray, Hash, Eq, Clone, Copy)]
pub enum StellarChannel {
    Frame,
    Synchornizations,
    WindowChanges, // synchronizations is more reliable for now
    Signaling
}

#[derive(Serialize, Deserialize, PartialEq, Debug, EnumString, Display, EnumIter, VariantArray, Hash, Eq, Clone, Copy)]
pub enum GraphicsAPI {
    Unknown,
    OpenGL,
    Vulkan,
    DirectX, // ewww I hope not
    Metal, // idk if i'll ever be able to
}

#[derive(Serialize, Deserialize, PartialEq, Debug, EnumString, Display, EnumIter, VariantArray, Hash, Eq, Clone, Copy)]
pub enum EncodingPreset {
    H264,
    H265,
    VP8,
    VP9, // bad
    AV1,
    Unknown, // dangerously passthrough all settings
}

#[derive(Serialize, Deserialize, PartialEq, Debug, EnumString, Display, EnumIter, VariantArray, Hash, Eq, Clone, Copy)]
pub enum PipelineOptimization {
    None,
    Intel,
    NVIDIA,
    AMD
}

use strum::IntoEnumIterator;
use strum::VariantArray;

use crate::util;
pub fn get_all_channels() -> Vec<StellarChannel> {
    let vec: Vec<StellarChannel> = StellarChannel::iter().collect();
    vec
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Handshake {
    pub resolution: (u32, u32),
    pub shimg_path: PathBuf,
    pub graphics_api: GraphicsAPI,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Copy, Clone)]
#[repr(u8)]
pub enum SessionState {
    Initalizing = 0,
    Handshaking = 1,
    Ready = 2,
    Disconnecting = 9,
}

pub fn session_state_to_u8(state: SessionState) -> u8 {
    match state {
        SessionState::Initalizing => 0,
        SessionState::Handshaking => 1,
        SessionState::Ready => 2,
        SessionState::Disconnecting => 9,
    }
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Copy, Clone)]
#[repr(u8)]
pub enum StreamerState {
    Initalizing = 0,
    Handshaking = 1,
    Running = 2,
}

pub fn streamer_state_to_u8(state: StreamerState) -> u8 {
    match state {
        StreamerState::Initalizing => 0,
        StreamerState::Handshaking => 1,
        StreamerState::Running => 2,
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct HostInfo {
    pub name: String,
    pub pid: u32,
    pub graphics_api: GraphicsAPI,
}

impl Default for HostInfo {
    fn default() -> Self {
        HostInfo {
            name: util::prog().unwrap_or_else(|| "unknown".to_string()),
            pid: std::process::id(),
            graphics_api: GraphicsAPI::Unknown,
        }
    }
}


// things that can change but all optional
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Synchornization {
    pub resolution: Option<(u32, u32)>,
    pub graphics_api: Option<GraphicsAPI>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct DebugInfo {
    pub message: String,
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
    DebugInfoRequest,
    DebugInfoResponse(DebugInfo),
    DebugInfoRequestV2,
    DebugInfoResponseV2(DebugInfo, String),
    UserInputEvent(InputEvent)
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(untagged)]
pub enum StellarFrontendMessage {
    Test {
        time: u64
    },
    Ping {
        ping_payload: String
    },
    // webrtc stuff reference: https://gitlab.freedesktop.org/gstreamer/gst-examples/-/blob/discontinued-for-monorepo/webrtc/sendrecv/gst-rust/src/main.rs?ref_type=heads#L51
    Ice {
        candidate: String,
        #[serde(rename = "sdpMLineIndex")]
        sdp_mline_index: u32,
    },
    Sdp {
        #[serde(rename = "type")]
        type_: String,
        sdp: String,
    },
    ProvisionWebRTC {
        rtc_provision_start: u64,
    },
    ProvisionWebRTCReply {
        provision_ok: bool
    },
    Error {
        error: String
    },
    DebugInfoRequest {
        debug_info_request: u64
    },
    DebugResponse {
        debug: String
    },
    OfferRequest {
        offer_request_source: String
    },
    DefineACL {
        acl: PrivligeDefinition,
        socket_id: String
    },
    HyperwarpDebugInfoRequest {
        hyperwarp_debug_info_request: u64
    },
    HyperwarpDebugResponse {
        hyperwarp_debug: String,
        source: String
    },
}

// js usable protocol
// bypass stargate so faster
// use rename for clientbound
#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(tag = "type")]
pub enum StellarDirectControlMessage {
    #[serde(rename = "update_window_title")]
    UpdateWindowTitle {
        title: String,
    },
    #[serde(alias = "keychange")]
    KeyChange {
        key: String,
        code: String,
        composition: bool,
        state: bool,
        timestamp: u64,
    },
    #[serde(rename = "update_window_size")]
    UpdateWindowSize {
        width: u32,
        height: u32,
    },
    #[serde(alias = "mouse_rel")]
    MouseMoveRelative {
        x: i32,
        y: i32,
        timestamp: u64,
    },
    #[serde(alias = "mouse_abs")]
    MouseMoveAbsolute { // tablet style input
        x: i32,
        y: i32,
        timestamp: u64,
    },
    #[serde(alias = "mouse_btn")]
    MouseButton {
        change: u8, // the button field in the js event
        buttons: u8, // the buttons field in the js event
        state: bool, // if it was an up or down event
        timestamp: u64,
    },
    #[serde(rename = "mouse_lock")]
    MouseLock {
        state: bool
    }
}

pub fn may_mutate_pipeline(message: &StellarFrontendMessage) -> bool {
    match message {
        StellarFrontendMessage::Ice { candidate, sdp_mline_index } => true,
        StellarFrontendMessage::Sdp { type_, sdp } => true,
        StellarFrontendMessage::ProvisionWebRTC { rtc_provision_start } => true,
        StellarFrontendMessage::DebugInfoRequest { debug_info_request } => true,
        StellarFrontendMessage::OfferRequest { offer_request_source } => true,
        _ => false
    }
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

#[derive(Serialize, Deserialize, PartialEq, Copy, Clone, Debug)]
pub struct PrivligeDefinition {
    pub can_chat: bool,
    pub can_resize: bool,
    pub can_mouse: bool,
    pub can_touchscreen: bool,
    pub can_keyboard: bool,
    pub can_controller: bool,
    pub can_manage_controllers: bool,
    pub can_admin: bool,
}

// TODO: adopt a strict by default model, for now it's full for debug
impl Default for PrivligeDefinition {
    fn default() -> Self {
        create_default_acl()
    }
}

pub const fn create_default_acl() -> PrivligeDefinition {
    PrivligeDefinition {
        can_chat: true,
        can_resize: true,
        can_mouse: true,
        can_touchscreen: true,
        can_keyboard: true,
        can_controller: true,
        can_manage_controllers: true,
        can_admin: true,
    }
}

// input
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputEvent {
    pub payload: InputEventPayload,
    pub metadata: InputMetadata,
    pub context: Option<InputContext>,
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputContext {
    pub modifiers: u16,
    pub buttons: u32,
}

impl InputContext {
    pub fn new() -> InputContext {
        InputContext {
            modifiers: 0,
            buttons: 0,
        }
    }
}

impl InputEvent {
    // untimestamped blank input event
    pub fn new(payload: InputEventPayload) -> InputEvent {
        let input_event = InputEvent {
            payload,
            metadata: InputMetadata::new(),
            context: None,
        };
        input_event
    }

    pub fn add_context(&mut self, context: InputContext) {
        self.context = Some(context);
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputMetadata {
    pub sdl2_timestamp_ticks: Option<u32>,
    pub sdl2_timestamp_ticks_u64: Option<u64>,
    pub sdl3_timestamp_ticks: Option<u32>,
    pub sdl3_timestamp_ticks_u64: Option<u64>,
}


impl InputMetadata {
    pub fn new() -> InputMetadata {
        InputMetadata {
            sdl2_timestamp_ticks: None,
            sdl2_timestamp_ticks_u64: None,
            // heh I will not deal with this for a while
            sdl3_timestamp_ticks: None,
            sdl3_timestamp_ticks_u64: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InputEventPayload {
    MouseMoveRelative {
        x: i32,
        y: i32,
        x_absolute: i32,
        y_absolute: i32,
    },
    MouseMoveAbsolute(i32, i32, i32, i32),
    KeyEvent {
        key: u32,
        scancode: u32,
        state: bool,
        modifiers: u16,
    },
    KeyEventLite {
        key: u32,
        state: bool,
    },
    MouseButtonsSet {
        buttons: u8,
    },
    MouseButtonsChange {
        change: u8,
        state: bool,
    }
}