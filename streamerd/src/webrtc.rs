use std::{collections::HashMap, sync::Arc};

// tips: I mostly sourced this from https://github.com/GStreamer/gst-examples/blob/master/webrtc/multiparty-sendrecv/gst-rust/src/main.rs

use gstreamer::{prelude::*, Buffer, BufferFlags, Element, ErrorMessage};
use gstreamer_video::{prelude::*, VideoInfo};
use gstreamer_webrtc::WebRTCSessionDescription;
use stellar_protocol::protocol::{EncodingPreset, PipelineOptimization};

use lazy_static::lazy_static;

use crate::streamerd::StreamerConfig;

// https://gitlab.freedesktop.org/gstreamer/gstreamer/-/blob/main/subprojects/gst-examples/webrtc/multiparty-sendrecv/gst-rust/src/main.rs?ref_type=heads#L305
pub struct WebRTCPeer {
    pub id: String,
    pub queue: gstreamer::Element,
    pub webrtcbin: gstreamer::Element,
}

impl WebRTCPeer {
    pub fn new(id: String) -> Self {
        let webrtcbin = gstreamer::ElementFactory::make("webrtcbin").build().expect("could not create webrtcbin element");
        webrtcbin.set_property_from_str("stun-server", "stun://stun.l.google.com:19302");
        webrtcbin.set_property_from_str("bundle-policy", "max-bundle");
        Self {
            id,
            queue: gstreamer::ElementFactory::make("queue").build().expect("could not create queue element"),
            webrtcbin,
        }
    }

    pub fn play(&self) -> anyhow::Result<()> {
        self.queue.set_state(gstreamer::State::Playing)?;
        self.webrtcbin.set_state(gstreamer::State::Playing)?;
        Ok(())
    }

    pub fn set_stun_server(&self, stun_server: &str) {
        self.webrtcbin.set_property_from_str("stun-server", &stun_server);
    }

    pub fn setup_with_pipeline(&self, pipeline: &gstreamer::Pipeline, tee: &gstreamer::Element) {
        self.add_to_pipeline(pipeline);
        self.link_with_pipeline(pipeline, tee);
    }

    pub fn add_to_pipeline(&self, pipeline: &gstreamer::Pipeline) {
        pipeline.add_many([&self.queue, &self.webrtcbin]).expect("adding elements to pipeline failed");
    }

    pub fn link_with_pipeline(&self, pipeline: &gstreamer::Pipeline, tee: &gstreamer::Element) {
        gstreamer::Element::link_many([tee, &self.queue, &self.webrtcbin]).expect("linking elements failed");
    }

    pub fn remove_from_pipeline(&self, pipeline: &gstreamer::Pipeline, tee: &gstreamer::Element) {

        // unlink queue and webrtcbin
        gstreamer::Element::unlink_many([&tee, &self.queue, &self.webrtcbin]);

        // stop queue and webrtcbin
        self.queue.set_state(gstreamer::State::Null).expect("could not set queue state to null");
        self.webrtcbin.set_state(gstreamer::State::Null).expect("could not set webrtcbin state to null");

        // remove queue and webrtcbin from pipeline
        pipeline.remove(&self.queue).expect("removing queue element failed");
        pipeline.remove(&self.webrtcbin).expect("removing webrtcbin element failed");
    }

    pub fn set_local_description(&self, desc_ref: &WebRTCSessionDescription) {
        self.webrtcbin.emit_by_name::<()>("set-local-description", &[desc_ref, &None::<gstreamer::Promise>]);
    }

    pub fn set_remote_description(&self, desc_ref: &WebRTCSessionDescription) {
        self.webrtcbin.emit_by_name::<()>("set-remote-description", &[desc_ref, &None::<gstreamer::Promise>]);
    }

    pub fn process_sdp_answer(&self, sdp: &str) -> anyhow::Result<()> {
        let sdp_message = gstreamer_sdp::SDPMessage::parse_buffer(sdp.as_bytes())?;
        let answer = gstreamer_webrtc::WebRTCSessionDescription::new(gstreamer_webrtc::WebRTCSDPType::Answer, sdp_message);
        self.set_remote_description(&answer);
        Ok(())
    }

    pub fn process_sdp_offer(&self, sdp: &str, on_answer: Box<dyn FnOnce(WebRTCSessionDescription) + Send>) -> anyhow::Result<()> {
        let sdp_message = gstreamer_sdp::SDPMessage::parse_buffer(sdp.as_bytes())?;
        let answer = gstreamer_webrtc::WebRTCSessionDescription::new(gstreamer_webrtc::WebRTCSDPType::Answer, sdp_message);
        self.set_remote_description(&answer);
        let promise = gstreamer::Promise::with_change_func(move |reply| {
            match reply {
                Ok(answer_option) => {
                    match answer_option {
                        Some(struct_ref) => {
                            let answer_result = struct_ref
                                .value("answer")
                                .unwrap()
                                .get::<gstreamer_webrtc::WebRTCSessionDescription>();
                            match answer_result {
                                Ok(answer) => {
                                    on_answer(answer);
                                },
                                Err(err) => {
                                    println!("failed at getting answer struct: {:?}", err);
                                }
                            }
                        },
                        None => {
                            println!("no answer created");
                        }
                    }
                },
                Err(err) => {
                    println!("early error creating answer: {:?}", err);
                }
            }
        });
        self.webrtcbin.emit_by_name::<()>("create-answer", &[&None::<gstreamer::Structure>, &promise]);

        Ok(())
    }
}

