use std::collections::HashMap;

use gstreamer::{prelude::*, Buffer, BufferFlags, Element, ErrorMessage};
use gstreamer_video::{prelude::*, VideoInfo};
use stellar_protocol::protocol::EncodingPreset;

use lazy_static::lazy_static;

pub struct WebRTCPeer {
    pub id: String,
    pub queue: gstreamer::Element,
    pub webrtcbin: gstreamer::Element,
}

impl WebRTCPeer {
    pub fn new(id: String) -> Self {
        Self {
            id,
            queue: gstreamer::ElementFactory::make("queue").build().expect("could not create queue element"),
            webrtcbin: gstreamer::ElementFactory::make("webrtcbin").build().expect("could not create webrtcbin element"),
        }
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
}

pub struct WebRTCPreprocessor {
    pub encoder: gstreamer::Element,
    pub payloader: gstreamer::Element,
    pub extra_prefix_elements: Vec<gstreamer::Element>,
    pub extra_suffix_elements: Vec<gstreamer::Element>,
    pub preset: EncodingPreset,
    settings: HashMap<String, serde_json::Value>, // this allows us to set the settings for the encoder and payloader regardless of format easily
}

lazy_static! {
    pub static ref SUPPORTS_TARGET_BITRATE: Vec<EncodingPreset> = vec![EncodingPreset::H264, EncodingPreset::H265, EncodingPreset::VP8, EncodingPreset::VP9, EncodingPreset::AV1];
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
        }
    }

    // maybe a better name for this is encodertype not preset
    pub fn new_preset(preset: EncodingPreset) -> Self {
        let prefix: Vec<gstreamer::Element> = vec![];
        let suffix: Vec<gstreamer::Element> = vec![];

        // TODO: support unknown
        let encoder_el_type = WebRTCPreprocessor::get_encoder_element_type(preset);
        let payloader_el_type = WebRTCPreprocessor::get_payloader_element_type(preset);
        Self {
            encoder: gstreamer::ElementFactory::make(&encoder_el_type).name("encoder").build().expect("could not create encoder element"),
            payloader: gstreamer::ElementFactory::make(&payloader_el_type).name("pauloader").build().expect("could not create payloader element"),
            preset,
            settings: HashMap::new(),
            extra_prefix_elements: prefix,
            extra_suffix_elements: suffix,
        }
    }

    pub fn get_encoder_element_type(preset: EncodingPreset) -> String {
        match preset {
            // wow supermaven so smart???
            EncodingPreset::H264 => "x264enc".to_string(),
            EncodingPreset::H265 => "x265enc".to_string(),
            EncodingPreset::VP8 => "vp8enc".to_string(),
            EncodingPreset::VP9 => "vp9enc".to_string(),
            EncodingPreset::AV1 => "rav1enc".to_string(),
            EncodingPreset::Unknown => panic!("unknown encoding preset should provide details on how to create the encoder"),
        }
    }

    pub fn get_payloader_element_type(preset: EncodingPreset) -> String {
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

    pub fn set_setting(&mut self, key: &str, value: serde_json::Value) {

        // match with preset
        let value_clone = value.clone();
        if matches!(self.preset, EncodingPreset::Unknown) {

        } else {
            match key {
                "target-bitrate" => {
                    if SUPPORTS_TARGET_BITRATE.contains(&self.preset) {
                        if let serde_json::Value::Number(value_number) = value {
                            let mut bitrate = value_number.as_u64().unwrap_or(1024 * 1024 * 4);
                            bitrate = num::clamp(bitrate, 1024 * 300, 1024 * 1024 * 100);
                            self.encoder.set_property("target-bitrate", bitrate);
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