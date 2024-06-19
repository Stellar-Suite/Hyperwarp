use std::collections::HashMap;

use gstreamer::{prelude::*, Buffer, BufferFlags, Element, ErrorMessage};
use gstreamer_video::{prelude::*, VideoInfo};
use stellar_protocol::protocol::EncodingPreset;

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

        // Self::new(encoder, payloader)
        todo!("impl");
    }

    pub fn get_element_type(preset: EncodingPreset) -> String {
        match preset {
            // wow supermaven so smart???
            EncodingPreset::H264 => "vp8enc".to_string(),
            EncodingPreset::H265 => "vp8enc".to_string(),
            EncodingPreset::VP8 => "vp8enc".to_string(),
            EncodingPreset::VP9 => "vp8enc".to_string(),
            EncodingPreset::AV1 => "vp8enc".to_string(),
            EncodingPreset::Unknown => "vp8enc".to_string(),
        }
    }

    pub fn set_setting(&mut self, key: &str, value: serde_json::Value) {

        // match with preset

        self.settings.insert(key.to_string(), value);
    }
}