pub struct WebRTCPreprocessor {
    pub encoder: gstreamer::Element,
    pub payloader: gstreamer::Element,
    pub extra_prefix_elements: Vec<gstreamer::Element>,
    pub extra_suffix_elements: Vec<gstreamer::Element>,
    pub preset: EncodingPreset,
    settings: HashMap<String, serde_json::Value>, // this allows us to set the settings for the encoder and payloader regardless of format easily
    pub config: Option<Arc<StreamerConfig>>,
}

lazy_static! {
    pub static ref SUPPORTS_TARGET_BITRATE: Vec<EncodingPreset> = vec![EncodingPreset::H264, EncodingPreset::H265, EncodingPreset::VP8, EncodingPreset::VP9];
    pub static ref SUPPORTS_BITRATE: Vec<EncodingPreset> = vec![EncodingPreset::H264, EncodingPreset::H265, EncodingPreset::VP8, EncodingPreset::VP9, EncodingPreset::AV1];
    pub static ref SUPPORTS_DEADLINE: Vec<EncodingPreset> = vec![EncodingPreset::VP8, EncodingPreset::VP9];
}

// TODO: pass config to this and do hardware accel
// this is fast enough for now
// also AV1 asm encoder go brrr?
impl WebRTCPreprocessor {
    pub fn new(encoder: gstreamer::Element, payloader: gstreamer::Element) -> Self {
        Self {
            encoder,
            payloader,
            preset: EncodingPreset::Unknown,
            settings: HashMap::new(),
            extra_prefix_elements: Vec::new(),
            extra_suffix_elements: Vec::new(),
            config: None,
        }
    }

    pub fn attach_to_pipeline(&self, pipeline: &gstreamer::Pipeline, after_element: &gstreamer::Element) {
        pipeline.add_many([&self.encoder, &self.payloader]).expect("adding elements to pipeline failed");
        // add prefix and suffix elements
        // I might just be able to add them directly with add_many but I'll check later
        for el in &self.extra_prefix_elements {
            pipeline.add_many([el]).expect("adding prefix elements to pipeline failed");
        }
        for el in &self.extra_suffix_elements {
            pipeline.add_many([el]).expect("adding suffix elements to pipeline failed");
        }

        let mut linkage = vec![after_element];
        for el in &self.extra_prefix_elements {
            linkage.push(el);
        }
        linkage.push(&self.encoder);
        linkage.push(&self.payloader);
        for el in &self.extra_suffix_elements {
            linkage.push(el);
        }

        gstreamer::Element::link_many(linkage).expect("linking elements failed");
    }

    pub fn get_last_element(&self) -> &gstreamer::Element {
        if self.extra_suffix_elements.len() > 0 {
            return &self.extra_suffix_elements[self.extra_suffix_elements.len() - 1];
        }
        &self.payloader
    }

    pub fn play(&self) -> anyhow::Result<()> {
        self.encoder.set_state(gstreamer::State::Playing)?;
        self.payloader.set_state(gstreamer::State::Playing)?;
        Ok(())
    }

    // maybe a better name for this is encodertype not preset
    pub fn new_preset(preset: EncodingPreset, optimizations: PipelineOptimization) -> Self {
        let mut prefix: Vec<gstreamer::Element> = vec![];
        let mut suffix: Vec<gstreamer::Element> = vec![];

        // TODO: support unknown
        let encoder_el_type = WebRTCPreprocessor::get_encoder_element_type(preset, optimizations);
        let payloader_el_type = WebRTCPreprocessor::get_payloader_element_type(preset, optimizations);

        match optimizations {
            PipelineOptimization::NVIDIA => {
                match preset {
                    EncodingPreset::H264 => {
                        suffix.push(gstreamer::ElementFactory::make("h264parse").build().expect("could not create h264parse element"));
                    },
                    EncodingPreset::H265 => {
                        suffix.push(gstreamer::ElementFactory::make("h265parse").build().expect("could not create h265parse element"));
                    },
                    _ => {

                    }
                }
            },
            _ => {

            }
        }

        Self {
            encoder: gstreamer::ElementFactory::make(&encoder_el_type).name("encoder").build().expect("could not create encoder element"),
            payloader: gstreamer::ElementFactory::make(&payloader_el_type).name("pauloader").build().expect("could not create payloader element"),
            preset,
            settings: HashMap::new(),
            extra_prefix_elements: prefix,
            extra_suffix_elements: suffix,
            config: None,
        }
    }

    pub fn set_config(&mut self, config: Option<Arc<StreamerConfig>>) {
        self.config = config;
    }

    pub fn get_optimizations(&self) -> PipelineOptimization {
        match &self.config {
            Some(config) => {
                config.optimizations
            },
            None => PipelineOptimization::None,
        }
    }

    // TODO: support audio

    pub fn get_encoder_element_type_simple(preset: EncodingPreset) -> String {
        Self::get_encoder_element_type(preset, PipelineOptimization::None)
    }

    pub fn get_encoder_element_type(preset: EncodingPreset, optimizations: PipelineOptimization) -> String {
        match preset {
            // wow supermaven so smart???
            EncodingPreset::H264 => {
                match optimizations {
                    PipelineOptimization::NVIDIA => "nvh264enc".to_string(),
                    _ => "x264enc".to_string(),
                }
            },
            EncodingPreset::H265 => {
                match optimizations {
                    PipelineOptimization::NVIDIA => "nvh265enc".to_string(),
                    _ => "x265enc".to_string(),
                }
            },
            EncodingPreset::VP8 => "vp8enc".to_string(),
            EncodingPreset::VP9 => "vp9enc".to_string(),
            EncodingPreset::AV1 => "rav1enc".to_string(),
            EncodingPreset::Unknown => panic!("unknown encoding preset should provide details on how to create the encoder"),
        }
    }

    pub fn get_payloader_element_type_simple(preset: EncodingPreset) -> String {
        Self::get_payloader_element_type(preset, PipelineOptimization::None)
    }

    pub fn get_payloader_element_type(preset: EncodingPreset, optimizations: PipelineOptimization) -> String {
        match preset {
            // wow supermaven so smart???
            EncodingPreset::H264 => "rtph264pay".to_string(),
            EncodingPreset::H265 => "rtph265pay".to_string(),
            EncodingPreset::VP8 => "rtpvp8pay".to_string(),
            EncodingPreset::VP9 => "rtpvp9pay".to_string(),
            EncodingPreset::AV1 => "rtpav1pay".to_string(), // https://gstreamer.freedesktop.org/documentation/rsrtp/rtpav1pay.html?gi-language=c
            EncodingPreset::Unknown => panic!("unknown encoding preset should provide details on how to create the payloader"),
        }
    }

    pub fn set_default_settings(&mut self){
        match self.preset {
            // TODO: fix this, but it defaults to 0 which is based off resolution
            // EncodingPreset::H264 => self.set_setting("bitrate", serde_json::json!(1024 * 1024 * 6)),
            // EncodingPreset::H265 => self.set_setting("bitrate", serde_json::json!(1024 * 1024 * 4)),
            EncodingPreset::VP8 => self.set_setting("target-bitrate", serde_json::json!(1024 * 1024 * 6)),
            EncodingPreset::VP9 => self.set_setting("target-bitrate", serde_json::json!(1024 * 1024 * 4)),
            EncodingPreset::AV1 => self.set_setting("target-bitrate", serde_json::json!(1024 * 1024 * 4)),
            _ => {
                // idk
            }
        }

        if SUPPORTS_DEADLINE.contains(&self.preset) {
            self.set_setting("deadline", serde_json::json!(0));
        }
    }

    pub fn set_setting(&mut self, key: &str, value: serde_json::Value) {

        // match with preset
        let value_clone = value.clone();
        if matches!(self.preset, EncodingPreset::Unknown) {

        } else {
            match key {
                "target-bitrate" => {
                    if SUPPORTS_TARGET_BITRATE.contains(&self.preset) {
                        if let serde_json::Value::Number(value_number) = value {
                            // makes a uint in gstreamer strictness
                            let mut bitrate: i32 = value_number.as_u64().unwrap_or(1024 * 1024 * 4) as i32;
                            bitrate = num::clamp(bitrate, 1024 * 300, 1024 * 1024 * 100);
                            self.encoder.set_property("target-bitrate", bitrate); // gint
                        }
                    }
                },
                "bitrate" => {
                    if SUPPORTS_BITRATE.contains(&self.preset) {
                        if let serde_json::Value::Number(value_number) = value {
                            // makes a uint in gstreamer strictness
                            let mut bitrate: u32 = value_number.as_u64().unwrap_or(1024 * 1024 * 4) as u32;
                            bitrate = num::clamp(bitrate, 1024 * 300, 1024 * 1024 * 100);
                            self.encoder.set_property("bitrate", bitrate); //guint
                        }
                    }
                },
                "deadline" => {
                    if SUPPORTS_DEADLINE.contains(&self.preset) {
                        if let serde_json::Value::Number(value_number) = value {
                            // makes a uint in gstreamer strictness
                            let mut deadline: i64 = value_number.as_i64().unwrap_or(0);
                            deadline = num::clamp(deadline, 0,1000);
                            self.encoder.set_property("deadline", deadline); 
                        }
                    }
                },
                _ => {
                    // unknown
                }
            }
        }

        self.settings.insert(key.to_string(), value_clone);
    }
}